use anyhow::{anyhow, Context, Error, Result};
use libc::{c_int, c_uint, statx};
use regex::Regex;
use std::ffi::CString;
use std::fs::{DirEntry, File, ReadDir};
use std::mem::MaybeUninit;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;
use std::{collections::HashMap, fs};

#[derive(Debug)]
struct KernelCmdline {
    params: HashMap<String, Option<String>>,
}

impl KernelCmdline {
    fn new() -> Self {
        Self {
            params: HashMap::new(),
        }
    }
    fn parse(mut self) -> Result<Self> {
        let raw = fs::read_to_string("/proc/cmdline").context("cannot open /proc/cmdline")?;
        let split: Vec<&str> = raw.trim().split(' ').collect();
        split.iter().for_each(|e| {
            if let Some((key, value)) = e.split_once('=') {
                self.params.insert(key.to_string(), Some(value.to_string()));
            } else {
                self.params.insert(e.to_string(), None);
            }
        });
        Ok(self)
    }
    fn get_bool(&self, key: &str) -> bool {
        self.params.contains_key(key)
    }

    fn get_str(&self, key: &str) -> Option<String> {
        self.params.get(key).and_then(|e| e.to_owned())
    }
}

#[derive(Debug, Default)]
struct InstallerConfig {
    eve_nuke_disks: Option<Vec<String>>,
    eve_nuke_all_disks: bool,
    eve_install_disk: Option<String>,
    eve_persist_disk: Option<Vec<String>>,
    eve_install_server: Option<String>,
    eve_install_skip_rootfs: bool,
    eve_install_skip_config: bool,
    eve_install_skip_persist: bool,
    eve_pause_before_install: bool,
    eve_pause_after_install: bool,
    eve_blackbox: bool,
    eve_soft_serial: Option<String>,
    eve_reboot_after_install: bool,
    eve_install_skip_zfs_checks: bool,
    // helper fields
    persist_fs_zfs: bool,
}

impl InstallerConfig {
    fn from_cmdline(cmdline: &KernelCmdline) -> Self {
        let mut config = Self::default();
        config.eve_nuke_all_disks = cmdline.get_bool("eve_nuke_all_disks");
        config.eve_blackbox = cmdline.get_bool("eve_blackbox");
        config.eve_install_skip_config = cmdline.get_bool("eve_install_skip_config");
        config.eve_install_skip_persist = cmdline.get_bool("eve_install_skip_persist");
        config.eve_install_skip_rootfs = cmdline.get_bool("eve_install_skip_rootfs");
        config.eve_pause_after_install = cmdline.get_bool("eve_pause_after_install");
        config.eve_pause_before_install = cmdline.get_bool("eve_pause_before_install");
        config.eve_reboot_after_install = cmdline.get_bool("eve_reboot_after_install");
        config.eve_install_skip_zfs_checks = cmdline.get_bool("eve_install_skip_zfs_checks");
        config.eve_soft_serial = cmdline.get_str("eve_soft_serial");

        config.eve_install_server = cmdline.get_str("eve_install_server");
        config.eve_install_disk = cmdline.get_str("eve_install_disk");

        let eve_persist_disk = cmdline.get_str("eve_persist_disk");

        config.eve_persist_disk = eve_persist_disk
            .as_ref()
            .map(|e| e.trim().split(",").map(|e| e.to_string()).collect());

        config.persist_fs_zfs = eve_persist_disk.map_or(false, |e| e.trim().ends_with(","));

        config.eve_nuke_disks = cmdline
            .get_str("eve_nuke_disks")
            .as_ref()
            .map(|e| e.trim().split(",").map(|e| e.to_string()).collect());

        config.eve_blackbox = cmdline.get_bool("eve_blackbox");
        config
    }
}

fn run_os_command(cmdline: &str) -> Result<()> {
    let output = if let Some((cmd, params)) = cmdline.trim().split_once(' ') {
        Command::new(cmd).args(params.split(" ")).output()?
    } else {
        Command::new(cmdline.trim()).output()?
    };

    if output.status.success() {
        String::from_utf8(output.stdout)?
            .lines()
            .for_each(|line| println!("{}", line));
    } else {
        String::from_utf8(output.stderr)?
            .lines()
            .for_each(|line| println!("{}", line));
    }

    Ok(())
}

#[derive(Debug)]
enum BlkTransport {
    Sata,
    Nvme,
}

impl FromStr for BlkTransport {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "nvme" => Ok(BlkTransport::Nvme),
            "sata" => Ok(BlkTransport::Sata),
            _ => Err(anyhow!("Unknown transport type")),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct MajMin {
    maj: u32,
    min: u32,
}

impl FromStatx for MajMin {
    type Err = Error;

