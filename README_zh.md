# Wallow 🧱

**Wallow** 是一个用 Rust 编写的现代命令行工具，专为壁纸爱好者设计。它可以从 Wallhaven 搜索并下载高质量壁纸，并利用 `gowall` 自动应用各种美化主题。

[English Documentation](README.md)

## ✨ 特性

- 🔍 **搜索与下载**: 强大的 Wallhaven API 搜索接口。
- 🎨 **主题转换**: 无缝集成 `gowall`，支持 Catppuccin, Dracula, Nord 等配色主题。
- 📅 **定时任务**: 内置 `schedule` 子命令，轻松集成 `crontab` 实现每日自动换壁纸。
- 🖼️ **交互式预览**: 集成 `fzf` 实现交互式壁纸选择，支持终端图片预览（WezTerm, Kitty 等）。
- 🌍 **多语言支持**: 自动检测系统语言（目前支持中英文）。
- ⚙️ **灵活配置**: 遵循 Unix 风格，通过 `~/.config/wallow/config.toml` 管理配置。
- ⌨️ **命令补全**: 支持 Zsh, Fish, Bash 等多种 Shell 的自动补全。

## 🚀 安装

### 一键安装 (推荐)

只需要 `curl` 和 `bash`。该脚本将自动下载适用于你系统（macOS/Linux）的最新预编译二进制文件，并安装到 `~/.local/bin`。

```bash
curl -sSL https://raw.githubusercontent.com/shlroland/wallow/master/install.sh | bash
```

### 源码编译

如果你已安装 Rust，也可以选择从源码编译：

#### 前提条件

- **gowall**: 用于主题转换。 [安装 gowall](https://github.com/Achno/gowall)。
- **Rust**: 用于从源码编译。

#### 编译步骤

```bash
git clone https://github.com/shlroland/wallow.git
cd wallow
cargo build --release
```

二进制文件将生成在 `target/release/wallow`。

## 🛠 使用方法

### 基础命令

```bash
# 搜索并下载壁纸
wallow fetch --query "nature" --count 3

# 对本地图片应用主题
wallow convert image.jpg --theme catppuccin

# 一键完成：下载并应用主题
wallow run --query "cyberpunk" --theme dracula

# 列表查看与交互式预览 (需要安装 fzf)
wallow list --fzf

# 将本地图片设为系统壁纸
wallow apply wallpapers/image.jpg

# 列出所有可用的 gowall 主题
wallow themes

# 管理配置项
wallow config show
wallow config dump
wallow config set query "nature"
```

### 自动化 (Schedule)

每天自动下载一张随机的新鲜壁纸：

```bash
wallow schedule
```
*执行后请根据提示将其加入 `crontab`。*

### Shell 自动补全

```bash
# Zsh 用户
wallow completions zsh > ~/.zsh/completions/_wallow

# Fish 用户
wallow completions fish > ~/.config/fish/completions/wallow.fish
```

## ⚙️ 配置

在 `~/.config/wallow/config.toml` 创建配置文件：

```toml
#:schema https://raw.githubusercontent.com/shlroland/wallow/master/wallow.schema.json

[common]
wallpaper_dir = "my_wallpapers"  # 壁纸保存目录

[common.search]
query = "nature"            # 默认搜索关键词
resolution = "3840x2160"        # 默认分辨率
sorting = "random"              # 默认排序

[source.wallhaven]
api_key = "你的_wallhaven_api_key" # 用于访问 NSFW 或提高频率限制
```

## 📄 许可证

本项目采用 MIT 许可证。
