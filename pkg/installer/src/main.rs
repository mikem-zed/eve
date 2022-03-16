#![feature(path_try_exists)]
use anyhow::{anyhow, Context, Error, Result};
use arrayvec::ArrayVec;
use gptman::linux::get_sector_size;
use gptman::{GPTPartitionEntry, GPT};
use lazy_static::lazy_static;
use linux::block::generate_random_uuid;
use regex::internal::Input;
use regex::Regex;
use std::ascii::AsciiExt;
use std::env::temp_dir;
use std::fs::{DirEntry, File, OpenOptions};
use std::hash::Hasher;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;
use std::{collections::HashMap, fs};
use tempfile::tempdir;

mod linux;
use crate::linux::block::{get_blk_devices, BlkDevice, BlkTransport, FromStat, MajMin};
use crate::linux::musl::stat;

//use gptman::

#[cfg(test)]
mod test;
#[cfg(test)]
mod test_guid {
    use crate::PartitionGUID;
    use anyhow::Result;

    #[test]
    fn test_guid_hyphened() -> Result<()> {
        let res = PartitionGUID::parse_guid_mixed_endian("C12A7328-F81F-11D2-BA4B-00A0C93EC93B");
        match res {
            Ok(bytes) => {
                assert_eq!(
                    bytes.bytes,
                    [
                        0x28, 0x73, 0x2A, 0xC1, 0x1F, 0xF8, 0xD2, 0x11, 0xBA, 0x4B, 0x00, 0xA0,
                        0xC9, 0x3E, 0xC9, 0x3B
                    ]
                );
                Ok(())
            }
            Err(err) => Err(err),
        }
    }
}
//FIXME: I had to wrap it into an object to amke lazy_static! happy
struct PartitionGUID {
    bytes: [u8; 16],
}

impl PartitionGUID {
    fn parse_guid_mixed_endian(guid: &str) -> Result<Self> {
        let guid = guid
            .as_bytes()
            .to_ascii_uppercase()
            .into_iter()
            .filter(|&e| e != b'-')
            .map(|e| {
                if e.is_ascii_hexdigit() {
                    Ok(e)
                } else {
                    Err(anyhow!("incorrect HEX character"))
                }
            })
            .collect::<Result<Vec<u8>>>()?;
        if guid.len() != 32 {
            return Err(anyhow!("Incorrect string length"));
        } else {
            let to_4bit = |a: u8| match a {
                b'0'..=b'9' => a - b'0',
                b'A'..=b'F' => a - b'A' + 10,
                _ => 0, // this is not really the case becasue we rejected incorrect input already
            };

            let first32_le: ArrayVec<_, 4> = guid[0..8]
                .chunks(2)
                .rev()
                //.inspect(|e| println!("0x{}{}", char::from(e[0]), char::from(e[1])))
                .map(|e| to_4bit(e[0]) << 4 | to_4bit(e[1]))
                .collect();

            let second16_le: ArrayVec<_, 2> = guid[8..12]
                .chunks(2)
                .rev()
                .map(|e| to_4bit(e[0]) << 4 | to_4bit(e[1]))
                .collect();

            let third16_le: ArrayVec<_, 2> = guid[12..16]
                .chunks(2)
                .rev()
                .map(|e| to_4bit(e[0]) << 4 | to_4bit(e[1]))
                .collect();

            let be_tail: Vec<u8> = guid[16..32]
                .chunks(2)
                .map(|e| to_4bit(e[0]) << 4 | to_4bit(e[1]))
                .collect();

            let result: ArrayVec<_, 16> = first32_le
                .into_iter()
                .chain(second16_le.into_iter())
                .chain(third16_le.into_iter())
                .chain(be_tail.into_iter())
                .collect();
            Ok(Self {
                bytes: result
                    .into_inner()
                    .map_err(|_| anyhow!("Error converting"))?,
            })
        }
    }
}

lazy_static! {
    static ref EFI_SYSTEM_UUID: PartitionGUID =
        PartitionGUID::parse_guid_mixed_endian("C12A7328-F81F-11D2-BA4B-00A0C93EC93B").unwrap();
    static ref PARTITION_TYPE_USR_X86_64: PartitionGUID =
        PartitionGUID::parse_guid_mixed_endian("5dfbf5f4-2848-4bac-aa5e-0d9a20b745a6").unwrap();
    static ref CONFIG_GUID: PartitionGUID =
        PartitionGUID::parse_guid_mixed_endian("13307e62-cd9c-4920-8f9b-91b45828b798").unwrap();
    static ref BOOTLOADER_RE: Regex = Regex::new(r"BOOT.*\.EFI").unwrap();
}

