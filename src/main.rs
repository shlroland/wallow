// main.rs — 程序入口
// 负责初始化异步运行时、解析命令行参数、分发子命令

extern crate libc;

mod cli; // 声明 cli 模块，对应 src/cli.rs
mod config; // 声明 config 模块，对应 src/config.rs
mod gowall; // 声明 gowall 模块，对应 src/gowall.rs
mod setter;
mod source;

// 初始化多语言支持，嵌入 locales 目录下的所有翻译
rust_i18n::i18n!("locales");

use clap::{CommandFactory, Parser}; // 引入 Parser trait 的 parse() 方法; CommandFactory 用于生成补全脚本
use clap_complete::generate; // 引入补全脚本生成函数
use cli::{Cli, Commands}; // 引入 CLI 结构体和子命令枚举
use config::AppConfig; // 引入应用配置
use rust_i18n::t; // 引入翻译宏
use source::{SearchOptions, WallpaperSource};
use source::wallhaven::WallhavenClient;
use source::unsplash::UnsplashClient; // 引入 Unsplash API 客户端

/// `#[tokio::main]` 宏将 async main 转换为同步 main + tokio 运行时
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 自动检测系统语言并设置
    let locale = std::env::var("LANG").unwrap_or_else(|_| "en".to_string());
    if locale.starts_with("zh") {
        rust_i18n::set_locale("zh-CN");
    } else {
        rust_i18n::set_locale("en");
    }

    // 解析命令行参数
    let cli = Cli::parse();

    // 创建应用配置（读取环境变量、设置路径）
    let mut config = AppConfig::new();

    // 确保壁纸目录存在
    config.ensure_dirs()?;

    // 根据子命令分发执行逻辑
    match &cli.command {
        Commands::Fetch {
            query,
            resolution,
            categories,
            purity,
            sorting,
            count,
            source,
        } => {
            handle_fetch(
                &config,
                query.as_deref(),
                resolution.as_deref(),
                categories.as_deref(),
                purity.as_deref(),
                sorting.as_deref(),
                *count,
                source.as_deref().unwrap_or(&config.default_source),
            )
            .await?;
        }

        Commands::Convert {
            image,
            theme,
            output,
        } => {
            gowall::check_installed()?;
            handle_convert(&config, image, theme, output.as_deref())?;
        }

        Commands::Themes => {
            gowall::check_installed()?;
            handle_themes()?;
        }

        Commands::Schedule { cron } => {
            handle_schedule(&mut config, cron.as_deref()).await?;
        }

        Commands::Completions { shell } => {
            generate(
                *shell,
                &mut Cli::command(),
                "wallow",
                &mut std::io::stdout(),
            );
        }

        Commands::Run {
            query,
            theme,
            resolution,
            categories,
            purity,
            sorting,
            source,
        } => {
            gowall::check_installed()?;
            handle_run(
                &config,
                query.as_deref(),
                Some(theme),
                resolution.as_deref(),
                categories.as_deref(),
                purity.as_deref(),
                sorting.as_deref(),
                source.as_deref().unwrap_or(&config.default_source),
            )
            .await?;
        }

        Commands::Set { query, theme, source } => {
            let image_path = handle_run(
                &config,
                query.as_deref(),
                theme.as_deref(),
                None,
                None,
                None,
                None,
                source.as_deref().unwrap_or(&config.default_source),
            )
            .await?;

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
        Commands::Upgrade => {
            handle_upgrade().await?;
        }
        Commands::Uninstall { keep_wallpapers } => {
            handle_uninstall(&config, *keep_wallpapers)?;
        }
        Commands::List { fzf } => {
            handle_list(&config, *fzf)?;
        }
        Commands::Apply { image } => {
            handle_apply(image)?;
        }
    }

    Ok(())
}
/// 处理 list 子命令：列出已下载的壁纸，可选 fzf 交互预览
fn handle_list(config: &AppConfig, use_fzf: bool) -> Result<(), Box<dyn std::error::Error>> {
    // 收集壁纸目录和转换目录中的所有图片文件
    let mut images: Vec<std::path::PathBuf> = Vec::new();
    for dir in [&config.wallpaper_dir, &config.converted_dir] {
        if dir.exists() {
            for entry in std::fs::read_dir(dir)? {
                let path = entry?.path();
                if path.is_file() {
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        if matches!(ext.to_lowercase().as_str(), "jpg" | "jpeg" | "png" | "webp") {
                            images.push(path);
                        }
                    }
                }
            }
        }
    }
    if images.is_empty() {
        println!("{}", t!("no_wallpapers"));
        return Ok(());
    }
    if !use_fzf {
        // 普通列表模式：直接打印路径
        for path in &images {
            println!("{}", path.display());
        }
        return Ok(());
    }
    // fzf 交互模式
    // 前置检查依赖
    if !which_exists("fzf") {
        return Err(t!("fzf_error").into());
    }
    let term_program = std::env::var("TERM_PROGRAM").unwrap_or_default();
    let is_wezterm = term_program == "WezTerm" || std::env::var("WEZTERM_EXECUTABLE").is_ok();
    if is_wezterm && !which_exists("chafa") {
        return Err(t!("chafa_required").into());
    }
    let preview_cmd = build_preview_cmd();
    println!("{}", t!("list_found", count => images.len()));
    let tmp = std::env::temp_dir().join("wallow_fzf_selection.txt");
    // fzf 的 UI 需要直接读写 /dev/tty。
    // 通过 sh -c 运行整条管道：printf 把路径列表喂给 fzf，
    // fzf 检测到 stdout 被重定向时自动用 /dev/tty 渲染 UI，
    // 选中结果写入 tmpfile。
    let escaped_paths = images
        .iter()
        .map(|p| {
            let s = p.to_string_lossy();
            format!("'{}'", s.replace('\'', "'\\''" ))
        })
        .collect::<Vec<_>>()
        .join(" ");
    let preview_escaped = preview_cmd.replace('"', "\\\"" );
    let shell_cmd = format!(
        "printf '%s\\n' {paths} | fzf --preview \"sh -c '{preview}'\" --preview-window=right:60% --ansi > {tmp}",
        paths = escaped_paths,
        preview = preview_escaped,
        tmp = tmp.display()
    );
    let status = std::process::Command::new("sh")
        .arg("-c")
        .arg(&shell_cmd)
        .status()
        .map_err(|_| t!("fzf_error"))?;
    if status.success() {
        if tmp.exists() {
            let selected = std::fs::read_to_string(&tmp)?;
            let selected = selected.trim().to_string();
            let _ = std::fs::remove_file(&tmp);
            if !selected.is_empty() {
                println!("{}", t!("setting_wallpaper"));
                let path = std::path::PathBuf::from(&selected);
                setter::set_from_path(&path)?;
                println!("{}", t!("set_done"));
            }
        }
    }
    Ok(())
}

