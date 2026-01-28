#!/bin/sh
# Metis GUI Installer
# Usage: curl -fsSL https://raw.githubusercontent.com/colliery-io/metis/main/scripts/install.sh | bash
#
# Downloads and installs the Metis desktop application.
# The GUI includes the CLI which is installed on first launch.

set -e

# Configuration
GITHUB_REPO="colliery-io/metis"

# Colors (check if terminal supports colors)
if [ -t 1 ]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[0;33m'
    BLUE='\033[0;34m'
    BOLD='\033[1m'
    NC='\033[0m'
else
    RED=''
    GREEN=''
    YELLOW=''
    BLUE=''
    BOLD=''
    NC=''
fi

info() {
    printf "${BLUE}[INFO]${NC} %s\n" "$1"
}

success() {
    printf "${GREEN}[OK]${NC} %s\n" "$1"
}

warn() {
    printf "${YELLOW}[WARN]${NC} %s\n" "$1"
}

error() {
    printf "${RED}[ERROR]${NC} %s\n" "$1" >&2
    exit 1
}

# Detect OS
detect_os() {
    case "$(uname -s)" in
        Darwin*)
            OS="macos"
            ;;
        Linux*)
            OS="linux"
            ;;
        MINGW*|MSYS*|CYGWIN*)
            OS="windows"
            ;;
        *)
            error "Unsupported operating system: $(uname -s)"
            ;;
    esac
}

# Detect architecture
detect_arch() {
    case "$(uname -m)" in
        x86_64|amd64)
            ARCH="x64"
            ;;
        arm64|aarch64)
            ARCH="aarch64"
            ;;
        *)
            error "Unsupported architecture: $(uname -m)"
            ;;
    esac
}

# Check for required commands
check_dependencies() {
    for cmd in curl; do
        if ! command -v "$cmd" >/dev/null 2>&1; then
            error "Required command not found: $cmd"
        fi
    done

    # macOS needs hdiutil for dmg mounting
    if [ "$OS" = "macos" ] && ! command -v hdiutil >/dev/null 2>&1; then
        error "Required command not found: hdiutil"
    fi
}

# Get the latest release version from GitHub
get_latest_version() {
    if [ -n "${METIS_VERSION:-}" ]; then
        VERSION="$METIS_VERSION"
        info "Using specified version: $VERSION"
    else
        info "Fetching latest release version..."

        # Get latest release, handling the app-v prefix that tauri uses
        VERSION=$(curl -sSL "https://api.github.com/repos/${GITHUB_REPO}/releases" | \
            grep '"tag_name":' | grep 'app-v' | head -1 | sed -E 's/.*"app-v([^"]+)".*/\1/')

        if [ -z "$VERSION" ]; then
            error "Could not determine latest version. Set METIS_VERSION environment variable to specify a version (e.g., 1.0.0)."
        fi
        info "Latest version: $VERSION"
    fi
}

# Build asset name based on OS/arch
get_asset_name() {
    case "$OS" in
        macos)
            echo "Metis_${VERSION}_${ARCH}.dmg"
            ;;
        linux)
            echo "Metis_${VERSION}_amd64.AppImage"
            ;;
        windows)
            echo "Metis_${VERSION}_x64-setup.exe"
            ;;
    esac
}

# Install on macOS
install_macos() {
    local dmg_file="$1"
    local mount_point

    info "Mounting disk image..."
    mount_point=$(hdiutil attach "$dmg_file" -nobrowse -quiet | tail -1 | awk '{print $3}')

    if [ -z "$mount_point" ]; then
        # Try alternate parsing
        mount_point="/Volumes/Metis"
    fi

    info "Installing to /Applications..."

    # Remove existing installation
    if [ -d "/Applications/Metis.app" ]; then
        warn "Removing existing Metis installation..."
        rm -rf "/Applications/Metis.app"
    fi

    # Copy app
    cp -R "${mount_point}/Metis.app" /Applications/

    # Unmount
    info "Cleaning up..."
    hdiutil detach "$mount_point" -quiet 2>/dev/null || true

    # Remove quarantine from app
    info "Removing quarantine attributes..."
    xattr -rd com.apple.quarantine /Applications/Metis.app 2>/dev/null || true

    # Also fix CLI if previously installed by GUI
    local cli_path="$HOME/Library/Application Support/io.colliery.metis/bin/metis"
    if [ -f "$cli_path" ]; then
        xattr -rd com.apple.quarantine "$cli_path" 2>/dev/null || true
        xattr -d com.apple.provenance "$cli_path" 2>/dev/null || true
        codesign --force --sign - "$cli_path" 2>/dev/null || true
        success "Fixed CLI binary signature"
    fi

    success "Metis installed to /Applications/Metis.app"
}

