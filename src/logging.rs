use crate::cstr::Utf8CStr;
use crate::result::SilentResultExt;
use crate::{cstr, logging, raw_cstr};
use core::fmt;
use libc::{
    c_char, dev_t, makedev, mknod, mode_t, syscall, unlink, write, SYS_dup3, O_CLOEXEC, O_WRONLY,
    STDERR_FILENO, STDIN_FILENO, STDOUT_FILENO, S_IFCHR,
};
use libc::{sleep, O_RDWR};
use std::io::stdout;
use std::mem::ManuallyDrop;
use std::process::exit;
use std::{
    fs::File,
    io::{IoSlice, Write},
    os::fd::{FromRawFd, IntoRawFd, RawFd},
};

pub(crate) type Formatter<'a> = &'a mut dyn fmt::Write;

mod LogFlag {
    pub const DisableError: u32 = 1 << 0;
    pub const DisableWarn: u32 = 1 << 1;
    pub const DisableInfo: u32 = 1 << 2;
    pub const DisableDebug: u32 = 1 << 3;
    pub const ExitOnError: u32 = 1 << 4;
}
#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    ErrorCxx,
    Error,
    Warn,
    Info,
    Debug,
}

impl LogLevel {
    fn as_disable_flag(&self) -> u32 {
        match *self {
            LogLevel::Error | LogLevel::ErrorCxx => LogFlag::DisableError,
            LogLevel::Warn => LogFlag::DisableWarn,
            LogLevel::Info => LogFlag::DisableInfo,
            LogLevel::Debug => LogFlag::DisableDebug,
        }
    }
}

pub static mut LOGGER: Logger = Logger {
    write: |_, _| {},
    flags: 0,
};

type LogWriter = fn(level: LogLevel, msg: &Utf8CStr);

#[derive(Copy, Clone)]
pub struct Logger {
    pub write: LogWriter,
    pub flags: u32,
}

// SAFETY: magiskinit is single threaded
static mut KMSG: RawFd = -1;

#[macro_export]
macro_rules! info {
    ($($args:tt)+) => {
        $crate::logging::log_with_formatter($crate::logging::LogLevel::Info, |w| writeln!(w, $($args)+))
    }
}

#[cfg(debug_assertions)]
#[macro_export]
macro_rules! debug {
    ($($args:tt)+) => {
        $crate::logging::log_with_formatter($crate::logging::LogLevel::Debug, |w| writeln!(w, $($args)+))
    }
}

#[cfg(not(debug_assertions))]
#[macro_export]
macro_rules! debug {
    ($($args:tt)+) => {};
}

#[macro_export]
macro_rules! log_with_args {
    ($level:expr, $($args:tt)+) => {
        log_with_formatter($level, |w| writeln!(w, $($args)+))
    }
}

fn log_with_writer<F: FnOnce(LogWriter)>(level: LogLevel, f: F) {
    let logger = unsafe { LOGGER };
    if (logger.flags & level.as_disable_flag()) != 0 {
        return;
    }
    f(logger.write);
    if matches!(level, LogLevel::ErrorCxx) && (logger.flags & LogFlag::ExitOnError) != 0 {
        exit(-1);
    }
}

pub fn log_with_formatter<F: FnOnce(Formatter) -> fmt::Result>(level: LogLevel, f: F) {
    log_with_writer(level, |_write| {
        let mut buf = cstr::buf::default();
        f(&mut buf).ok();
        _write(level, &buf);
    });
}

pub fn setup_klog() {
    unsafe {
        let mut fd = cstr!("/dev/null").open(O_RDWR | O_CLOEXEC).silent();
        if fd.is_err() {
            mknod(raw_cstr!("/null"), S_IFCHR | 0o666, makedev(1, 3));
            fd = cstr!("/null").open(O_RDWR | O_CLOEXEC).silent();
            cstr!("/null").remove().ok();
        }
        if let Ok(ref fd) = fd {
            syscall(SYS_dup3, fd, STDIN_FILENO, O_CLOEXEC);
            syscall(SYS_dup3, fd, STDOUT_FILENO, O_CLOEXEC);
            syscall(SYS_dup3, fd, STDERR_FILENO, O_CLOEXEC);
        }
        let mut fd = cstr!("/dev/kmsg").open(O_WRONLY | O_CLOEXEC).silent();
        if fd.is_err() {
            mknod(raw_cstr!("/kmsg"), S_IFCHR | 0o666, makedev(1, 11));
            fd = cstr!("/kmsg").open(O_WRONLY | O_CLOEXEC).silent();
            cstr!("/kmsg").remove().ok();
        }
        KMSG = fd.map(|fd| fd.into_raw_fd()).unwrap_or(-1);
    };

    if let Ok(mut rate) = cstr!("/proc/sys/kernel/printk_devkmsg").open(O_WRONLY | O_CLOEXEC) {
        writeln!(rate, "on").ok();
    }
    fn kmsg_log_write(_: LogLevel, msg: &Utf8CStr) {
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

pub fn set_log_level_state(level: LogLevel, enabled: bool) {
    let flag = level.as_disable_flag();
    unsafe {
        if enabled {
            LOGGER.flags &= !flag
        } else {
            LOGGER.flags |= flag
        }
    }
}

#[test]
fn test_debug_log() {
    fn stdout_log_write(_: LogLevel, msg: &Utf8CStr) {
        let io1 = IoSlice::new("magiskinit: ".as_bytes());
        let io2 = IoSlice::new(msg.as_bytes());

        let mut out = stdout();
        let _ = out.write_vectored(&[io1, io2]).ok();
        let _ = out.write_all(b"\n").ok(); // 添加换行
    }

    let logger = Logger {
        write: stdout_log_write,
        flags: 0,
    };
    unsafe {
        LOGGER = logger;
    }
    debug!(" test_debug_log ");

    // let kmsg = CString::new("/kmsg").unwrap();
    // log_with_writer(LogLevel::Error, |write| {
    //     write(LogLevel::Error, &kmsg);
    // });

    log_with_args!(LogLevel::Debug, "log_with_args");
}
