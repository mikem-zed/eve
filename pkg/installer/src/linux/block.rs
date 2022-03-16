use crate::linux::musl::stat as linux_stat;
use anyhow::{anyhow, Context, Error, Result};
use gptman::{linux::get_sector_size, GPTPartitionEntry, GPT};
use lazy_static::lazy_static;
use libc::{major, minor, stat};
use rand::Rng;
use regex::Regex;
use std::{
    fmt,
    fs::{self, DirEntry},
    io::{Seek, SeekFrom},
    path::PathBuf,
    str::FromStr,
};

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
    pub fn device_path(&self) -> String {
        format!("/sys/dev/block/{}", self)
    }
}

impl fmt::Display for MajMin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.maj, self.min)
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
    pub label: Option<String>,
    pub uuid: Option<[u8; 16]>,
}

pub fn generate_random_uuid() -> [u8; 16] {
    rand::thread_rng().gen()
}

impl BlkDevice {
    pub fn device_path_str(&self) -> String {
        self.device_path.to_string_lossy().to_string()
    }

    pub fn find_part(&self, label: &str) -> Option<&BlkDevice> {
        let ret = self.partitions.as_ref().and_then(|e| {
            e.iter()
                .find(|p| p.label.as_ref().and_then(|e| Some(e == label)) == Some(true))
        });
        ret
    }
    pub fn new_gpt(&self) -> Result<(GPT, u64)> {
        if self.part_index.is_some() {
            return Err(anyhow!(
                "Cannot create GPT on a prtition {}",
                self.device_path_str()
            ));
        }
        let mut fd = fs::File::open(&self.device_path)?;
        let gpt = GPT::new_from(&mut fd, self.sector_size, generate_random_uuid())?;
        let len = fd.seek(SeekFrom::End(0))?;

        Ok((gpt, len))
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
        lazy_static! {
            static ref MAJMIN_RE: Regex = Regex::new(r"^([0-9]+):([0-9]+)$").unwrap();
        }

        MAJMIN_RE
            .captures(s)
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
                "Device node for {}:{} not found under /dev",
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
    let device_path = find_dev_node(&majmin)
        .with_context(|| format!("Cannot find /dev node for device {}", majmin))?;

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

    //FIXME: collect::<Resilt<T>> on empty itterutor return OK()
    let part_devs = part_dirs
        .iter()
        .map(|e| read_blk_device(e))
        .collect::<Result<Vec<_>>>();

    let mut fd = fs::File::open(&device_path)
        .with_context(|| format!("Couldn't get file descriptor for {:?}", device_path))?;
    let gpt = gptman::GPT::find_from(&mut fd);
    let uuid = if let Ok(gpt) = &gpt {
        Some(gpt.header.disk_guid)
    } else {
        None
    };

    let partitons = if let Ok(mut partitions) = part_devs {
        if partitions.is_empty() {
            None
        } else {
            // we MUST have a partition table if we have partitions
            //FIXME: it may be MBR without GPT
            let gpt = gpt.unwrap();
            // this device have partitions. detect partition lables
            let find_gpt_entry = |idx: u32, part: &GPTPartitionEntry, part_idx: Option<u32>| {
                if let Some(part_idx) = part_idx {
                    if part_idx == idx {
                        return Some((part.partition_name.to_owned(), part.unique_partition_guid));
                    }
                }
                None
            };

            partitions.iter_mut().for_each(|dev| {
                if let Some((label, uuid)) = gpt
                    .iter()
                    .filter(|(_, p)| p.is_used())
                    .find_map(|(idx, p)| find_gpt_entry(idx, p, dev.part_index))
                {
                    dev.label = Some(label.to_string());
                    dev.uuid = Some(uuid);
                }
            });

            Some(partitions)
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

    Ok(BlkDevice {
        transport: BlkTransport::Sata,
        is_virtual: is_virtual,
        majmin: majmin,
        partitions: partitons,
        part_index: part_index,
        sector_size: get_sector_size(&mut fd)?, //FIXME: will panic for non-block devices
        device_path: device_path,
        label: None,
        uuid: uuid,
    })
}

pub fn get_blk_devices(include_virtual: bool) -> Result<Vec<BlkDevice>> {
    let mut result = Vec::new();

    // read devices but not partitions
    for dir in fs::read_dir("/sys/block")? {
        let dir = dir?;
        let b = read_blk_device(&dir).with_context(|| format!("Couldn't read {:?}", dir.path()));
        match b {
            Ok(b) => {
                if !include_virtual && b.is_virtual {
                    continue;
                }
                result.push(b);
            }
            Err(e) => {
                println!("WARNING: Couldn't read a block device {}", e);
            }
        }
    }

    Ok(result)
}
