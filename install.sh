#!/usr/bin/env bash
set -euo pipefail

REPO="SushanthK07/ytmusic"
INSTALL_DIR="${INSTALL_DIR:-/usr/local/bin}"
BINARY="ytmusic"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

info()  { echo -e "${CYAN}[info]${NC}  $*"; }
ok()    { echo -e "${GREEN}[ok]${NC}    $*"; }
warn()  { echo -e "${YELLOW}[warn]${NC}  $*"; }
error() { echo -e "${RED}[error]${NC} $*"; exit 1; }

detect_platform() {
    local os arch
    os="$(uname -s)"
    arch="$(uname -m)"

    case "$os" in
        Linux*)  os="linux" ;;
        Darwin*) os="macos" ;;
        MINGW*|MSYS*|CYGWIN*) os="windows" ;;
        *) error "Unsupported OS: $os" ;;
    esac

    case "$arch" in
        x86_64|amd64)  arch="x86_64" ;;
        aarch64|arm64) arch="aarch64" ;;
        *) error "Unsupported architecture: $arch" ;;
    esac

    local suffix=""
    if [ "$os" = "windows" ]; then
        suffix=".exe"
    fi

    echo "${BINARY}-${os}-${arch}${suffix}"
}

get_latest_version() {
    curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
        | grep '"tag_name"' \
        | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/'
}

install_binary() {
    local asset="$1"
    local version="$2"
    local url="https://github.com/${REPO}/releases/download/${version}/${asset}"

    info "Downloading ${BINARY} ${version} for $(uname -s)/$(uname -m)..."
    local tmp
    tmp="$(mktemp)"
    if ! curl -fsSL -o "$tmp" "$url"; then
        error "Download failed. Check https://github.com/${REPO}/releases for available builds."
    fi

    chmod +x "$tmp"

    if [ -w "$INSTALL_DIR" ]; then
        mv "$tmp" "${INSTALL_DIR}/${BINARY}"
    else
        info "Installing to ${INSTALL_DIR} (requires sudo)..."
        sudo mv "$tmp" "${INSTALL_DIR}/${BINARY}"
    fi

    ok "Installed ${BINARY} to ${INSTALL_DIR}/${BINARY}"
}

check_dependency() {
    local cmd="$1"
    local desc="$2"
    if command -v "$cmd" &>/dev/null; then
        ok "$cmd found: $(command -v "$cmd")"
        return 0
    else
        warn "$cmd not found — $desc"
        return 1
    fi
}

install_dependencies() {
    local missing=0
    echo ""
    info "Checking dependencies..."
    check_dependency mpv "audio playback engine" || missing=1
    check_dependency yt-dlp "YouTube stream extractor" || missing=1
    echo ""

    if [ "$missing" -eq 0 ]; then
        return
    fi

    info "Installing missing dependencies..."

    if command -v brew &>/dev/null; then
        brew install mpv yt-dlp
    elif command -v apt-get &>/dev/null; then
        sudo apt-get update -qq
        sudo apt-get install -y -qq mpv
        if ! command -v yt-dlp &>/dev/null; then
            if command -v pip3 &>/dev/null; then
                pip3 install --user yt-dlp
            elif command -v pipx &>/dev/null; then
                pipx install yt-dlp
            else
                warn "Install yt-dlp manually: https://github.com/yt-dlp/yt-dlp#installation"
            fi
        fi
    elif command -v pacman &>/dev/null; then
        sudo pacman -S --noconfirm mpv yt-dlp
    elif command -v dnf &>/dev/null; then
        sudo dnf install -y mpv
        pip3 install --user yt-dlp
    elif command -v scoop &>/dev/null; then
        scoop install mpv yt-dlp
    else
        echo ""
        warn "Could not auto-install dependencies. Please install manually:"
        warn "  mpv:    https://mpv.io/installation"
        warn "  yt-dlp: https://github.com/yt-dlp/yt-dlp#installation"
    fi

    echo ""
    check_dependency mpv "https://mpv.io" || true
    check_dependency yt-dlp "https://github.com/yt-dlp/yt-dlp" || true
}

main() {
    echo ""
    echo -e "${CYAN}  ♫ ytmusic installer${NC}"
    echo ""

    local asset version
    asset="$(detect_platform)"
    version="$(get_latest_version)"

    if [ -z "$version" ]; then
        error "Could not determine latest version. Check https://github.com/${REPO}/releases"
    fi

    install_binary "$asset" "$version"
    install_dependencies

    echo ""
    echo -e "${GREEN}  ✓ Ready! Run '${BINARY}' to start.${NC}"
    echo ""
}

main "$@"
