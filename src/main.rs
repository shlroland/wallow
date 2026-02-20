// main.rs — 程序入口
// 负责初始化异步运行时、解析命令行参数、分发子命令

mod cli;       // 声明 cli 模块，对应 src/cli.rs
mod config;    // 声明 config 模块，对应 src/config.rs
mod gowall;    // 声明 gowall 模块，对应 src/gowall.rs
mod wallhaven; // 声明 wallhaven 模块，对应 src/wallhaven.rs
mod source;

use clap::Parser;                    // 引入 Parser trait 的 parse() 方法
use cli::{Cli, Commands};            // 引入 CLI 结构体和子命令枚举
use config::AppConfig;               // 引入应用配置
use wallhaven::WallhavenClient;      // 引入 Wallhaven API 客户端
use source::{SearchOptions, WallpaperSource};

/// `#[tokio::main]` 宏将 async main 转换为同步 main + tokio 运行时
/// 等价于：
/// ```rust
/// fn main() {
///     tokio::runtime::Runtime::new().unwrap().block_on(async { ... })
/// }
/// ```
/// 这是 tokio 异步运行时的标准入口写法
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 解析命令行参数
    // Cli::parse() 由 #[derive(Parser)] 自动生成
    // 如果参数不合法，clap 会自动打印错误信息并退出
    let cli = Cli::parse();

    // 创建应用配置（读取环境变量、设置路径）
    let config = AppConfig::new();

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
            handle_fetch(&config, query.as_deref(), resolution, categories, purity, sorting, *count)
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
            handle_run(&config, query.as_deref(), theme, resolution, categories, purity, sorting)
                .await?;
        }
    }

    Ok(())
}

/// 处理 fetch 子命令：搜索并下载壁纸
///
/// # 参数
/// - `config`: 应用配置的不可变借用
/// - `query`: 搜索关键词，`Option<&str>` 是 `Option<String>` 的借用形式
/// - `resolution` ~ `sorting`: 搜索参数，都是 `&str`（字符串借用）
/// - `count`: 下载数量，`usize` 是无符号整数（Copy 类型，按值传递）
///
/// # 异步说明
/// - `async fn` 返回 Future，需要 `.await` 驱动执行
/// - 函数内部的 `.await` 点是异步挂起点，让出线程给其他任务
async fn handle_fetch(
    config: &AppConfig,
    query: Option<&str>,
    resolution: &str,
    categories: &str,
    purity: &str,
    sorting: &str,
    count: usize,
) -> Result<(), Box<dyn std::error::Error>> {
    // 创建 Wallhaven 客户端
    // config.api_key.clone() 深拷贝 Option<String>
    // 因为 WallhavenClient::new() 需要获取 api_key 的所有权
    let client = WallhavenClient::new(config.api_key.clone());

    println!("正在搜索壁纸...");

    let options = SearchOptions {
        query,
        resolution,
        categories,
        purity,
        sorting,
    };

    let wallpapers = client.search(options).await?;

    if wallpapers.is_empty() {
        println!("未找到符合条件的壁纸。");
        return Ok(());
    }

    let selected = wallpapers.iter().take(count);

    for (i, wallpaper) in selected.enumerate() {
        println!(
            "[{}/{}] 正在下载: {} ({})",
            i + 1,
            count.min(wallpapers.len()),
            wallpaper.id,
            wallpaper.resolution
        );

        let save_path = client.download(wallpaper, &config.wallpaper_dir).await?;

        println!("已保存: {}", save_path.display());
    }

    println!("下载完成！共 {} 张壁纸。", count.min(wallpapers.len()));
    Ok(())
}

/// 处理 convert 子命令：调用 gowall 转换壁纸主题
///
/// 这个函数不是 async 的，因为 gowall 通过 std::process::Command 同步调用
fn handle_convert(
    config: &AppConfig,
    image: &str,
    theme: &str,
    output: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("正在转换壁纸主题: {} -> {}", image, theme);

    // 确定输出路径
    // 如果用户指定了 --output，使用用户指定的路径
    // 否则使用默认的 converted 目录
    // .map() 对 Some 值应用闭包，None 保持不变
    // .unwrap_or_else() 在 None 时执行闭包生成默认值（惰性求值）
    let output_path = output
        .map(|o| o.to_string())
        .unwrap_or_else(|| config.converted_dir.display().to_string());

    // 调用 gowall convert
    // Some(output_path.as_str()) 将 &str 包装为 Option
    // gowall::convert 的第三个参数是 Option<impl AsRef<Path>>
    gowall::convert(image, theme, Some(output_path.as_str()))?;

    println!("转换完成！输出目录: {}", output_path);
    Ok(())
}

/// 处理 themes 子命令：列出所有可用主题
fn handle_themes() -> Result<(), Box<dyn std::error::Error>> {
    let themes = gowall::list_themes()?;

    println!("可用的 gowall 主题 ({} 个):", themes.len());
    println!("{}", "-".repeat(30));

    // .iter() 创建不可变引用的迭代器
    // |theme| 闭包参数，类型为 &&String（迭代器产生 &String，再被 for 借用）
    for theme in themes.iter() {
        println!("  {}", theme);
    }

    Ok(())
}

/// 处理 run 子命令：一键下载 + 转换
///
/// 组合 handle_fetch 和 handle_convert 的逻辑
/// 先下载一张壁纸，然后立即应用指定主题
async fn handle_run(
    config: &AppConfig,
    query: Option<&str>,
    theme: &str,
    resolution: &str,
    categories: &str,
    purity: &str,
    sorting: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = WallhavenClient::new(config.api_key.clone());

    println!("正在搜索壁纸...");

    let options = SearchOptions {
        query,
        resolution,
        categories,
        purity,
        sorting,
    };

    let wallpapers = client.search(options).await?;

    let wallpaper = wallpapers.first().ok_or("未找到符合条件的壁纸")?;

    println!("找到壁纸: {} ({})", wallpaper.id, wallpaper.resolution);

    println!("正在下载...");
    let save_path = client.download(wallpaper, &config.wallpaper_dir).await?;
    println!("已保存: {}", save_path.display());

    // 转换主题
    println!("正在应用主题: {}...", theme);

    // save_path.to_str() 将 PathBuf 转为 Option<&str>
    // .ok_or()? 在路径包含非 UTF-8 字符时返回错误
    let image_str = save_path
        .to_str()
        .ok_or("路径包含非 UTF-8 字符")?;

    let output_dir = config.converted_dir.display().to_string();
    gowall::convert(image_str, theme, Some(output_dir.as_str()))?;

    println!("全部完成！壁纸已下载并应用 {} 主题。", theme);
    Ok(())
}
