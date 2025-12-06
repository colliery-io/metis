#!/bin/sh
# Metis Installer
# Usage: curl -sSL https://raw.githubusercontent.com/colliery-io/metis/main/scripts/install.sh | sh

set -e

# Configuration
GITHUB_REPO="colliery-io/metis"
# metis CLI includes MCP server via `metis mcp` subcommand
# metis-tui is the standalone terminal UI
BINARIES="metis metis-tui"
DEFAULT_INSTALL_DIR="$HOME/.local/bin"

# Colors (check if terminal supports colors)
if [ -t 1 ]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[0;33m'
    BLUE='\033[0;34m'
    BOLD='\033[1m'
    NC='\033[0m' # No Color
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
            OS="darwin"
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
            ARCH="x86_64"
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
    for cmd in curl tar; do
        if ! command -v "$cmd" >/dev/null 2>&1; then
            error "Required command not found: $cmd"
        fi
    done
}

# Get the latest release version from GitHub
get_latest_version() {
    if [ -n "${METIS_VERSION:-}" ]; then
        VERSION="$METIS_VERSION"
        info "Using specified version: $VERSION"
    else
        info "Fetching latest release version..."
        VERSION=$(curl -sSL "https://api.github.com/repos/${GITHUB_REPO}/releases/latest" | \
            grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

        if [ -z "$VERSION" ]; then
            error "Could not determine latest version. Set METIS_VERSION environment variable to specify a version."
        fi
        info "Latest version: $VERSION"
    fi
}

# Download and install all binaries
download_and_install() {
    local install_dir="${METIS_INSTALL_DIR:-$DEFAULT_INSTALL_DIR}"
    local temp_dir
    temp_dir=$(mktemp -d)

    # Create install directory if it doesn't exist
    mkdir -p "$install_dir"

    info "Downloading Metis ${VERSION} for ${OS}/${ARCH}..."

    # Download and install each binary
    for binary in $BINARIES; do
        # Expected asset name pattern: {binary}-{version}-{arch}-{os}.tar.gz
        local asset_name="${binary}-${VERSION}-${ARCH}-${OS}.tar.gz"
        local download_url="https://github.com/${GITHUB_REPO}/releases/download/${VERSION}/${asset_name}"

        info "Downloading ${binary}..."

        # Download the tarball
        if ! curl -sSL "$download_url" -o "${temp_dir}/${binary}.tar.gz" 2>/dev/null; then
            warn "Failed to download ${binary} - skipping (may not be available for this platform)"
            continue
        fi

        # Check if download was successful (file exists and is not empty)
        if [ ! -s "${temp_dir}/${binary}.tar.gz" ]; then
            warn "Empty download for ${binary} - skipping"
            continue
        fi

        # Extract
        if ! tar -xzf "${temp_dir}/${binary}.tar.gz" -C "$temp_dir" 2>/dev/null; then
            warn "Failed to extract ${binary} - skipping"
            continue
        fi

        # Find the binary in extracted files
        local binary_path
        binary_path=$(find "$temp_dir" -name "$binary" -type f 2>/dev/null | head -1)

        if [ -z "$binary_path" ]; then
            # Try to find executable files
            binary_path=$(find "$temp_dir" -name "${binary}*" -type f -perm +111 2>/dev/null | head -1)
        fi

        if [ -z "$binary_path" ]; then
            warn "Could not find ${binary} in archive - skipping"
            continue
        fi

        # Install the binary
        cp "$binary_path" "${install_dir}/${binary}"
        chmod +x "${install_dir}/${binary}"

        # macOS: Remove quarantine attribute
        if [ "$OS" = "darwin" ]; then
            xattr -d com.apple.quarantine "${install_dir}/${binary}" 2>/dev/null || true
        fi

        success "Installed ${binary}"

        # Clean up extracted files for this binary
        rm -f "${temp_dir}/${binary}.tar.gz"
        find "$temp_dir" -name "${binary}*" -delete 2>/dev/null || true
    done

    # Cleanup temp directory
    rm -rf "$temp_dir"

    # Store for PATH check
    INSTALL_DIR="$install_dir"
}

# Check if install directory is in PATH
check_path() {
    case ":$PATH:" in
        *":${INSTALL_DIR}:"*)
            success "${INSTALL_DIR} is already in your PATH"
            ;;
        *)
            warn "${INSTALL_DIR} is not in your PATH"
            echo ""
            echo "Add it to your shell configuration:"
            echo ""

            # Detect shell and provide appropriate instructions
            case "${SHELL##*/}" in
                bash)
                    echo "  echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.bashrc"
                    echo "  source ~/.bashrc"
                    ;;
                zsh)
                    echo "  echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.zshrc"
                    echo "  source ~/.zshrc"
                    ;;
                fish)
                    echo "  fish_add_path ~/.local/bin"
                    ;;
                *)
                    echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
                    ;;
            esac
            echo ""
            ;;
    esac
}

# Verify installation
verify_installation() {
    echo ""
    info "Verifying installation..."

    local installed_count=0
    for binary in $BINARIES; do
        if [ -x "${INSTALL_DIR}/${binary}" ]; then
            local version_output
            version_output=$("${INSTALL_DIR}/${binary}" --version 2>/dev/null || echo "installed")
            success "${binary}: ${version_output}"
            installed_count=$((installed_count + 1))
        fi
    done

    if [ "$installed_count" -eq 0 ]; then
        error "No binaries were installed successfully"
    fi
}

# Print usage information
print_usage() {
    echo ""
    echo "${BOLD}Installed Components:${NC}"
    echo ""
    echo "  ${BOLD}metis${NC}      - CLI with built-in MCP server (via 'metis mcp')"
    echo "  ${BOLD}metis-tui${NC}  - Interactive terminal user interface"
    echo ""
    echo "${BOLD}Quick Start:${NC}"
    echo ""
    echo "  # Initialize a new project"
    echo "  metis init"
    echo ""
    echo "  # Launch the TUI"
    echo "  metis-tui"
    echo ""
    echo "${BOLD}MCP Server Configuration (Claude Code):${NC}"
    echo ""
    echo "  Add to your MCP settings:"
    echo "  {\"mcpServers\": {\"metis\": {\"command\": \"${INSTALL_DIR}/metis\", \"args\": [\"mcp\"]}}}"
    echo ""
    echo "For more information: https://github.com/${GITHUB_REPO}"
}

# Main installation flow
main() {
    echo ""
    echo "=============================="
    echo "      Metis Installer"
    echo "=============================="
    echo ""

    check_dependencies
    detect_os
    detect_arch
    get_latest_version
    download_and_install
    check_path
    verify_installation
    print_usage

    echo ""
    success "Installation complete!"
    echo ""
}

main "$@"
