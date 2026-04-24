#!/usr/bin/env bash
set -euo pipefail

BUILD_DIR="${BUILD_DIR:-/usr/local/share/crawlds}"
QS_DIR="${QS_DIR:-$HOME/.config/quickshell/crawldesktopshell}"
SOURCE_URL="https://github.com/me-osano/crawlds.git"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

log_info() { echo -e "${CYAN}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[SUCCESS]${NC} $1"; } 
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }
log_step() { echo -e "${BLUE}[STEP]${NC} $1"; }

BANNER="
    █████████  ███████████     █████████   █████   ███   █████ █████       ██████████    █████████
  ███░░░░░███░░███░░░░░███   ███░░░░░███ ░░███   ░███  ░░███ ░░███       ░░███░░░░███  ███░░░░░███
 ███     ░░░  ░███    ░███  ░███    ░███  ░███   ░███   ░███  ░███        ░███   ░░███░███    ░░░ 
░███          ░██████████   ░███████████  ░███   ░███   ░███  ░███        ░███    ░███░░█████████ 
░███          ░███░░░░░███  ░███░░░░░███  ░░███  █████  ███   ░███        ░███    ░███ ░░░░░░░░███
░░███     ███ ░███    ░███  ░███    ░███   ░░░█████░█████░    ░███      █ ░███    ███  ███    ░███
░░█████████  █████   █████ █████   █████    ░░███ ░░███      ███████████ ██████████  ░░█████████ 
░░░░░░░░░  ░░░░░   ░░░░░ ░░░░░   ░░░░░      ░░░   ░░░      ░░░░░░░░░░░  ░░░░░░░░░░    ░░░░░░░░░  
"

# ======= Clone source ===================
setup_temp_source() {
    TMP_DIR=$(mktemp -d)
    export TMP_DIR
    
    log_info "Cloning CrawlDS repo..."
    if ! command -v git >/dev/null; then
        sudo pacman -S --needed git
    fi
    
    git clone --depth 1 "$SOURCE_URL" "$TMP_DIR" >/dev/null 2>&1 || {
        log_error "Failed to clone repository"
        exit 1
    }
}

cleanup() {
    [[ -n "${TMP_DIR:-}" ]] && rm -rf "$TMP_DIR"
}
trap cleanup EXIT

# =========== Install dependencies ===============
install_deps() {    
    log_info "Installing dependencies..."
    if command -v pacman >/dev/null; then
        log_info "Installing dependencies..."
        sudo pacman -S --needed rustup quickshell qt6ct qt6-multimedia imagemagick gum
    else
        log_info "Install dependencies manually: git rust quickshell imagemagick"
    fi
}

install_choice() {
    local prompt="$1"; shift   # e.g. "Window Manager"
    log_step "Select $prompt:"
    local choice
    if command -v gum >/dev/null; then
        choice=$(gum choose "$@" "skip")
    else
        options=("$@" "skip")
        for i in "${!options[@]}"; do
            echo "$((i+1))) ${options[$i]}"
        done
        while true; do
            read -rp "Enter choice: " input
            if [[ "$input" =~ ^[0-9]+$ ]] && (( input >= 1 && input <= ${#options[@]} )); then
                choice="${options[$((input-1))]}"
                break
            else
                echo "Invalid choice"
            fi
        done
    fi
    
    if [[ "$choice" != "skip" ]]; then
        log_info "Installing $choice..."
        sudo pacman -S --needed "$choice"
        # niri needs xwayland-satellite alongside it
        [[ "$choice" == "niri" ]] && sudo pacman -S --needed xwayland-satellite
    fi
}

# ========== Building crawlds daemon ==============
install_daemon() {
    log_info "Installing crawlds daemon..."

    if ! command -v cargo >/dev/null; then
        rustup default stable
    fi

    if [[ ! -d "$TMP_DIR/core" ]]; then
        log_error "Core directory not found in repository"
        exit 1
    fi

    cd "$TMP_DIR/core"

    export RUSTUP_TOOLCHAIN=stable
    export CARGO_TARGET_DIR="$BUILD_DIR/core/target"

    log_step "Building crawlds (this may take a few minutes)..."
    if ! cargo build --release --workspace --bins 2>&1; then
        log_error "Build failed"
        exit 1
    fi

    if [[ ! -f "$CARGO_TARGET_DIR/release/crawlds-daemon" ]]; then
        log_error "Build failed - daemon not found"
        exit 1
    fi

    log_info "Installing crawlds-daemon binaries..."
    sudo install -Dm755 "$CARGO_TARGET_DIR/release/crawlds-daemon" /usr/local/bin/crawlds-daemon
    sudo install -Dm755 "$CARGO_TARGET_DIR/release/crawlds" /usr/local/bin/crawlds
    
    log_info "Copying default config"
    mkdir -p ~/.config/crawlds
    cp assets/config/core.toml ~/.config/crawlds/core.toml
    
    log_info "Setting up crawlds service"
    mkdir -p ~/.config/systemd/user
    cp assets/systemd/crawlds.service ~/.config/systemd/user/
    systemctl --user enable --now crawlds || true

    log_success "crawlds daemon installed and running!"
}

# ======== Setup Quickshell ========
install_quickshell() {
    log_info "Installing Crawl Desktop Shell(Quickshell)..."

    if command -v qs >/dev/null; then
        log_info "Installing Quickshell UI..."
        rm -rf "$QS_DIR"
        mkdir -p "$(dirname "$QS_DIR")"
        cp -r "$TMP_DIR/quickshell" "$QS_DIR"
        
        log_info "Quickshell shell installed!"
        echo
        echo "Run with: crawlds run"
    else
        log_info "Skipping shell (qs not found)"
    fi
}

echo -e "${BANNER}"
echo -e "  CrawlDS Installation"
echo "========================================="
echo

setup_temp_source
install_deps
install_choice "Window Manager" "niri" "hyprland"
install_choice "Terminal"       "ghostty" "alacritty" "kitty"
install_daemon
install_quickshell

echo
echo "========================================="
echo -e "  ${GREEN}Installation Complete!${NC}"
echo "========================================="
echo
echo "Run shell: crawlds run"