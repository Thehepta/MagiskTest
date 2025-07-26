use std::io::Write;
use libc::{execve, exit, fork, getpid, mount, sleep, O_CLOEXEC, O_CREAT, O_RDONLY, O_WRONLY};
use std::ffi::{c_char, c_void, CStr, CString};
use std::os::unix::fs::PermissionsExt;
use std::fs;
use std::fs::OpenOptions;
use std::io;
use std::ptr::null;
use Fuseisk::cstr::Utf8CStr;
use Fuseisk::{cstr, debug, info, raw_cstr, OverlayAttr,file::MappedFile};
use Fuseisk::cstr::buf::default;
use Fuseisk::file::MutBytesExt;
use Fuseisk::logging::setup_klog;
use Fuseisk::result::{LibcReturn, LoggedResult, ResultExt};
use crate::bootconfig::BootConfig;
// use Fuseisk::{cstr, debug, info, logging, raw_cstr};

unsafe extern "C" {
    static environ: *const *mut libc::c_char;
}

const INIT_RC: &str = "/system/etc/init/hw/init.rc";


pub struct MagiskInit {
    preinit_dev: String,
    mount_list: Vec<String>,
    argv: *mut *mut c_char,
    config: BootConfig,
    overlay_con: Vec<OverlayAttr>,
}

pub(crate) fn hexpatch_init_for_second_stage(writable: bool) {
    let init = if writable {
        MappedFile::open_rw(cstr!("/init"))
    } else {
        MappedFile::open(cstr!("/init"))
    };

    let Ok(mut init) = init else {
        info!("Failed to open /init for hexpatch");
        return;
    };

    // Redirect original init to magiskinit
    let from = "/system/bin/init";
    let to = "/data/magiskinit";
    let v = init.patch(from.as_bytes(), to.as_bytes());
    #[allow(unused_variables)]
    for off in &v {
        debug!("Patch @ {:#010X} [{}] -> [{}]", off, from, to);
    }

    if !writable {
        // If we cannot directly modify /init, we need to bind mount a replacement on top of it
        let src = cstr!("/init");
        let dest = cstr!("/data/init");
        let _: LoggedResult<()> = (|| {
            let mut fd = dest.create(O_CREAT | O_WRONLY, 0)?;
            fd.write_all(init.as_ref())?;

            let attr = src.follow_link().get_attr()?;
            dest.set_attr(&attr)?;
            dest.bind_mount_to(src, false)?;
            Ok(())
        })();
    }
}



