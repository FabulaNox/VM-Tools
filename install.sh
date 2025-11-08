#!/bin/bash

################################################################################
# VM-Tools Quick Install Script
################################################################################
# This script provides multiple installation methods for vmtools:
# 1. Quick install from repository
# 2. Development setup
# 3. System-wide installation
# 4. User-local installation
#
# Usage: 
#   curl -sSL https://raw.githubusercontent.com/FabulaNox/VM-Tools/main/install.sh | bash
#   OR
#   ./install.sh [quick|dev|system|user|uninstall]
################################################################################

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Configuration
REPO_URL="https://github.com/FabulaNox/VM-Tools.git"
REPO_NAME="VM-Tools"
BINARY_NAME="vmtools"
INSTALL_DIR="/usr/local/bin"
LOCAL_INSTALL_DIR="$HOME/.local/bin"
TEMP_DIR="/tmp/vmtools-install-$$"

# Function to print colored messages
print_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_step() {
    echo -e "${BLUE}[STEP]${NC} $1"
}

print_success() {
    echo -e "${PURPLE}[SUCCESS]${NC} $1"
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to detect OS
detect_os() {
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        if command_exists apt-get; then
            echo "ubuntu"
        elif command_exists yum; then
            echo "centos"
        elif command_exists pacman; then
            echo "arch"
        else
            echo "linux"
        fi
    elif [[ "$OSTYPE" == "darwin"* ]]; then
        echo "macos"
    else
        echo "unknown"
    fi
}

# Function to install system dependencies
install_dependencies() {
    local os=$(detect_os)
    print_step "Installing system dependencies for $os..."
    
    case $os in
        "ubuntu")
            sudo apt-get update -qq
            sudo apt-get install -y git curl build-essential libvirt-clients qemu-utils qemu-kvm libvirt-daemon-system
            ;;
        "centos")
            sudo yum update -y
            sudo yum groupinstall -y "Development Tools"
            sudo yum install -y git curl libvirt-client qemu-img qemu-kvm libvirt-daemon
            ;;
        "arch")
            sudo pacman -Sy --noconfirm git curl base-devel libvirt qemu-img qemu
            ;;
        "macos")
            if command_exists brew; then
                brew install git curl
                print_warning "QEMU/KVM not available on macOS. Some features will be limited."
            else
                print_error "Homebrew not found. Please install Homebrew first: https://brew.sh"
                exit 1
            fi
            ;;
        *)
            print_warning "Unknown OS. Please install git, curl, and QEMU/KVM manually."
            ;;
    esac
}

# Function to install Rust
install_rust() {
    if command_exists cargo; then
        print_info "Rust is already installed: $(rustc --version)"
        return
    fi
    
    print_step "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    
    # Source the environment
    if [[ -f "$HOME/.cargo/env" ]]; then
        source "$HOME/.cargo/env"
    fi
    
    # Add to PATH for current session
    export PATH="$HOME/.cargo/bin:$PATH"
    
    if command_exists cargo; then
        print_success "Rust installed successfully: $(rustc --version)"
    else
        print_error "Failed to install Rust"
        exit 1
    fi
}

# Function to setup libvirt permissions
setup_libvirt() {
    if [[ "$OSTYPE" == "linux-gnu"* ]]; then
        print_step "Setting up libvirt permissions..."
        
        # Add user to libvirt group
        if getent group libvirt >/dev/null; then
            if ! groups "$USER" | grep -q '\blibvirt\b'; then
                sudo usermod -aG libvirt "$USER"
                print_warning "Added $USER to libvirt group. Please log out and back in for changes to take effect."
            fi
        fi
        
        # Start libvirtd service
        if command_exists systemctl; then
            sudo systemctl enable libvirtd || true
            sudo systemctl start libvirtd || true
        fi
    fi
}

# Function to clone or update repository
setup_repository() {
    print_step "Setting up repository..."
    
    if [[ -d "$TEMP_DIR" ]]; then
        rm -rf "$TEMP_DIR"
    fi
    
    mkdir -p "$TEMP_DIR"
    cd "$TEMP_DIR"
    
    print_info "Cloning repository..."
    git clone "$REPO_URL" "$REPO_NAME"
    cd "$REPO_NAME"
}

# Function to build vmtools
build_vmtools() {
    print_step "Building vmtools..."
    
    # Ensure Rust is in PATH
    if [[ -f "$HOME/.cargo/env" ]]; then
        source "$HOME/.cargo/env"
    fi
    export PATH="$HOME/.cargo/bin:$PATH"
    
    if ! command_exists cargo; then
        print_error "Cargo not found. Please ensure Rust is properly installed."
        exit 1
    fi
    
    # Build in release mode
    cargo build --release
    
    if [[ ! -f "target/release/$BINARY_NAME" ]]; then
        print_error "Build failed. Binary not found."
        exit 1
    fi
    
    print_success "Build completed successfully"
}

