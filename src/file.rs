use crate::cstr;
use crate::cstr::{FsPathFollow, Utf8CStr, Utf8CStrBuf};
use crate::dir::Directory;
use crate::result::{LibcReturn, OsError, OsResult, OsResultStatic};
use libc::{c_uint, mode_t, stat, EEXIST, ENOENT, O_CLOEXEC, O_CREAT, O_PATH, O_RDONLY, O_RDWR, O_TRUNC, O_WRONLY};
use nix::errno::{errno, Errno};
use std::ffi::CStr;
use std::fmt::Display;
use std::fs::File;
use std::{mem, ptr, slice};
use std::os::fd::{AsFd, AsRawFd, BorrowedFd, FromRawFd, OwnedFd, RawFd};
use std::os::unix::ffi::OsStrExt;
use std::path::Path;

fn open_fd(path: &Utf8CStr, flags: i32, mode: mode_t) -> OsResult<OwnedFd> {
    unsafe {
        let fd = libc::open(path.as_ptr(), flags, mode as c_uint).as_os_result(
            "open",
            Some(path),
            None,
        )?;
        Ok(OwnedFd::from_raw_fd(fd))
    }
}
pub fn fd_path(fd: RawFd, buf: &mut dyn Utf8CStrBuf) -> OsResult<'static, ()> {
    let path = cstr::buf::new::<64>()
        .join_path("/proc/self/fd")
        .join_path_fmt(fd);
    path.read_link(buf).map_err(|e| e.set_args(None, None))
}

pub struct FileAttr {
    pub st: libc::stat,
    #[cfg(feature = "selinux")]
    pub con: crate::Utf8CStrBufArr<128>,
}

impl FileAttr {
    fn new() -> Self {
        FileAttr {
            st: unsafe { mem::zeroed() },
            #[cfg(feature = "selinux")]
            con: crate::Utf8CStrBufArr::new(),
        }
    }

    #[inline(always)]
    #[allow(clippy::unnecessary_cast)]
    fn is(&self, mode: mode_t) -> bool {
        (self.st.st_mode & libc::S_IFMT as c_uint) as mode_t == mode
    }

    pub fn is_dir(&self) -> bool {
        self.is(libc::S_IFDIR)
    }

    pub fn is_file(&self) -> bool {
        self.is(libc::S_IFREG)
    }

    pub fn is_symlink(&self) -> bool {
        self.is(libc::S_IFLNK)
    }

    pub fn is_block_device(&self) -> bool {
        self.is(libc::S_IFBLK)
    }

    pub fn is_char_device(&self) -> bool {
        self.is(libc::S_IFCHR)
    }

    pub fn is_fifo(&self) -> bool {
        self.is(libc::S_IFIFO)
    }

    pub fn is_socket(&self) -> bool {
        self.is(libc::S_IFSOCK)
    }

    pub fn is_whiteout(&self) -> bool {
        self.is_char_device() && self.st.st_rdev == 0
    }
}

const XATTR_NAME_SELINUX: &CStr = c"security.selinux";

impl Utf8CStr {
    pub fn follow_link(&self) -> &FsPathFollow {
        unsafe { mem::transmute(self) }
    }

    pub fn open(&self, flags: i32) -> OsResult<File> {
        Ok(File::from(open_fd(self, flags, 0)?))
    }

    pub fn create(&self, flags: i32, mode: mode_t) -> OsResult<File> {
        Ok(File::from(open_fd(self, O_CREAT | flags, mode)?))
    }

    pub fn exists(&self) -> bool {
        unsafe {
            let mut st: stat = mem::zeroed();
            libc::lstat(self.as_ptr(), &mut st) == 0
        }
    }

    pub fn rename_to<'a>(&'a self, name: &'a Utf8CStr) -> OsResult<'a, ()> {
        unsafe {
            libc::rename(self.as_ptr(), name.as_ptr()).check_os_err(
                "rename",
                Some(self),
                Some(name),
            )
        }
    }

    pub fn remove(&self) -> OsResult<()> {
        unsafe { libc::remove(self.as_ptr()).check_os_err("remove", Some(self), None) }
    }

