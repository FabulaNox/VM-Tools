#!/bin/bash

################################################################################
# VM-Tools Build Script
################################################################################
# This script builds the vmtools binary and sets up the development environment
#
# Usage: ./build.sh [release|debug|install|clean]
################################################################################

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

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

# Check if Rust is installed
check_rust() {
    if ! command -v cargo &> /dev/null; then
        print_error "Rust/Cargo is not installed"
        print_info "Please install Rust from https://rustup.rs/"
        exit 1
    fi
    
    print_info "Rust version: $(rustc --version)"
}

# Check system dependencies
check_dependencies() {
    print_step "Checking system dependencies..."
    
    local missing_deps=()
    
    # Check for libvirt
    if ! command -v virsh &> /dev/null; then
        missing_deps+=("libvirt-clients")
    fi
    
    # Check for qemu-img
    if ! command -v qemu-img &> /dev/null; then
        missing_deps+=("qemu-utils")
    fi
    
    # Check for KVM
    if [[ ! -e /dev/kvm ]]; then
        print_warning "KVM device not found - virtualization may not work"
    fi
    
    if [[ ${#missing_deps[@]} -gt 0 ]]; then
        print_error "Missing dependencies: ${missing_deps[*]}"
        print_info "Please install them using:"
        print_info "  sudo apt-get install ${missing_deps[*]}"
        exit 1
    fi
    
    print_info "All dependencies are satisfied"
}

# Build function
build_project() {
    local build_type=${1:-debug}
    
    print_step "Building vmtools in $build_type mode..."
    
    if [[ "$build_type" == "release" ]]; then
        cargo build --release
        print_info "Release build completed: target/release/vmtools"
    else
        cargo build
        print_info "Debug build completed: target/debug/vmtools"
    fi
}

# Install function
install_project() {
    print_step "Installing vmtools..."
    
    # Build release version
    cargo build --release
    
    # Install binary
    sudo cp target/release/vmtools /usr/local/bin/
    sudo chmod +x /usr/local/bin/vmtools
    
    # Create config directory
    mkdir -p "$HOME/.config/vmtools"
    
    print_info "vmtools installed to /usr/local/bin/vmtools"
    print_info "Configuration directory: $HOME/.config/vmtools"
    
    # Test installation
    if vmtools --version &> /dev/null; then
        print_info "Installation verified successfully"
    else
        print_error "Installation verification failed"
        exit 1
    fi
}

# Clean function
clean_project() {
    print_step "Cleaning build artifacts..."
    cargo clean
    print_info "Build artifacts cleaned"
}

# Run tests
run_tests() {
    print_step "Running tests..."
    cargo test
    print_info "All tests passed"
}

# Lint code
lint_code() {
    print_step "Running clippy (Rust linter)..."
    cargo clippy -- -D warnings
    print_info "Code linting completed"
}

# Format code
format_code() {
    print_step "Formatting code..."
    cargo fmt
    print_info "Code formatting completed"
}

# Show help
show_help() {
    echo "VM-Tools Build Script"
    echo ""
    echo "Usage: $0 [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  debug     Build in debug mode (default)"
    echo "  release   Build in release mode"
    echo "  install   Build and install to system"
    echo "  clean     Clean build artifacts"
    echo "  test      Run all tests"
    echo "  lint      Run clippy linter"
    echo "  format    Format code"
    echo "  deps      Check dependencies only"
    echo "  all       Run format, lint, test, and build"
    echo "  help      Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0                # Build in debug mode"
    echo "  $0 release        # Build in release mode"
    echo "  $0 install        # Install to system"
    echo "  $0 all            # Complete development cycle"
}

# Main execution
main() {
    local command=${1:-debug}
    
    echo "=============================================="
    echo "  VM-Tools Build Script"
    echo "=============================================="
    echo ""
    
    case "$command" in
        "debug")
            check_rust
            check_dependencies
            build_project debug
            ;;
        "release")
            check_rust
            check_dependencies
            build_project release
            ;;
        "install")
            check_rust
            check_dependencies
            install_project
            ;;
        "clean")
            clean_project
            ;;
        "test")
            check_rust
            run_tests
            ;;
        "lint")
            check_rust
            lint_code
            ;;
        "format")
            check_rust
            format_code
            ;;
        "deps")
            check_dependencies
            ;;
        "all")
            check_rust
            check_dependencies
            format_code
            lint_code
            run_tests
            build_project release
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
    
    echo ""
    print_info "Build script completed successfully!"
}

# Run main function
main "$@"