/// 根据当前终端类型构建 fzf --preview 使用的图片显示命令
///
/// 优先级：WezTerm(chafa iterm) → Kitty → iTerm2 → chafa → 文件名 fallback
///
/// 注意：wezterm imgcat 在 fzf preview 中有已知 bug，会永远处于 loading 状态而不显示图片。
/// 参考：https://github.com/wezterm/wezterm/issues/6088
///         https://github.com/junegunn/fzf/issues/3646
/// WezTerm 支持 iTerm2 协议，因此在 WezTerm 中使用 chafa -f iterm 可得到真实图片渲染。
fn build_preview_cmd() -> String {
    let term_program = std::env::var("TERM_PROGRAM").unwrap_or_default();
    let term = std::env::var("TERM").unwrap_or_default();
    // 在 Rust 进程里读取终端尺寸，传给 chafa
    // fzf preview 子进程的 stdout 被重定向，无法自动检测终端尺寸
    let (cols, rows) = term_size();
    // 预览窗格占右边 60%，高度留一行给 fzf 界面
    let preview_w = (cols * 60 / 100).max(20);
    let preview_h = rows.saturating_sub(2).max(10);
    let size_arg = format!("-s {}x{}", preview_w, preview_h);
    if term_program == "WezTerm" || std::env::var("WEZTERM_EXECUTABLE").is_ok() {
        if which_exists("chafa") {
            format!("chafa -f iterm {} --animate false {{}}", size_arg)
        } else {
            "echo 'Install chafa for image preview: brew install chafa'".to_string()
        }
    } else if term == "xterm-kitty" || std::env::var("KITTY_WINDOW_ID").is_ok() {
        "kitty +kitten icat --clear --transfer-mode=memory --stdin=no --place=${FZF_PREVIEW_COLUMNS}x${FZF_PREVIEW_LINES}@0x0 {}".to_string()
    } else if term_program == "iTerm.app" {
        "imgcat -W ${FZF_PREVIEW_COLUMNS} -H ${FZF_PREVIEW_LINES} {}".to_string()
    } else if which_exists("chafa") {
        format!("chafa {} --animate false {{}}", size_arg)
    } else {
        "echo {}".to_string()
    }
}