    pub fn remove_all(&self) -> OsResultStatic<()> {
        let attr = self.get_attr()?;
        if attr.is_dir() {
            let mut dir = Directory::try_from(open_fd(self, O_RDONLY | O_CLOEXEC, 0)?)?;
            dir.remove_all()?;
        }
        Ok(self.remove()?)
    }

    #[allow(clippy::unnecessary_cast)]
    pub fn read_link(&self, buf: &mut dyn Utf8CStrBuf) -> OsResult<()> {
        buf.clear();
        unsafe {
            let r = libc::readlink(self.as_ptr(), buf.as_mut_ptr(), buf.capacity() - 1)
                .as_os_result("readlink", Some(self), None)? as isize;
            *(buf.as_mut_ptr().offset(r) as *mut u8) = b'\0';
            buf.set_len(r as usize);
        }
        Ok(())
    }

    pub fn mkdir(&self, mode: mode_t) -> OsResult<()> {
        unsafe {
            if libc::mkdir(self.as_ptr(), mode) < 0 {
                if Errno::last_raw() == EEXIST {
                    libc::chmod(self.as_ptr(), mode).check_os_err("chmod", Some(self), None)?;
                } else {
                    return Err(OsError::last_os_error("mkdir", Some(self), None));
                }
            }
        }
        Ok(())
    }

    pub fn mkdirs(&self, mode: mode_t) -> OsResultStatic<()> {
        if self.is_empty() {
            return Ok(());
        }

        let mut path = cstr::buf::default();
        let mut components = self.split('/').filter(|s| !s.is_empty());

        if self.starts_with('/') {
            path.append_path("/");
        }

        loop {
            let Some(s) = components.next() else {
                break;
            };
            path.append_path(s);

            unsafe {
                if libc::mkdir(path.as_ptr(), mode) < 0 && Errno::last_raw() != EEXIST {
                    return Err(OsError::last_os_error("mkdir", Some(&path), None))?;
                }
            }
        }
        Errno::set_raw(0);
        // *errno() = 0;
        Ok(())
    }

    // Inspired by https://android.googlesource.com/platform/bionic/+/master/libc/bionic/realpath.cpp
    pub fn realpath(&self, buf: &mut dyn Utf8CStrBuf) -> OsResult<()> {
        let fd = self.open(O_PATH | O_CLOEXEC)?;
        let mut st1: libc::stat;
        let mut st2: libc::stat;
        let mut skip_check = false;
        unsafe {
            st1 = mem::zeroed();
            if libc::fstat(fd.as_raw_fd(), &mut st1) < 0 {
                // This will only fail on Linux < 3.6
                skip_check = true;
            }
        }
        fd_path(fd.as_raw_fd(), buf)?;
        unsafe {
            st2 = mem::zeroed();
            libc::stat(buf.as_ptr(), &mut st2).check_os_err("stat", Some(self), None)?;
            if !skip_check && (st2.st_dev != st1.st_dev || st2.st_ino != st1.st_ino) {
                Errno::set_raw(ENOENT);
                return Err(OsError::last_os_error("realpath", Some(self), None));
            }
        }
        Ok(())
    }

    pub fn get_attr(&self) -> OsResult<FileAttr> {
        let mut attr = FileAttr::new();
        unsafe {
            libc::lstat(self.as_ptr(), &mut attr.st).check_os_err("lstat", Some(self), None)?;

            #[cfg(feature = "selinux")]
            self.get_secontext(&mut attr.con)?;
        }
        Ok(attr)
    }

    pub fn set_attr<'a>(&'a self, attr: &'a FileAttr) -> OsResult<'a, ()> {
        unsafe {
            if !attr.is_symlink() && libc::chmod(self.as_ptr(), (attr.st.st_mode & 0o777)) < 0 {
                let self_attr = self.get_attr()?;
                if !self_attr.is_symlink() {
                    return Err(OsError::last_os_error("chmod", Some(self), None));
                }
            }
            libc::lchown(self.as_ptr(), attr.st.st_uid, attr.st.st_gid).check_os_err(
                "lchown",
                Some(self),
                None,
            )?;

            #[cfg(feature = "selinux")]
            if !attr.con.is_empty() {
                self.set_secontext(&attr.con)?;
            }
        }
        Ok(())
    }

