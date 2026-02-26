# Wallow 🧱

[![Language](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**Wallow** is a modern CLI tool written in Rust designed for wallpaper enthusiasts. It allows you to search and download high-quality wallpapers from Wallhaven and automatically apply aesthetic color themes using `gowall`.

[中文文档 (Chinese Documentation)](README_zh.md)

## ✨ Features

- 🔍 **Search & Fetch**: Powerful search interface for Wallhaven API.
- 🎨 **Theme Conversion**: Seamless integration with `gowall` to apply themes like Catppuccin, Dracula, Nord, and more.
- 📅 **Schedule**: Built-in support for daily wallpaper automation with `crontab` integration.
- 🖼️ **Interactive Preview**: Integration with `fzf` for interactive wallpaper selection with image previews. Supports WezTerm (`chafa` + iTerm2 protocol), Kitty, iTerm2, and any terminal with `chafa` installed.
- 🌍 **I18n**: Automatic language detection (Supports English and Chinese).
- ⚙️ **Configurable**: Unix-style configuration via `~/.config/wallow/config.toml`.
- ⌨️ **Auto-completion**: Support for Zsh, Fish, and Bash.

## 🚀 Installation

### One-line Install (Recommended)

Requires `curl` and `bash`. This script will download the latest pre-built binary for your system (macOS/Linux) and install it to `~/.local/bin`.

```bash
curl -sSL https://raw.githubusercontent.com/shlroland/wallow/master/install.sh | bash
```

### Build from source

If you have Rust installed and prefer building from source:

#### Prerequisites

- **gowall**: Required for theme conversion. [Install gowall](https://github.com/Achno/gowall).
- **Rust**: To compile from source.

#### Build

```bash
git clone https://github.com/shlroland/wallow.git
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

# List and interactively preview wallpapers (requires fzf)
# List and interactively preview wallpapers (requires fzf + chafa)
wallow list --fzf

# Set a local image as system wallpaper
wallow apply wallpapers/image.jpg

# List all available gowall themes
wallow themes

# Manage configuration
wallow config show
wallow config dump
wallow config set query "nature"
```

### Interactive Preview (`list --fzf`)

Opens an interactive wallpaper picker with image preview. Selecting an entry sets it as your system wallpaper.

**Requirements:**
- [`fzf`](https://github.com/junegunn/fzf): `brew install fzf`
- [`chafa`](https://hpjansson.org/chafa/): `brew install chafa`

**Terminal support:**

| Terminal | Protocol | Notes |
|----------|----------|-------|
| WezTerm  | iTerm2 (`chafa -f iterm`) | `wezterm imgcat` has a [known fzf bug](https://github.com/wezterm/wezterm/issues/6088); chafa is used instead |
| Kitty    | Kitty graphics | via `kitty +kitten icat` |
| iTerm2   | iTerm2 inline | via `imgcat` |
| Others   | auto (`chafa`) | best available protocol |

Register or update a crontab job to automatically download a fresh wallpaper on a schedule:

```bash
# Register with a cron expression (also saves it to config.toml)
wallow schedule "0 8 * * *"

# Re-register using the cron expression already saved in config.toml
wallow schedule
```

The cron expression is saved to `~/.config/wallow/config.toml` under `[schedule]`. Any existing `wallow schedule` crontab entry is replaced, so running the command again is safe.

### Shell Completion

```bash
# For Zsh
wallow completions zsh > ~/.zsh/completions/_wallow

# For Fish
wallow completions fish > ~/.config/fish/completions/wallow.fish
```

## ⚙️ Configuration

Create a config file at `~/.config/wallow/config.toml`:

```toml
#:schema https://raw.githubusercontent.com/shlroland/wallow/master/wallow.schema.json

[common]
wallpaper_dir = "my_wallpapers"

[common.search]
query = "nature"
resolution = "3840x2160"
sorting = "random"

[source.wallhaven]
api_key = "your_wallhaven_api_key_here"

[schedule]
# Cron expression for the scheduled wallpaper job
# Example: every day at 08:00
cron = "0 8 * * *"
```

## 📄 License

This project is licensed under the MIT License.
