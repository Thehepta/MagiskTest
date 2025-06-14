use libc::O_RDWR;
use libc::{
    c_char, dev_t, makedev, mknod, mode_t, syscall, unlink, SYS_dup3, O_CLOEXEC, STDERR_FILENO,O_WRONLY,
    STDIN_FILENO, STDOUT_FILENO, S_IFCHR,
};
use std::ffi::CString;
use std::{
    fs::File,
    io::{IoSlice, Write},
    os::fd::{FromRawFd, IntoRawFd, RawFd},
};
use std::mem::ManuallyDrop;

use crate::{cstr, cstr::CStrOpenExt, result::Silent};



mod LogFlag {
    pub const DisableError: u32 = 1 << 0;
    pub const DisableWarn: u32 = 1 << 1;
    pub const DisableInfo: u32 = 1 << 2;
    pub const DisableDebug: u32 = 1 << 3;
    pub const ExitOnError: u32 = 1 << 4;
}

pub enum LogLevel {
    ErrorCxx,
    Error,
    Warn,
    Info,
    Debug,
}

pub static mut LOGGER: Logger = Logger {
    write: |_, _| {},
    flags: 0,
};


type LogWriter = fn(level: LogLevel, msg: &CString);


#[derive(Copy, Clone)]
pub struct Logger {
    pub write: LogWriter,
    pub flags: u32,
}


// SAFETY: magiskinit is single threaded
static mut KMSG: RawFd = -1;

pub fn setup_klog() {
    unsafe {
        let mut fd: Result<File, std::io::Error> = cstr!("/dev/null").open(O_RDWR | O_CLOEXEC);
        if fd.is_err() {
            let path_null = CString::new("/null").unwrap();
            let mode: mode_t = S_IFCHR | 0o666;
            let dev: dev_t = makedev(1, 3) as dev_t;
            mknod(path_null.as_ptr(), mode, dev);
            fd = cstr!("/null").open(O_RDWR | O_CLOEXEC);
            unlink(path_null.as_ptr());
        }
        if let Ok(ref fd) = fd {
            syscall(SYS_dup3, fd, STDIN_FILENO, O_CLOEXEC);
            syscall(SYS_dup3, fd, STDOUT_FILENO, O_CLOEXEC);
            syscall(SYS_dup3, fd, STDERR_FILENO, O_CLOEXEC);
        }
        let mut fd = cstr!("/dev/kmsg").open(O_RDWR | O_CLOEXEC);
        if fd.is_err() {
            let kmsg_null = CString::new("/kmsg").unwrap();
            let mode: mode_t = S_IFCHR | 0o666;
            let dev: dev_t = makedev(1, 3) as dev_t;
            mknod(kmsg_null.as_ptr(), mode, dev);
            fd = cstr!("/kmsg").open(O_RDWR | O_CLOEXEC);
            unlink(kmsg_null.as_ptr());
        }

        KMSG = fd.map(|fd| fd.into_raw_fd()).unwrap_or(-1)
    };


    if let Ok(mut rate) = cstr!("/proc/sys/kernel/printk_devkmsg").open(O_WRONLY | O_CLOEXEC) {
        writeln!(rate, "on").ok();
    }
    fn kmsg_log_write(_: LogLevel, msg: &CString) {
        let fd = unsafe { KMSG };
        if fd >= 0 {
            let io1 = IoSlice::new("magiskinit: ".as_bytes());
            let io2 = IoSlice::new(msg.as_bytes());
            let mut kmsg = ManuallyDrop::new(unsafe { File::from_raw_fd(fd) });
            let _ = kmsg.write_vectored(&[io1, io2]).ok();
        }
    }

    let logger = Logger {
        write: kmsg_log_write,
        flags: 0,
    };
    unsafe {
        LOGGER = logger;
    }
}
