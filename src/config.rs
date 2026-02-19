// config.rs — 配置管理模块
// 遵循 Unix 风格：优先从 ~/.config/wallow/config.toml 读取配置

use serde::Deserialize; // 引入反序列化 trait，用于解析 TOML
use std::env; // 环境变量模块
use std::fs; // 文件系统模块
use std::path::{Path, PathBuf}; // 路径处理类型

/// 映射 config.toml 文件内容的结构体
/// `#[derive(Deserialize)]` 允许 toml 库将字符串解析为此结构体
#[derive(Debug, Deserialize)]
struct ConfigFile {
    /// 对应 TOML 中的 api_key 字段
    api_key: Option<String>,
}

/// 应用全局配置项
pub struct AppConfig {
    /// Wallhaven API Key (优先级：ENV > TOML)
    pub api_key: Option<String>,
    /// 壁纸保存根目录
    pub wallpaper_dir: PathBuf,
    /// 转换后壁纸的保存目录
    pub converted_dir: PathBuf,
    /// 配置文件所在路径（用于提示用户）
    pub config_path: PathBuf,
}

impl AppConfig {
    /// 初始化配置
    ///
    /// # 逐行详解
    pub fn new() -> Self {
        // 1. 确定配置路径：遵循 Unix 风格的 ~/.config/wallow/config.toml
        // 获取 $HOME 环境变量。env::var 返回 Result，如果取不到则通过 expect 抛出 panic
        let home = env::var("HOME").expect("无法获取 $HOME 环境变量");

        // 使用 PathBuf::from 创建路径，并用 .join() 拼接子目录
        // 这样在 Unix 下会自动生成 ~/.config/wallow 路径
        let config_dir = PathBuf::from(home).join(".config").join("wallow");
        let config_path = config_dir.join("config.toml");

        // 2. 尝试从文件读取配置
        let config_file = Self::load_config_from_file(&config_path);

        // 3. 合并优先级：环境变量 > 配置文件内容
        // env::var("...").ok()：获取环境变量，并将 Result 转为 Option
        // .or(...)：如果前面是 None，则尝试取后面的配置内容
        // .and_then(|cf| cf.api_key)：如果配置对象存在，则取出其中的 api_key
        let api_key = env::var("WALLHAVEN_API_KEY")
            .ok()
            .or(config_file.and_then(|cf| cf.api_key));

        // 4. 设置壁纸存储路径
        let wallpaper_dir = PathBuf::from("wallpapers");
        let converted_dir = wallpaper_dir.join("converted");

        Self {
            api_key,
            wallpaper_dir,
            converted_dir,
            config_path,
        }
    }

    /// 辅助函数：解析 TOML 配置文件
    ///
    /// # 逐行详解
    fn load_config_from_file(path: &Path) -> Option<ConfigFile> {
        // fs::read_to_string(path)：尝试从给定路径读取整个文件为字符串
        // .ok()：如果读取失败（如文件不存在），忽略错误并返回 None
        fs::read_to_string(path).ok().and_then(|content| {
            // toml::from_str(&content)：将字符串解析为 ConfigFile 结构体
            // 这里的 &content 是对字符串内容的不可变借用
            toml::from_str(&content).ok()
        })
    }

    /// 确保所有必要的目录都存在
    ///
    /// # 逐行详解
    pub fn ensure_dirs(&self) -> std::io::Result<()> {
        // self.config_path.parent()：获取配置文件的父目录（即 ~/.config/wallow）
        if let Some(parent) = self.config_path.parent() {
            // fs::create_dir_all(parent)：递归创建目录（类似 mkdir -p）
            // ? 操作符：如果成功则继续，如果失败则返回 Err(io::Error)
            fs::create_dir_all(parent)?;
        }

        // 创建壁纸相关的存储目录
        fs::create_dir_all(&self.wallpaper_dir)?;
        fs::create_dir_all(&self.converted_dir)?;

        Ok(())
    }
}