# Function to install binary
install_binary() {
    local target_dir="$1"
    local use_sudo="$2"
    
    print_step "Installing binary to $target_dir..."
    
    # Create target directory
    if [[ "$use_sudo" == "true" ]]; then
        sudo mkdir -p "$target_dir"
        sudo cp "target/release/$BINARY_NAME" "$target_dir/"
        sudo chmod +x "$target_dir/$BINARY_NAME"
    else
        mkdir -p "$target_dir"
        cp "target/release/$BINARY_NAME" "$target_dir/"
        chmod +x "$target_dir/$BINARY_NAME"
    fi
    
    print_success "Binary installed to $target_dir/$BINARY_NAME"
}

# Function to setup configuration
setup_configuration() {
    print_step "Setting up default configuration..."
    
    # Create config directory
    local config_dir="$HOME/.config/vmtools"
    mkdir -p "$config_dir"
    
    # Run vmtools to create default config
    if [[ -x "$INSTALL_DIR/$BINARY_NAME" ]]; then
        "$INSTALL_DIR/$BINARY_NAME" config --show >/dev/null 2>&1 || true
    elif [[ -x "$LOCAL_INSTALL_DIR/$BINARY_NAME" ]]; then
        "$LOCAL_INSTALL_DIR/$BINARY_NAME" config --show >/dev/null 2>&1 || true
    fi
    
    print_success "Configuration directory created at $config_dir"
}

# Function to verify installation
verify_installation() {
    print_step "Verifying installation..."
    
    local binary_path=""
    if command_exists "$BINARY_NAME"; then
        binary_path=$(which "$BINARY_NAME")
        print_success "vmtools found at: $binary_path"
    else
        print_error "vmtools not found in PATH"
        print_info "You may need to:"
        print_info "  1. Log out and back in (for group changes)"
        print_info "  2. Add $LOCAL_INSTALL_DIR to your PATH"
        print_info "  3. Restart your terminal"
        return 1
    fi
    
    # Test basic functionality
    if "$BINARY_NAME" --version >/dev/null 2>&1; then
        print_success "vmtools is working correctly"
        
        # Show version and basic info
        echo ""
        echo "=============================================="
        print_success "Installation completed successfully!"
        echo "=============================================="
        echo ""
        echo "vmtools version: $("$BINARY_NAME" --version 2>/dev/null || echo "Unknown")"
        echo "Installed at: $binary_path"
        echo "Config directory: $HOME/.config/vmtools"
        echo ""
        echo "Quick start:"
        echo "  $BINARY_NAME --help           # Show help"
        echo "  $BINARY_NAME list --all       # List VMs"
        echo "  $BINARY_NAME config --show    # Show config"
        echo ""
        echo "For examples: ./examples.sh"
        echo "For documentation: README.md"
        echo ""
    else
        print_error "vmtools installation verification failed"
        return 1
    fi
}

# Function to cleanup
cleanup() {
    if [[ -d "$TEMP_DIR" ]]; then
        print_step "Cleaning up temporary files..."
        rm -rf "$TEMP_DIR"
    fi
}

# Function for quick install
quick_install() {
    echo "=============================================="
    echo "  VM-Tools Quick Install"
    echo "=============================================="
    echo ""
    
    install_dependencies
    install_rust
    setup_libvirt
    setup_repository
    build_vmtools
    install_binary "$INSTALL_DIR" "true"
    setup_configuration
    verify_installation
    cleanup
}

# Function for development setup
dev_install() {
    echo "=============================================="
    echo "  VM-Tools Development Setup"
    echo "=============================================="
    echo ""
    
    install_dependencies
    install_rust
    setup_libvirt
    
    # Clone to current directory instead of temp
    if [[ ! -d "$REPO_NAME" ]]; then
        git clone "$REPO_URL"
    fi
    
    cd "$REPO_NAME"
    
    print_step "Setting up development environment..."
    chmod +x build.sh
    ./build.sh deps
    ./build.sh debug
    
    print_success "Development environment ready!"
    echo ""
    echo "Development commands:"
    echo "  ./build.sh debug      # Debug build"
    echo "  ./build.sh release    # Release build"
    echo "  ./build.sh test       # Run tests"
    echo "  ./build.sh install    # Install system-wide"
    echo ""
    echo "Run the tool:"
    echo "  ./target/debug/vmtools --help"
}

