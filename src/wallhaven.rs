// wallhaven.rs — Wallhaven API 异步客户端模块
// 负责与 Wallhaven API 交互：搜索壁纸和下载图片

use crate::source::{SearchOptions, WallpaperInfo, WallpaperSource};
use async_trait::async_trait;
use serde::Deserialize; // 反序列化 trait，用于将 JSON 转为 Rust 结构体
use std::path::{Path, PathBuf}; // 路径的不可变借用类型（Borrowed），用于函数参数
use tokio::fs::File; // tokio 提供的异步文件操作
use tokio::io::AsyncWriteExt; // 异步写入 trait，提供 write_all() 等方法

/// Wallhaven API 搜索响应的顶层结构
#[derive(Deserialize, Debug)]
pub struct SearchResponse {
    /// 搜索结果列表
    /// Wallhaven API 每页最多返回 24 条结果
    pub data: Vec<Wallpaper>,
}

/// 单张壁纸的数据结构
#[derive(Deserialize, Debug)]
pub struct Wallpaper {
    /// 壁纸唯一标识符（如 "94x38z"）
    pub id: String,

    /// 壁纸原图的直接下载 URL
    /// 格式如：https://w.wallhaven.cc/full/94/wallhaven-94x38z.jpg
    pub path: String,

    /// 壁纸分辨率（如 "3840x2160"）
    pub resolution: String,
}

/// Wallhaven API 异步客户端
///
/// 封装了 reqwest::Client 和 API 配置，提供搜索和下载方法。
pub struct WallhavenClient {
    /// HTTP 客户端（内部有连接池，应复用）
    client: reqwest::Client,

    /// API 基础 URL
    base_url: String,

    /// 可选的 API Key
    api_key: Option<String>,
}

#[async_trait]
impl WallpaperSource for WallhavenClient {
    async fn search(
        &self,
        options: SearchOptions<'_>,
    ) -> Result<Vec<WallpaperInfo>, Box<dyn std::error::Error>> {
        let url = format!("{}/search", self.base_url);

        let mut params: Vec<(&str, &str)> = vec![
            ("resolutions", options.resolution),
            ("categories", options.categories),
            ("purity", options.purity),
            ("sorting", options.sorting),
        ];

        if let Some(q) = options.query {
            params.push(("q", q));
        }

        if let Some(key) = self.api_key.as_deref() {
            params.push(("apikey", key));
        }

        let response = self.client.get(&url).query(&params).send().await?;

        let search_response: SearchResponse = response.json().await?;

        let info_list = search_response
            .data
            .into_iter()
            .map(|w| WallpaperInfo {
                id: w.id,
                url: w.path,
                resolution: w.resolution,
                source: "wallhaven".to_string(),
            })
            .collect();

        Ok(info_list)
    }

    async fn download(
        &self,
        info: &WallpaperInfo,
        save_dir: &Path,
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        // 从 URL 中提取原始文件名
        let original_filename = info.url.rsplit('/').next().unwrap_or("wallpaper.jpg");
        // 为文件名添加 wallow- 前缀，方便后续统一清理
        let filename = format!("wallow-{}", original_filename);

        let save_path = save_dir.join(filename);

        let response = self.client.get(&info.url).send().await?;
        let bytes = response.bytes().await?;

        let mut file = File::create(&save_path).await?;
        file.write_all(&bytes).await?;

        Ok(save_path)
    }
}

impl WallhavenClient {
    /// 创建新的 Wallhaven 客户端
    pub fn new(api_key: Option<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: String::from("https://wallhaven.cc/api/v1"),
            api_key,
        }
    }

    /// 搜索壁纸 (Raw)
    #[allow(dead_code)]
    pub async fn search_raw(
        &self,
        query: Option<&str>,
        resolution: &str,
        categories: &str,
        purity: &str,
        sorting: &str,
    ) -> Result<Vec<Wallpaper>, Box<dyn std::error::Error>> {
        let url = format!("{}/search", self.base_url);

        let mut params: Vec<(&str, &str)> = vec![
            ("resolutions", resolution),
            ("categories", categories),
            ("purity", purity),
            ("sorting", sorting),
        ];

        if let Some(q) = query {
            params.push(("q", q));
        }

        if let Some(key) = self.api_key.as_deref() {
            params.push(("apikey", key));
        }

        let response = self.client.get(&url).query(&params).send().await?;
        let search_response: SearchResponse = response.json().await?;

        Ok(search_response.data)
    }

    /// 下载单张壁纸到指定目录 (Raw)
    #[allow(dead_code)]
    pub async fn download_raw(
        &self,
        wallpaper: &Wallpaper,
        save_dir: impl AsRef<Path>,
    ) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
        // 从 URL 中提取原始文件名
        let original_filename = wallpaper
            .path
            .rsplit('/')
            .next()
            .unwrap_or("wallpaper.jpg");
        
        // 为文件名添加 wallow- 前缀
        let filename = format!("wallow-{}", original_filename);

        // 构建完整的保存路径
        let save_path = save_dir.as_ref().join(filename);

        let response = self.client.get(&wallpaper.path).send().await?;
        let bytes = response.bytes().await?;

        let mut file = File::create(&save_path).await?;
        file.write_all(&bytes).await?;

        Ok(save_path)
    }
}
