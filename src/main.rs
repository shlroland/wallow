// main.rs — 程序入口
// 负责初始化异步运行时、解析命令行参数、分发子命令

mod cli; // 声明 cli 模块，对应 src/cli.rs
mod config; // 声明 config 模块，对应 src/config.rs
mod gowall; // 声明 gowall 模块，对应 src/gowall.rs
mod setter;
mod source;
mod wallhaven; // 声明 wallhaven 模块，对应 src/wallhaven.rs

// 初始化多语言支持，嵌入 locales 目录下的所有翻译
rust_i18n::i18n!("locales");

use clap::{CommandFactory, Parser}; // 引入 Parser trait 的 parse() 方法; CommandFactory 用于生成补全脚本
use clap_complete::generate; // 引入补全脚本生成函数
use cli::{Cli, Commands}; // 引入 CLI 结构体和子命令枚举
use config::AppConfig; // 引入应用配置
use rust_i18n::t; // 引入翻译宏
use source::{SearchOptions, WallpaperSource};
use wallhaven::WallhavenClient; // 引入 Wallhaven API 客户端

/// `#[tokio::main]` 宏将 async main 转换为同步 main + tokio 运行时
/// 等价于：
/// ```rust
/// fn main() {
///     tokio::runtime::Runtime::new().unwrap().block_on(async { ... })
/// }
/// ```
/// 这是 tokio 异步运行时的 standard 入口写法
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 自动检测系统语言并设置
    // rust-i18n 默认会读取 LANG 环境变量，但我们可以显式设置以防万一
    let locale = std::env::var("LANG").unwrap_or_else(|_| "en".to_string());
    if locale.starts_with("zh") {
        rust_i18n::set_locale("zh-CN");
    } else {
        rust_i18n::set_locale("en");
    }

    // 解析命令行参数
    // Cli::parse() 由 #[derive(Parser)] 自动生成
    // 如果参数不合法，clap 会自动打印错误信息并退出
    let cli = Cli::parse();

    // 创建应用配置（读取环境变量、设置路径）
    let mut config = AppConfig::new();

    // 确保壁纸目录存在
    // ? 操作符：如果 ensure_dirs() 返回 Err，main 函数立即返回该错误
    config.ensure_dirs()?;

    // 根据子命令分发执行逻辑
    // match 是 Rust 的模式匹配，必须穷尽所有变体（exhaustive）
    // &cli.command 借用命令枚举，避免移动（Move）所有权
    match &cli.command {
        // 解构 Fetch 变体，提取所有字段
        // ref 关键字：借用字段值而非移动，因为外层已经是 & 借用
        Commands::Fetch {
            query,
            resolution,
            categories,
            purity,
            sorting,
            count,
        } => {
            handle_fetch(
                &config,
                query.as_deref(),
                resolution.as_deref(),
                categories.as_deref(),
                purity.as_deref(),
                sorting.as_deref(),
                *count,
            )
            .await?;
        }

        // 解构 Convert 变体
        Commands::Convert {
            image,
            theme,
            output,
        } => {
            gowall::check_installed()?;
            handle_convert(&config, image, theme, output.as_deref())?;
        }

        // Themes 变体没有字段，直接匹配
        Commands::Themes => {
            gowall::check_installed()?;
            handle_themes()?;
        }

        Commands::Schedule => {
            handle_schedule(&config).await?;
        }

        Commands::Completions { shell } => {
            generate(
                *shell,
                &mut Cli::command(),
                "wallow",
                &mut std::io::stdout(),
            );
        }

        // 解构 Run 变体（一键完成：下载 + 转换）
        Commands::Run {
            query,
            theme,
            resolution,
            categories,
            purity,
            sorting,
        } => {
            gowall::check_installed()?;
            handle_run(
                &config,
                query.as_deref(),
                Some(theme), // 将 &String 转为 Option<&str>
                resolution.as_deref(),
                categories.as_deref(),
                purity.as_deref(),
                sorting.as_deref(),
            )
            .await?;
        }

        Commands::Set { query, theme } => {
            // 1. 下载并转换（如果指定了主题）
            let image_path = handle_run(
                &config,
                query.as_deref(),
                theme.as_deref(),
                None, // 使用配置默认值
                None,
                None,
                None,
            )
            .await?;

            // 2. 设置壁纸
            println!("{}", t!("setting_wallpaper"));
            setter::set_from_path(&image_path)?;
            println!("{}", t!("set_done"));
        }
        Commands::Config { action } => {
            handle_config(&mut config, action)?;
        }
        Commands::Clean => {
            handle_clean(&config)?;
        }
    }

    Ok(())
}

