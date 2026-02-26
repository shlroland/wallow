# Wallow 🧱

[![Language](https://img.shields.io/badge/language-Rust-orange.svg)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**Wallow** is a modern CLI tool written in Rust for wallpaper enthusiasts. Search and download high-quality wallpapers from multiple sources, and automatically apply aesthetic color themes using `gowall`.

[中文文档 (Chinese Documentation)](README_zh.md)

## ✨ Features

- 🔍 **Search & Fetch**: Pluggable source system — supports multiple wallpaper providers with per-source configuration.
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
# Search and download wallpapers (default source: wallhaven)
wallow fetch --query "nature" --count 3
# Use a specific source
wallow fetch --query "landscape" --source unsplash
# Convert a local image to a theme
wallow convert image.jpg --theme catppuccin
# One-click: Search, download, and apply theme
wallow run --query "cyberpunk" --theme dracula
# Use default theme from config (no --theme needed if configured)
wallow run --query "nature"
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
# Output directories for converted wallpapers (supports multiple)
converted_dirs = [
  "Pictures/wallow/converted",
  ".config/wezterm/backgrounds",
]
# Default source: wallhaven (default) or unsplash
source = "wallhaven"
# Default theme — run/set will auto-convert without --theme
theme = "catppuccin"
[common.search]
query = "nature"
resolution = "3840x2160"
sorting = "random"
[source.wallhaven]
api_key = "your_wallhaven_api_key_here"
[source.unsplash]
access_key = "your_unsplash_access_key_here"
[schedule]
# Cron expression for the scheduled wallpaper job
# Example: every day at 08:00
cron = "0 8 * * *"
```

## 🖼️ Wallpaper Sources

Wallow uses a pluggable source system. Use `--source <name>` to switch per-command, or set a default in `config.toml` under `[common] source`.

| Source | Requires | Notes |
|--------|----------|-------|
| `wallhaven` | API Key (optional) | Default source |
| `unsplash` | Access Key (required) | Demo: 50 req/hr |

### wallhaven

The default source. Works out of the box without any configuration. An API Key is only needed to access NSFW content.

**Setup (optional):**

1. Get your API Key at [wallhaven.cc/settings/account](https://wallhaven.cc/settings/account)
2. Add to config or set as environment variable:

```toml
[source.wallhaven]
api_key = "your_wallhaven_api_key_here"
```

```bash
export WALLHAVEN_API_KEY="your_wallhaven_api_key_here"
```

### unsplash

High-quality editorial photos from [unsplash.com](https://unsplash.com). Requires a free Access Key.

**Setup (required):**

1. Register a new app at [unsplash.com/developers](https://unsplash.com/developers)
2. Copy the **Access Key** (not the Secret Key)
3. Add to config or set as environment variable:

```toml
[source.unsplash]
access_key = "your_unsplash_access_key_here"
```

```bash
export UNSPLASH_ACCESS_KEY="your_unsplash_access_key_here"
```

> Demo apps are rate-limited to **50 requests/hour**. Apply for Production access on the Unsplash developer dashboard to raise it to 5000/hr.

**Usage:**

```bash
wallow fetch --query "nature" --count 3
wallow fetch --query "landscape" --source unsplash
wallow run --query "cyberpunk" --theme dracula --source unsplash
```

## 📄 License
## 📄 License

This project is licensed under the MIT License.
