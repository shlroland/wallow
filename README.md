# Wallow 🧱

[![Language](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**Wallow** is a modern CLI tool written in Rust designed for wallpaper enthusiasts. It allows you to search and download high-quality wallpapers from Wallhaven and automatically apply aesthetic color themes using `gowall`.

[中文文档 (Chinese Documentation)](README_zh.md)

## ✨ Features

- 🔍 **Search & Fetch**: Powerful search interface for Wallhaven API.
- 🎨 **Theme Conversion**: Seamless integration with `gowall` to apply themes like Catppuccin, Dracula, Nord, and more.
- 📅 **Schedule**: Built-in support for daily wallpaper automation with `crontab` integration.
- 🌍 **I18n**: Automatic language detection (Supports English and Chinese).
- ⚙️ **Configurable**: Unix-style configuration via `~/.config/wallow/config.toml`.
- ⌨️ **Auto-completion**: Support for Zsh, Fish, and Bash.

## 🚀 Installation

### Prerequisites

- **gowall**: Required for theme conversion. [Install gowall](https://github.com/Achno/gowall).
- **Rust**: To compile from source.

### Build from source

```bash
git clone https://github.com/your-username/wallow.git
cd wallow
cargo build --release
```

The binary will be available at `target/release/wallow`.

## 🛠 Usage

### Basic Commands

```bash
# Search and download wallpapers
wallow fetch --query "nature" --count 3

# Convert a local image to a theme
wallow convert image.jpg --theme catppuccin

# One-click: Search, download, and apply theme
wallow run --query "cyberpunk" --theme dracula

# List all available gowall themes
wallow themes
```

### Automation (Schedule)

Download a random fresh wallpaper daily:

```bash
wallow schedule
```
*Follow the on-screen instructions to integrate with `crontab`.*

### Shell Completion

```bash
# For Zsh
wallow completions zsh > ~/.zsh/completions/_wallow
```

## ⚙️ Configuration

Create a config file at `~/.config/wallow/config.toml`:

```toml
[common]
wallpaper_dir = "my_wallpapers"

[common.search]
resolution = "3840x2160"
sorting = "random"

[source.wallhaven]
api_key = "your_wallhaven_api_key_here"
```

## 📄 License

This project is licensed under the MIT License.
