// source/mod.rs — 壁纸源模块入口
pub mod unsplash;
pub mod wallhaven;

// source.rs — 壁纸源抽象接口模块
// 定义了所有壁纸站（如 Wallhaven）必须实现的通用 Trait

use std::path::{Path, PathBuf}; // 路径相关类型
use async_trait::async_trait;   // 异步 Trait 支持宏

/// 统一的壁纸元数据结构
/// 不论来自哪个壁纸站，都转换成这个结构体供上层使用
#[derive(Debug, Clone)]
pub struct WallpaperInfo {
    /// 壁纸在原站的 ID
    pub id: String,
    /// 壁纸原图的直接下载 URL
    pub url: String,
    /// 分辨率描述
    pub resolution: String,
    /// 来源站名称（如 "wallhaven"）
    #[allow(dead_code)]
    #[allow(dead_code)]
    pub source: String,
    /// 来源特定的附加数据（如 Unsplash 的 download_location）
    #[allow(dead_code)]
    pub extra: Option<String>,
}

/// 搜索参数结构体
/// 抽象了通用的搜索需求
pub struct SearchOptions<'a> {
    pub query: Option<&'a str>,
    pub resolution: &'a str,
    pub categories: &'a str,
    pub purity: &'a str,
    pub sorting: &'a str,
}

/// 壁纸源的抽象 Trait
/// 所有的壁纸站客户端（如 WallhavenClient）都应该实现这个 Trait
///
/// # 异步 Trait 说明
/// Rust 原生目前对 Trait 中的 async fn 支持有限，
/// 这里使用 `async_trait` 宏来支持异步接口。
#[async_trait]
pub trait WallpaperSource {
    /// 搜索壁纸
    /// 返回统一的 WallpaperInfo 列表
    async fn search(&self, options: SearchOptions<'_>) -> Result<Vec<WallpaperInfo>, Box<dyn std::error::Error>>;

    /// 下载壁纸
    /// 接收一个 WallpaperInfo 和保存目录，返回保存后的完整路径
    async fn download(&self, info: &WallpaperInfo, save_dir: &Path) -> Result<PathBuf, Box<dyn std::error::Error>>;
}