# Function for user-local install
user_install() {
    echo "=============================================="
    echo "  VM-Tools User Install"
    echo "=============================================="
    echo ""
    
    install_dependencies
    install_rust
    setup_libvirt
    setup_repository
    build_vmtools
    install_binary "$LOCAL_INSTALL_DIR" "false"
    setup_configuration
    
    # Add to PATH if not already there
    local shell_rc=""
    if [[ "$SHELL" == *"zsh"* ]]; then
        shell_rc="$HOME/.zshrc"
    elif [[ "$SHELL" == *"bash"* ]]; then
        shell_rc="$HOME/.bashrc"
    fi
    
    if [[ -n "$shell_rc" && -f "$shell_rc" ]]; then
        if ! grep -q "$LOCAL_INSTALL_DIR" "$shell_rc"; then
            echo "" >> "$shell_rc"
            echo "# Added by vmtools installer" >> "$shell_rc"
            echo "export PATH=\"$LOCAL_INSTALL_DIR:\$PATH\"" >> "$shell_rc"
            print_info "Added $LOCAL_INSTALL_DIR to PATH in $shell_rc"
        fi
    fi
    
    export PATH="$LOCAL_INSTALL_DIR:$PATH"
    verify_installation
    cleanup
}

# Function to uninstall (basic removal)
uninstall() {
    echo "=============================================="
    echo "  VM-Tools Quick Uninstall"
    echo "=============================================="
    echo ""
    
    print_warning "For complete and safe uninstall options, use: ./uninstall.sh"
    print_info "This performs basic removal only"
    echo ""
    
    print_step "Removing vmtools..."
    
    # Remove from system locations
    if [[ -f "$INSTALL_DIR/$BINARY_NAME" ]]; then
        sudo rm -f "$INSTALL_DIR/$BINARY_NAME"
        print_info "Removed $INSTALL_DIR/$BINARY_NAME"
    fi
    
    # Remove from user location
    if [[ -f "$LOCAL_INSTALL_DIR/$BINARY_NAME" ]]; then
        rm -f "$LOCAL_INSTALL_DIR/$BINARY_NAME"
        print_info "Removed $LOCAL_INSTALL_DIR/$BINARY_NAME"
    fi
    
    # Ask about config directory
    if [[ -d "$HOME/.config/vmtools" ]]; then
        read -p "Remove configuration directory? [y/N]: " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            rm -rf "$HOME/.config/vmtools"
            print_info "Removed configuration directory"
        fi
    fi
    
    print_success "vmtools uninstalled"
    print_info "For complete cleanup, run: ./uninstall.sh --complete"
}

# Function to show help
show_help() {
    echo "VM-Tools Installer"
    echo ""
    echo "Usage: $0 [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  quick     Quick install (system-wide, requires sudo)"
    echo "  dev       Development setup (clone repo locally)"
    echo "  system    System-wide install (same as quick)"
    echo "  user      User-local install (no sudo required)"
    echo "  uninstall Remove vmtools (basic - use ./uninstall.sh for complete removal)"
    echo "  help      Show this help"
    echo ""
    echo "Environment variables:"
    echo "  INSTALL_DIR          Custom install directory (default: /usr/local/bin)"
    echo "  SKIP_DEPENDENCIES    Skip system dependency installation"
    echo "  SKIP_RUST           Skip Rust installation"
    echo ""
    echo "Examples:"
    echo "  $0 quick                    # Quick system install"
    echo "  $0 user                     # Install to ~/.local/bin"
    echo "  INSTALL_DIR=/opt/bin $0 quick  # Install to custom location"
    echo ""
    echo "Remote install:"
    echo "  curl -sSL https://raw.githubusercontent.com/FabulaNox/VM-Tools/main/install.sh | bash"
}

# Trap to ensure cleanup
trap cleanup EXIT

# Main execution
main() {
    local command=${1:-quick}
    
    # Handle environment variables
    if [[ -n "$SKIP_DEPENDENCIES" ]]; then
        print_info "Skipping dependency installation (SKIP_DEPENDENCIES set)"
        install_dependencies() { :; }
    fi
    
    if [[ -n "$SKIP_RUST" ]]; then
        print_info "Skipping Rust installation (SKIP_RUST set)"
        install_rust() { :; }
    fi
    
    case "$command" in
        "quick"|"system")
            quick_install
            ;;
        "dev"|"development")
            dev_install
            ;;
        "user"|"local")
            user_install
            ;;
        "uninstall"|"remove")
            uninstall
            ;;
        "help"|"--help"|"-h")
            show_help
            ;;
        *)
            print_error "Unknown command: $command"
            show_help
            exit 1
            ;;
    esac
}

# Check if script is being sourced or executed
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi