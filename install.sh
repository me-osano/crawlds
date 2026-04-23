#!/usr/bin/env bash
set -euo pipefail

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${GREEN}[INFO]${NC} $1"; }
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

usage() {
    cat <<EOF
CrawlDS Installation Script

Usage: curl -fsSL https://raw.githubusercontent.com/me-osano/crawlds/master/install.sh | sh [OPTIONS]

OPTIONS:
    --daemon-only         Install only the Rust daemon (crawlds)
    --shell-only          Install only the Quickshell shell
    --all                 Install both daemon and shell (default)
    --install-dir PATH    Installation directory for daemon (default: ~/.local/share/crawlds)
    --qs-dir PATH         Quickshell config directory (default: ~/.config/quickshell/crawldesktopshell)
    --system              Install daemon system-wide (requires sudo)
    --enable              Enable and start the daemon service
    --branch BRANCH       Git branch to install from (default: master)
    --window-manager WM    Install window manager deps (niri|hyprland)
    --terminal TERM       Install terminal deps (ghostty|alacritty|kitty)
    -h, --help            Show this help message


EXAMPLES:
    curl -fsSL https://raw.githubusercontent.com/me-osano/crawlds/master/install.sh | sh
    curl -fsSL https://raw.githubusercontent.com/me-osano/crawlds/master/install.sh | sh --system --enable
    curl -fsSL https://raw.githubusercontent.com/me-osano/crawlds/master/install.sh | sh --window-manager niri --terminal ghostty
EOF
    exit 0
}

DAEMON_ONLY=false
SHELL_ONLY=false
ALL=true
SYSTEM_WIDE=false
ENABLE_SERVICE=false
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/share/crawlds}"
QS_INSTALL_DIR="${QS_INSTALL_DIR:-$HOME/.config/quickshell/crawldesktopshell}"
BRANCH="master"
SOURCE_URL="https://github.com/me-osano/crawlds.git"
WINDOW_MANAGER=""
TERMINAL=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --daemon-only) DAEMON_ONLY=true; ALL=false; shift ;;
        --shell-only) SHELL_ONLY=true; ALL=false; shift ;;
        --all) ALL=true; shift ;;
        --install-dir) INSTALL_DIR="$2"; shift 2 ;;
        --qs-dir) QS_INSTALL_DIR="$2"; shift 2 ;;
        --system) SYSTEM_WIDE=true; shift ;;
        --enable) ENABLE_SERVICE=true; shift ;;
        --branch) BRANCH="$2"; shift 2 ;;
        --window-manager) WINDOW_MANAGER="$2"; shift 2 ;;
        --terminal) TERMINAL="$2"; shift 2 ;;
        -h|--help) usage ;;
        *) log_error "Unknown option: $1"; exit 1 ;;
    esac
done

if [[ $ALL == true ]]; then
    DAEMON_ONLY=false
    SHELL_ONLY=false
fi

if [[ -n "${INSTALL_DIR:-}" && ! -d "$(dirname "$INSTALL_DIR")" ]]; then
    log_error "Directory does not exist: $(dirname "$INSTALL_DIR")"
    exit 1
fi

if [[ -n "${QS_INSTALL_DIR:-}" && ! -d "$(dirname "$QS_INSTALL_DIR")" ]]; then
    log_error "Directory does not exist: $(dirname "$QS_INSTALL_DIR")"
    exit 1
fi

check_command() {
    command -v "$1" >/dev/null 2>&1
}

get_pkg_manager() {
    if check_command pacman; then
        echo "pacman"
    elif check_command apt; then
        echo "apt"
    elif check_command dnf; then
        echo "dnf"
    else
        echo ""
    fi
}

run_pkg_install() {
    local pkgs=("$@")
    if [[ ${#pkgs[@]} -eq 0 ]]; then
        return
    fi

    local pm
    pm=$(get_pkg_manager)

    case "$pm" in
        pacman)
            if [[ $EUID -eq 0 ]]; then
                pacman -S --needed "${pkgs[@]}"
            else
                sudo pacman -S --needed "${pkgs[@]}" 2>/dev/null || \
                    log_warn "Could not auto-install. Please install manually: pacman -S ${pkgs[*]}"
            fi
            ;;
        apt)
            if [[ $EUID -eq 0 ]]; then
                apt-get install -y "${pkgs[@]}"
            else
                sudo apt-get install -y "${pkgs[@]}" 2>/dev/null || \
                    log_warn "Could not auto-install. Please install manually: apt-get install -y ${pkgs[*]}"
            fi
            ;;
        dnf)
            if [[ $EUID -eq 0 ]]; then
                dnf install -y "${pkgs[@]}"
            else
                sudo dnf install -y "${pkgs[@]}" 2>/dev/null || \
                    log_warn "Could not auto-install. Please install manually: dnf install -y ${pkgs[*]}"
            fi
            ;;
        *)
            log_warn "Could not detect package manager. Please install manually: ${pkgs[*]}"
            ;;
    esac
}

