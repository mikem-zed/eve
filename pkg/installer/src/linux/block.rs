use anyhow::{anyhow, Error, Result};
use libc::{major, minor, stat};
use regex::Regex;
use std::{
    fs::{self, DirEntry, File, ReadDir},
    path::PathBuf,
    str::FromStr,
};

use gptman::{linux::get_sector_size, GPT};

#[derive(Debug, Clone, PartialEq)]
pub enum BlkTransport {
    Sata,
    Nvme,
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct MajMin {
    maj: u32,
    min: u32,
}

impl MajMin {
    fn device_path(&self) -> String {
        format!("/sys/dev/block/{}:{}", self.maj, self.min)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlkDevice {
    pub transport: BlkTransport,
    pub device_path: PathBuf,
    pub is_virtual: bool,
    pub majmin: MajMin,
    pub partitions: Option<Vec<BlkDevice>>,
    pub part_index: Option<u32>,
    pub sector_size: u64,
    pub label: Option<String>
}

impl BlkDevice {
    fn device_path_str(&self) -> &str {
        &self.device_path.to_string_lossy()
    }
    fn read_partition_table(&self) -> Result<GPT> {
        if let Some(index) =  {
            
        }
    }
}

pub trait FromStat: Sized {
    type Err;
    fn from_stat(st: &stat) -> Result<Self, Self::Err>;
}

impl FromStat for MajMin {
    type Err = Error;

    fn from_stat(st: &stat) -> Result<Self, Self::Err> {
        if (st.st_mode & libc::S_IFMT) == libc::S_IFBLK {
            Ok(Self {
                maj: unsafe { major(st.st_rdev) },
                min: unsafe { minor(st.st_rdev) },
            })
        } else {
            Ok(Self {
                maj: unsafe { major(st.st_dev) },
                min: unsafe { minor(st.st_dev) },
            })
        }
    }
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

use crate::linux::musl::stat as linux_stat;

fn find_dev_node(mm: &MajMin) -> Result<PathBuf> {
    let cmp_dev = |e: &DirEntry| {
        let r = linux_stat(e.path());
        r.and_then(|e| Ok(MajMin::from_stat(&e)? == *mm)).unwrap()
    };
    fs::read_dir("/dev")?
        .map(|e| e.map_err(|err| anyhow!(err)))
        .collect::<Result<Vec<DirEntry>>>()
        .and_then(|e| {
            e.into_iter().find(|x| cmp_dev(x)).ok_or(anyhow!(
                "Device {}:{} not found",
                mm.maj,
                mm.min
            ))
        })
        .map(|dir| dir.path())
}

fn read_blk_device(dir: &DirEntry) -> Result<BlkDevice> {
    let dev_name = dir.file_name();
    let link = dir.path().canonicalize()?;

    // we can do it safly becasue we are working with /sys and it is always ASCI
    let is_virtual = link.to_string_lossy().contains("virtual");

    let majmin = MajMin::from_str(fs::read_to_string(dir.path().join("dev"))?.trim())?;

    let parts_path = majmin.device_path();

    let part_dirs: Vec<_> = fs::read_dir(parts_path)?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.file_name()
                .to_str()
                .unwrap()
                .starts_with(dev_name.to_str().unwrap())
        })
        .collect();

    let part_devs = part_dirs
        .iter()
        .map(|e| read_blk_device(e)) //FIXME: i do not like ignoring error with this ok()
        .collect::<Result<Vec<_>>>();

    let partitons = if let Ok(partition) = part_devs {
        if partition.is_empty() {
            None
        } else {
            Some(partition)
        }
    } else {
        None
    };

    let part_index_path = dir.path().join("partition");

    let part_index = if part_index_path.exists() {
        Some(fs::read_to_string(part_index_path)?.trim().parse::<u32>()?)
    } else {
        None
    };

    let device_path = find_dev_node(&majmin)?;

    Ok(BlkDevice {
        transport: BlkTransport::Sata,
        is_virtual: is_virtual,
        majmin: majmin,
        partitions: partitons,
        part_index: part_index,
        sector_size: get_sector_size(&mut File::open(&device_path)?)?,
        device_path: device_path,
    })
}

pub fn get_blk_devices(include_virtual: bool) -> Result<Vec<BlkDevice>> {
    let mut result = Vec::new();

    // read devices but not partitions
    for dir in fs::read_dir("/sys/block")? {
        let b = read_blk_device(&dir?)?;
        if !include_virtual && b.is_virtual {
            continue;
        }
        result.push(b);
    }

    Ok(result)
}

// struct BlkDeviceTree<'a> {
//     // /sys/block
//     sys_block: Vec<Result<DirEntry>>,
//     blk_devices: Vec<BlkDevice>,
// }

// impl<'a> BlkDeviceTree<'a> {
//     fn new() -> Result<Self> {
//         let devices: Vec<_> = fs::read_dir("/sys/block")?
//             .filter(|e| e.is_ok())
//             .map(|e| e.map_err(|e| anyhow!(e)))
//             .collect();
//         Ok(Self {
//             sys_block: devices,
//             blk_devices: Vec::new(),
//         })
//     }
// }

// struct IntoIteratorHelper {
//     iter: ::std::vec::IntoIter<BlkDevice>,
// }

// impl<'a> IntoIterator for BlkDeviceTree<'a> {
//     type Item = &'a BlkDevice;
//     type IntoIter = IntoIteratorHelper;

//     // note that into_iter() is consuming self
//     fn into_iter(self) -> Self::IntoIter {
//         IntoIteratorHelper {
//             iter: self.blk_devices.into_iter(),
//         }
//     }
// }

// impl Iterator for IntoIteratorHelper {
//     type Item = BlkDevice;

//     // just return the reference
//     fn next(&mut self) -> Option<Self::Item> {
//         self.iter.next()
//     }
// }

// impl Iterator for BlkDeviceTree {
//     type Item = Result<BlkDevice>;

//     fn next(&mut self) -> Option<Self::Item> {
//         let next = self.current.next();
//         if let Some(next) = next {}
//         None
//     }
// }
//represents the whole /sys/device tree
// // can return itterators over individual device types e.g. block devices
// struct DeviceTree {

// }

// impl DeviceTree {
//     fn new() -> Self { Self {  } }

// }
