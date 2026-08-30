#!/usr/bin/env bash
# ==============================================================================
#  BOOMBOX-RS: Retro Cyberpunk Cassette Boombox & Worldwide Radio Explorer
#  Automated One-Line Installer for Linux & macOS
#  Repository: https://github.com/dannie203/tui-radio
# ==============================================================================

set -euo pipefail

REPO="dannie203/tui-radio"
BIN_DIR="${HOME}/.local/bin"
DESKTOP_DIR="${HOME}/.local/share/applications"
ICON_DIR="${HOME}/.local/share/icons/hicolor"

# Colors & Formatting
RED='\033[0;31m'
GREEN='\033[0;32m'
GOLD='\033[0;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

echo -e "${GOLD}${BOLD}"
cat << "EOF"
  ____   ____   ____  __  __ ____   ______  __
 | __ ) / __ \ / __ \|  \/  | __ ) / __ \ \/ /
 |  _ \| |  | | |  | | |\/| |  _ \| |  | |\  / 
 | |_) | |__| | |__| | |  | | |_) | |__| |/  \ 
 |____/ \____/ \____/|_|  |_|____/ \____//_/\_\ RX-505
EOF
echo -e "${CYAN}📼 Automated Installer for Linux & macOS${NC}\n"

# 1. Detect Architecture & OS
OS="$(uname -s | tr '[:upper:]' '[:lower:]')"
ARCH="$(uname -m)"

if [ "$OS" != "linux" ]; then
    echo -e "${RED}❌ Unsupported operating system: ${OS}. Currently Linux x86_64 is supported.${NC}"
    exit 1
fi

if [ "$ARCH" != "x86_64" ]; then
    echo -e "${RED}❌ Unsupported architecture: ${ARCH}. Prebuilt releases target x86_64.${NC}"
    echo -e "${CYAN}ℹ️  You can build from source using: cargo build --release${NC}"
    exit 1
fi

# 2. Check Dependencies
echo -e "${CYAN}🔍 Checking required dependencies...${NC}"

check_dep() {
    local cmd="$1"
    local desc="$2"
    if command -v "$cmd" >/dev/null 2>&1; then
        echo -e "  ${GREEN}✓${NC} ${cmd} (${desc}) found: $(command -v "$cmd")"
    else
        echo -e "  ${GOLD}⚠${NC} ${cmd} (${desc}) not found in PATH."
    fi
}

check_dep "mpv" "Audio engine & streaming decoder"
check_dep "yt-dlp" "Online YouTube & SoundCloud stream extractor"

if ! command -v mpv >/dev/null 2>&1; then
    echo -e "${GOLD}💡 Tip: Please install 'mpv' via your package manager (e.g. sudo pacman -S mpv / sudo apt install mpv).${NC}"
fi

# 3. Create Directories
mkdir -p "$BIN_DIR" "$DESKTOP_DIR"

# 4. Download Latest Release
TMP_DIR="$(mktemp -d)"
cleanup() {
    rm -rf "$TMP_DIR"
}
trap cleanup EXIT

RELEASE_URL="https://github.com/${REPO}/releases/latest/download/boombox-rs-linux-x86_64.tar.gz"

echo -e "\n${CYAN}📥 Downloading latest Boombox release from GitHub...${NC}"
if command -v curl >/dev/null 2>&1; then
    curl -sSL "$RELEASE_URL" -o "${TMP_DIR}/boombox.tar.gz"
elif command -v wget >/dev/null 2>&1; then
    wget -q "$RELEASE_URL" -O "${TMP_DIR}/boombox.tar.gz"
else
    echo -e "${RED}❌ Neither curl nor wget was found. Please install curl or wget.${NC}"
    exit 1
fi

echo -e "${CYAN}📦 Extracting and installing binaries...${NC}"
tar -xzf "${TMP_DIR}/boombox.tar.gz" -C "${TMP_DIR}"

# Install binary
install -m 755 "${TMP_DIR}/boombox-rs" "${BIN_DIR}/boombox-rs"
ln -sf "${BIN_DIR}/boombox-rs" "${BIN_DIR}/boombox"

# 5. Download & Install Desktop Entry and Icons
echo -e "${CYAN}🎨 Installing desktop entries and icons...${NC}"
RAW_BASE="https://raw.githubusercontent.com/${REPO}/main"

curl -sSL "${RAW_BASE}/boombox-toggle" -o "${BIN_DIR}/boombox-toggle" && chmod +x "${BIN_DIR}/boombox-toggle"
curl -sSL "${RAW_BASE}/assets/boombox.desktop" -o "${DESKTOP_DIR}/boombox.desktop"
curl -sSL "${RAW_BASE}/assets/boombox.desktop" -o "${DESKTOP_DIR}/org.omarchy.boombox.desktop"

# Download scalable SVG icons
mkdir -p "${ICON_DIR}/scalable/apps"
curl -sSL "${RAW_BASE}/assets/icons/hicolor/scalable/apps/boombox.svg" -o "${ICON_DIR}/scalable/apps/boombox.svg" 2>/dev/null || true
curl -sSL "${RAW_BASE}/assets/icons/hicolor/scalable/apps/boombox-tray.svg" -o "${ICON_DIR}/scalable/apps/boombox-tray.svg" 2>/dev/null || true
curl -sSL "${RAW_BASE}/assets/icons/hicolor/scalable/apps/boombox-tray-playing.svg" -o "${ICON_DIR}/scalable/apps/boombox-tray-playing.svg" 2>/dev/null || true
curl -sSL "${RAW_BASE}/assets/icons/hicolor/scalable/apps/boombox-tray-paused.svg" -o "${ICON_DIR}/scalable/apps/boombox-tray-paused.svg" 2>/dev/null || true

# Download standard PNG icons
for size in 16 24 32 48 64 128 256 512; do
    mkdir -p "${ICON_DIR}/${size}x${size}/apps"
    for icon in "boombox.png" "boombox-tray.png" "boombox-tray-playing.png" "boombox-tray-paused.png"; do
        curl -sSL "${RAW_BASE}/assets/icons/hicolor/${size}x${size}/apps/${icon}" -o "${ICON_DIR}/${size}x${size}/apps/${icon}" 2>/dev/null || true
    done
done

# Update desktop icon cache if available
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$DESKTOP_DIR" 2>/dev/null || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -q -t "${HOME}/.local/share/icons/hicolor" 2>/dev/null || true
fi

# Ensure ~/.local/bin is in PATH
if [[ ":$PATH:" != *":$BIN_DIR:"* ]]; then
    echo -e "\n${GOLD}⚠️  Notice: ${BIN_DIR} is not in your current PATH.${NC}"
    echo -e "   Add this line to your ~/.bashrc or ~/.zshrc:"
    echo -e "   ${CYAN}export PATH=\"\$HOME/.local/bin:\$PATH\"${NC}"
fi

echo -e "\n${GREEN}${BOLD}✨ BOOMBOX RX-505 installed successfully!${NC}"
echo -e "${GOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "  ▶ Launch via CLI:     ${CYAN}boombox${NC} or ${CYAN}boombox-rs${NC}"
echo -e "  ▶ Toggle Scratchpad:  ${CYAN}boombox-toggle${NC}"
echo -e "  ▶ Desktop Launcher:   ${CYAN}Boombox RX-505${NC} in your Application Menu"
echo -e "${GOLD}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}\n"
