use std::{ffi::c_char, fs};
use Fuseisk::{debug, info};

pub struct KeyValue {
    key: String,
    value: String,
}
pub struct BootConfig {
    pub(crate) skip_initramfs: bool,
    pub(crate) force_normal_boot: bool,
    pub(crate) rootwait: bool,
    pub(crate) emulator: bool,
    pub(crate) slot: String,
    pub(crate) dt_dir: String,
    pub(crate) fstab_suffix: String,
    pub(crate) hardware: String,
    pub(crate) hardware_plat: String,
    pub(crate) partition_map: Vec<KeyValue>,
}

const DEFAULT_DT_DIR: &str = "/proc/device-tree/firmware/android";
impl BootConfig {
    pub fn init(&mut self) -> () {
        self.set(parse_cmdline(&fs::read_to_string("/proc/cmdline").unwrap()));
        self.set(parse_bootconfig(
            &fs::read_to_string("/proc/bootconfig").unwrap(),
        ));
        debug!("Device config:\n");
        info!("Device config:\n");
        self.print();
    }
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
                }
                "androidboot.slot" => {
                    let s_ = String::from("_");
                    self.slot = s_ + &value;
                }
                "skip_initramfs" => {
                    self.skip_initramfs = true;
                }
                "androidboot.force_normal_boot" => {
                    if value == "1" {
                        self.force_normal_boot = true;
                    }
                }
                "rootwait" => {
                    self.rootwait = true;
                }
                "androidboot.android_dt_dir" => {
                    self.dt_dir = value;
                }
                "androidboot.hardware" => {
                    self.hardware = value;
                }
                "androidboot.hardware.platform" => {
                    self.hardware_plat = value;
                }
                "androidboot.fstab_suffix" => {
                    self.fstab_suffix = value;
                }
                "qemu" => {
                    self.emulator = true;
                }
                // "androidboot.partition_map" => {
                //     for (k, v) in parse_partition_map(value) {
                //         self.partition_map.push((k, v));
                //     }
                // }
                _ => {}
            }
        }
    }

    #[allow(unused_imports, unused_unsafe)]
    pub(crate) fn print(&self) {
        info!("skip_initramfs=[{}]", self.skip_initramfs);
        debug!("force_normal_boot=[{}]", self.force_normal_boot);
        debug!("rootwait=[{}]", self.rootwait);
        debug!("slot=[{}]", self.slot);
        debug!("dt_dir=[{}]", self.dt_dir);
        debug!("fstab_suffix=[{}]", self.fstab_suffix);
        debug!("hardware=[{}]", self.hardware);
        debug!("hardware.platform=[{}]", self.hardware_plat);
        debug!("emulator=[{}]", self.emulator);
        // debug!("partition_map=[{:?}]", self.partition_map);
    }
}

pub fn parse_cmdline(input: &str) -> Vec<(String, String)> {
    input
        .split_whitespace() // 使用空白字符分割每一项
        .map(|token| {
            if let Some(idx) = token.find('=') {
                let (key, value) = token.split_at(idx);
                (key.to_string(), value[1..].to_string()) // 跳过 '='
            } else {
                (token.to_string(), String::new())
            }
        })
        .collect()
}

/// 解析 key=value 格式的字符串，支持引号包裹的内容
pub fn parse_bootconfig(input: &str) -> Vec<(String, String)> {
    input
        .split("\n") // 使用空白字符分割每一项
        .map(|token| {
            if let Some(idx) = token.find('=') {
                let (key_part, value_part) = token.split_at(idx);

                // 提取 key 并去除前后空白
                let key = key_part.trim().to_string();

                // 提取 value（跳过 '=' 后）
                let mut value = value_part[1..].trim().to_string();

                // 去除两边的双引号（如果存在）
                if value.starts_with('"') && value.ends_with('"') && value.len() >= 2 {
                    value = value[1..value.len() - 1].to_string();
                }

                (key, value)
            } else {
                (token.to_string(), String::new())
            }
        })
        .collect()
}
