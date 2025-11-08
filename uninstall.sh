#!/bin/bash

################################################################################
# VM-Tools Complete Uninstall Script
################################################################################
# This script safely removes VM-Tools while preserving all VMs and their data.
# It provides options for complete removal or selective cleanup.
#
# Usage: 
#   ./uninstall.sh [options]
#
# Options:
#   --help              Show this help
#   --binary-only       Remove only the vmtools binary
#   --config-only       Remove only configuration files
#   --complete          Remove everything except VMs (default)
#   --dry-run           Show what would be removed without doing it
#   --force             Skip confirmations
#   --preserve-config   Keep configuration files
################################################################################

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
BINARY_NAME="vmtools"
SYSTEM_INSTALL_DIR="/usr/local/bin"
LOCAL_INSTALL_DIR="$HOME/.local/bin"
CONFIG_DIR="$HOME/.config/vmtools"
REPO_DIR="$HOME/VM-Tools"
CACHE_DIR="$HOME/.cache/vmtools"

# Default options
DRY_RUN=false
FORCE=false
BINARY_ONLY=false
CONFIG_ONLY=false
COMPLETE=true
PRESERVE_CONFIG=false

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
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_dry_run() {
    echo -e "${CYAN}[DRY RUN]${NC} Would $1"
}

# Function to confirm action
confirm() {
    if [[ "$FORCE" == "true" ]]; then
        return 0
    fi
    
    read -p "$1 [y/N]: " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        return 0
    else
        return 1
    fi
}

# Function to safely remove file/directory
safe_remove() {
    local target="$1"
    local description="$2"
    local use_sudo="${3:-false}"
    
    if [[ ! -e "$target" ]]; then
        print_info "$description not found (already removed)"
        return 0
    fi
    
    if [[ "$DRY_RUN" == "true" ]]; then
        print_dry_run "remove $description: $target"
        return 0
    fi
    
    if [[ "$use_sudo" == "true" ]]; then
        sudo rm -rf "$target"
    else
        rm -rf "$target"
    fi
    
    print_success "Removed $description"
}

# Function to check what VMs exist
check_existing_vms() {
    print_step "Checking for existing VMs..."
    
    # Check if virsh is available
    if command -v virsh >/dev/null 2>&1; then
        local vm_count=$(virsh list --all --name 2>/dev/null | grep -v '^$' | wc -l)
        if [[ $vm_count -gt 0 ]]; then
            print_info "Found $vm_count existing VM(s):"
            virsh list --all --name 2>/dev/null | grep -v '^$' | sed 's/^/  - /'
            print_warning "These VMs will NOT be affected by uninstallation"
        else
            print_info "No VMs found"
        fi
    else
        print_info "virsh not available - cannot check for VMs"
    fi
    echo
}

# Function to show what will be removed
show_removal_plan() {
    echo "=============================================="
    echo "  VM-Tools Removal Plan"
    echo "=============================================="
    echo
    
    print_step "The following items will be checked for removal:"
    echo
    
    if [[ "$BINARY_ONLY" == "true" ]]; then
        echo "📦 BINARY REMOVAL ONLY:"
        echo "  - $SYSTEM_INSTALL_DIR/$BINARY_NAME (if exists)"
        echo "  - $LOCAL_INSTALL_DIR/$BINARY_NAME (if exists)"
    elif [[ "$CONFIG_ONLY" == "true" ]]; then
        echo "⚙️  CONFIGURATION REMOVAL ONLY:"
        echo "  - $CONFIG_DIR (if exists)"
        echo "  - $CACHE_DIR (if exists)"
    else
        echo "🗂️  VMTOOLS SOFTWARE:"
        echo "  - $SYSTEM_INSTALL_DIR/$BINARY_NAME (if exists)"
        echo "  - $LOCAL_INSTALL_DIR/$BINARY_NAME (if exists)"
        echo "  - Repository clone (if exists)"
        
        if [[ "$PRESERVE_CONFIG" == "false" ]]; then
            echo "  - $CONFIG_DIR (if exists)"
            echo "  - $CACHE_DIR (if exists)"
        fi
    fi
    
    echo
    echo "✅ WILL NOT BE REMOVED:"
    echo "  - Your VMs and their disk images"
    echo "  - libvirt/QEMU system configuration"
    echo "  - VM networks and storage pools"
    echo "  - Any VM data or snapshots"
    
    if [[ "$PRESERVE_CONFIG" == "true" ]]; then
        echo "  - vmtools configuration files"
    fi
    
    echo
}

