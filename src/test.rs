#[cfg(test)]
mod tests {
    use std::{
        ffi::CString,
        fs,
        io::{stdout, IoSlice, Write},
    };
    use crate::bootconfig::{parse_bootconfig, parse_cmdline};
    // 注意这个惯用法：在 tests 模块中，从外部作用域导入所有名字。
    use super::*;

    #[test]
    fn test_cmdline_parst() {
        // 获取当前工作目录
        let current_dir = std::env::current_dir().unwrap();
        println!("当前工作目录: {:?}", current_dir);

        // 构建文件路径
        let file_path = current_dir.join("cmdline.test");

        // 检查文件是否存在
        if !file_path.exists() {
            eprintln!("文件不存在: {:?}", file_path);
            std::process::exit(1);
        }

        // 读取文件内容
        let content = fs::read_to_string(file_path).unwrap();

        // 使用 parse_kv 解析
        let kv_pairs = parse_cmdline(&content);

        // 打印结果
        for (key, value) in kv_pairs {
            println!("{} = {}", key, value);
        }
    }

    #[test]
    fn test_bootconfig_parst() {
        // 获取当前工作目录
        let current_dir = std::env::current_dir().unwrap();
        println!("当前工作目录: {:?}", current_dir);

        // 构建文件路径
        let file_path = current_dir.join("bootconfig.test");

        // 检查文件是否存在
        if !file_path.exists() {
            eprintln!("文件不存在: {:?}", file_path);
            std::process::exit(1);
        }

        // 读取文件内容
        let content = fs::read_to_string(file_path).unwrap();

        // 使用 parse_kv 解析
        let kv_pairs = parse_bootconfig(&content);

        // 打印结果
        for (key, value) in kv_pairs {
            println!("{} = {}", key, value);
        }
    }
}
