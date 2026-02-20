// cli.rs — 命令行接口定义模块
// 使用 clap 的 derive 模式定义所有子命令和参数

use clap::{Parser, Subcommand}; // Parser: 解析命令行参数的 trait; Subcommand: 定义子命令的 trait
use clap_complete::Shell; // Shell 枚举：Bash, Zsh, Fish, Elvish, PowerShell

// `#[derive(Parser)]` 自动为结构体实现命令行解析逻辑
// `#[command(...)]` 属性宏用于配置命令的元信息
// `name` 设置二进制名称，`about` 设置简短描述
// `///` 文档注释会被 clap 自动用作 `--help` 的长描述
// 注意：面向用户的帮助文本用 `///`，面向开发者的教学注释用 `//`
//       否则教学内容会泄漏到 `--help` 输出中
/// 壁纸下载与主题转换工具
///
/// 从 Wallhaven 下载壁纸，使用 gowall 应用配色主题，
/// 生成适合终端软件的背景图片。
#[derive(Parser)]
#[command(name = "wallow")]
#[command(about = "壁纸下载与主题转换工具 — 从 Wallhaven 获取壁纸，用 gowall 应用配色主题")]
pub struct Cli {
    // `#[command(subcommand)]` 告诉 clap 这个字段是子命令枚举
    // 用户必须提供一个子命令，否则 clap 会自动显示帮助信息
    #[command(subcommand)]
    pub command: Commands,
}

// `#[derive(Subcommand)]` 让枚举的每个变体成为一个子命令
// 每个变体的 `///` 文档注释会成为该子命令的帮助描述
// 变体名自动转为 kebab-case（如 `Fetch` -> `fetch`）
#[derive(Subcommand)]
pub enum Commands {
    /// 从 Wallhaven 搜索并下载壁纸
    Fetch {
        // `#[arg(short, long)]` 同时支持 `-q` 短参数和 `--query` 长参数
        // `Option<String>` 表示该参数可选，不提供时为 None
        /// 搜索关键词（如 "nature", "anime", "landscape"）
        #[arg(short, long)]
        query: Option<String>,

        /// 壁纸分辨率
        #[arg(short, long)]
        resolution: Option<String>,

        // 每一位对应一个分类：第1位=general, 第2位=anime, 第3位=people
        /// 壁纸分类开关 (general/anime/people)，如 "111"=全部, "100"=仅general
        #[arg(short, long)]
        categories: Option<String>,

        // 访问 NSFW 内容需要设置 WALLHAVEN_API_KEY 环境变量
        /// 内容纯净度开关 (sfw/sketchy/nsfw)，如 "100"=仅SFW
        #[arg(short, long)]
        purity: Option<String>,

        /// 排序方式 (date_added/relevance/random/views/favorites/toplist)
        #[arg(short, long)]
        sorting: Option<String>,

        // `value_name` 自定义帮助信息中的参数占位符名称
        /// 下载数量
        #[arg(short = 'n', long, default_value = "1", value_name = "N")]
        count: usize,
    },

    /// 使用 gowall 对壁纸应用配色主题
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
    Themes,

    /// 生成 shell 补全脚本（支持 bash, zsh, fish, elvish, powershell）
    ///
    /// 用法示例：
    ///   wallow completions zsh > ~/.zsh/completions/_wallow
    ///   wallow completions fish > ~/.config/fish/completions/wallow.fish
    ///   wallow completions bash > ~/.local/share/bash-completion/completions/wallow
    Completions {
        /// 目标 shell 类型
        shell: Shell,
    },

    /// 定时任务：根据配置自动下载一张随机壁纸
    Schedule,

    /// 一键完成：下载壁纸 + 应用主题（如 `wallow run -q nature -t catppuccin`）
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
}
