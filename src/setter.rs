// setter.rs — 系统壁纸设置模块

use rust_i18n::t;
use std::path::Path;

/// 将指定路径的图片设置为系统壁纸
///
/// # 参数
/// - `path`: 图片的绝对路径
pub fn set_from_path(path: impl AsRef<Path>) -> Result<(), Box<dyn std::error::Error>> {
    // wallpaper::set_from_path 接受一个字符串路径
    // 我们先将路径转为字符串
    let path_str = path.as_ref().to_str().ok_or(t!("error_utf8"))?;

    // 调用第三方库设置壁纸
    // 这个库会自动识别操作系统并调用相应的 API
    wallpaper::set_from_path(path_str)
        .map_err(|e| format!("{}: {}", t!("error_set_failed", reason => ""), e).into())
}