setup_temp_source() {
    if [[ -n "${SCRIPT_DIR:-}" ]]; then
        return 0
    fi

    if [[ -d "core" && -d "quickshell" ]]; then
        SCRIPT_DIR="$(pwd)"
        return 0
    fi

    SCRIPT_DIR=$(mktemp -d)
    CLEANUP_NEEDED=true

    log_info "Downloading CrawlDS..."
    git clone --depth 1 --branch "$BRANCH" "$SOURCE_URL" "$SCRIPT_DIR" >/dev/null 2>&1 || {
        log_error "Failed to clone repository"
        exit 1
    }
}

cleanup() {
    if [[ "${CLEANUP_NEEDED:-false}" == "true" ]]; then
        rm -rf "$SCRIPT_DIR"
    fi
}
trap cleanup EXIT

install_dependencies() {
    log_info "Checking dependencies..."

    local missing=()

    if ! check_command cargo; then
        missing+=("rust")
    fi

    if ! check_command git; then
        missing+=("git")
    fi
    
    if ! check_command qs; then
        missing+=("quickshell" "qt6ct" "qt6-multimedia")
    fi
    
    if ! check_command matugen; then
        missing+=("matugen")
    fi
    
    if ! check_command magick; then
        missing+=("imagemagick")
    fi

    if [[ ${#missing[@]} -gt 0 ]]; then
        log_info "Installing missing dependencies: ${missing[*]}"
        run_pkg_install "${missing[@]}"
    fi
}

install_wm() {
    case "$WINDOW_MANAGER" in
        niri)
            log_info "Installing Niri ..."
            local deps=("niri" "xwayland-satellite")
            install_pkg "${deps[@]}" ;;
        hyprland)
            log_info "Installing Hyprland dependencies..."
            local deps=("hyprland")
            install_pkg "${deps[@]}" ;;
        *)
            [[ -n "$WINDOW_MANAGER" ]] && log_warn "Unknown window manager: $WINDOW_MANAGER"
            ;;
    esac
}

install_terminal() {
    case "$TERMINAL" in
        ghostty)
            log_info "Installing Ghostty dependencies..."
            local deps=("ghostty")
            install_pkg "${deps[@]}" ;;
        alacritty)
            log_info "Installing Alacritty dependencies..."
            local deps=("alacritty")
            install_pkg "${deps[@]}" ;;
        kitty)
            log_info "Installing Kitty dependencies..."
            local deps=("kitty")
            install_pkg "${deps[@]}" ;;
        *)
            [[ -n "$TERMINAL" ]] && log_warn "Unknown terminal: $TERMINAL"
            ;;
    esac
}

