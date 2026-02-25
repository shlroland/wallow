#!/bin/bash

# wallow 安装脚本
# 该脚本将从 GitHub 下载预编译的二进制文件并配置环境

set -e

# 仓库信息
REPO="shlroland/wallow"
# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}开始安装 wallow...${NC}"

# 1. 检测系统和架构
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)

case "$OS" in
    linux)
        if [ "$ARCH" == "x86_64" ]; then
            ARTIFACT="wallow-linux-x64"
        else
            echo -e "${RED}抱歉，目前仅支持 Linux x86_64 架构。${NC}"
            exit 1
        fi
        ;;
    darwin)
        if [ "$ARCH" == "x86_64" ]; then
            ARTIFACT="wallow-macos-x64"
        elif [ "$ARCH" == "arm64" ]; then
            ARTIFACT="wallow-macos-arm64"
        else
            echo -e "${RED}不支持的 macOS 架构: $ARCH${NC}"
            exit 1
        fi
        ;;
    *)
        echo -e "${RED}不支持的操作系统: $OS${NC}"
        exit 1
        ;;
esac

# 2. 获取最新版本并下载
echo -e "${BLUE}步骤 1: 正在从 GitHub 获取最新版本...${NC}"
LATEST_RELEASE_URL="https://github.com/$REPO/releases/latest/download/$ARTIFACT"

TMP_DIR=$(mktemp -d)
echo -e "${BLUE}步骤 2: 正在下载二进制文件 ($ARTIFACT)...${NC}"
# 使用 -L 处理重定向
curl -L "$LATEST_RELEASE_URL" -o "$TMP_DIR/wallow"

# 3. 准备配置目录
CONFIG_DIR="$HOME/.config/wallow"
echo -e "${BLUE}步骤 3: 正在准备配置目录 ($CONFIG_DIR)...${NC}"
mkdir -p "$CONFIG_DIR"

# 4. 安装默认配置文件
if [ ! -f "$CONFIG_DIR/config.toml" ]; then
    echo -e "${BLUE}步骤 4: 正在安装默认配置文件...${NC}"
    # 如果在源码目录下，直接用 assets/config.toml，否则可以考虑从 github 下载
    if [ -f "assets/config.toml" ]; then
        cp assets/config.toml "$CONFIG_DIR/config.toml"
    else
        echo -e "${BLUE}正在从 GitHub 下载默认配置模板...${NC}"
        curl -L "https://raw.githubusercontent.com/$REPO/master/assets/config.toml" -o "$CONFIG_DIR/config.toml"
    fi
    echo -e "${GREEN}默认配置已安装到 $CONFIG_DIR/${NC}"
else
    echo -e "${BLUE}步骤 4: 配置文件已存在，跳过覆盖。${NC}"
fi

# 5. 安装二进制文件
BIN_PATH="$HOME/.local/bin"
echo -e "${BLUE}步骤 5: 正在安装二进制文件到 $BIN_PATH...${NC}"
mkdir -p "$BIN_PATH"
mv "$TMP_DIR/wallow" "$BIN_PATH/"
chmod +x "$BIN_PATH/wallow"

# 清理临时目录
rm -rf "$TMP_DIR"

# 6. 检查 PATH 并提示
echo -e "\n${GREEN}安装完成！${NC}"
echo -e "--------------------------------------------------"

if [[ ":$PATH:" != *":$BIN_PATH:"* ]]; then
    echo -e "${RED}警告: $BIN_PATH 不在你的 PATH 环境变量中。${NC}"
    echo -e "你需要将以下行添加到你的 Shell 配置文件（如 ~/.bashrc 或 ~/.zshrc）中："
    echo -e "${BLUE}export PATH=\"\$HOME/.local/bin:\$PATH\"${NC}\n"
fi

echo -e "你可以通过运行 ${BLUE}wallow --version${NC} 来验证安装。"
echo -e ""
echo -e "提示：为了获得更好的体验，建议配置 Shell 自动补全："
echo -e "Fish: ${BLUE}wallow completions fish > ~/.config/fish/completions/wallow.fish${NC}"
echo -e "Zsh:  ${BLUE}wallow completions zsh > ~/.zsh/completions/_wallow${NC}"
echo -e "\n提示：VS Code 用户安装 Taplo 插件后可享受配置自动补全（依赖网络）。"
echo -e "--------------------------------------------------"
