// setter.rs — 系统壁纸设置模块

use rust_i18n::t;
use std::path::Path;

/// 将指定路径的图片设置为系统壁纸
///
/// # 参数
/// - `path`: 图片的绝对路径
pub fn set_from_path(path: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
    let path_ref = path.as_ref();
    let path_str = path_ref.to_str().ok_or(t!("error_utf8"))?;

    // 打印调试信息，让用户知道到底在设置哪张图
    println!("  -> {}", path_ref.display());

    // 调用第三方库设置壁纸
    // 这个库会自动识别操作系统并调用相应的 API
    wallpaper::set_from_path(path_str)
        .map_err(|e| format!("{}: {}", t!("error_set_failed", reason => ""), e).into())
}
