use std::os::unix::ffi::OsStrExt;
use std::{ffi::CString, path::Path};
use std::fs::File;
use std::os::unix::io::FromRawFd;
use libc::{open};

#[macro_export]
macro_rules! cstr {
    ($s:expr) => {
        ::std::ffi::CString::new($s).expect("CString conversion failed")
    };
}
// 给 CString 实现 open 方法
pub trait CStrOpenExt {
    fn open(&self, flags: i32) -> std::io::Result<File>;
    fn exists(&self) -> bool;

}

impl CStrOpenExt for CString {
    fn open(&self, flags: i32) -> std::io::Result<File> {
        let fd = unsafe { open(self.as_ptr(), flags) };
        if fd < 0 {
            Err(std::io::Error::last_os_error())
        } else {
            // SAFETY: File takes ownership of the fd
            Ok(unsafe { File::from_raw_fd(fd) })
        }
    }
    fn exists(&self) -> bool {
        // 将 CString 转为 Path
        let path = Path::new(std::ffi::OsStr::from_bytes(self.as_bytes()));
        return path.exists();
    }
}

