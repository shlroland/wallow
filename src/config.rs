// config.rs — 配置管理模块
// 负责管理 Wallhaven API Key 和壁纸存储路径

use std::env; // 标准库的环境变量模块，用于读取 WALLHAVEN_API_KEY
use std::path::PathBuf; // 跨平台的路径类型，自动处理不同操作系统的路径分隔符

/// 应用配置结构体
/// 集中管理所有运行时配置项
pub struct AppConfig {
    /// Wallhaven API Key（可选）
    /// 用于访问 NSFW 内容和个性化搜索设置
    /// 从环境变量 `WALLHAVEN_API_KEY` 读取
    pub api_key: Option<String>,

    /// 壁纸保存根目录
    /// 默认为项目下的 `wallpapers/` 目录
    pub wallpaper_dir: PathBuf,

    /// 转换后壁纸的保存目录
    /// 默认为 `wallpapers/converted/`
    pub converted_dir: PathBuf,
}

impl AppConfig {
    /// 创建默认配置
    ///
    /// - `api_key`：尝试从环境变量 `WALLHAVEN_API_KEY` 读取，读不到则为 `None`
    /// - `wallpaper_dir`：当前工作目录下的 `wallpapers/`
    /// - `converted_dir`：当前工作目录下的 `wallpapers/converted/`
    ///
    /// # Rust 特性说明
    /// - `env::var()` 返回 `Result<String, VarError>`，用 `.ok()` 转为 `Option<String>`
    /// - `PathBuf::from()` 从字符串字面量创建路径对象，拥有该路径字符串的所有权（Owned）
    pub fn new() -> Self {
        // env::var() 尝试读取环境变量，返回 Result
        // .ok() 将 Result 转为 Option：Ok(val) -> Some(val)，Err(_) -> None
        let api_key = env::var("WALLHAVEN_API_KEY").ok();

        // PathBuf::from() 创建一个拥有所有权的路径对象
        let wallpaper_dir = PathBuf::from("wallpapers");

        // .join() 在路径末尾追加子路径，返回新的 PathBuf（不修改原路径）
        let converted_dir = wallpaper_dir.join("converted");

        Self {
            api_key,
            wallpaper_dir,
            converted_dir,
        }
    }

    /// 确保壁纸目录存在
    /// 如果目录不存在则递归创建（包括父目录）
    ///
    /// # 返回值
    /// - `Ok(())`：目录已存在或创建成功
    /// - `Err(io::Error)`：创建目录失败（如权限不足）
    ///
    /// # Rust 特性说明
    /// - `std::fs::create_dir_all` 类似 `mkdir -p`，递归创建所有缺失的父目录
    /// - `?` 操作符：如果 Result 是 Err，立即返回该错误；如果是 Ok，解包内部值
    pub fn ensure_dirs(&self) -> std::io::Result<()> {
        // create_dir_all 递归创建目录，如果已存在则不报错
        // &self.wallpaper_dir 是对 PathBuf 的不可变借用（Borrow），不转移所有权
        std::fs::create_dir_all(&self.wallpaper_dir)?;
        std::fs::create_dir_all(&self.converted_dir)?;
        Ok(())
    }
}