install_pkg() {
    local pkgs=("$@")
    if [[ ${#pkgs[@]} -eq 0 ]]; then
        return
    fi
    run_pkg_install "${pkgs[@]}"
}

install_daemon() {
    log_info "Installing crawlds daemon..."

    if [[ ! -d "$SCRIPT_DIR/core" ]]; then
        log_error "Core directory not found in repository"
        exit 1
    fi

    cd "$SCRIPT_DIR/core"

    if [[ ! -f "Cargo.toml" ]]; then
        log_error "Cargo.toml not found in core/"
        exit 1
    fi

    export RUSTUP_TOOLCHAIN=stable
    export CARGO_TARGET_DIR="$INSTALL_DIR/target"

    log_step "Building crawlds (this may take a few minutes)..."
    if ! cargo build --release --workspace --bins 2>&1; then
        log_error "Build failed"
        exit 1
    fi

    if [[ ! -f "$CARGO_TARGET_DIR/release/crawlds-daemon" ]]; then
        log_error "Build failed - daemon not found"
        exit 1
    fi

    if [[ $SYSTEM_WIDE == true ]]; then
        log_info "Installing system-wide..."
        sudo install -Dm755 "$CARGO_TARGET_DIR/release/crawlds-daemon" /usr/local/bin/crawlds-daemon
        sudo install -Dm755 "$CARGO_TARGET_DIR/release/crawlds" /usr/local/bin/crawlds

        mkdir -p ~/.config/systemd/user
        cp assets/systemd/crawlds.service ~/.config/systemd/user/
    else
        log_info "Installing to user directory..."
        mkdir -p "$HOME/.local/bin"
        install -Dm755 "$CARGO_TARGET_DIR/release/crawlds-daemon" "$HOME/.local/bin/crawlds-daemon"
        install -Dm755 "$CARGO_TARGET_DIR/release/crawlds" "$HOME/.local/bin/crawlds"

        mkdir -p ~/.config/systemd/user
        cp assets/systemd/crawlds.service ~/.config/systemd/user/

        if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
            USER_BIN_DIR=$(realpath "$HOME/.local/bin")
            if [[ ":$PATH:" != *":$USER_BIN_DIR:"* ]]; then
                log_warn "$HOME/.local/bin not in PATH"
                echo -e "\nAdd to your shell profile (~/.bashrc or ~/.zshrc):"
                echo -e "  ${YELLOW}export PATH=\"\$HOME/.local/bin:\$PATH\"${NC}"
            fi
        fi
    fi
    
    log_info "Copying default config"
    mkdir -p ~/.config/crawlds
    cp assets/config/core.toml ~/.config/crawlds/core.toml

    log_info "crawlds daemon installed!"
}

enable_daemon_service() {
    log_info "Enabling crawlds service..."

    if systemctl --user list-unit-files 2>/dev/null | grep -q "crawlds.service"; then
        systemctl --user enable --now crawlds
        log_info "Service enabled and started"
    else
        log_warn "Service file not found in expected location"
        log_info "Enable manually: systemctl --user enable --now crawlds"
    fi
}

install_quickshell() {
    log_info "Installing Quickshell shell..."

    if [[ ! -d "$SCRIPT_DIR/quickshell" ]]; then
        log_error "Quickshell directory not found in repository"
        exit 1
    fi

    mkdir -p "$(dirname "$QS_INSTALL_DIR")"

    if [[ -L "$QS_INSTALL_DIR" ]] || [[ -d "$QS_INSTALL_DIR" ]]; then
        log_warn "Quickshell directory already exists: $QS_INSTALL_DIR"
        read -rp "Overwrite? [y/N] " -n 1 -r REPLY
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            log_info "Skipping shell installation."
            return 0
        fi
        rm -rf "$QS_INSTALL_DIR"
    fi

    log_info "Copying Quickshell shell to $QS_INSTALL_DIR..."
    cp -r "$SCRIPT_DIR/quickshell" "$QS_INSTALL_DIR"

    log_info "Quickshell shell installed!"
    echo
    echo "Run with: crawlds run"
}

check_quickshell() {
    if ! check_command qs; then
        log_warn "Quickshell (qs) not found in PATH"
        log_info "Install Quickshell first: https://quickshell.outfoxxed.me/"
        return 1
    fi
    return 0
}

check_daemon_running() {
    if systemctl is-active --user crawlds >/dev/null 2>&1; then
        log_info "crawlds daemon is running"
        return 0
    elif pgrep -x "crawlds-daemon" >/dev/null 2>&1; then
        log_info "crawlds daemon is running"
        return 0
    else
        log_warn "crawlds daemon is not running"
        return 1
    fi
}

main() {
    setup_temp_source

    echo -e "${BANNER}"
    echo -e "  CrawlDS Installation"
    echo "========================================="
    echo

    INTERACTIVE=false
    if [[ -z "$WINDOW_MANAGER" && -z "$TERMINAL" ]]; then
        INTERACTIVE=true
        install_pkg "gum"
    fi

    if [[ "$INTERACTIVE" == true ]]; then
        log_step "Select Window Manager:"
        WINDOW_MANAGER=$(gum choose "niri" "hyprland" "skip")
        if [[ "$WINDOW_MANAGER" == "skip" ]]; then
            WINDOW_MANAGER=""
        fi

        log_step "Select Terminal:"
        TERMINAL=$(gum choose "ghostty" "alacritty" "kitty" "skip")
        if [[ "$TERMINAL" == "skip" ]]; then
            TERMINAL=""
        fi
    fi

    if [[ -n "$WINDOW_MANAGER" ]]; then
        install_wm
    fi

    if [[ -n "$TERMINAL" ]]; then
        install_terminal
    fi

    if [[ $ALL == true || $DAEMON_ONLY == true ]]; then
        install_dependencies
        install_daemon
        if [[ $ENABLE_SERVICE == true ]]; then
            enable_daemon_service
        fi
    fi

    if [[ $ALL == true || $SHELL_ONLY == true ]]; then
        if check_quickshell; then
            install_quickshell
        else
            log_error "Cannot install shell without Quickshell"
            exit 1
        fi
    fi

    echo
    echo "========================================="
    echo -e "  ${GREEN}Installation Complete!${NC}"
    echo "========================================="
    echo

    if [[ $ALL == true || $DAEMON_ONLY == true ]]; then
        check_daemon_running || log_warn "Start daemon with: crawlds-daemon &"
    fi

    if [[ $ALL == true || $SHELL_ONLY == true ]]; then
        echo "Run shell: crawlds run"
    fi
}

main "$@"