    pub fn get_secontext(&self, con: &mut dyn Utf8CStrBuf) -> OsResult<()> {
        unsafe {
            let sz = libc::lgetxattr(
                self.as_ptr(),
                XATTR_NAME_SELINUX.as_ptr(),
                con.as_mut_ptr().cast(),
                con.capacity(),
            );
            if sz < 1 {
                con.clear();
                if Errno::last_raw() != libc::ENODATA {
                    return Err(OsError::last_os_error("lgetxattr", Some(self), None));
                }
            } else {
                con.set_len((sz - 1) as usize);
            }
        }
        Ok(())
    }

    pub fn set_secontext<'a>(&'a self, con: &'a Utf8CStr) -> OsResult<'a, ()> {
        unsafe {
            libc::lsetxattr(
                self.as_ptr(),
                XATTR_NAME_SELINUX.as_ptr(),
                con.as_ptr().cast(),
                con.len() + 1,
                0,
            )
            .check_os_err("lsetxattr", Some(self), Some(con))
        }
    }

    pub fn copy_to(&self, path: &Utf8CStr) -> OsResultStatic<()> {
        let attr = self.get_attr()?;
        if attr.is_dir() {
            path.mkdir(0o777)?;
            let mut src = Directory::open(self)?;
            let dest = Directory::open(path)?;
            src.copy_into(&dest)?;
        } else {
            // It's OK if remove failed
            path.remove().ok();
            if attr.is_file() {
                let mut src = self.open(O_RDONLY | O_CLOEXEC)?;
                let mut dest = path.create(O_WRONLY | O_CREAT | O_TRUNC | O_CLOEXEC, 0o777)?;
                std::io::copy(&mut src, &mut dest)?;
            } else if attr.is_symlink() {
                let mut buf = cstr::buf::default();
                self.read_link(&mut buf)?;
                unsafe {
                    libc::symlink(buf.as_ptr(), path.as_ptr()).check_os_err(
                        "symlink",
                        Some(&buf),
                        Some(path),
                    )?;
                }
            }
        }
        path.set_attr(&attr)?;
        Ok(())
    }

    pub fn move_to(&self, path: &Utf8CStr) -> OsResultStatic<()> {
        if path.exists() {
            let attr = path.get_attr()?;
            if attr.is_dir() {
                let mut src = Directory::open(self)?;
                let dest = Directory::open(path)?;
                return src.move_into(&dest);
            } else {
                path.remove()?;
            }
        }
        self.rename_to(path)?;
        Ok(())
    }

    pub fn parent_dir(&self) -> Option<&str> {
        Path::new(self.as_str())
            .parent()
            .map(Path::as_os_str)
            // SAFETY: all substring of self is valid UTF-8
            .map(|s| unsafe { std::str::from_utf8_unchecked(s.as_bytes()) })
    }

    pub fn file_name(&self) -> Option<&str> {
        Path::new(self.as_str())
            .file_name()
            // SAFETY: all substring of self is valid UTF-8
            .map(|s| unsafe { std::str::from_utf8_unchecked(s.as_bytes()) })
    }

    // ln self path
    pub fn link_to(&self, path: &Utf8CStr) -> OsResultStatic<()> {
        let attr = self.get_attr()?;
        if attr.is_dir() {
            path.mkdir(0o777)?;
            path.set_attr(&attr)?;
            let mut src = Directory::open(self)?;
            let dest = Directory::open(path)?;
            Ok(src.link_into(&dest)?)
        } else {
            unsafe {
                libc::link(self.as_ptr(), path.as_ptr()).check_os_err(
                    "link",
                    Some(self),
                    Some(path),
                )?;
            }
            Ok(())
        }
    }

    // ln -s target self
    pub fn create_symlink_to<'a>(&'a self, target: &'a Utf8CStr) -> OsResult<'a, ()> {
        unsafe {
            libc::symlink(target.as_ptr(), self.as_ptr()).check_os_err(
                "symlink",
                Some(target),
                Some(self),
            )
        }
    }

    pub fn mkfifo(&self, mode: mode_t) -> OsResult<()> {
        unsafe { libc::mkfifo(self.as_ptr(), mode).check_os_err("mkfifo", Some(self), None) }
    }
}

