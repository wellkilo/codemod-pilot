#!/usr/bin/env bash
# codemod-pilot development environment setup
# Usage: ./scripts/setup-dev.sh

set -euo pipefail

# --- Colors ---

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
BLUE='\033[0;34m'
NC='\033[0m'

info() {
    printf "${BLUE}[setup]${NC} %s\n" "$1"
}

success() {
    printf "${GREEN}[setup]${NC} %s\n" "$1"
}

warn() {
    printf "${YELLOW}[setup]${NC} %s\n" "$1"
}

error() {
    printf "${RED}[setup]${NC} %s\n" "$1" >&2
    exit 1
}

# --- Checks ---

check_command() {
    local cmd="$1"
    local install_hint="$2"
    if ! command -v "$cmd" &>/dev/null; then
        error "'$cmd' is not installed. $install_hint"
    fi
}

# --- Main ---

main() {
    echo ""
    echo "  codemod-pilot development setup"
    echo "  ================================"
    echo ""

    # Check prerequisites
    info "Checking prerequisites..."

    check_command "git" "Install git: https://git-scm.com/"

    if ! command -v rustup &>/dev/null; then
        warn "rustup not found. Installing..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
        source "${HOME}/.cargo/env"
        success "rustup installed"
    else
        success "rustup found: $(rustup --version 2>/dev/null | head -1)"
    fi

    # Install/update stable toolchain
    info "Installing stable Rust toolchain..."
    rustup toolchain install stable
    rustup default stable
    success "Rust $(rustc --version | awk '{print $2}') installed"

    # Install components
    info "Installing required components..."
    rustup component add rustfmt clippy
    success "rustfmt and clippy installed"

    # Install optional development tools
    info "Installing development tools..."

    if ! command -v cargo-insta &>/dev/null; then
        info "Installing cargo-insta (snapshot testing)..."
        cargo install cargo-insta
        success "cargo-insta installed"
    else
        success "cargo-insta already installed"
    fi

    if ! command -v cargo-watch &>/dev/null; then
        info "Installing cargo-watch (auto-rebuild on save)..."
        cargo install cargo-watch
        success "cargo-watch installed"
    else
        success "cargo-watch already installed"
    fi

    if ! command -v cargo-nextest &>/dev/null; then
        info "Installing cargo-nextest (better test runner)..."
        cargo install cargo-nextest
        success "cargo-nextest installed"
    else
        success "cargo-nextest already installed"
    fi

    # Check C compiler (needed for tree-sitter)
    info "Checking C compiler (required for tree-sitter)..."
    if command -v cc &>/dev/null; then
        success "C compiler found: $(cc --version 2>/dev/null | head -1)"
    elif command -v gcc &>/dev/null; then
        success "C compiler found: $(gcc --version 2>/dev/null | head -1)"
    elif command -v clang &>/dev/null; then
        success "C compiler found: $(clang --version 2>/dev/null | head -1)"
    else
        warn "No C compiler found. Tree-sitter grammars require a C compiler."
        warn "Install one with:"
        warn "  Ubuntu/Debian: sudo apt-get install build-essential"
        warn "  macOS: xcode-select --install"
        warn "  Fedora: sudo dnf install gcc"
    fi

    # Build the workspace
    echo ""
    info "Building workspace..."
    if cargo build --workspace 2>&1; then
        success "Build successful"
    else
        error "Build failed. Check the output above for errors."
    fi

    # Run tests
    info "Running tests..."
    if cargo test --workspace 2>&1; then
        success "All tests passed"
    else
        warn "Some tests failed. This may be expected for a fresh checkout."
    fi

    # Run lints
    info "Running clippy..."
    if cargo clippy --workspace --all-targets -- -D warnings 2>&1; then
        success "No clippy warnings"
    else
        warn "Clippy found some warnings. Please fix them before committing."
    fi

    # Check formatting
    info "Checking formatting..."
    if cargo fmt --all -- --check 2>&1; then
        success "Code is properly formatted"
    else
        warn "Some files need formatting. Run 'cargo fmt --all' to fix."
    fi

    # Summary
    echo ""
    echo "  ==============================="
    echo "  Development setup complete!"
    echo "  ==============================="
    echo ""
    echo "  Useful commands:"
    echo "    cargo build --workspace        Build all crates"
    echo "    cargo test --workspace         Run all tests"
    echo "    cargo run -p codemod-cli -- learn --before 'foo()' --after 'bar()'"
    echo "    cargo watch -x 'test -p codemod-core'   Auto-run tests on save"
    echo "    cargo insta test --workspace   Run snapshot tests"
    echo "    cargo insta review             Review snapshot changes"
    echo "    cargo clippy --workspace       Run lints"
    echo "    cargo fmt --all                Format all code"
    echo ""
}

main "$@"