/// 读取当前终端宽度和高度（列数 x 行数）
/// 通过 ioctl(TIOCGWINSZ) 直接读取终端宽高（列数 x 行数）
/// 不依赖外部命令，失败时返回 80x24
fn term_size() -> (usize, usize) {
    #[cfg(unix)]
    {
        use std::os::unix::io::AsRawFd;
        // winsize 结构体对应 struct winsize { ws_row, ws_col, ws_xpixel, ws_ypixel }
        #[repr(C)]
        struct Winsize {
            ws_row: u16,
            ws_col: u16,
            ws_xpixel: u16,
            ws_ypixel: u16,
        }
        // TIOCGWINSZ 在 macOS 上的值为 0x40087468
        const TIOCGWINSZ: u64 = 0x40087468;
        let mut ws = Winsize { ws_row: 0, ws_col: 0, ws_xpixel: 0, ws_ypixel: 0 };
        // 尝试 stdout(fd=1)，如果失败尝试 stderr(fd=2)，再失败尝试 stdin(fd=0)
        let fds = [
            std::io::stdout().as_raw_fd(),
            std::io::stderr().as_raw_fd(),
            std::io::stdin().as_raw_fd(),
        ];
        for fd in fds {
            let ret = unsafe { libc::ioctl(fd, TIOCGWINSZ, &mut ws) };
            if ret == 0 && ws.ws_col > 0 && ws.ws_row > 0 {
                return (ws.ws_col as usize, ws.ws_row as usize);
            }
        }
    }
    (80, 24)
}

/// 检查某个命令是否在 PATH 中存在
fn which_exists(cmd: &str) -> bool {
    std::process::Command::new("which")
        .arg(cmd)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// 处理 apply 子命令：将本地文件设为壁纸
fn handle_apply(image: &str) -> Result<(), Box<dyn std::error::Error>> {
    let path = std::path::PathBuf::from(image);
    if !path.exists() {
        return Err(format!("文件不存在: {}", image).into());
    }
    println!("{}", t!("setting_wallpaper"));
    setter::set_from_path(&path)?;
    println!("{}", t!("set_done"));
    Ok(())
}

/// 处理 clean 子命令：清理所有以 wallow- 开头的文件
fn handle_clean(config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    let dirs = vec![&config.wallpaper_dir, &config.converted_dir];

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
    source: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", t!("search_start"));

    let options = SearchOptions {
        query: query.or(config.search_defaults.query.as_deref()),
        resolution: resolution.unwrap_or(&config.search_defaults.resolution),
        categories: categories.unwrap_or(&config.search_defaults.categories),
        purity: purity.unwrap_or(&config.search_defaults.purity),
        sorting: sorting.unwrap_or(&config.search_defaults.sorting),
    };

    // 根据 source 参数选择对应的壁纸源客户端
    let wallpapers: Vec<source::WallpaperInfo> = match source {
        "unsplash" => {
            let key = config.unsplash_access_key.clone()
                .ok_or("Unsplash Access Key 未配置，请在 config.toml 的 [source.unsplash] 中设置 access_key，或设置 UNSPLASH_ACCESS_KEY 环境变量")?;
            UnsplashClient::new(key).search(options).await?
        }
        _ => {
            WallhavenClient::new(config.api_key.clone()).search(options).await?
        }
    };

    if wallpapers.is_empty() {
        println!("{}", t!("no_wallpapers"));
        return Ok(());
    }

    let selected: Vec<&source::WallpaperInfo> = wallpapers.iter().take(count).collect();
    let total = count.min(wallpapers.len());

    for (i, wallpaper) in selected.iter().enumerate() {
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

        let save_path = match source {
            "unsplash" => {
                let key = config.unsplash_access_key.clone().unwrap();
                UnsplashClient::new(key).download(wallpaper, &config.wallpaper_dir).await?
            }
            _ => {
                WallhavenClient::new(config.api_key.clone()).download(wallpaper, &config.wallpaper_dir).await?
            }
        };
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
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    println!("{}", t!("convert_start", image => image, theme => theme));

    let input_path = std::path::Path::new(image);
    let original_filename = input_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("image.jpg");

    // 生成带主题前缀的文件名
    // 如果原名是 wallow-wallhaven-xxx.jpg，改为 wallow-catppuccin-wallhaven-xxx.jpg
    let new_filename = if original_filename.starts_with("wallow-") {
        format!("wallow-{}-{}", theme, &original_filename[7..])
    } else {
        format!("wallow-{}-{}", theme, original_filename)
    };

    // 确定输出完整路径
    let output_file_path = if let Some(out) = output {
        let p = std::path::PathBuf::from(out);
        if p.is_dir() { p.join(new_filename) } else { p }
    } else {
        config.converted_dir.join(new_filename)
    };

    gowall::convert(image, theme, Some(output_file_path.to_str().unwrap()))?;

    println!("{}", t!("convert_done", path => output_file_path.display()));
    Ok(output_file_path)
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
    source: &str,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    println!("{}", t!("search_start"));
    let options = SearchOptions {
        query: query.or(config.search_defaults.query.as_deref()),
        resolution: resolution.unwrap_or(&config.search_defaults.resolution),
        categories: categories.unwrap_or(&config.search_defaults.categories),
        purity: purity.unwrap_or(&config.search_defaults.purity),
        sorting: sorting.unwrap_or(&config.search_defaults.sorting),
    };
    let (wallpapers, save_path) = match source {
        "unsplash" => {
            let key = config.unsplash_access_key.clone()
                .ok_or("Unsplash Access Key 未配置")?;
            let client = UnsplashClient::new(key);
            let wallpapers = client.search(options).await?;
            let wallpaper = wallpapers.first().ok_or(t!("error_no_wallpapers"))?;
            println!(
                "{}",
                t!("download_info", current => 1, total => 1,
                   id => wallpaper.id, res => wallpaper.resolution)
            );
            let path = client.download(wallpaper, &config.wallpaper_dir).await?;
            (wallpapers, path)
        }
        _ => {
            let client = WallhavenClient::new(config.api_key.clone());
            let wallpapers = client.search(options).await?;
            let wallpaper = wallpapers.first().ok_or(t!("error_no_wallpapers"))?;
            println!(
                "{}",
                t!("download_info", current => 1, total => 1,
                   id => wallpaper.id, res => wallpaper.resolution)
            );
            let path = client.download(wallpaper, &config.wallpaper_dir).await?;
            (wallpapers, path)
        }
    };
    let _ = wallpapers; // 防止 unused 警告
    println!("{}", t!("save_path", path => save_path.display()));
    if let Some(theme_name) = theme {
        let image_str = save_path.to_str().ok_or(t!("error_utf8"))?;
        let converted_path = handle_convert(config, image_str, theme_name, None)?;
        println!("{}", t!("all_done", theme => theme_name));
        Ok(converted_path)
    } else {
        Ok(save_path)
    }
}

/// 处理 schedule 子命令：注册或更新 crontab 定时任务
async fn handle_schedule(
    config: &mut AppConfig,
    cron_arg: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    // 确定最终使用的 cron 表达式：命令行参数 > toml 配置
    let cron = match cron_arg {
        Some(expr) => expr.to_string(),
        None => config
            .schedule
            .cron
            .clone()
            .ok_or("请提供 cron 表达式，或在 config.toml 的 [schedule] 节中设置 cron 字段")?,
    };

    // 如果是通过命令行传入的，写入配置文件持久化
    if cron_arg.is_some() {
        config.set_cron(cron.clone())?;
        println!("已将 cron 表达式 '{}' 写入配置文件", cron);
    }

    // 获取当前可执行文件路径，用于构造 crontab 条目
    let bin_path = std::env::current_exe()?;
    let bin_str = bin_path.to_string_lossy();
    // crontab 条目格式: "<cron表达式> <可执行文件> schedule --run"
    // 用 --run 标志区分「注册模式」和「执行模式」，避免 crontab 触发时再次进入注册逻辑
    let cron_entry = format!("{} {} schedule --run", cron, bin_str);

    // 读取当前 crontab 内容
    let existing = std::process::Command::new("crontab")
        .arg("-l")
        .output();

    let current = match existing {
        Ok(out) if out.status.success() => String::from_utf8_lossy(&out.stdout).to_string(),
        // crontab -l 在无任何条目时返回非零退出码，视为空内容
        _ => String::new(),
    };

    // 移除旧的 wallow schedule 条目（避免重复），再追加新条目
    let filtered: String = current
        .lines()
        .filter(|line| !line.contains("wallow") || !line.contains("schedule"))
        .map(|line| format!("{}
", line))
        .collect();
    let new_crontab = format!("{}{}
", filtered, cron_entry);

    // 通过 `crontab -` 写入新的 crontab
    let mut child = std::process::Command::new("crontab")
        .arg("-")
        .stdin(std::process::Stdio::piped())
        .spawn()?;
    {
        use std::io::Write;
        let stdin = child.stdin.as_mut().ok_or("无法获取 crontab stdin")?;
        stdin.write_all(new_crontab.as_bytes())?;
    }
    let status = child.wait()?;
    if !status.success() {
        return Err("写入 crontab 失败".into());
    }

    println!("定时任务已注册: {}", cron_entry);
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
            println!(
                "{}",
                t!("config_path", path => config.config_path.display())
            );
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
            println!("{}", t!("config_sorting", sorting => config.search_defaults.sorting));
            println!("  source: {}", config.default_source);
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

/// 处理 upgrade 子命令：从 GitHub Releases 下载最新版本并替换当前二进制
async fn handle_upgrade() -> Result<(), Box<dyn std::error::Error>> {
    // 从 GitHub API 获取最新 release 信息
    println!("{}", t!("upgrade_checking"));

    let client = reqwest::Client::builder()
        // 设置 User-Agent，GitHub API 要求必须携带
        .user_agent("wallow-upgrade")
        .build()?;

    // 请求 GitHub latest release API，返回 JSON 包含 tag_name 和 assets
    let release: serde_json::Value = client
        .get("https://api.github.com/repos/shlroland/wallow/releases/latest")
        .send()
        .await?
        .json()
        .await?;

    // 提取版本号字符串，如 "v0.1.3"
    let latest_version = release["tag_name"]
        .as_str()
        .ok_or("无法解析最新版本号")?;

    // 与当前编译时版本对比（env!("CARGO_PKG_VERSION") 在编译期展开）
    let current_version = env!("CARGO_PKG_VERSION");
    let latest_stripped = latest_version.trim_start_matches('v');

    if latest_stripped == current_version {
        println!("{}", t!("upgrade_already_latest", version => current_version));
        return Ok(());
    }

    println!("{}", t!("upgrade_found", current => current_version, latest => latest_version));

    // 根据当前系统和架构确定 artifact 名称（与 install.sh 保持一致）
    let artifact = detect_artifact()?;

    // 构造下载 URL
    let download_url = format!(
        "https://github.com/shlroland/wallow/releases/latest/download/{}",
        artifact
    );

    println!("{}", t!("upgrade_downloading", url => &download_url));

    // 下载新二进制到临时文件
    let bytes = client
        .get(&download_url)
        .send()
        .await?
        .bytes()
        .await?;

    // 获取当前可执行文件路径
    let current_exe = std::env::current_exe()?;

    // 写入临时文件（与目标同目录，保证 rename 是原子操作）
    // 同目录下的 rename 在 Unix 上是原子的，避免写到一半被中断
    let tmp_path = current_exe.with_extension("tmp");
    std::fs::write(&tmp_path, &bytes)?;

    // 赋予可执行权限
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        // 0o755 = rwxr-xr-x
        std::fs::set_permissions(&tmp_path, std::fs::Permissions::from_mode(0o755))?;
    }

    // 原子替换：将临时文件重命名为当前可执行文件
    std::fs::rename(&tmp_path, &current_exe)?;

    println!("{}", t!("upgrade_done", version => latest_version));
    Ok(())
}

/// 根据当前操作系统和 CPU 架构返回对应的 artifact 文件名
fn detect_artifact() -> Result<String, Box<dyn std::error::Error>> {
    // std::env::consts::OS 返回 "macos", "linux", "windows" 等
    // std::env::consts::ARCH 返回 "x86_64", "aarch64" 等
    let artifact = match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "x86_64") => "wallow-macos-x64",
        ("macos", "aarch64") => "wallow-macos-arm64",
        ("linux", "x86_64") => "wallow-linux-x64",
        (os, arch) => {
            return Err(format!("不支持的平台: {os}/{arch}").into());
        }
    };
    Ok(artifact.to_string())
}

/// 处理 uninstall 子命令：删除二进制、配置目录，可选删除壁纸缓存
fn handle_uninstall(
    config: &AppConfig,
    keep_wallpapers: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", t!("uninstall_start"));

    // 1. 删除壁纸缓存目录（除非用户指定 --keep-wallpapers）
    if !keep_wallpapers {
        for dir in [&config.wallpaper_dir, &config.converted_dir] {
            if dir.exists() {
                std::fs::remove_dir_all(dir)?;
                println!("{}", t!("uninstall_removed_dir", path => dir.display()));
            }
        }
    } else {
        println!("{}", t!("uninstall_kept_wallpapers"));
    }

    // 2. 删除配置目录 ~/.config/wallow/
    // config_path 是 ~/.config/wallow/config.toml，取其父目录
    if let Some(config_dir) = config.config_path.parent() {
        if config_dir.exists() {
            std::fs::remove_dir_all(config_dir)?;
            println!("{}", t!("uninstall_removed_dir", path => config_dir.display()));
        }
    }

    // 3. 删除当前可执行文件本身
    // 在 Unix 上，正在运行的进程可以删除自身的 inode，进程仍可继续运行直到退出
    let current_exe = std::env::current_exe()?;
    std::fs::remove_file(&current_exe)?;
    println!("{}", t!("uninstall_removed_bin", path => current_exe.display()));

    println!("{}", t!("uninstall_done"));
    Ok(())
}
