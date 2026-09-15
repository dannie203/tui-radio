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
echo -e "${CYAN}📼 Automated Installer for Linux (v3.8.12+)${NC}\n"

# ------------------------------------------------------------------------------
# 1. Detect Architecture & OS
# ------------------------------------------------------------------------------
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

# Create target directories early
mkdir -p "$BIN_DIR" "$DESKTOP_DIR"

# ------------------------------------------------------------------------------
# 2. Check & Install Required Dependencies
# ------------------------------------------------------------------------------
echo -e "${CYAN}🔍 Checking & preparing system dependencies...${NC}"

# 2.1 Audio Engine: MPV (BẮT BUỘC)
if command -v mpv >/dev/null 2>&1; then
    echo -e "  ${GREEN}✓${NC} mpv (Audio engine & streaming decoder) found: $(command -v mpv)"
else
    echo -e "  ${GOLD}⚠${NC} mpv not found. Attempting automated installation via package manager..."
    
    PKG_MGR=""
    if command -v pacman >/dev/null 2>&1; then
        PKG_MGR="pacman"
    elif command -v apt-get >/dev/null 2>&1; then
        PKG_MGR="apt-get"
    elif command -v dnf >/dev/null 2>&1; then
        PKG_MGR="dnf"
    elif command -v zypper >/dev/null 2>&1; then
        PKG_MGR="zypper"
    elif command -v apk >/dev/null 2>&1; then
        PKG_MGR="apk"
    elif command -v xbps-install >/dev/null 2>&1; then
        PKG_MGR="xbps"
    fi

    SUDO=""
    if [ "${EUID:-$(id -u)}" -ne 0 ]; then
        if command -v sudo >/dev/null 2>&1; then
            SUDO="sudo"
        fi
    fi

    TTY_INPUT=""
    if [ -t 0 ]; then
        TTY_INPUT=""
    elif [ -e /dev/tty ]; then
        TTY_INPUT="</dev/tty"
    fi

    if [ -n "$PKG_MGR" ] && { [ "${EUID:-$(id -u)}" -eq 0 ] || [ -n "$SUDO" ]; }; then
        echo -e "  ${CYAN}▶ Installing 'mpv' via ${PKG_MGR}...${NC}"
        case "$PKG_MGR" in
            pacman)
                eval "${SUDO} pacman -S --noconfirm --needed mpv ${TTY_INPUT}" || true
                ;;
            apt-get)
                eval "${SUDO} apt-get update -y ${TTY_INPUT} && ${SUDO} apt-get install -y mpv ${TTY_INPUT}" || true
                ;;
            dnf)
                eval "${SUDO} dnf install -y mpv ${TTY_INPUT}" || true
                ;;
            zypper)
                eval "${SUDO} zypper install -y mpv ${TTY_INPUT}" || true
                ;;
            apk)
                eval "${SUDO} apk add mpv ${TTY_INPUT}" || true
                ;;
            xbps)
                eval "${SUDO} xbps-install -Sy mpv ${TTY_INPUT}" || true
                ;;
        esac
    fi

    if command -v mpv >/dev/null 2>&1; then
        echo -e "  ${GREEN}✓${NC} mpv installed successfully: $(command -v mpv)"
    else
        echo -e "  ${RED}❌ Could not automatically install 'mpv'.${NC}"
        echo -e "     ${GOLD}Please install 'mpv' manually (e.g. sudo pacman -S mpv / sudo apt install mpv).${NC}"
    fi
fi

# 2.2 Stream Helper: yt-dlp (Khuyến nghị cho YouTube & SoundCloud)
if command -v yt-dlp >/dev/null 2>&1 || [ -f "${BIN_DIR}/yt-dlp" ]; then
    echo -e "  ${GREEN}✓${NC} yt-dlp (Online stream extractor) found"
else
    echo -e "  ${CYAN}📥 Downloading standalone yt-dlp binary to ${BIN_DIR}/yt-dlp...${NC}"
    if curl -sSL "https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp" -o "${BIN_DIR}/yt-dlp"; then
        chmod +x "${BIN_DIR}/yt-dlp"
        echo -e "  ${GREEN}✓${NC} yt-dlp installed to ${BIN_DIR}/yt-dlp"
    else
        echo -e "  ${GOLD}⚠${NC} Could not auto-download yt-dlp. Online YouTube streams may be limited."
    fi
fi

# 2.3 Audio Transcoder: FFmpeg (Tùy chọn cho tính năng Tape Recorder)
if command -v ffmpeg >/dev/null 2>&1; then
    echo -e "  ${GREEN}✓${NC} ffmpeg (Audio recorder & format encoder) found: $(command -v ffmpeg)"
else
    echo -e "  ${GOLD}ℹ${NC} ffmpeg not detected. (Optional: install 'ffmpeg' to enable Cassette Tape Recording)."
