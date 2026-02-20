// gowall.rs — gowall CLI 集成模块
// 通过 std::process::Command 调用系统安装的 gowall 二进制文件

use rust_i18n::t;
use std::path::Path; // 路径的不可变借用类型
use std::process::Command; // 用于创建和执行子进程 // 引入翻译宏

/// 检测系统是否已安装 gowall 命令行工具
///
/// # 实现原理
/// - 尝试执行 `gowall --version`，通过返回结果判断是否可用
/// - `Command::new("gowall")` 会在系统 `$PATH` 中查找 gowall 二进制文件
/// - `.arg("--version")` 添加参数，让 gowall 快速返回而不执行实际操作
/// - `.output()` 同步执行命令并捕获输出，返回 `Result<Output, io::Error>`
///
/// # 返回值
/// - `Ok(())`: gowall 已安装且可正常执行
/// - `Err(...)`: gowall 未安装或不可执行，错误信息包含安装指引
///
/// # Rust 特性说明
/// - `match` 结合 `if` 守卫（guard）：`Ok(output) if output.status.success()`
///   先匹配 `Ok` 变体，再检查 `status.success()`，两个条件同时满足才进入该分支
/// - `_` 通配符匹配所有其他情况（命令不存在、权限不足、非零退出码等）
/// - `.into()` 利用 `From` trait 将 `String` 自动转换为 `Box<dyn Error>`
pub fn check_installed() -> Result<(), Box<dyn std::error::Error>> {
    match Command::new("gowall").arg("--version").output() {
        // 命令执行成功且退出码为 0，说明 gowall 已安装
        Ok(output) if output.status.success() => Ok(()),
        // 其他所有情况：命令不存在、执行失败、非零退出码
        _ => Err(t!("error_gowall_not_installed").into()),
    }
}

/// 调用 `gowall convert` 对图片应用配色主题
///
/// # 参数
/// - `image_path`: 输入图片路径，`impl AsRef<Path>` 接受多种路径类型
/// - `theme`: 主题名称（如 "catppuccin", "dracula"），`&str` 是字符串的不可变借用
/// - `output_path`: 可选的输出路径，`Option<impl AsRef<Path>>` 组合了可选性和路径泛型
///
/// # 返回值
/// - `Ok(String)`: 转换成功，返回 gowall 的标准输出
/// - `Err(...)`: 命令执行失败或 gowall 返回非零退出码
///
/// # Rust 特性说明
/// - `impl AsRef<Path>` 是泛型约束的简写（impl Trait 语法），等价于 `<P: AsRef<Path>>`
/// - `Option<impl AsRef<Path>>` 表示输出路径是可选的，且支持多种路径类型
/// - 这个函数不是 async 的，因为 `std::process::Command` 是同步的
///   对于 CLI 工具调用，同步执行更简单且足够用
pub fn convert(
    image_path: impl AsRef<Path>,
    theme: &str,
    output_path: Option<impl AsRef<Path>>,
) -> Result<String, Box<dyn std::error::Error>> {
    // 创建子进程命令
    // Command::new("gowall") 指定要执行的程序名
    // 系统会在 $PATH 中查找 gowall 二进制文件
    let mut cmd = Command::new("gowall");

    // 添加子命令和参数
    // .arg() 逐个添加命令行参数
    // image_path.as_ref() 将泛型转为 &Path
    // .as_os_str() 将 Path 转为操作系统原生字符串格式（OsStr）
    cmd.arg("convert")
        .arg(image_path.as_ref().as_os_str())
        .arg("-t")
        .arg(theme);

    // 如果指定了输出路径，添加 --output 参数
    // if let Some(path) = output_path 解构 Option，只在 Some 时执行
    if let Some(path) = output_path {
        cmd.arg("--output").arg(path.as_ref().as_os_str());
    }

    // 执行命令并捕获输出
    // .output() 同步执行命令，等待完成，捕获 stdout 和 stderr
    // 返回 Result<Output, io::Error>
    // ? 在 io::Error 时提前返回
    let output = cmd.output()?;

    // 检查命令是否成功执行
    // output.status.success() 检查退出码是否为 0
    if output.status.success() {
        // String::from_utf8_lossy() 将字节切片转为字符串
        // 遇到无效 UTF-8 字节时用 U+FFFD 替换，而非报错
        // .to_string() 将 Cow<str>（写时复制字符串）转为拥有所有权的 String
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(stdout)
    } else {
        // 命令执行失败，将 stderr 内容作为错误信息返回
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        // .into() 将 String 自动转换为 Box<dyn Error>
        // 这利用了 Rust 的 From trait 自动转换机制
        Err(t!("error_gowall_convert_failed", reason => stderr).into())
    }
}

/// 调用 `gowall list` 获取所有可用主题
///
/// # 返回值
/// - `Ok(Vec<String>)`: 主题名称列表
/// - `Err(...)`: 命令执行失败
///
/// # Rust 特性说明
/// - 返回 `Vec<String>` 而非原始字符串，方便调用方处理
/// - 每个主题名是独立的 `String`，拥有自己的堆内存
pub fn list_themes() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    // 执行 gowall list 命令
    let output = Command::new("gowall").arg("list").output()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);

        // 解析输出：每行一个主题名
        // .lines() 按换行符分割字符串，返回迭代器（惰性求值）
        // .map(|line| line.trim().to_string()) 对每行：
        //   - trim() 去除首尾空白字符，返回 &str（借用）
        //   - to_string() 创建拥有所有权的 String（因为 stdout 会被释放）
        // .filter(|s| !s.is_empty()) 过滤掉空行
        // .collect() 将迭代器收集为 Vec<String>
        //   Rust 根据返回类型自动推断 collect 的目标类型（类型推导）
        let themes: Vec<String> = stdout
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Ok(themes)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(t!("error_gowall_list_failed", reason => stderr).into())
    }
}