# Function to remove binary files
remove_binaries() {
    print_step "Removing vmtools binaries..."
    
    # Remove from system location
    safe_remove "$SYSTEM_INSTALL_DIR/$BINARY_NAME" "system binary" true
    
    # Remove from user location  
    safe_remove "$LOCAL_INSTALL_DIR/$BINARY_NAME" "user binary" false
    
    # Check PATH modifications
    for shell_config in "$HOME/.bashrc" "$HOME/.zshrc" "$HOME/.profile"; do
        if [[ -f "$shell_config" ]] && grep -q "$LOCAL_INSTALL_DIR" "$shell_config" 2>/dev/null; then
            print_warning "Found PATH modification in $shell_config"
            print_info "You may want to manually remove vmtools PATH entries"
        fi
    done
}

# Function to remove configuration
remove_configuration() {
    print_step "Removing configuration files..."
    
    safe_remove "$CONFIG_DIR" "configuration directory" false
    safe_remove "$CACHE_DIR" "cache directory" false
}

# Function to remove repository
remove_repository() {
    print_step "Looking for repository clone..."
    
    # Check common locations where the repo might be
    local possible_locations=(
        "$HOME/VM-Tools"
        "$HOME/Repositories/VM-Tools"
        "$HOME/Repositories/VM-Tools/VM-Tools"
        "$HOME/Downloads/VM-Tools"
        "$HOME/workspace/VM-Tools"
    )
    
    for location in "${possible_locations[@]}"; do
        if [[ -d "$location" ]] && [[ -f "$location/Cargo.toml" ]]; then
            if confirm "Remove repository clone at $location?"; then
                safe_remove "$location" "repository clone" false
            fi
        fi
    done
}

# Function to perform complete uninstall
complete_uninstall() {
    show_removal_plan
    
    if ! confirm "Proceed with uninstallation?"; then
        print_info "Uninstallation cancelled"
        exit 0
    fi
    
    echo
    check_existing_vms
    
    if [[ "$BINARY_ONLY" == "true" ]]; then
        remove_binaries
    elif [[ "$CONFIG_ONLY" == "true" ]]; then
        remove_configuration
    else
        remove_binaries
        
        if [[ "$PRESERVE_CONFIG" == "false" ]]; then
            remove_configuration
        fi
        
        remove_repository
    fi
    
    echo
    print_success "VM-Tools uninstallation completed!"
    echo
    print_info "Your VMs and VM data remain completely untouched"
    if command -v virsh >/dev/null 2>&1; then
        print_info "You can still manage VMs using: virsh, virt-manager, or virt-viewer"
    fi
    
    if [[ "$PRESERVE_CONFIG" == "true" ]]; then
        print_info "Configuration preserved at: $CONFIG_DIR"
    fi
}