pub trait FsPathBuilder {
    fn join_path<T: AsRef<str>>(mut self, path: T) -> Self
    where
        Self: Sized,
    {
        self.append_path(path);
        self
    }
    fn join_path_fmt<T: Display>(mut self, name: T) -> Self
    where
        Self: Sized,
    {
        self.append_path_fmt(name);
        self
    }
    fn append_path<T: AsRef<str>>(&mut self, path: T) -> &mut Self;
    fn append_path_fmt<T: Display>(&mut self, name: T) -> &mut Self;
}

fn append_path_impl(buf: &mut dyn Utf8CStrBuf, path: &str) {
    if path.starts_with('/') {
        buf.clear();
    }
    if !buf.is_empty() && !buf.ends_with('/') {
        buf.push_str("/");
    }
    buf.push_str(path);
}

impl<S: Utf8CStrBuf + Sized> FsPathBuilder for S {
    fn append_path<T: AsRef<str>>(&mut self, path: T) -> &mut Self {
        append_path_impl(self, path.as_ref());
        self
    }

    fn append_path_fmt<T: Display>(&mut self, name: T) -> &mut Self {
        self.write_fmt(format_args!("/{name}")).ok();
        self
    }
}

impl FsPathBuilder for dyn Utf8CStrBuf + '_ {
    fn append_path<T: AsRef<str>>(&mut self, path: T) -> &mut Self {
        append_path_impl(self, path.as_ref());
        self
    }

    fn append_path_fmt<T: Display>(&mut self, name: T) -> &mut Self {
        self.write_fmt(format_args!("/{name}")).ok();
        self
    }
}

pub fn fd_get_attr(fd: RawFd) -> OsResult<'static, FileAttr> {
    let mut attr = FileAttr::new();
    unsafe {
        libc::fstat(fd, &mut attr.st).check_os_err("fstat", None, None)?;

        #[cfg(feature = "selinux")]
        fd_get_secontext(fd, &mut attr.con)?;
    }
    Ok(attr)
}

pub fn fd_set_attr(fd: RawFd, attr: &FileAttr) -> OsResult<()> {
    unsafe {
        libc::fchmod(fd, (attr.st.st_mode & 0o777)).check_os_err("fchmod", None, None)?;
        libc::fchown(fd, attr.st.st_uid, attr.st.st_gid).check_os_err("fchown", None, None)?;

        #[cfg(feature = "selinux")]
        if !attr.con.is_empty() {
            fd_set_secontext(fd, &attr.con)?;
        }
    }
    Ok(())
}

pub fn fd_get_secontext(fd: RawFd, con: &mut dyn Utf8CStrBuf) -> OsResult<'static, ()> {
    unsafe {
        let sz = libc::fgetxattr(
            fd,
            XATTR_NAME_SELINUX.as_ptr(),
            con.as_mut_ptr().cast(),
            con.capacity(),
        );
        if sz < 1 {
            if Errno::last_raw() != libc::ENODATA {
                return Err(OsError::last_os_error("fgetxattr", None, None));
            }
        } else {
            con.set_len((sz - 1) as usize);
        }
    }
    Ok(())
}

pub fn fd_set_secontext(fd: RawFd, con: &Utf8CStr) -> OsResult<()> {
    unsafe {
        libc::fsetxattr(
            fd,
            XATTR_NAME_SELINUX.as_ptr(),
            con.as_ptr().cast(),
            con.len() + 1,
            0,
        )
        .check_os_err("fsetxattr", Some(con), None)
    }
}

pub fn clone_attr<'a>(a: &'a Utf8CStr, b: &'a Utf8CStr) -> OsResult<'a, ()> {
    let attr = a.get_attr().map_err(|e| e.set_args(Some(a), None))?;
    b.set_attr(&attr).map_err(|e| e.set_args(Some(b), None))
}

pub fn fclone_attr(a: RawFd, b: RawFd) -> OsResult<'static, ()> {
    let attr = fd_get_attr(a)?;
    fd_set_attr(b, &attr).map_err(|e| e.set_args(None, None))
}

