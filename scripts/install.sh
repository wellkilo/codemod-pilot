#!/usr/bin/env bash
# codemod-pilot installer
# Usage: curl -fsSL https://raw.githubusercontent.com/codemod-pilot/codemod-pilot/main/scripts/install.sh | sh
#
# Environment variables:
#   CODEMOD_PILOT_VERSION  - Specific version to install (default: latest)
#   CODEMOD_PILOT_DIR      - Installation directory (default: ~/.codemod-pilot/bin)

set -euo pipefail

# --- Configuration ---

REPO="codemod-pilot/codemod-pilot"
BINARY_NAME="codemod-pilot"
DEFAULT_INSTALL_DIR="${HOME}/.codemod-pilot/bin"
INSTALL_DIR="${CODEMOD_PILOT_DIR:-$DEFAULT_INSTALL_DIR}"
GITHUB_API="https://api.github.com"
GITHUB_RELEASES="https://github.com/${REPO}/releases"

# --- Colors ---

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# --- Functions ---

info() {
    printf "${BLUE}info${NC}: %s\n" "$1"
}

success() {
    printf "${GREEN}success${NC}: %s\n" "$1"
}

warn() {
    printf "${YELLOW}warn${NC}: %s\n" "$1"
}

error() {
    printf "${RED}error${NC}: %s\n" "$1" >&2
    exit 1
}

detect_os() {
    local os
    os="$(uname -s)"
    case "$os" in
        Linux*)  echo "unknown-linux-gnu" ;;
        Darwin*) echo "apple-darwin" ;;
        MINGW*|MSYS*|CYGWIN*) echo "pc-windows-msvc" ;;
        *)       error "Unsupported operating system: $os" ;;
    esac
}

detect_arch() {
    local arch
    arch="$(uname -m)"
    case "$arch" in
        x86_64|amd64)  echo "x86_64" ;;
        aarch64|arm64) echo "aarch64" ;;
        *)             error "Unsupported architecture: $arch" ;;
    esac
}

get_latest_version() {
    local url="${GITHUB_API}/repos/${REPO}/releases/latest"
    if command -v curl &>/dev/null; then
        curl -fsSL "$url" | grep '"tag_name"' | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/'
    elif command -v wget &>/dev/null; then
        wget -qO- "$url" | grep '"tag_name"' | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/'
    else
        error "Neither curl nor wget found. Please install one of them."
    fi
}

download() {
    local url="$1"
    local output="$2"
    if command -v curl &>/dev/null; then
        curl -fsSL "$url" -o "$output"
    elif command -v wget &>/dev/null; then
        wget -q "$url" -O "$output"
    else
        error "Neither curl nor wget found. Please install one of them."
    fi
}

verify_checksum() {
    local file="$1"
    local checksums_url="$2"
    local expected_name="$3"

    info "Verifying checksum..."
    local checksums_file
    checksums_file="$(mktemp)"
    download "$checksums_url" "$checksums_file"

    local expected_hash
    expected_hash="$(grep "$expected_name" "$checksums_file" | awk '{print $1}')"
    rm -f "$checksums_file"

    if [ -z "$expected_hash" ]; then
        warn "Could not find checksum for $expected_name, skipping verification"
        return 0
    fi

    local actual_hash
    if command -v sha256sum &>/dev/null; then
        actual_hash="$(sha256sum "$file" | awk '{print $1}')"
    elif command -v shasum &>/dev/null; then
        actual_hash="$(shasum -a 256 "$file" | awk '{print $1}')"
    else
        warn "sha256sum/shasum not found, skipping checksum verification"
        return 0
    fi

    if [ "$actual_hash" != "$expected_hash" ]; then
        error "Checksum mismatch! Expected: $expected_hash, Got: $actual_hash"
    fi

    success "Checksum verified"
}

install_binary() {
    local archive_path="$1"
    local archive_name="$2"

    mkdir -p "$INSTALL_DIR"

    info "Extracting archive..."
    case "$archive_name" in
        *.tar.gz)
            tar xzf "$archive_path" -C "$INSTALL_DIR"
            ;;
        *.zip)
            unzip -o "$archive_path" -d "$INSTALL_DIR"
            ;;
        *)
            error "Unknown archive format: $archive_name"
            ;;
    esac

    chmod +x "${INSTALL_DIR}/${BINARY_NAME}"
    success "Installed ${BINARY_NAME} to ${INSTALL_DIR}/${BINARY_NAME}"
}

update_path() {
    if [[ ":$PATH:" == *":${INSTALL_DIR}:"* ]]; then
        return 0
    fi

    local shell_config=""
    case "$(basename "${SHELL:-/bin/sh}")" in
        bash) shell_config="${HOME}/.bashrc" ;;
        zsh)  shell_config="${HOME}/.zshrc" ;;
        fish) shell_config="${HOME}/.config/fish/config.fish" ;;
        *)    shell_config="" ;;
    esac

    echo ""
    warn "${INSTALL_DIR} is not in your PATH."
    echo ""

    if [ -n "$shell_config" ]; then
        echo "  Add it by running:"
        echo ""
        if [[ "$shell_config" == *"fish"* ]]; then
            echo "    fish_add_path ${INSTALL_DIR}"
        else
            echo "    echo 'export PATH=\"${INSTALL_DIR}:\$PATH\"' >> ${shell_config}"
        fi
        echo "    source ${shell_config}"
    else
        echo "  Add ${INSTALL_DIR} to your PATH manually."
    fi
    echo ""
}

# --- Main ---

main() {
    echo ""
    echo "  codemod-pilot installer"
    echo "  ======================="
    echo ""

    # Detect platform
    local arch
    arch="$(detect_arch)"
    local os
    os="$(detect_os)"
    local target="${arch}-${os}"
    info "Detected platform: ${target}"

    # Get version
    local version="${CODEMOD_PILOT_VERSION:-}"
    if [ -z "$version" ]; then
        info "Fetching latest version..."
        version="$(get_latest_version)"
    fi

    if [ -z "$version" ]; then
        error "Could not determine version to install"
    fi

    info "Installing codemod-pilot ${version}..."

    # Determine archive format
    local ext="tar.gz"
    if [[ "$os" == "pc-windows-msvc" ]]; then
        ext="zip"
    fi

    local archive_name="${BINARY_NAME}-${version}-${target}.${ext}"
    local download_url="${GITHUB_RELEASES}/download/${version}/${archive_name}"
    local checksums_url="${GITHUB_RELEASES}/download/${version}/SHA256SUMS.txt"

    # Download
    info "Downloading ${archive_name}..."
    local tmp_dir
    tmp_dir="$(mktemp -d)"
    local archive_path="${tmp_dir}/${archive_name}"
    download "$download_url" "$archive_path"

    # Verify checksum
    verify_checksum "$archive_path" "$checksums_url" "$archive_name"

    # Install
    install_binary "$archive_path" "$archive_name"

    # Cleanup
    rm -rf "$tmp_dir"

    # Path guidance
    update_path

    # Verify installation
    if "${INSTALL_DIR}/${BINARY_NAME}" --version &>/dev/null; then
        local installed_version
        installed_version="$("${INSTALL_DIR}/${BINARY_NAME}" --version)"
        success "Installation complete! ${installed_version}"
    else
        success "Installation complete! Run '${BINARY_NAME} --version' to verify."
    fi

    echo ""
    echo "  Get started:"
    echo "    ${BINARY_NAME} --help"
    echo ""
}

main "$@"