# Function to verify current installation
verify_installation() {
    echo "=============================================="
    echo "  Current VM-Tools Installation Status"
    echo "=============================================="
    echo
    
    local found_something=false
    
    # Check binaries
    if [[ -f "$SYSTEM_INSTALL_DIR/$BINARY_NAME" ]]; then
        print_info "System binary: $SYSTEM_INSTALL_DIR/$BINARY_NAME"
        found_something=true
    fi
    
    if [[ -f "$LOCAL_INSTALL_DIR/$BINARY_NAME" ]]; then
        print_info "User binary: $LOCAL_INSTALL_DIR/$BINARY_NAME"
        found_something=true
    fi
    
    # Check configuration
    if [[ -d "$CONFIG_DIR" ]]; then
        local config_size=$(du -sh "$CONFIG_DIR" 2>/dev/null | cut -f1)
        print_info "Configuration: $CONFIG_DIR ($config_size)"
        found_something=true
    fi
    
    # Check cache
    if [[ -d "$CACHE_DIR" ]]; then
        local cache_size=$(du -sh "$CACHE_DIR" 2>/dev/null | cut -f1)
        print_info "Cache: $CACHE_DIR ($cache_size)"
        found_something=true
    fi
    
    # Check if vmtools is in PATH
    if command -v vmtools >/dev/null 2>&1; then
        local vmtools_path=$(which vmtools)
        local vmtools_version=$(vmtools --version 2>/dev/null || echo "unknown")
        print_info "Active binary: $vmtools_path ($vmtools_version)"
        found_something=true
    fi
    
    if [[ "$found_something" == "false" ]]; then
        print_info "No VM-Tools installation found"
    fi
    
    echo
}

# Function to show help
show_help() {
    echo "VM-Tools Uninstall Script"
    echo
    echo "DESCRIPTION:"
    echo "  Safely removes VM-Tools while preserving all VMs and their data."
    echo "  Never touches your virtual machines, disk images, or VM configurations."
    echo
    echo "USAGE:"
    echo "  $0 [OPTIONS]"
    echo
    echo "OPTIONS:"
    echo "  --help              Show this help message"
    echo "  --verify            Show current installation status"
    echo "  --binary-only       Remove only the vmtools binary files"
    echo "  --config-only       Remove only configuration and cache files"
    echo "  --complete          Remove everything except VMs (default)"
    echo "  --dry-run           Show what would be removed without doing it"
    echo "  --force             Skip confirmation prompts"
    echo "  --preserve-config   Keep configuration files during removal"
    echo
    echo "EXAMPLES:"
    echo "  $0                  # Interactive complete uninstall"
    echo "  $0 --verify         # Check what's currently installed"
    echo "  $0 --dry-run        # See what would be removed"
    echo "  $0 --binary-only    # Remove only the binary, keep config"
    echo "  $0 --force          # Uninstall without prompts"
    echo
    echo "SAFETY:"
    echo "  ✅ Your VMs and their data are NEVER touched"
    echo "  ✅ libvirt/QEMU configuration is preserved"  
    echo "  ✅ VM disk images remain untouched"
    echo "  ✅ Networks and storage pools are preserved"
    echo
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --help|-h)
            show_help
            exit 0
            ;;
        --verify)
            verify_installation
            exit 0
            ;;
        --binary-only)
            BINARY_ONLY=true
            COMPLETE=false
            shift
            ;;
        --config-only)
            CONFIG_ONLY=true
            COMPLETE=false
            shift
            ;;
        --complete)
            COMPLETE=true
            BINARY_ONLY=false
            CONFIG_ONLY=false
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --force)
            FORCE=true
            shift
            ;;
        --preserve-config)
            PRESERVE_CONFIG=true
            shift
            ;;
        *)
            print_error "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Main execution
echo "=============================================="
echo "  VM-Tools Safe Uninstaller"
echo "=============================================="
echo

# Validate conflicting options
if [[ "$BINARY_ONLY" == "true" && "$CONFIG_ONLY" == "true" ]]; then
    print_error "Cannot specify both --binary-only and --config-only"
    exit 1
fi

if [[ "$CONFIG_ONLY" == "true" && "$PRESERVE_CONFIG" == "true" ]]; then
    print_error "Cannot specify both --config-only and --preserve-config"
    exit 1
fi

# Show dry run notice
if [[ "$DRY_RUN" == "true" ]]; then
    print_warning "DRY RUN MODE - No changes will be made"
    echo
fi

# Perform uninstallation
complete_uninstall