impl MagiskInit {
    pub fn new(arg: *mut *mut c_char) -> Self {
        Self {
            preinit_dev: String::new(),
            mount_list: Vec::new(),
            overlay_con: Vec::new(),
            argv: arg,
            config: BootConfig {
                skip_initramfs: false,
                force_normal_boot: false,
                rootwait: false,
                emulator: false,
                slot: "".to_owned(),
                dt_dir: "".to_owned(),
                fstab_suffix: "".to_owned(),
                hardware: "".to_owned(),
                hardware_plat: "".to_owned(),
                partition_map: Vec::new(),
            },
        }
    }
    pub fn start(&mut self) -> LoggedResult<()> {
        if !cstr!("/proc/cmdline").exists() {
            cstr!("/proc").mkdir(0o755)?;
            unsafe {
                mount(
                    raw_cstr!("proc"),
                    raw_cstr!("/proc"),
                    raw_cstr!("proc"),
                    0,
                    null(),
                )
            }.check_io_err()?;
            self.mount_list.push("/proc".to_string());
        }

        if !cstr!("/sys/block").exists() {
            cstr!("/sys").mkdir(0o755)?;
            unsafe {
                mount(
                    raw_cstr!("sysfs"),
                    raw_cstr!("/sys"),
                    raw_cstr!("sysfs"),
                    0,
                    null(),
                )
            }
                .check_io_err()?;
            self.mount_list.push("/sys".to_string());
        }

        setup_klog();
        self.config.init();

        let argv1 = unsafe { *self.argv.offset(1) };
        if !argv1.is_null() && unsafe { CStr::from_ptr(argv1) == c"selinux_setup" } {
                self.second_stage();
            } else if self.config.skip_initramfs {
                self.legacy_system_as_root();
            } else if self.config.force_normal_boot {
                self.first_stage();
            } else if cstr!("/sbin/recovery").exists() || cstr!("/system/bin/recovery").exists() {
                self.recovery();
            } else if self.check_two_stage() {
                self.first_stage();
            } else {
                self.rootfs();
        }

        // // Finally execute the original init
        self.exec_init();
        Ok(())
    }
    fn patch_ro_root(&mut self){
        self.mount_list.push("/data".to_string());
        if cstr!(INIT_RC).exists(){
            debug!("file {} exists", INIT_RC);
            cstr!(INIT_RC).copy_to(cstr!("/data/init.rc")).log_ok();
            if cstr!("/data/init.rc").bind_mount_to(cstr!(INIT_RC), false).is_ok()
            {
                debug!("Bind mount /data/init.rc -> {}",INIT_RC);
                let mut file = OpenOptions::new().append(true).open(INIT_RC).unwrap();
                writeln!(file, "{}", "#rzxrzfewfewfewf");
            } else {
                debug!("Bind mount /data/init.rc -> {} failed",INIT_RC);
            }
            // mou
        }else {
            debug!("file {} is not exists", INIT_RC);
        }
        // let result =  unsafe { libc::fork() };
        // if result < 0{
        //     debug!("Fork failed");
        // }else if result == 0 {
        //     setup_klog();
        //
        //     loop {
        //         unsafe {
        //             sleep(1);
        //             debug!("patch_ro_root:{}",getpid());
        //
        //         }
        //     }
        // }



    }
    fn second_stage(&mut self) {
        info!("Second Stage Init");

        cstr!("/init").unmount().ok();
        cstr!("/system/bin/init").unmount().ok(); // just in case
        cstr!("/data/init").remove().ok();

        unsafe {
            // Make sure init dmesg logs won't get messed up
            *self.argv = raw_cstr!("/system/bin/init") as *mut _;
        }

        /*
        Some weird devices like meizu, uses 2SI but still have legacy rootfs
        if is_rootfs() {
        // We are still on rootfs, so make sure we will execute the init of the 2nd stage
        let init_path = cstr!("/init");
        init_path.remove().ok();
        init_path
        .create_symlink_to(cstr!("/system/bin/init"))
        .log_ok();
        self.patch_rw_root();
        } else {
        */
        self.patch_ro_root();
        // }
    }
    fn legacy_system_as_root(&mut self) {}
    fn recovery(&mut self) {}
    fn rootfs(&mut self) {}
    fn first_stage(&self) {
        info!("First Stage Init");
        self.prepare_data();

        if !cstr!("/sdcard").exists() && !cstr!("/first_stage_ramdisk/sdcard").exists() {
            self.hijack_init_with_switch_root();
            self.restore_ramdisk_init();
        } else {
            info!("First Stage start error, /sdcard or /first_stage_ramdisk/sdcard is exits");
            self.restore_ramdisk_init();
            // Fallback to hexpatch if /sdcard exists
            hexpatch_init_for_second_stage(true);
        }
    }
    fn restore_ramdisk_init(&self) {
        cstr!("/init").remove().ok();

        let orig_init = cstr!("init_back");

        if orig_init.exists() {
            orig_init.rename_to(cstr!("/init")).log_ok();
        } else {
            // If the backup init is missing, this means that the boot ramdisk
            // was created from scratch, and the real init is in a separate CPIO,
            // which is guaranteed to be placed at /system/bin/init.
            cstr!("/init")
                .create_symlink_to(cstr!("/system/bin/init"))
                .log_ok();
        }
    }
    fn check_two_stage(&self) -> bool {
        return true;
    }
    pub(crate) fn exec_init(&mut self) {
        for path in self.mount_list.iter_mut().rev() {
            let path = Utf8CStr::from_string(path);
            if path.unmount().log().is_ok() {
                debug!("Unmount [{}]", path);
            }
        }
        unsafe {
            execve(raw_cstr!("/init"), self.argv.cast(), environ.cast())
                .check_io_err()
                .log_ok();
            exit(1);
        }
    }

