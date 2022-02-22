use std::{ffi::CString, mem::MaybeUninit, path::PathBuf};

use anyhow::{anyhow, Result};

pub fn stat(path: PathBuf) -> Result<libc::stat> {
    let mut st = MaybeUninit::uninit();

    let c_str = CString::new(path.to_str().ok_or(anyhow!(
        "Couldn't convert a pth to CString {}",
        path.to_string_lossy()
    ))?)?;
    unsafe {
        match libc::stat(c_str.as_ptr(), st.as_mut_ptr()) {
            0 => Ok(st.assume_init()),
            x => Err(anyhow!("Error calling stat: {}", x)),
        }
    }
}

// fn mystatx(path: PathBuf) -> Result<statx> {
//     let mut st = MaybeUninit::uninit();
//     let flags: c_int = libc::AT_SYMLINK_NOFOLLOW;
//     let mask: c_uint = libc::STATX_ALL;
//     let c_str = CString::new(path.to_str().ok_or(anyhow!(
//         "Couldn't convert a pth to CString {}",
//         path.to_string_lossy()
//     ))?)?;
//     unsafe {
//         match statx(0, c_str.as_ptr(), flags, mask, st.as_mut_ptr()) {
//             0 => Ok(st.assume_init()),
//             x => Err(anyhow!("Error calling statx: {}", x)),
//         }
//     }
// }
