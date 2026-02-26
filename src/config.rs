// config.rs — 配置管理模块
// 遵循 Unix 风格：优先从 ~/.config/wallow/config.toml 读取配置

use schemars::JsonSchema; // 引入用于生成 JSON Schema 的 trait
use serde::{Deserialize, Serialize}; // 引入序列化与反序列化 trait
use std::env; // 环境变量模块
use std::fs; // 文件系统模块
use std::path::{Path, PathBuf}; // 路径处理类型

/// 映射 config.toml 文件内容的嵌套结构体
#[derive(Debug, Deserialize, Serialize, Default, JsonSchema)]
struct ConfigFile {
    #[serde(default)]
    common: CommonConfig,
    #[serde(default)]
    source: SourceConfigs,
    #[serde(default)]
    schedule: ScheduleConfig,
}

#[derive(Debug, Deserialize, Serialize, Default, JsonSchema)]
struct CommonConfig {
    /// 壁纸保存根目录 (支持绝对路径，相对路径则相对于 $HOME)
    wallpaper_dir: Option<String>,
    /// 默认搜索参数
    #[serde(default)]
    search: SearchDefaults,
}

#[derive(Debug, Deserialize, Serialize, JsonSchema)]
pub struct SearchDefaults {
    /// 默认搜索关键词
    #[serde(default)]
    pub query: Option<String>,
    #[serde(default = "default_resolution")]
    pub resolution: String,
    #[serde(default = "default_categories")]
    pub categories: String,
    #[serde(default = "default_purity")]
    pub purity: String,
    #[serde(default = "default_sorting")]
    pub sorting: String,
}

impl Default for SearchDefaults {
    fn default() -> Self {
        Self {
            query: None,
            resolution: default_resolution(),
            categories: default_categories(),
            purity: default_purity(),
            sorting: default_sorting(),
        }
    }
}

fn default_resolution() -> String {
    "3840x2160".to_string()
}
fn default_categories() -> String {
    "111".to_string()
}
fn default_purity() -> String {
    "100".to_string()
}
fn default_sorting() -> String {
    "random".to_string()
}

#[derive(Debug, Deserialize, Serialize, Default, JsonSchema)]
struct SourceConfigs {
    #[serde(default)]
    wallhaven: WallhavenConfig,
}

#[derive(Debug, Deserialize, Serialize, Default, JsonSchema)]
struct WallhavenConfig {
    api_key: Option<String>,
}

/// 定时任务配置
#[derive(Debug, Deserialize, Serialize, Default, JsonSchema)]
pub struct ScheduleConfig {
    /// Cron 表达式，定义定时执行频率 (例: "0 8 * * *" 表示每天 8:00)
    #[serde(default)]
    pub cron: Option<String>,
}

/// 应用全局配置项
pub struct AppConfig {
    /// Wallhaven API Key (优先级：ENV > TOML)
    pub api_key: Option<String>,
    /// 壁纸保存根目录
    pub wallpaper_dir: PathBuf,
    /// 转换后壁纸的保存目录
    pub converted_dir: PathBuf,
    /// 配置文件所在路径
    pub config_path: PathBuf,
    /// 默认搜索参数
    pub search_defaults: SearchDefaults,
    /// 定时任务配置 (cron 表达式)
    pub schedule: ScheduleConfig,
}

impl AppConfig {
    /// 初始化配置
    pub fn new() -> Self {
        let home = env::var("HOME").expect("无法获取 $HOME 环境变量");
        let home_path = PathBuf::from(&home);
        let config_dir = home_path.join(".config").join("wallow");
        let config_path = config_dir.join("config.toml");

        let config_file = Self::load_config_from_file(&config_path).unwrap_or_default();

        // 优先级：环境变量 > 配置文件内容
        let api_key = env::var("WALLHAVEN_API_KEY")
            .ok()
            .or(config_file.source.wallhaven.api_key);

        // 壁纸目录：
        // 1. 如果配置了路径：如果是相对路径则相对于 $HOME，否则使用绝对路径
        // 2. 如果未配置：默认使用 $HOME/Pictures/wallow
        let wallpaper_dir = if let Some(dir_str) = config_file.common.wallpaper_dir {
            let p = PathBuf::from(dir_str);
            if p.is_absolute() {
                p
            } else {
                home_path.join(p)
            }
        } else {
            home_path.join("Pictures").join("wallow")
        };

        let converted_dir = wallpaper_dir.join("converted");

        Self {
            api_key,
            wallpaper_dir,
            converted_dir,
            config_path,
            search_defaults: config_file.common.search,
            schedule: config_file.schedule,
        }
    }

    /// 辅助函数：解析 TOML 配置文件
    fn load_config_from_file(path: &Path) -> Option<ConfigFile> {
        fs::read_to_string(path)
            .ok()
            .and_then(|content| toml::from_str(&content).ok())
    }

    /// 确保所有必要的目录都存在
    pub fn ensure_dirs(&self) -> std::io::Result<()> {
        if let Some(parent) = self.config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::create_dir_all(&self.wallpaper_dir)?;
        fs::create_dir_all(&self.converted_dir)?;

        Ok(())
    }

    /// 将配置保存回文件
    pub fn save(&self) -> std::io::Result<()> {
        let config_file = ConfigFile {
            common: CommonConfig {
                wallpaper_dir: Some(self.wallpaper_dir.to_string_lossy().to_string()),
                search: SearchDefaults {
                    query: self.search_defaults.query.clone(),
                    resolution: self.search_defaults.resolution.clone(),
                    categories: self.search_defaults.categories.clone(),
                    purity: self.search_defaults.purity.clone(),
                    sorting: self.search_defaults.sorting.clone(),
                },
            },
            source: SourceConfigs {
                wallhaven: WallhavenConfig {
                    api_key: self.api_key.clone(),
                },
            },
            schedule: ScheduleConfig {
                cron: self.schedule.cron.clone(),
            },
        };

        let toml_str = toml::to_string_pretty(&config_file)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        fs::write(&self.config_path, toml_str)
    }
    /// 更新 schedule.cron 并保存到配置文件
    pub fn set_cron(&mut self, cron: String) -> std::io::Result<()> {
        self.schedule.cron = Some(cron);
        self.save()
    }

    /// 获取配置文件的 JSON Schema
    pub fn get_schema() -> String {
        let schema = schemars::schema_for!(ConfigFile);
        serde_json::to_string_pretty(&schema).unwrap()
    }

    /// 将当前配置转换为 TOML 字符串
    pub fn to_toml(&self) -> String {
        let config_file = ConfigFile {
            common: CommonConfig {
                wallpaper_dir: Some(self.wallpaper_dir.to_string_lossy().to_string()),
                search: SearchDefaults {
                    query: self.search_defaults.query.clone(),
                    resolution: self.search_defaults.resolution.clone(),
                    categories: self.search_defaults.categories.clone(),
                    purity: self.search_defaults.purity.clone(),
                    sorting: self.search_defaults.sorting.clone(),
                },
            },
            source: SourceConfigs {
                wallhaven: WallhavenConfig {
                    api_key: self.api_key.clone(),
                },
            },
            schedule: ScheduleConfig {
                cron: self.schedule.cron.clone(),
            },
        };

        toml::to_string_pretty(&config_file)
            .unwrap_or_else(|_| "# Error serializing config".to_string())
    }
}