    pub(crate) fn prepare_data(&self) {
        debug!("Setup data tmp");
        cstr!("/data").mkdir(0o755).log_ok();
        unsafe {
            mount(
                raw_cstr!("magisk"),
                raw_cstr!("/data"),
                raw_cstr!("tmpfs"),
                0,
                raw_cstr!("mode=755").cast(),
            )
        }
            .check_io_err()
            .log_ok();

        cstr!("/init").copy_to(cstr!("/data/magiskinit")).log_ok();
        // cstr!("/.backup").copy_to(cstr!("/data/.backup")).log_ok();
        // cstr!("/overlay.d")
        //     .copy_to(cstr!("/data/overlay.d"))
        //     .log_ok();
    }
    pub(crate) fn hijack_init_with_switch_root(&self) {
        // We make use of original init's `SwitchRoot` to help us bind mount
        // magiskinit to /system/bin/init to hijack second stage init.
        //
        // Two important assumption about 2SI:
        // - The second stage init is always /system/bin/init
        // - After `SwitchRoot`, /sdcard is always a symlink to `/storage/self/primary`.
        //
        // `SwitchRoot` will perform the following:
        // - Recursive move all mounts under `/` to `/system`
        // - chroot to `/system`
        //
        // The trick here is that in Magisk's first stage init, we can mount magiskinit to /sdcard,
        // and create a symlink at /storage/self/primary pointing to /system/system/bin/init.
        //
        // During init's `SwitchRoot`, it will mount move /sdcard (which is magiskinit)
        // to /system/sdcard, which is a symlink to /storage/self/primary, which is a
        // symlink to /system/system/bin/init, which will eventually become /system/bin/init after
        // chroot to /system. The effective result is that we coerce the original init into bind
        // mounting magiskinit to /system/bin/init, successfully hijacking the second stage init.
        //
        // An edge case is that some devices (like meizu) use 2SI but does not switch root.
        // In that case, they must already have a /sdcard in ramfs, thus we can check if
        // /sdcard exists and fallback to using hexpatch.

        if self.config.force_normal_boot {
            cstr!("/first_stage_ramdisk/storage/self")
                .mkdirs(0o755)
                .log_ok();
            cstr!("/first_stage_ramdisk/storage/self/primary")
                .create_symlink_to(cstr!("/system/system/bin/init"))
                .log_ok();
            debug!("Symlink /first_stage_ramdisk/storage/self/primary -> /system/system/bin/init");
            cstr!("/first_stage_ramdisk/sdcard")
                .create(O_RDONLY | O_CREAT | O_CLOEXEC, 0)
                .log_ok();
        } else {
            cstr!("/storage/self").mkdirs(0o755).log_ok();
            cstr!("/storage/self/primary")
                .create_symlink_to(cstr!("/system/system/bin/init"))
                .log_ok();
            debug!("Symlink /storage/self/primary -> /system/system/bin/init");
        }
        cstr!("/init").rename_to(cstr!("/sdcard")).log_ok();

        // First try to mount magiskinit from rootfs to workaround Samsung RKP
        if cstr!("/sdcard")
            .bind_mount_to(cstr!("/sdcard"), false)
            .is_ok()
        {
            debug!("Bind mount /sdcard -> /sdcard");
        } else {
            // Binding mounting from rootfs is not supported before Linux 3.12
            cstr!("/data/magiskinit")
                .bind_mount_to(cstr!("/sdcard"), false)
                .log_ok();
            debug!("Bind mount /data/magiskinit -> /sdcard");
        }
    }
}