/// 处理 clean 子命令：清理所有以 wallow- 开头的文件
fn handle_clean(config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    let dirs = vec![
        &config.wallpaper_dir,
        &config.converted_dir,
        &config.schedule_dir,
    ];

    let mut deleted_count = 0;

    for dir in dirs {
        if !dir.exists() {
            continue;
        }

        println!("{}", t!("cleaning_dir", path => dir.display()));

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    if filename.starts_with("wallow-") {
                        std::fs::remove_file(&path)?;
                        deleted_count += 1;
                        println!("  {} {}", t!("deleted"), filename);
                    }
                }
            }
        }
    }

    println!("{}", t!("clean_done", count => deleted_count));
    Ok(())
}

/// 处理 fetch 子命令：搜索并下载壁纸
async fn handle_fetch(
    config: &AppConfig,
    query: Option<&str>,
    resolution: Option<&str>,
    categories: Option<&str>,
    purity: Option<&str>,
    sorting: Option<&str>,
    count: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    // 创建 Wallhaven 客户端
    let client = WallhavenClient::new(config.api_key.clone());

    println!("{}", t!("search_start"));

    // 合并配置优先级：命令行参数 > 配置文件默认值
    let options = SearchOptions {
        query: query.or(config.search_defaults.query.as_deref()),
        resolution: resolution.unwrap_or(&config.search_defaults.resolution),
        categories: categories.unwrap_or(&config.search_defaults.categories),
        purity: purity.unwrap_or(&config.search_defaults.purity),
        sorting: sorting.unwrap_or(&config.search_defaults.sorting),
    };

    let wallpapers = client.search(options).await?;

    if wallpapers.is_empty() {
        println!("{}", t!("no_wallpapers"));
        return Ok(());
    }

    let selected = wallpapers.iter().take(count);
    let total = count.min(wallpapers.len());

    for (i, wallpaper) in selected.enumerate() {
        println!(
            "{}",
            t!(
                "download_info",
                current => i + 1,
                total => total,
                id => wallpaper.id,
                res => wallpaper.resolution
            )
        );

        let save_path = client.download(wallpaper, &config.wallpaper_dir).await?;

        println!("{}", t!("save_path", path => save_path.display()));
    }

    println!("{}", t!("download_done", count => total));
    Ok(())
}

/// 处理 convert 子命令：调用 gowall 转换壁纸主题
fn handle_convert(
    config: &AppConfig,
    image: &str,
    theme: &str,
    output: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", t!("convert_start", image => image, theme => theme));

    // 确定输出路径
    let output_path = output
        .map(|o| o.to_string())
        .unwrap_or_else(|| config.converted_dir.display().to_string());

    // 调用 gowall convert
    gowall::convert(image, theme, Some(output_path.as_str()))?;

    println!("{}", t!("convert_done", path => output_path));
    Ok(())
}

/// 处理 themes 子命令：列出所有可用主题
fn handle_themes() -> Result<(), Box<dyn std::error::Error>> {
    let themes = gowall::list_themes()?;

    println!("{}", t!("themes_title", count => themes.len()));
    println!("{}", "-".repeat(30));

    for theme in themes.iter() {
        println!("  {}", theme);
    }

    Ok(())
}

