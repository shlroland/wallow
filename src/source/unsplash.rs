// unsplash.rs — Unsplash API 异步客户端模块
// 负责与 Unsplash API 交互：搜索壁纸和下载图片
//
// 注意：根据 Unsplash API Guidelines，每次实际下载图片前
// 必须先调用 links.download_location 触发下载统计。

use super::{SearchOptions, WallpaperInfo, WallpaperSource};
use async_trait::async_trait;
use serde::Deserialize; // 反序列化 trait，用于将 JSON 转为 Rust 结构体
use std::path::{Path, PathBuf};
use tokio::fs::File; // tokio 提供的异步文件操作
use tokio::io::AsyncWriteExt; // 异步写入 trait，提供 write_all() 等方法

/// Unsplash 搜索响应的顶层结构
/// GET /search/photos 返回的 JSON 根对象
#[derive(Deserialize, Debug)]
pub struct SearchResponse {
    /// 搜索结果列表
    pub results: Vec<Photo>,
}

/// 单张图片的数据结构
#[derive(Deserialize, Debug)]
pub struct Photo {
    /// 图片唯一标识符（如 "LBI7cgq3pbM"）
    pub id: String,

    /// 图片宽度（像素）
    pub width: u32,

    /// 图片高度（像素）
    pub height: u32,

    /// 各尺寸图片 URL 集合
    pub urls: PhotoUrls,

    /// 图片相关链接，包含触发下载统计所需的 download_location
    pub links: PhotoLinks,
}

/// 图片 URL 集合
/// Unsplash 提供多种尺寸，raw 为原始无损图片
#[derive(Deserialize, Debug)]
pub struct PhotoUrls {
    /// 原始图片 URL，不带任何处理参数
    /// 可追加 &w=3840&h=2160&fit=crop 等 Imgix 参数自定义尺寸
    pub raw: String,

    /// 最高质量图片（带 q=80&fm=jpg）
    #[allow(dead_code)]
    pub full: String,
}

/// 图片链接集合
#[derive(Deserialize, Debug)]
pub struct PhotoLinks {
    /// 触发下载统计的 API 地址（必须在下载前调用）
    /// 根据 Unsplash API Guidelines，这是强制要求
    pub download_location: String,
}

/// 触发下载统计后返回的响应结构
#[derive(Deserialize, Debug)]
struct DownloadResponse {
    /// 实际可下载的图片 URL
    url: String,
}

/// Unsplash API 异步客户端
///
/// 封装了 reqwest::Client 和 API 配置，提供搜索和下载方法。
/// Access Key 通过 `Authorization: Client-ID <key>` header 传递。
pub struct UnsplashClient {
    /// HTTP 客户端（内部有连接池，应复用）
    client: reqwest::Client,

    /// API 基础 URL
    base_url: String,

    /// Unsplash Access Key（必填，用于 Authorization header）
    access_key: String,
}

impl UnsplashClient {
    /// 创建新的 Unsplash 客户端
    ///
    /// # 参数
    /// - `access_key`: 从 Unsplash Developer 后台获取的 Access Key
    pub fn new(access_key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: String::from("https://api.unsplash.com"),
            access_key,
        }
    }

    /// 构建带 Authorization header 的请求
    /// Unsplash 使用 "Client-ID <key>" 格式，而非 Bearer token
    fn auth_header(&self) -> String {
        format!("Client-ID {}", self.access_key)
    }
}

#[async_trait]
impl WallpaperSource for UnsplashClient {
    async fn search(
        &self,
        options: SearchOptions<'_>,
    ) -> Result<Vec<WallpaperInfo>, Box<dyn std::error::Error>> {
        // Unsplash 搜索必须提供 query，若未提供则使用通用关键词
        let query = options.query.unwrap_or("wallpaper");

        // 解析分辨率字符串（如 "3840x2160"）用于构建 raw URL 参数
        // 格式不合法时静默降级，不中断搜索
        let (req_w, req_h) = parse_resolution(options.resolution);

        // 将 sorting 映射到 Unsplash 的 order_by 参数
        // Unsplash 只支持 relevant / latest，其他值降级为 relevant
        let order_by = match options.sorting {
            "latest" | "date_added" => "latest",
            _ => "relevant",
        };

        let url = format!("{}/search/photos", self.base_url);

        let response = self
            .client
            .get(&url)
            .header("Authorization", self.auth_header())
            // Unsplash 每页最多 30 条
            .query(&[
                ("query", query),
                ("per_page", "30"),
                ("order_by", order_by),
                ("orientation", "landscape"), // 壁纸场景优先横向
                ("content_filter", "low"),
            ])
            .send()
            .await?;

        let search_response: SearchResponse = response.json().await?;

        let info_list = search_response
            .results
            .into_iter()
            .map(|photo| {
                // 在 raw URL 后追加尺寸参数，获取目标分辨率图片
                // fit=crop 保证裁剪到精确尺寸，cs=srgb 保证色彩空间正确
                let download_url = if req_w > 0 && req_h > 0 {
                    format!(
                        "{}&w={}&h={}&fit=crop&cs=srgb&fm=jpg",
                        photo.urls.raw, req_w, req_h
                    )
                } else {
                    format!("{}&fm=jpg&q=85", photo.urls.raw)
                };

                WallpaperInfo {
                    id: photo.id,
                    url: download_url,
                    resolution: format!("{}x{}", photo.width, photo.height),
                    source: "unsplash".to_string(),
                    // 将 download_location 存入 extra，供 download() 调用统计接口
                    extra: Some(photo.links.download_location),
                }
            })
            .collect();

        Ok(info_list)
    }

    async fn download(
        &self,
        info: &WallpaperInfo,
        save_dir: &Path,
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        // 第一步：调用 download_location 触发 Unsplash 下载统计（API Guidelines 强制要求）
        // 同时获取带签名的真实下载 URL
        if let Some(download_location) = &info.extra {
            let dl_response: DownloadResponse = self
                .client
                .get(download_location)
                .header("Authorization", self.auth_header())
                .send()
                .await?
                .json()
                .await?;

            // 第二步：用统计接口返回的 URL 下载实际图片
            let bytes = self
                .client
                .get(&dl_response.url)
                .send()
                .await?
                .bytes()
                .await?;

            let filename = format!("wallow-unsplash-{}.jpg", info.id);
            let save_path = save_dir.join(filename);

            let mut file = File::create(&save_path).await?;
            file.write_all(&bytes).await?;

            Ok(save_path)
        } else {
            // 降级：直接用 url 字段下载（不触发统计）
            let bytes = self
                .client
                .get(&info.url)
                .send()
                .await?
                .bytes()
                .await?;

            let filename = format!("wallow-unsplash-{}.jpg", info.id);
            let save_path = save_dir.join(filename);

            let mut file = File::create(&save_path).await?;
            file.write_all(&bytes).await?;

            Ok(save_path)
        }
    }
}

/// 解析 "WxH" 格式的分辨率字符串，返回 (width, height)
/// 解析失败时返回 (0, 0)
fn parse_resolution(resolution: &str) -> (u32, u32) {
    let parts: Vec<&str> = resolution.splitn(2, 'x').collect();
    if parts.len() == 2 {
        let w = parts[0].parse::<u32>().unwrap_or(0);
        let h = parts[1].parse::<u32>().unwrap_or(0);
        (w, h)
    } else {
        (0, 0)
    }
}
