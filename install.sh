#!/bin/bash

# wallow 安装脚本
# 该脚本将编译项目并配置默认环境

set -e

# 颜色定义
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}开始安装 wallow...${NC}"

# 1. 编译项目
echo -e "${BLUE}步骤 1: 正在编译 release 版本...${NC}"
cargo build --release

# 2. 准备配置目录
CONFIG_DIR="$HOME/.config/wallow"
echo -e "${BLUE}步骤 2: 正在准备配置目录 ($CONFIG_DIR)...${NC}"
mkdir -p "$CONFIG_DIR"

# 3. 安装默认配置文件
if [ ! -f "$CONFIG_DIR/config.toml" ]; then
    echo -e "${BLUE}步骤 3: 正在安装默认配置文件...${NC}"
    cp assets/config.toml "$CONFIG_DIR/config.toml"
    echo -e "${GREEN}默认配置已安装到 $CONFIG_DIR/${NC}"
else
    echo -e "${BLUE}步骤 3: 配置文件已存在，跳过覆盖。${NC}"
fi

# 4. 安装二进制文件
BIN_PATH="$HOME/.cargo/bin"
echo -e "${BLUE}步骤 4: 正在安装二进制文件到 $BIN_PATH...${NC}"
mkdir -p "$BIN_PATH"
cp target/release/wallow "$BIN_PATH/"

# 5. 提示自动补全
echo -e "\n${GREEN}安装完成！${NC}"
echo -e "--------------------------------------------------"
echo -e "你可以通过运行 ${BLUE}wallow --version${NC} 来验证安装。"
echo -e ""
echo -e "提示：为了获得更好的体验，建议配置 Shell 自动补全："
echo -e "Fish: ${BLUE}wallow completions fish > ~/.config/fish/completions/wallow.fish${NC}"
echo -e "Zsh:  ${BLUE}wallow completions zsh > ~/.zsh/completions/_wallow${NC}"
echo -e "\n提示：VS Code 用户安装 Taplo 插件后可享受配置自动补全（依赖网络）。"
echo -e "--------------------------------------------------"
