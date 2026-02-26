# Wallow 🧱

**Wallow** 是一个用 Rust 编写的现代命令行工具，专为壁纸爱好者设计。它可以从 Wallhaven 搜索并下载高质量壁纸，并利用 `gowall` 自动应用各种美化主题。

[English Documentation](README.md)

## ✨ 特性

- 🔍 **搜索与下载**: 强大的 Wallhaven API 搜索接口。
- 🎨 **主题转换**: 无缝集成 `gowall`，支持 Catppuccin, Dracula, Nord 等配色主题。
- 📅 **定时任务**: 内置 `schedule` 子命令，轻松集成 `crontab` 实现每日自动换壁纸。
- 🖼️ **交互式预览**: 集成 `fzf` 实现交互式壁纸选择，支持终端图片预览。支持 WezTerm（`chafa` + iTerm2 协议）、Kitty、iTerm2 及安装了 `chafa` 的任意终端。
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
# 列表查看与交互式预览 (需要安装 fzf 和 chafa)
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
### 交互式预览 (`list --fzf`)
打开交互式壁纸选择界面，选中后自动设为系统壁纸。
**前置依赖：**
- [`fzf`](https://github.com/junegunn/fzf)：`brew install fzf`
- [`chafa`](https://hpjansson.org/chafa/)：`brew install chafa`
**终端支持情况：**
| 终端 | 协议 | 备注 |
|------|------|------|
| WezTerm  | iTerm2（`chafa -f iterm`） | `wezterm imgcat` 在 fzf 中有[已知 bug](https://github.com/wezterm/wezterm/issues/6088)，改用 chafa |
| Kitty    | Kitty graphics | 通过 `kitty +kitten icat` |
| iTerm2   | iTerm2 inline | 通过 `imgcat` |
| 其他终端 | 自动（`chafa`） | 自动选择最佳协议 |

注册或更新 crontab 定时任务，按指定频率自动下载新壁纸：
```bash
# 传入 cron 表达式注册定时任务（同时写入 config.toml）
wallow schedule "0 8 * * *"
# 使用 config.toml 中已保存的 cron 表达式重新注册
wallow schedule
```
cron 表达式会保存到 `~/.config/wallow/config.toml` 的 `[schedule]` 节。重复执行会替换旧的 crontab 条目，不会产生重复记录。

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
[schedule]
# 定时任务的 cron 表达式
# 示例：每天 08:00 执行
cron = "0 8 * * *"
```

## 📄 许可证

本项目采用 MIT 许可证。
