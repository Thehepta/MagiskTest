use std::ptr;
use crate::cstr::Utf8CStr;
use crate::result::{LibcReturn, OsResult};

impl Utf8CStr {

    pub fn bind_mount_to<'a>(&'a self, path: &'a Utf8CStr, rec: bool) -> OsResult<'a, ()> {
        let flag = if rec { libc::MS_REC } else { 0 };
        unsafe {
            libc::mount(
                self.as_ptr(),
                path.as_ptr(),
                ptr::null(),
                libc::MS_BIND | flag,
                ptr::null(),
            )
                .check_os_err("bind_mount", Some(self), Some(path))
        }
    }
    
    pub fn unmount(&self) -> OsResult<()> {
        unsafe {
            libc::umount2(self.as_ptr(), libc::MNT_DETACH).check_os_err("unmount", Some(self), None)
        }
    }
}
