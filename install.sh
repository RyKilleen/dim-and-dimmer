#!/bin/bash
set -e

# Dim and Dimmer Installer
# Downloads the latest release and installs it system-wide

REPO="ryankilleen/dim-and-dimmer"
INSTALL_DIR="/usr/local/bin"
DESKTOP_DIR="/usr/share/applications"
ICON_DIR="/usr/share/icons/hicolor/scalable/apps"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

info() { echo -e "${GREEN}[INFO]${NC} $1"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
error() { echo -e "${RED}[ERROR]${NC} $1"; exit 1; }

# Check for root/sudo
check_privileges() {
    if [ "$EUID" -ne 0 ]; then
        error "This script requires root privileges. Please run with sudo."
    fi
}

# Check for required dependencies
check_dependencies() {
    info "Checking dependencies..."

    local missing=()

    if ! command -v ddcutil &> /dev/null; then
        missing+=("ddcutil")
    fi

    if ! command -v xrandr &> /dev/null; then
        missing+=("xrandr")
    fi

    if [ ${#missing[@]} -ne 0 ]; then
        warn "Missing dependencies: ${missing[*]}"
        echo ""
        echo "Install them using your package manager:"
        echo "  Debian/Ubuntu: sudo apt install ddcutil x11-xserver-utils"
        echo "  Fedora:        sudo dnf install ddcutil xrandr"
        echo "  Arch Linux:    sudo pacman -S ddcutil xorg-xrandr"
        echo ""
    else
        info "All dependencies found"
    fi
}

# Check i2c group membership for the invoking user
check_i2c_group() {
    local real_user="${SUDO_USER:-$USER}"

    if ! groups "$real_user" | grep -q '\bi2c\b'; then
        warn "User '$real_user' is not in the 'i2c' group"
        echo "  DDC-CI control may not work without i2c group membership."
        echo "  Add yourself to the group: sudo usermod -aG i2c $real_user"
        echo "  Then log out and back in for changes to take effect."
        echo ""
    fi
}

# Detect architecture
get_arch() {
    local arch=$(uname -m)
    case "$arch" in
        x86_64) echo "x86_64" ;;
        aarch64) echo "aarch64" ;;
        *) error "Unsupported architecture: $arch" ;;
    esac
}

# Download and install the binary
install_binary() {
    local arch=$(get_arch)
    local tmp_dir=$(mktemp -d)
    local binary_name="dim-and-dimmer-linux-${arch}"

    info "Downloading latest release for ${arch}..."

    # Get latest release URL from Codeberg API
    local release_url="https://codeberg.org/api/v1/repos/${REPO}/releases/latest"
    local download_url=$(curl -s "$release_url" | grep -o "https://codeberg.org/${REPO}/releases/download/[^\"]*${binary_name}[^\"]*" | head -1)

    if [ -z "$download_url" ]; then
        error "Could not find release for architecture: ${arch}"
    fi

    curl -L -o "${tmp_dir}/dim-and-dimmer" "$download_url"
    chmod +x "${tmp_dir}/dim-and-dimmer"

    info "Installing binary to ${INSTALL_DIR}..."
    mv "${tmp_dir}/dim-and-dimmer" "${INSTALL_DIR}/dim-and-dimmer"

    rm -rf "$tmp_dir"
}

# Install desktop file
install_desktop_file() {
    info "Installing desktop file..."

    mkdir -p "$DESKTOP_DIR"

    # Get script directory to find assets
    local script_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    local desktop_src="${script_dir}/assets/dim-and-dimmer.desktop"

    if [ -f "$desktop_src" ]; then
        cp "$desktop_src" "${DESKTOP_DIR}/dim-and-dimmer.desktop"
    else
        # Download from repo for curl-based installation
        curl -fsSL "https://codeberg.org/${REPO}/raw/branch/main/assets/dim-and-dimmer.desktop" \
            -o "${DESKTOP_DIR}/dim-and-dimmer.desktop"
    fi

    # Update desktop database if available
    if command -v update-desktop-database &> /dev/null; then
        update-desktop-database "$DESKTOP_DIR" 2>/dev/null || true
    fi
}

# Uninstall function
uninstall() {
    info "Uninstalling Dim and Dimmer..."

    rm -f "${INSTALL_DIR}/dim-and-dimmer"
    rm -f "${DESKTOP_DIR}/dim-and-dimmer.desktop"
    rm -f "${ICON_DIR}/dim-and-dimmer.svg"

    if command -v update-desktop-database &> /dev/null; then
        update-desktop-database "$DESKTOP_DIR" 2>/dev/null || true
    fi

    info "Uninstallation complete"
}

# Main
main() {
    echo "================================"
    echo "  Dim and Dimmer Installer"
    echo "================================"
    echo ""

    if [ "$1" = "--uninstall" ] || [ "$1" = "-u" ]; then
        check_privileges
        uninstall
        exit 0
    fi

    check_privileges
    check_dependencies
    check_i2c_group
    install_binary
    install_desktop_file

    echo ""
    info "Installation complete!"
    echo "  You can now run 'dim-and-dimmer' or find it in your application menu."
}

main "$@"
