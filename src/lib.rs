mod logging;

pub mod cstr;
pub mod result;
pub mod MagiskLib {

    use crate::cstr;
    use crate::cstr::CStrOpenExt;
    use libc::{execve, exit, mount, umount, O_CLOEXEC, O_WRONLY};
    use std::ffi::{c_char, c_void, CStr, CString};
    use std::os::unix::fs::PermissionsExt;
    use std::path::Path;
    use std::{fs, ptr};

    unsafe extern "C" {
        static environ: *const *mut libc::c_char;
    }

    pub struct Utf8CString(String);

    pub struct OverlayAttr(Utf8CString, Utf8CString);

    struct KeyValue {
        key: String,
        value: String,
    }

    pub struct BootConfig {
        skip_initramfs: bool,
        force_normal_boot: bool,
        rootwait: bool,
        emulator: bool,
        slot: String,
        dt_dir: [c_char; 64],
        fstab_suffix: [c_char; 32],
        hardware: [c_char; 32],
        hardware_plat: [c_char; 32],
        partition_map: Vec<KeyValue>,
    }

    pub struct MagiskInit {
        preinit_dev: String,
        mount_list: Vec<String>,
        argv: *mut *mut c_char,
        config: BootConfig,
        overlay_con: Vec<OverlayAttr>,
    }

    impl BootConfig {
        pub fn init(&mut self) -> () {}
        pub fn set(&mut self, mut kv: Vec<(String, String)>) {
            for (key, value) in kv {
                match key.as_str() {
                    "androidboot.slot_suffix" => {
                        // Many Amlogic devices are A-only but have slot_suffix...
                        if value == "normal" {
                            // log::warn!("Skip invalid androidboot.slot_suffix=[normal]");
                            continue;
                        }
                        self.slot = value;
                        // copy_str_to_array(&mut self.slot, value);
                    }
                    // "androidboot.slot" => {
                    //     self.slot[0] = b'_';
                    //     let mut rest = &mut self.slot[1..];
                    //     copy_str_to_array(&mut rest, value);
                    // }
                    // "skip_initramfs" => {
                    //     self.skip_initramfs = true;
                    // }
                    // "androidboot.force_normal_boot" => {
                    //     self.force_normal_boot = !value.is_empty() && value.as_bytes()[0] == b'1';
                    // }
                    // "rootwait" => {
                    //     self.rootwait = true;
                    // }
                    // "androidboot.android_dt_dir" => {
                    //     copy_str_to_array(&mut self.dt_dir, value);
                    // }
                    // "androidboot.hardware" => {
                    //     copy_str_to_array(&mut self.hardware, value);
                    // }
                    // "androidboot.hardware.platform" => {
                    //     copy_str_to_array(&mut self.hardware_plat, value);
                    // }
                    // "androidboot.fstab_suffix" => {
                    //     copy_str_to_array(&mut self.fstab_suffix, value);
                    // }
                    // "qemu" => {
                    //     self.emulator = true;
                    // }
                    // "androidboot.partition_map" => {
                    //     for (k, v) in parse_partition_map(value) {
                    //         self.partition_map.push((k, v));
                    //     }
                    // }
                    _ => {}
                }
            }
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
                    dt_dir: [0; 64],
                    fstab_suffix: [0; 32],
                    hardware: [0; 32],
                    hardware_plat: [0; 32],
                    partition_map: Vec::new(),
                },
            }
        }
        pub fn start(&mut self) -> () {
            if !cstr!("/proc/cmdline").exists() {
                let dir_path = "/proc";
                fs::create_dir(dir_path);
                let perms = fs::Permissions::from_mode(0o755);
                fs::set_permissions(dir_path, perms);
                let source = CString::new("proc").unwrap();
                let target = CString::new("/proc").unwrap();
                let fstype = CString::new("proc").unwrap();
                let flags = 0; // 没有 MsFlags 就直接用 0
                let data = ptr::null::<c_void>();

                let res = unsafe {
                    mount(
                        source.as_ptr(),
                        target.as_ptr(),
                        fstype.as_ptr(),
                        flags,
                        data,
                    )
                };
                if res == 0 {
                    self.mount_list.push("/proc".to_string());
                }
            }

            if !cstr!("/sys/block").exists() {
                let dir_path = "/sys";
                fs::create_dir(dir_path);
                let perms = fs::Permissions::from_mode(0o755);
                fs::set_permissions(dir_path, perms);
                let source = CString::new("sysfs").unwrap();
                let target = CString::new("/sys").unwrap();
                let fstype = CString::new("sysfs").unwrap();
                let flags = 0; // 没有 MsFlags 就直接用 0
                let data = ptr::null::<c_void>();

                let res = unsafe {
                    mount(
                        source.as_ptr(),
                        target.as_ptr(),
                        fstype.as_ptr(),
                        flags,
                        data,
                    )
                };
                if res == 0 {
                    self.mount_list.push("/proc".to_string());
                }
                self.mount_list.push("/sys".to_string());
            }

            crate::logging::setup_klog();
            self.config.init();

            let argv1 = unsafe { *self.argv.offset(1) };
            // if !argv1.is_null() && unsafe { CStr::from_ptr(argv1) == c"selinux_setup" } {
            //     self.second_stage();
            // } else if self.config.skip_initramfs {
            //     self.legacy_system_as_root();
            // } else if self.config.force_normal_boot {
            //     self.first_stage();
            // } else if cstr!("/sbin/recovery").exists() || cstr!("/system/bin/recovery").exists() {
            //     self.recovery();
            // } else if self.check_two_stage() {
            //     self.first_stage();
            // } else {
            //     self.rootfs();
            // }

            // Finally execute the original init
            self.exec_init();
        }

        pub(crate) fn exec_init(&mut self) {
            for path in self.mount_list.iter_mut().rev() {
                // umount(path.);
                // let path = CString::from_string(path);
                // if path.unmount().log().is_ok() {
                //     debug!("Unmount [{}]", path);
                // }
            }
            unsafe {
                let init = CString::new("/init").unwrap();

                execve(init.as_ptr(), self.argv.cast(), environ.cast());
                exit(1);
            }
        }
    }
}
