#![no_main]
use std::ffi::c_char;
use Fuseisk::MagiskLib::MagiskInit;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn main(
    argc: i32,
    argv: *mut *mut c_char,
    _envp: *const *const c_char,
) -> i32 {
    unsafe {
        // umask(0);
        libc::umask(0);
        if libc::getpid() == 1 {
            MagiskInit::new(argv).start();
        }
        println!("Hello, world!2");

        return 0;
    }
}