//EFI partition size in bytes
const EFI_PART_SIZE: u64 = 36 * 1024 * 1024;
// rootfs partition size in bytes
const ROOTFS_PART_SIZE: u64 = 300 * 1024 * 1024;
// conf partition size in bytes
const CONF_PART_SIZE: u64 = 1024 * 1024;
// installer inventory parition size in bytes
const WIN_INVENTORY_PART_SIZE: u64 = 40240 * 1024;
// installer system parition size in bytes
const INSTALLER_SYS_PART_SIZE: u64 = 300 * 1024 * 1024;
// sector where the first partition starts on a blank disk
const FIRST_PART_SEC: u64 = 2048;
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

        // if e have more than one disk - this is ZFS,
        // if we do not specify the disk but have a ',' this is ZFS as well
        // if we have one disk and a ',' this is ZFS
        config.persist_fs_zfs = eve_persist_disk.map_or(false, |e| e.contains(","));

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

    fn create_partition(
        &self,
        gpt: &mut GPT,
        idx: u32,
        label: &str,
        size: u64,
        type_uuid: [u8; 16],
    ) -> Result<()> {
        let sec_size = (size - 1) / gpt.sector_size + 1;

        let start = gpt.find_first_place(sec_size).ok_or(anyhow!(
            "Couldn't find a place for partition {}: size={}",
            label,
            size
        ))?;

        let aling_size = |size: u64| {
            let aligned_up = ((sec_size - 1) / gpt.align + 1) * gpt.align;
            aligned_up
        };

        gpt[idx] = GPTPartitionEntry {
            starting_lba: start,
            ending_lba: start + aling_size(size) + 1,
            partition_type_guid: type_uuid,
            unique_partition_guid: generate_random_uuid(),
            attribute_bits: 0,
            partition_name: label.into(),
        };
        Ok(())
    }

    fn create_efi_partition(&self, gpt: &mut GPT) -> Result<()> {
        self.create_partition(gpt, 1, "EFI System", EFI_PART_SIZE, EFI_SYSTEM_UUID.bytes)?;
        gpt[1].attribute_bits = 1 << 2; //legacy bootable
        Ok(())
    }

    fn create_rootfs_partition(&self, gpt: &mut GPT, label: &str, primary: bool) -> Result<()> {
        let index = if primary { 2 } else { 3 };
        self.create_partition(
            gpt,
            index,
            label,
            ROOTFS_PART_SIZE,
            PARTITION_TYPE_USR_X86_64.bytes,
        )?;
        if primary {
            gpt[index].attribute_bits = 1 << 49 | 1 << 56;
        }
        Ok(())
    }

    fn create_config_partition(&self, gpt: &mut GPT) -> Result<()> {
        self.create_partition(gpt, 4, "CONFIG", CONF_PART_SIZE, CONFIG_GUID.bytes)
    }

    fn create_partition_table(&self, dev: &BlkDevice) -> Result<BlkDevice> {
        let (mut gpt, size) = dev.new_gpt()?;

        println!("GPT: {:#?}", &gpt.header);

        self.create_efi_partition(&mut gpt)?;
        self.create_rootfs_partition(&mut gpt, "IMGA", true)?;
        self.create_rootfs_partition(&mut gpt, "IMGB", false)?;
        self.create_config_partition(&mut gpt)?;

        let mut fd = OpenOptions::new().write(true).open(&dev.device_path)?;
        //TODO: replace with hybrid MBR
        GPT::write_protective_mbr_into(&mut fd, gpt.sector_size)?;
        gpt.write_into(&mut fd)
            .with_context(|| format!("Cannot writr GPT to {}", dev.device_path_str()))?;
        // this call affects only a subtree of the owning block device
        gptman::linux::reread_partition_table(&mut fd)?;

        //rescan block devices
        let block_devices = get_blk_devices(false)?;

        let install_disk = block_devices
            .iter()
            .find(|e| e.device_path_str() == dev.device_path_str())
            .ok_or(anyhow!("Could not get updated install disck"))
            .map(|e| e.to_owned());
        install_disk
    }

    fn populate_config(&self, src_part: &BlkDevice, dst_part: &BlkDevice) -> Result<()> {
        mkfs_vfat(dst_part.device_path_str().as_str(), "EVE")?;

        let o_path = tempdir()?;
        let i_path = tempdir()?;

        mount_vfat(dst_part.device_path_str().as_str(), o_path.path())?;
        mount_vfat(src_part.device_path_str().as_str(), i_path.path())?;

        // pass tmpdir by reference or the folder will be dropped
        copy_dir_all(&i_path, &o_path)?;

        // overwrite the server if provided in config
        if let Some(server) = &self.config.eve_install_server {
            fs::write(&o_path.path().join("server"), server)?;
        }

        umount(dst_part.device_path_str().as_str())?;
        umount(src_part.device_path_str().as_str())?;
        Ok(())
    }

    fn find_grub(&self) -> Result<PathBuf> {
        let dir = fs::read_dir("/hostfs/media/boot/EFI/BOOT")?;
        let bootloader = dir
            .filter_map(|e| e.ok())
            .find(|e| BOOTLOADER_RE.is_match(&e.file_name().to_string_lossy()));
        bootloader
            .and_then(|e| Some(Path::new("/EFI/BOOT").join(e.file_name())))
            .ok_or(anyhow!("Couldn't get bootloader path"))
    }

    fn generate_grub_config(&self, path: &Path) -> Result<()> {
        let grub = self.find_grub()?;
        let mut cfg = fs::read_to_string("/grub.cfg.in")?;
        cfg = cfg.replace("@PATH_TO_GRUB@", &grub.strip_prefix("/")?.to_string_lossy());
        fs::write(path.join("grub.cfg"), cfg)?;
        Ok(())
    }

    fn populate_efi(&self, dst_part: &BlkDevice) -> Result<()> {
        mkfs_vfat(dst_part.device_path_str().as_str(), "EVE")?;

        let dst_efi_path = tempdir()?;

        mount_vfat(dst_part.device_path_str().as_str(), dst_efi_path.path())?;

        let grub_path = dst_efi_path.path().join("EFI/BOOT");

        fs::create_dir_all(&grub_path)?;

        // pass tmpdir by reference or the folder will be dropped
        copy_dir_all(Path::new("/hostfs/media/boot/EFI/BOOT/"), &grub_path)?;
        self.generate_grub_config(&grub_path)?;

        umount(dst_part.device_path_str().as_str())?;
        Ok(())
    }

    fn populate_partitions(&self, install_dev: &BlkDevice, boot_dev: &BlkDevice) -> Result<()> {
        let imga = install_dev
            .find_part("IMGA")
            .ok_or(anyhow!("Couldn't find IMGA"))?;
        // let imgb = dev.find_part("IMGB").ok_or(anyhow!("Couldn't find IMGB"))?;
        let dst_config_part = install_dev
            .find_part("CONFIG")
            .ok_or(anyhow!("Couldn't find CONFIG"))?;

        let src_config_part = boot_dev
            .find_part("CONFIG")
            .ok_or(anyhow!("Couldn't find CONFIG"))?;

        let dst_efi_part = install_dev
            .find_part("EFI System")
            .ok_or(anyhow!("Couldn't find EFI System"))?;

        println!("config_i: {:#?}", src_config_part);

        self.populate_config(src_config_part, dst_config_part)?;
        self.populate_efi(dst_efi_part)?;

        dd(
            "/hostfs/media/boot/rootfs.img",
            imga.device_path_str().as_str(),
        )?;
        // We do not populate IMGB
        // dd(
        //     "/hostfs/media/boot/rootfs.img",
        //     imgb.device_path_str().as_str(),
        // )?;
        Ok(())
    }

    fn do_install(&mut self) -> Result<()> {
        self.boot_device = Some(self.detect_boot_device()?);

        if let Some(boot_device) = &self.boot_device {
            println!("Boot device: {}", boot_device.device_path_str());

            println!("{:#?}", boot_device.find_part("CONFIG"));
            println!("{:#?}", boot_device.find_part("I do not exist"));

            // skip boot device
            let target_disks = self
                .get_target_disks()
                .into_iter()
                .filter(|d| d.majmin != boot_device.majmin)
                .collect::<Vec<BlkDevice>>();
            println!("TARGET: {:#?}", target_disks);

            //FIXME: take the first available for now
            //TODO: sort by transport
            let install_disk = &target_disks[0];
            println!(
                "INSTALLER: installing onto {}",
                install_disk.device_path_str()
            );

            //TODO: check existent partition table
            let install_disk = self.create_partition_table(install_disk)?;

            println!("NEW BOOT: {:#?}", install_disk);

            self.populate_partitions(&install_disk, &boot_device)?;

            // skip boot device and install_disk
            let persist_disks = self
                .get_persist_disks()
                .into_iter()
                .filter(|d| d.majmin != boot_device.majmin)
                .filter(|d| d.majmin != install_disk.majmin)
                .collect::<Vec<BlkDevice>>();

            println!("PERSIST: {:#?}", persist_disks);
        }

        // let mut disk = fs::File::open(&self.boot_device.as_ref().unwrap().device_path)?;
        // let len = disk.seek(SeekFrom::End(0))?;

        // if GPT::find_from(&mut disk).is_ok() {}

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

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) -> Result<()> {
    println!(
        "Copy all from {} to {}",
        src.as_ref().to_string_lossy(),
        dst.as_ref().to_string_lossy()
    );
    fs::create_dir_all(&dst)?;
    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        if ty.is_dir() {
            copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

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

    println!("Detecting run mode");

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

fn dd(inf: &str, of: &str) -> Result<()> {
    run_os_command(format!("dd if={} of={} bs=1M", inf, of).as_str())
}

fn mkfs_vfat(dev: &str, label: &str) -> Result<()> {
    println!("Formating {} as VFAT", dev);
    run_os_command(format!("mkfs.vfat -n {} {}", label, dev).as_str())
}

fn mount(dev: &str, path: &Path) -> Result<()> {
    println!("Mounting {} as {}", dev, path.to_string_lossy());
    run_os_command(format!("mount {} {}", dev, path.to_string_lossy()).as_str())
}

fn mount_vfat(dev: &str, path: &Path) -> Result<()> {
    run_os_command(
        format!(
            "mount -t vfat -o iocharset=iso8859-1 {} {}",
            dev,
            path.to_string_lossy()
        )
        .as_str(),
    )
}

fn umount(dev: &str) -> Result<()> {
    println!("Unmounting {}", dev);
    run_os_command(format!("umount {}", dev).as_str())
}

fn main() -> Result<()> {
    println!("Starting EVE installer...");
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
}