    fn from_statx(st: &statx) -> Result<Self, Self::Err> {
        if u32::from(st.stx_mode as u32 & libc::S_IFMT) == libc::S_IFBLK {
            Ok(Self {
                maj: st.stx_rdev_major,
                min: st.stx_rdev_minor,
            })
        } else {
            Err(anyhow!("Incorrect device mode"))
        }
    }
}

trait FromStatx: Sized {
    type Err;
    fn from_statx(st: &statx) -> Result<Self, Self::Err>;
}

impl FromStr for MajMin {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = Regex::new(r"^([0-9]+):([0-9]+)$")?;

        re.captures(s)
            .ok_or(anyhow!("Couldn't convert MAJ:MIN for {}", s))
            .and_then(|caps| {
                Ok(MajMin {
                    maj: caps
                        .get(1)
                        .ok_or(anyhow!("Couldn't convert MAJ:MIN for {}", s))?
                        .as_str()
                        .parse::<u32>()?,
                    min: caps
                        .get(2)
                        .ok_or(anyhow!("Couldn't convert MAJ:MIN for {}", s))?
                        .as_str()
                        .parse::<u32>()?,
                })
            })
    }
}
#[derive(Debug)]
struct BlkDevice {
    transport: BlkTransport,
    device_path: PathBuf,
    is_virtual: bool,
    majmin: MajMin,
}
// /sys/devices/pci0000:00/0000:00:02.1/0000:02:00.1/ata2/host1/scsi_host/host1/proc_name
// /sys/block/sda -> ../devices/pci0000:00/0000:00:02.1/0000:02:00.1/ata2/host1/target1:0:0/1:0:0:0/block/sda

fn read_blk_device(dir: DirEntry) -> Result<BlkDevice> {
    let link = fs::read_link(dir.path())?;

    // we can do it safly becasue we are working with /sys and it is always ASCI
    let is_virtual = link.to_string_lossy().contains("virtual");
    let is_scsi_host = link.to_string_lossy().contains("virtual");

    let dev_name_prefix = &dir.file_name().to_string_lossy()[0..3];

    // if let Err(_) = BlkTransport::from_str(dev_name_prefix) {

    // }

    // let is_virtual = link
    //     .components()
    //     .find(|x| x.as_os_str().eq_ignore_ascii_case("virtual"))
    //     .is_some();

    // if link.to_str().or_else(|| anyhow!("Couldn't convert MAJ:MIN for {}", s)).contains("pci") {
    //     link.to_str()
    // }

    let majmin = MajMin::from_str(fs::read_to_string(dir.path().join("dev"))?.trim())?;

    Ok(BlkDevice {
        transport: BlkTransport::Sata,
        device_path: PathBuf::from("/dev").join(dir.file_name()),
        is_virtual: is_virtual,
        majmin,
    })
}

fn get_blk_devices(include_virtual: bool) -> Result<Vec<BlkDevice>> {
    let mut result = Vec::new();

    for dir in fs::read_dir("/sys/block")? {
        let b = read_blk_device(dir?)?;
        if !include_virtual && b.is_virtual {
            continue;
        }
        result.push(b);
    }

    Ok(result)
}

fn mystatx(path: PathBuf) -> Result<statx> {
    let mut st = MaybeUninit::uninit();
    let flags: c_int = libc::AT_SYMLINK_NOFOLLOW;
    let mask: c_uint = libc::STATX_ALL;
    let c_str = CString::new(path.to_str().ok_or(anyhow!(
        "Couldn't convert a pth to CString {}",
        path.to_string_lossy()
    ))?)?;
    unsafe {
        match statx(0, c_str.as_ptr(), flags, mask, st.as_mut_ptr()) {
            0 => Ok(st.assume_init()),
            x => Err(anyhow!("Error calling statx: {}", x)),
        }
    }
}

//fn get_root_device() -> Result<PuthBuf> {}

fn main() -> Result<()> {
    let cmd_line = KernelCmdline::new()
        .parse()
        .with_context(|| "Cannot parse kernel command line")?;
    let config = InstallerConfig::from_cmdline(&cmd_line);

    println!("{:#?}", cmd_line);
    println!("{:#?}", config);

    let devices = get_blk_devices(false).with_context(|| "Couldn't get a list of block devices")?;

    for d in &devices {
        println!("{:?}", d);
    }

    //run_os_command("mount")?;
    //run_os_command("lsblk -anlb -o TYPE,NAME,SIZE,TRAN")?;
    let st = mystatx(PathBuf::from("/dev/sda"))?;

    print!("Stats: {:#?}", st);

    if u32::from(st.stx_mode as u32 & libc::S_IFMT) == libc::S_IFBLK {
        println!("this is a block device");
    }
    if u32::from(st.stx_mode as u32 & libc::S_IFMT) == libc::S_IFCHR {
        println!("this is a char device");
    }

    let mm = MajMin::from_statx(&st)?;

    let dev = devices.into_iter().find(|e| e.majmin == mm);

    println!("Root device: {:#?}", dev);

    let supported = match SupportedFilesystems::new() {
        Ok(supported) => supported,
        Err(why) => {
            eprintln!("failed to get supported file systems: {}", why);
            exit(1);
        }
    };

    Ok(())
}
