use anyhow::{anyhow, Context, Result};
use std::process::Command;
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

fn main() -> Result<()> {
    let cmd_line = KernelCmdline::new().parse()?;
    let config = InstallerConfig::from_cmdline(&cmd_line);
    println!("{:#?}", cmd_line);
    println!("{:#?}", config);

    //run_os_command("mount")?;
    //run_os_command("lsblk -anlb -o TYPE,NAME,SIZE,TRAN")?;

    Ok(())
}
