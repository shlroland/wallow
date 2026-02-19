// wallhaven.rs — Wallhaven API 异步客户端模块
// 负责与 Wallhaven API 交互：搜索壁纸和下载图片

use crate::source::{SearchOptions, WallpaperInfo, WallpaperSource};
use async_trait::async_trait;
use serde::Deserialize; // 反序列化 trait，用于将 JSON 转为 Rust 结构体
use std::path::{Path, PathBuf}; // 路径的不可变借用类型（Borrowed），用于函数参数
use tokio::fs::File; // tokio 提供的异步文件操作
use tokio::io::AsyncWriteExt; // 异步写入 trait，提供 write_all() 等方法

/// Wallhaven API 搜索响应的顶层结构
///
/// # serde 说明
/// - `#[derive(Deserialize)]` 自动实现 JSON -> Rust 结构体的反序列化
/// - `Debug` trait 允许使用 `{:?}` 格式化输出，方便调试
/// - 字段名必须与 JSON 的 key 完全匹配（或使用 `#[serde(rename)]`）
#[derive(Deserialize, Debug)]
pub struct SearchResponse {
    /// 搜索结果列表
    /// Wallhaven API 每页最多返回 24 条结果
    pub data: Vec<Wallpaper>,
}

/// 单张壁纸的数据结构
///
/// 只提取我们需要的字段，JSON 中多余的字段会被 serde 自动忽略
/// （这是 serde 的默认行为，无需额外配置）
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
///
/// # Rust 特性说明
/// - `reqwest::Client` 内部维护连接池，应该复用而非每次请求都创建新的
/// - `Option<String>` 用于可选的 API Key
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
        let filename = info.url.rsplit('/').next().unwrap_or("wallpaper.jpg");

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
    ///
    /// # 参数
    /// - `api_key`: 可选的 API Key，`Option<String>` 类型
    ///   传入 `Some("key".to_string())` 或 `None`
    ///
    /// # Rust 特性说明
    /// - `reqwest::Client::new()` 创建带默认配置的 HTTP 客户端
    /// - `String::from()` 从字符串字面量（`&str`）创建拥有所有权的 `String`
    /// - `Self` 是当前类型 `WallhavenClient` 的别名
    pub fn new(api_key: Option<String>) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: String::from("https://wallhaven.cc/api/v1"),
            api_key,
        }
    }

    /// 搜索壁纸
    ///
    /// 向 Wallhaven API 发送搜索请求，返回匹配的壁纸列表。
    ///
    /// # 参数
    /// - `query`: 搜索关键词（可选），`Option<&str>` 是对 `Option<String>` 的借用形式
    /// - `resolution`: 分辨率筛选（如 "3840x2160"），`&str` 是字符串的不可变借用
    /// - `categories`: 分类开关（如 "111"）
    /// - `purity`: 纯净度开关（如 "100"）
    /// - `sorting`: 排序方式（如 "random"）
    ///
    /// # 返回值
    /// - `Result<Vec<Wallpaper>, Box<dyn std::error::Error>>`
    ///   成功返回壁纸列表，失败返回动态错误类型
    #[allow(dead_code)]
    pub async fn search_raw(
        &self,
        query: Option<&str>,
        resolution: &str,
        categories: &str,
        purity: &str,
        sorting: &str,
    ) -> Result<Vec<Wallpaper>, Box<dyn std::error::Error>> {
        // 构建请求 URL
        // format!() 宏类似 println!()，但返回 String 而非打印到控制台
        let url = format!("{}/search", self.base_url);

        // 构建查询参数列表
        // Vec<(&str, &str)> 是键值对的动态数组
        // &str 是字符串切片（借用），不拥有数据的所有权
        let mut params: Vec<(&str, &str)> = vec![
            ("resolutions", resolution),
            ("categories", categories),
            ("purity", purity),
            ("sorting", sorting),
        ];

        // 如果提供了搜索关键词，添加到参数列表
        // if let 是模式匹配的简写形式，只处理 Some 的情况
        if let Some(q) = query {
            params.push(("q", q));
        }

        // 如果有 API Key，添加到参数列表
        // .as_deref() 将 Option<String> 转为 Option<&str>
        // 这是因为 String -> &str 的转换（解引用强制转换，Deref Coercion）
        if let Some(key) = self.api_key.as_deref() {
            params.push(("apikey", key));
        }

        // 发送 GET 请求
        // self.client.get(&url) 创建请求构建器，&url 借用 url 的值
        // .query(&params) 将参数编码为 URL 查询字符串（如 ?key=value&...）
        // .send().await 发送请求并异步等待响应
        // ? 操作符在遇到 Err 时提前返回错误
        let response = self
            .client
            .get(&url)
            .query(&params)
            .send()
            .await?;

        // 将响应体解析为 JSON
        // .json::<SearchResponse>() 使用 turbofish 语法指定目标类型
        // serde 会自动将 JSON 反序列化为 SearchResponse 结构体
        let search_response: SearchResponse = response.json().await?;

        // 返回壁纸列表
        // search_response.data 的所有权从 SearchResponse 移动（Move）到返回值
        Ok(search_response.data)
    }

    /// 下载单张壁纸到指定目录
    #[allow(dead_code)]
    pub async fn download_raw(
        &self,
        wallpaper: &Wallpaper,
        save_dir: impl AsRef<Path>,
    ) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
        // 从 URL 中提取文件名
        // .rsplit('/') 从右向左按 '/' 分割字符串，返回迭代器
        // .next() 取第一个元素（即最后一段路径），返回 Option<&str>
        // .unwrap_or("wallpaper.jpg") 如果是 None 则使用默认文件名
        let filename = wallpaper
            .path
            .rsplit('/')
            .next()
            .unwrap_or("wallpaper.jpg");

        // 构建完整的保存路径
        // save_dir.as_ref() 将泛型参数转为 &Path
        // .join(filename) 拼接目录和文件名，返回新的 PathBuf
        let save_path = save_dir.as_ref().join(filename);

        // 发送 GET 请求下载图片
        // .bytes().await 将整个响应体读取为字节数组（Bytes 类型）
        let response = self.client.get(&wallpaper.path).send().await?;
        let bytes = response.bytes().await?;

        // 异步创建文件并写入数据
        // File::create() 是 tokio 的异步版本，如果文件已存在会覆盖
        // write_all(&bytes) 将所有字节写入文件
        // &bytes 借用字节数据，避免不必要的复制
        let mut file = File::create(&save_path).await?;
        file.write_all(&bytes).await?;

        // 返回保存路径的所有权
        Ok(save_path)
    }
}