/// 处理 run 子命令：一键下载 + 转换
async fn handle_run(
    config: &AppConfig,
    query: Option<&str>,
    theme: Option<&str>,
    resolution: Option<&str>,
    categories: Option<&str>,
    purity: Option<&str>,
    sorting: Option<&str>,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let client = WallhavenClient::new(config.api_key.clone());

    println!("{}", t!("search_start"));

    let options = SearchOptions {
        query: query.or(config.search_defaults.query.as_deref()),
        resolution: resolution.unwrap_or(&config.search_defaults.resolution),
        categories: categories.unwrap_or(&config.search_defaults.categories),
        purity: purity.unwrap_or(&config.search_defaults.purity),
        sorting: sorting.unwrap_or(&config.search_defaults.sorting),
    };

    let wallpapers = client.search(options).await?;

    let wallpaper = wallpapers.first().ok_or(t!("error_no_wallpapers"))?;

    println!(
        "{}",
        t!(
            "download_info",
            current => 1,
            total => 1,
            id => wallpaper.id,
            res => wallpaper.resolution
        )
    );

    let save_path = client.download(wallpaper, &config.wallpaper_dir).await?;
    println!("{}", t!("save_path", path => save_path.display()));

    // 转换主题（如果指定了主题）
    if let Some(theme_name) = theme {
        println!(
            "{}",
            t!("convert_start", image => save_path.display(), theme => theme_name)
        );

        let image_str = save_path.to_str().ok_or(t!("error_utf8"))?;
        let output_dir = config.converted_dir.display().to_string();
        gowall::convert(image_str, theme_name, Some(output_dir.as_str()))?;

        // 转换后的路径逻辑
        let file_name = save_path.file_name().ok_or("无法获取文件名")?;
        let converted_path = config.converted_dir.join(file_name);

        println!("{}", t!("all_done", theme => theme_name));
        Ok(converted_path)
    } else {
        Ok(save_path)
    }
}

/// 处理 schedule 子命令：自动下载每日壁纸
async fn handle_schedule(config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    let client = WallhavenClient::new(config.api_key.clone());

    println!("{}", t!("search_start"));

    // 使用配置中的默认参数进行随机搜索
    let options = SearchOptions {
        query: config.search_defaults.query.as_deref(),
        resolution: &config.search_defaults.resolution,
        categories: &config.search_defaults.categories,
        purity: &config.search_defaults.purity,
        sorting: "random", // 定时任务强制使用随机以获得新鲜感
    };

    let wallpapers = client.search(options).await?;
    let wallpaper = wallpapers.first().ok_or(t!("error_no_wallpapers"))?;

    println!(
        "{}",
        t!(
            "download_info",
            current => 1,
            total => 1,
            id => wallpaper.id,
            res => wallpaper.resolution
        )
    );

    // 保存到指定的定时任务目录
    let save_path = client.download(wallpaper, &config.schedule_dir).await?;
    println!("{}", t!("save_path", path => save_path.display()));

    // 获取当前程序的绝对路径用于指引
    let bin_path = std::env::current_exe()?;
    let bin_str = bin_path.to_string_lossy();

    println!("{}", t!("schedule_tip", bin_path => bin_str));

    Ok(())
}

/// 处理 config 子命令：查看或修改配置
fn handle_config(
    config: &mut AppConfig,
    action: &cli::ConfigAction,
) -> Result<(), Box<dyn std::error::Error>> {
    match action {
        cli::ConfigAction::Show => {
            println!("{}", t!("config_title"));
            println!("{}", t!("config_path", path => config.config_path.display()));
            println!(
                "{}",
                t!("config_wallpaper_dir", path => config.wallpaper_dir.display())
            );
            println!("{}", t!("config_search_defaults"));
            let query_str = config.search_defaults.query.as_deref().unwrap_or("None");
            println!("{}", t!("config_query", query => query_str));
            println!(
                "{}",
                t!("config_res", res => config.search_defaults.resolution)
            );
            println!(
                "{}",
                t!("config_sorting", sorting => config.search_defaults.sorting)
            );
        }
        cli::ConfigAction::Schema => {
            println!("{}", AppConfig::get_schema());
        }
        cli::ConfigAction::Dump => {
            println!("{}", config.to_toml());
        }
        cli::ConfigAction::Set { key, value } => {
            match key.as_str() {
                "query" => config.search_defaults.query = Some(value.clone()),
                "res" | "resolution" => config.search_defaults.resolution = value.clone(),
                "sorting" => config.search_defaults.sorting = value.clone(),
                _ => return Err(t!("config_error_unknown_key", key => key).into()),
            }
            config.save()?;
            println!("{}", t!("config_updated", key => key, value => value));
        }
    }
    Ok(())
}
