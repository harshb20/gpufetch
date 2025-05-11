#!/bin/bash
# Build script for gpufetch

set -e

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "Error: Rust and Cargo are required to build gpufetch."
    echo "Please install Rust from https://rustup.rs/"
    exit 1
fi

# Check for required packages
check_deps() {
    local missing=0

    echo "Checking for required dependencies..."

    # Check for libpci-dev
    if ! pkg-config --exists libpci 2>/dev/null; then
        echo "Warning: libpci development package not found"
        echo "On Debian/Ubuntu, install with: sudo apt install libpci-dev"
        echo "On Fedora/RHEL, install with: sudo dnf install pciutils-devel"
        echo "On Arch Linux, install with: sudo pacman -S pciutils"
        missing=1
    fi

    # Check for libglvnd
    if ! pkg-config --exists libglvnd 2>/dev/null; then
        echo "Warning: libglvnd development package not found (optional for GPU detection)"
        echo "On Debian/Ubuntu, install with: sudo apt install libglvnd-dev"
        echo "On Fedora/RHEL, install with: sudo dnf install libglvnd-devel"
        echo "On Arch Linux, install with: sudo pacman -S libglvnd"
    fi

    if [ $missing -eq 1 ]; then
        echo "Some dependencies may be missing. The build might still succeed, but functionality may be limited."
        read -p "Continue with build? [Y/n] " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Nn]$ ]]; then
            exit 1
        fi
    else
        echo "All dependencies found!"
    fi
}

build() {
    echo "Building gpufetch..."
    cargo build --release
    echo "Build complete! Binary is at target/release/gpufetch"
}

install() {
    echo "Installing gpufetch..."
    
    # Create directories if they don't exist
    mkdir -p "$HOME/.local/bin"
    
    # Copy binary
    cp target/release/gpufetch "$HOME/.local/bin/"
    
    # Make executable
    chmod +x "$HOME/.local/bin/gpufetch"
    
    echo "gpufetch installed to $HOME/.local/bin/gpufetch"
    
    # Check if $HOME/.local/bin is in PATH
    if [[ ":$PATH:" != *":$HOME/.local/bin:"* ]]; then
        echo "Warning: $HOME/.local/bin is not in your PATH"
        echo "Add the following line to your ~/.bashrc or ~/.zshrc:"
        echo "  export PATH=\"\$HOME/.local/bin:\$PATH\""
    fi
}

system_install() {
    echo "Installing gpufetch system-wide (requires sudo)..."
    
    sudo install -Dm755 target/release/gpufetch /usr/local/bin/gpufetch
    
    echo "gpufetch installed to /usr/local/bin/gpufetch"
}

# Parse command line arguments
case "$1" in
    "check")
        check_deps
        ;;
    "build")
        check_deps
        build
        ;;
    "install")
        check_deps
        build
        install
        ;;
    "system-install")
        check_deps
        build
        system_install
        ;;
    "clean")
        cargo clean
        echo "Build artifacts cleaned!"
        ;;
    *)
        echo "gpufetch build script"
        echo "Usage: $0 [command]"
        echo ""
        echo "Commands:"
        echo "  check           - Check for dependencies"
        echo "  build           - Build gpufetch (default)"
        echo "  install         - Install to ~/.local/bin"
        echo "  system-install  - Install system-wide to /usr/local/bin (requires sudo)"
        echo "  clean           - Clean build artifacts"
        echo ""
        echo "If no command is specified, 'build' will be executed."
        if [ -z "$1" ]; then
            check_deps
            build
        fi
        ;;
esac

exit 0
