#![feature(path_try_exists)]
use anyhow::{anyhow, Context, Error, Result};
use gptman::linux::get_sector_size;
use gptman::GPT;
use lazy_static::lazy_static;
use std::fs::DirEntry;
use std::io::{Seek, SeekFrom};
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;
use std::{collections::HashMap, fs};
use uuid::Uuid;

mod linux;
use crate::linux::block::{get_blk_devices, BlkDevice, BlkTransport, FromStat, MajMin};
use crate::linux::musl::stat;

lazy_static! {
    static ref EVE_CONFIG_UUID: Uuid =
        Uuid::from_str("13307e62-cd9c-4920-8f9b-91b45828b798").unwrap();
}

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
    fn from_proc(self) -> Result<Self> {
        let raw = fs::read_to_string("/proc/cmdline").context("cannot open /proc/cmdline")?;
        self.parse(&raw)
    }

    #[allow(dead_code)]
    fn from_str(self, s: &str) -> Result<Self> {
        self.parse(s)
    }

    fn parse(mut self, s: &str) -> Result<Self> {
        let split: Vec<&str> = s.trim().split(' ').collect();
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

enum RunMode {
    Installer,
    StorageInit,
}
struct InstallerCtx {
    config: InstallerConfig,
    boot_device: Option<BlkDevice>,
    block_devices: Vec<BlkDevice>,
}

// # content of rootfs partition
// ROOTFS_IMG=/parts/rootfs.img
// # content of conf partition
// CONF_FILE=/parts/config.img
// # content of persist partition
// PERSIST_FILE=/parts/persist.img
// # EFI boot directory
// EFI_DIR=/parts/EFI
// # early bootloader directory (optional)
// BOOT_DIR=/parts/boot
// # content of initrd installer image (optional)
// INITRD_IMG=/parts/initrd.img
// # content of installer ECO (optional)
// INSTALLER_IMG=/parts/installer.img
// # GRUB cfg override for our installer
// INSTALLER_GRUB_CFG=/parts/grub.cfg

#[cfg(test)]
mod test;

impl InstallerCtx {
    fn build() -> Result<Self> {
        let cmd_line = KernelCmdline::new()
            .from_proc()
            .with_context(|| "Cannot parse kernel command line")?;
        let config = InstallerConfig::from_cmdline(&cmd_line);
        Ok(Self {
            config: config,
            boot_device: None,
            block_devices: Vec::new(),
        })
    }

    fn detect_boot_device(&mut self) -> Result<BlkDevice> {
        let st = stat(PathBuf::from("/hostfs/media/boot"))
            .with_context(|| "Couldn't get stats for '/hostfs/media/boot'")?;

        println!("{:#?}", st);

        let mm = MajMin::from_stat(&st)?;

        println!("{:#?}", mm);

        self.block_devices =
            get_blk_devices(false).with_context(|| "Couldn't get a list of block devices")?;

        println!("{:#?}", self.block_devices);

        // let boot_device = self
        //     .block_devices
        //     .iter()
        //     .find(|e| e.majmin == mm)
        //     .map(|e| e.to_owned())
        //     .ok_or(anyhow!("Cannot detect boot device!"))?;

        // if (boot_device.part_index.is_some()) {
        let has_partition = |dev: &BlkDevice, part: &MajMin| {
            dev.partitions.as_ref().map_or(false, |parts| {
                parts.iter().find(|p| p.majmin == *part).is_some()
            })
        };
        let boot_device = self
            .block_devices
            .iter()
            .find(|blk| has_partition(blk, &mm))
            .map(ToOwned::to_owned)
            .ok_or(anyhow!("Cannot detect boot device!"));
        // } else {
        //     self.boot_device = Some(boot_device);
        // }

        boot_device
    }

    fn get_target_disks(&self) -> Vec<BlkDevice> {
        let mut result = Vec::new();
        if let Some(boot_dev) = &self.boot_device {
            result = self
                .block_devices
                .iter()
                .filter(|e| *e != boot_dev)
                .filter(|dev| {
                    // if we have a filter return only suitable drives
                    if let Some(install_disk) = &self.config.eve_install_disk {
                        dev.device_path.to_string_lossy() == *install_disk
                    } else {
                        true
                    }
                })
                .map(|e| e.to_owned())
                .collect();
        }
        result
    }

    fn get_persist_disks(&self) -> Vec<BlkDevice> {
        let mut result = Vec::new();
        if let Some(boot_dev) = &self.boot_device {
            result = self
                .block_devices
                .iter()
                // skip boot device
                .filter(|e| *e != boot_dev)
                .filter(|dev| {
                    // we have a filter. include only matched entries
                    // FIXME: we may have a situation when eve_persist_disk have more disks that we detected
                    if let Some(disks) = &self.config.eve_persist_disk {
                        disks
                            .iter()
                            .find(|e| **e == dev.device_path.to_string_lossy())
                            .is_some()
                    } else {
                        true // always include disk if there is not filter
                    }
                })
                .map(|e| e.to_owned())
                .collect();
        }
        result
    }

    fn open_disk(&mut self, dev: &str) -> Result<(GPT, u64)> {
        let mut f = fs::File::open(&dev)?;
        let gpt = GPT::find_from(&mut f)?;
        let len = f.seek(SeekFrom::End(0))?;

        Ok((gpt, len))
    }

    fn do_install(&mut self) -> Result<()> {
        self.boot_device = Some(self.detect_boot_device()?);

        if let Some(boot_device) = self.boot_device {
            println!("Boot device: {}", boot_device.device_path);
        }

        let mut disk = fs::File::open(&self.boot_device.as_ref().unwrap().device_path)?;
        let len = disk.seek(SeekFrom::End(0))?;

        if GPT::find_from(&mut disk).is_ok() {}

        // let mut gpt =
        //     gptman::GPT::new_from(&mut disk, sector_size, Uuid::new_v4().as_bytes().to_owned())?;

        Ok(())

        // if let dev = &self.boot_device {
        //     println!("Boot device: {:#?}", dev);
        //     let persist = self.get_persist_disks();
        //     println!("Perist: {:#?}", &persist);

        //     Ok(())
        // } else {
        //     Err(anyhow!("Couldn't detext boot device"))
        // }
    }
}

// pub fn generate_random_uuid() -> [u8; 16] {
//     rand::thread_rng().gen()
// }

fn detect_run_mode() -> Result<RunMode> {
    match fs::try_exists("/hostfs/media/boot") {
        Ok(true) => Ok(RunMode::Installer),
        Ok(false) => Ok(RunMode::StorageInit),
        Err(err) => Err(err)
            .map_err(anyhow::Error::from)
            .with_context(|| "detect_run_mode failed!"),
    }
}

fn run_installer() -> Result<InstallerCtx> {
    // run_os_command("mount")?;
    // run_os_command("ls -la /dev")?;

    let mut ctx = InstallerCtx::build()?;

    match detect_run_mode()? {
        RunMode::Installer => {
            println!("Running in INSTALLER mode");
            ctx.do_install()?;
        }
        RunMode::StorageInit => {
            println!("Running in STORAGE-INIT mode");
        }
    }

    Ok(ctx)
}

fn main() -> Result<()> {
    let res = get_blk_devices(false);

    println!("{:#?}", res);

    match run_installer() {
        Ok(_) => {
            println!("Installation completed");
            run_os_command("/run-console.sh")?;
            Ok(())
        }
        Err(er) => {
            println!("Installation failed! {:?}", er);
            println!("==== entering shell ====");
            run_os_command("/run-console.sh")?;
            Err(er)
        }
    }

    // //run_os_command("mount")?;
    // //run_os_command("lsblk -anlb -o TYPE,NAME,SIZE,TRAN")?;
    // res.map(|_| ())
}
