// cli.rs — 命令行接口定义模块
// 使用 clap 的 derive 模式定义所有子命令和参数

use clap::{Parser, Subcommand}; // Parser: 解析命令行参数的 trait; Subcommand: 定义子命令的 trait
use clap_complete::Shell; // Shell 枚举：Bash, Zsh, Fish, Elvish, PowerShell

/// 壁纸下载与主题转换工具
///
/// 从 Wallhaven 下载壁纸，使用 gowall 应用配色主题，
/// 生成适合终端软件的背景图片。
#[derive(Parser)]
#[command(name = "wallow")]
#[command(version)] // 自动从 Cargo.toml 读取 version 字段
#[command(author)] // 自动从 Cargo.toml 读取 authors 字段（如有）
#[command(about = "壁纸下载与主题转换工具 — 从 Wallhaven 获取壁纸，用 gowall 应用配色主题")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// 从 Wallhaven 搜索并下载壁纸
    ///
    /// 用法示例:
    ///   wallow fetch --query nature
    ///   wallow fetch -q anime -n 5
    ///   wallow fetch --resolution 1920x1080 --purity 110
    Fetch {
        /// 搜索关键词（如 "nature", "anime", "landscape"）
        #[arg(short, long)]
        query: Option<String>,

        /// 壁纸分辨率
        #[arg(short, long)]
        resolution: Option<String>,

        /// 壁纸分类开关 (general/anime/people)，如 "111"=全部, "100"=仅general
        #[arg(short, long)]
        categories: Option<String>,

        /// 内容纯净度开关 (sfw/sketchy/nsfw)，如 "100"=仅SFW
        #[arg(short, long)]
        purity: Option<String>,

        /// 排序方式 (date_added/relevance/random/views/favorites/toplist)
        #[arg(short, long)]
        sorting: Option<String>,

        /// 下载数量
        #[arg(short = 'n', long, default_value = "1", value_name = "N")]
        count: usize,
    },

    /// 使用 gowall 对壁纸应用配色主题
    ///
    /// 用法示例:
    ///   wallow convert image.jpg --theme catppuccin
    ///   wallow convert wallow-xxx.jpg -t dracula
    Convert {
        /// 要转换的图片路径
        image: String,

        /// 目标主题名称（使用 `wallow themes` 查看可用主题）
        #[arg(short, long)]
        theme: String,

        /// 输出路径（不指定则保存到 wallpapers/converted/）
        #[arg(short, long)]
        output: Option<String>,
    },

    /// 列出所有可用的 gowall 主题
    ///
    /// 用法示例:
    ///   wallow themes
    Themes,

    /// 生成 shell 补全脚本（支持 bash, zsh, fish, elvish, powershell）
    ///
    /// 用法示例：
    ///   wallow completions zsh > ~/.zsh/completions/_wallow
    ///   wallow completions fish > ~/.config/fish/completions/wallow.fish
    Completions {
        /// 目标 shell 类型
        shell: Shell,
    },

    /// 定时任务：根据配置自动下载一张随机壁纸
    ///
    /// 用法示例:
    ///   wallow schedule
    Schedule,

    /// 一键更换：下载、转换并设置为系统壁纸
    ///
    /// 用法示例:
    ///   wallow set --query nature --theme catppuccin
    Set {
        /// 搜索关键词
        #[arg(short, long)]
        query: Option<String>,

        /// 目标主题名称（若不指定则使用原图）
        #[arg(short, long)]
        theme: Option<String>,
    },

    /// 一键完成：下载壁纸 + 应用主题
    ///
    /// 用法示例:
    ///   wallow run -q nature -t catppuccin
    ///   wallow run --query "cyberpunk" --theme dracula
    Run {
        /// 搜索关键词
        #[arg(short, long)]
        query: Option<String>,

        /// 目标主题名称
        #[arg(short, long)]
        theme: String,

        /// 壁纸分辨率
        #[arg(short, long)]
        resolution: Option<String>,

        /// 壁纸分类开关
        #[arg(short, long)]
        categories: Option<String>,

        /// 内容纯净度开关
        #[arg(short, long)]
        purity: Option<String>,

        /// 排序方式
        #[arg(short, long)]
        sorting: Option<String>,
    },

    /// 配置管理操作
    ///
    /// 用法示例:
    ///   wallow config show
    ///   wallow config dump
    ///   wallow config set query "anime"
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// 列出已下载的壁纸图片
    ///
    /// 用法示例:
    ///   wallow list
    ///   wallow list --fzf
    List {
        /// 使用 fzf 进行交互式选择与预览
        #[arg(short = 'F', long)]
        fzf: bool,
    },

    /// 将本地指定的图片设置为系统壁纸
    ///
    /// 用法示例:
    ///   wallow apply image.jpg
    Apply {
        /// 图片的本地路径
        image: String,
    },

    /// 清理所有带有 wallow- 前缀的下载文件
    ///
    /// 用法示例:
    ///   wallow clean
    Clean,
}

/// 配置管理操作
#[derive(Subcommand)]
pub enum ConfigAction {
    /// 查看当前所有配置简报
    Show,
    /// 生成配置文件对应的 JSON Schema
    Schema,
    /// 以 TOML 格式打印当前完整配置内容
    Dump,
    /// 设置配置项的值项 (支持: query, resolution, sorting)
    Set {
        /// 要设置的键 (query, res, sorting)
        key: String,
        /// 要设置的值
        value: String,
    },
}