# Install on Linux
install_linux() {
    local appimage_file="$1"
    local install_dir="$HOME/.local/bin"

    mkdir -p "$install_dir"

    info "Installing to ${install_dir}/metis..."
    cp "$appimage_file" "${install_dir}/metis"
    chmod +x "${install_dir}/metis"

    success "Metis installed to ${install_dir}/metis"

    # Check PATH
    case ":$PATH:" in
        *":${install_dir}:"*)
            ;;
        *)
            warn "${install_dir} is not in your PATH"
            echo ""
            echo "Add it to your shell configuration:"
            echo "  echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.bashrc"
            echo ""
            ;;
    esac
}

# Install on Windows (just download, user runs manually)
install_windows() {
    local exe_file="$1"
    local downloads_dir="$HOME/Downloads"

    mkdir -p "$downloads_dir"
    cp "$exe_file" "${downloads_dir}/Metis_Setup.exe"

    success "Installer downloaded to ${downloads_dir}/Metis_Setup.exe"
    echo ""
    echo "Please run the installer manually to complete installation."
}

# Main download and install
download_and_install() {
    local asset_name
    asset_name=$(get_asset_name)
    local download_url="https://github.com/${GITHUB_REPO}/releases/download/app-v${VERSION}/${asset_name}"

    local temp_dir
    temp_dir=$(mktemp -d)
    local download_path="${temp_dir}/${asset_name}"

    info "Downloading Metis ${VERSION} for ${OS}/${ARCH}..."
    info "URL: ${download_url}"

    if ! curl -fSL "$download_url" -o "$download_path" 2>/dev/null; then
        rm -rf "$temp_dir"
        error "Failed to download Metis. Check that version ${VERSION} exists for ${OS}/${ARCH}."
    fi

    # Verify download
    if [ ! -s "$download_path" ]; then
        rm -rf "$temp_dir"
        error "Downloaded file is empty"
    fi

    success "Downloaded ${asset_name}"

    # Install based on OS
    case "$OS" in
        macos)
            install_macos "$download_path"
            ;;
        linux)
            install_linux "$download_path"
            ;;
        windows)
            install_windows "$download_path"
            ;;
    esac

    # Cleanup
    rm -rf "$temp_dir"
}

# Print post-install info
print_info() {
    echo ""
    echo "${BOLD}Installation Complete${NC}"
    echo ""
    echo "The Metis desktop application has been installed."
    echo ""
    echo "${BOLD}What's Included:${NC}"
    echo "  - Visual kanban interface for project management"
    echo "  - Built-in CLI (installed/updated on GUI launch)"
    echo "  - MCP server for AI assistant integration"
    echo ""
    echo "${BOLD}Getting Started:${NC}"
    case "$OS" in
        macos)
            echo "  Open Metis from /Applications or Spotlight"
            echo ""
            echo "  On first launch, the CLI will be installed automatically."
            echo "  This enables terminal and AI assistant usage."
            ;;
        linux)
            echo "  Run: metis"
            echo ""
            echo "  Or add to your application menu."
            ;;
        windows)
            echo "  Run the installer, then launch Metis from the Start menu."
            ;;
    esac
    echo ""
    echo "${BOLD}${YELLOW}Important for Updates:${NC}"
    echo "  After updating, launch the Metis GUI at least once to update the CLI."
    echo "  Then restart Claude Code to pick up the new MCP server."
    echo ""
    echo "For more information: https://github.com/${GITHUB_REPO}"
}

# Main
main() {
    echo ""
    echo "=============================="
    echo "     Metis GUI Installer"
    echo "=============================="
    echo ""

    detect_os
    detect_arch
    check_dependencies
    get_latest_version
    download_and_install
    print_info

    echo ""
    success "Done!"
    echo ""
}

main "$@"