pub struct MappedFile(&'static mut [u8]);

impl MappedFile {
    pub fn open(path: &Utf8CStr) -> OsResult<MappedFile> {
        Ok(MappedFile(map_file(path, false)?))
    }

    pub fn open_rw(path: &Utf8CStr) -> OsResult<MappedFile> {
        Ok(MappedFile(map_file(path, true)?))
    }

    pub fn openat<'a, T: AsFd>(dir: &T, path: &'a Utf8CStr) -> OsResult<'a, MappedFile> {
        Ok(MappedFile(map_file_at(dir.as_fd(), path, false)?))
    }

    pub fn openat_rw<'a, T: AsFd>(dir: &T, path: &'a Utf8CStr) -> OsResult<'a, MappedFile> {
        Ok(MappedFile(map_file_at(dir.as_fd(), path, true)?))
    }

    pub fn create(fd: BorrowedFd, sz: usize, rw: bool) -> OsResult<MappedFile> {
        Ok(MappedFile(map_fd(fd, sz, rw)?))
    }
}


impl AsRef<[u8]> for MappedFile {
    fn as_ref(&self) -> &[u8] {
        self.0
    }
}

impl AsMut<[u8]> for MappedFile {
    fn as_mut(&mut self) -> &mut [u8] {
        self.0
    }
}

impl Drop for MappedFile {
    fn drop(&mut self) {
        unsafe {
            libc::munmap(self.0.as_mut_ptr().cast(), self.0.len());
        }
    }
}


unsafe extern "C" {
    // Don't use the declaration from the libc crate as request should be u32 not i32
    fn ioctl(fd: RawFd, request: u32, ...) -> i32;
}

// We mark the returned slice static because it is valid until explicitly unmapped
pub(crate) fn map_file(path: &Utf8CStr, rw: bool) -> OsResult<&'static mut [u8]> {
    unsafe { map_file_at(BorrowedFd::borrow_raw(libc::AT_FDCWD), path, rw) }
}


pub(crate) fn map_file_at<'a>(
    dirfd: BorrowedFd,
    path: &'a Utf8CStr,
    rw: bool,
) -> OsResult<'a, &'static mut [u8]> {
    #[cfg(target_pointer_width = "64")]
    const BLKGETSIZE64: u32 = 0x80081272;

    #[cfg(target_pointer_width = "32")]
    const BLKGETSIZE64: u32 = 0x80041272;

    let flag = if rw { O_RDWR } else { O_RDONLY };
    let fd = unsafe {
        OwnedFd::from_raw_fd(
            libc::openat(dirfd.as_raw_fd(), path.as_ptr(), flag | O_CLOEXEC).as_os_result(
                "openat",
                Some(path),
                None,
            )?,
        )
    };

    let attr = fd_get_attr(fd.as_raw_fd())?;
    let sz = if attr.is_block_device() {
        let mut sz = 0_u64;
        unsafe {
            ioctl(fd.as_raw_fd(), BLKGETSIZE64, &mut sz).check_os_err("ioctl", Some(path), None)?;
        }
        sz
    } else {
        attr.st.st_size as u64
    };

    map_fd(fd.as_fd(), sz as usize, rw).map_err(|e| e.set_args(Some(path), None))
}

pub(crate) fn map_fd(fd: BorrowedFd, sz: usize, rw: bool) -> OsResult<'static, &'static mut [u8]> {
    let flag = if rw {
        libc::MAP_SHARED
    } else {
        libc::MAP_PRIVATE
    };
    unsafe {
        let ptr = libc::mmap(
            ptr::null_mut(),
            sz,
            libc::PROT_READ | libc::PROT_WRITE,
            flag,
            fd.as_raw_fd(),
            0,
        );
        if ptr == libc::MAP_FAILED {
            return Err(OsError::last_os_error("mmap", None, None));
        }
        Ok(slice::from_raw_parts_mut(ptr.cast(), sz))
    }
}

pub trait MutBytesExt {
    fn patch(&mut self, from: &[u8], to: &[u8]) -> Vec<usize>;
}

impl<T: AsMut<[u8]> + ?Sized> MutBytesExt for T {
    fn patch(&mut self, from: &[u8], to: &[u8]) -> Vec<usize> {
        use memchr::memmem;
        
        // 第一步：只做不可变借用，查找所有 offset
        let offsets: Vec<_> = memmem::find_iter(self.as_mut(), from).collect();

        // 第二步：执行 patch（可变借用）
        for &offset in &offsets {
            self.as_mut()[offset..offset + to.len()].copy_from_slice(to);
        }

        offsets
        
    }

}