fi

# ------------------------------------------------------------------------------
# 3. Install Boombox Binary (Local build or GitHub Release)
# ------------------------------------------------------------------------------
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" 2>/dev/null && pwd || true)"
LOCAL_BIN=""
if [ -n "$SCRIPT_DIR" ] && [ -f "${SCRIPT_DIR}/target/release/boombox-rs" ]; then
    LOCAL_BIN="${SCRIPT_DIR}/target/release/boombox-rs"
elif [ -n "$SCRIPT_DIR" ] && [ -f "${SCRIPT_DIR}/boombox-rs" ]; then
    LOCAL_BIN="${SCRIPT_DIR}/boombox-rs"
fi

if [ -n "$LOCAL_BIN" ]; then
    echo -e "\n${CYAN}📦 Installing Boombox-RS from local build: ${LOCAL_BIN}...${NC}"
    install -m 755 "$LOCAL_BIN" "${BIN_DIR}/boombox-rs"
    ln -sf "${BIN_DIR}/boombox-rs" "${BIN_DIR}/boombox"
    echo -e "  ${GREEN}✓${NC} Binary installed to ${BIN_DIR}/boombox-rs"
else
    TMP_DIR="$(mktemp -d)"
    cleanup() {
        rm -rf "$TMP_DIR"
    }
    trap cleanup EXIT

    RELEASE_URL="https://github.com/${REPO}/releases/latest/download/boombox-rs-linux-x86_64.tar.gz"

    echo -e "\n${CYAN}📥 Downloading latest Boombox-RS release from GitHub...${NC}"
    if command -v curl >/dev/null 2>&1; then
        curl -sSL "$RELEASE_URL" -o "${TMP_DIR}/boombox.tar.gz"
    elif command -v wget >/dev/null 2>&1; then
        wget -q "$RELEASE_URL" -O "${TMP_DIR}/boombox.tar.gz"
    else
        echo -e "${RED}❌ Neither curl nor wget was found. Please install curl or wget.${NC}"
        exit 1
    fi

    echo -e "${CYAN}📦 Extracting and installing binary...${NC}"
    tar -xzf "${TMP_DIR}/boombox.tar.gz" -C "${TMP_DIR}"

    # Install binary
    install -m 755 "${TMP_DIR}/boombox-rs" "${BIN_DIR}/boombox-rs"
    ln -sf "${BIN_DIR}/boombox-rs" "${BIN_DIR}/boombox"
fi

# ------------------------------------------------------------------------------
# 4. Download & Install Desktop Entry, Toggle Script, and Icons
# ------------------------------------------------------------------------------
echo -e "\n${CYAN}🎨 Installing desktop integration and icons...${NC}"
RAW_BASE="https://raw.githubusercontent.com/${REPO}/main"

# 4.1 boombox-toggle scratchpad script
if [ -n "$SCRIPT_DIR" ] && [ -f "${SCRIPT_DIR}/boombox-toggle" ]; then
    install -m 755 "${SCRIPT_DIR}/boombox-toggle" "${BIN_DIR}/boombox-toggle"
else
    curl -sSL "${RAW_BASE}/boombox-toggle" -o "${BIN_DIR}/boombox-toggle" && chmod +x "${BIN_DIR}/boombox-toggle"
fi

# 4.2 .desktop launchers
if [ -n "$SCRIPT_DIR" ] && [ -f "${SCRIPT_DIR}/assets/boombox.desktop" ]; then
    cp "${SCRIPT_DIR}/assets/boombox.desktop" "${DESKTOP_DIR}/boombox.desktop"
    cp "${SCRIPT_DIR}/assets/boombox.desktop" "${DESKTOP_DIR}/org.omarchy.boombox.desktop"
else
    curl -sSL "${RAW_BASE}/assets/boombox.desktop" -o "${DESKTOP_DIR}/boombox.desktop"
    curl -sSL "${RAW_BASE}/assets/boombox.desktop" -o "${DESKTOP_DIR}/org.omarchy.boombox.desktop"
fi

# 4.3 Icons (Local copy if in repo, otherwise GitHub download)
if [ -n "$SCRIPT_DIR" ] && [ -d "${SCRIPT_DIR}/assets/icons/hicolor" ]; then
    cp -r "${SCRIPT_DIR}/assets/icons/hicolor/"* "${ICON_DIR}/"
else
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
fi

# 4.4 Update icon and desktop cache
if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$DESKTOP_DIR" 2>/dev/null || true
fi
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -q -t "${HOME}/.local/share/icons/hicolor" 2>/dev/null || true
fi

# ------------------------------------------------------------------------------
# 5. Environment Check & Completion Banner
# ------------------------------------------------------------------------------
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
