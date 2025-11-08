#!/bin/bash

################################################################################
# QEMU/KVM + virt-manager Installation Script for Ubuntu
################################################################################
# This script installs and configures QEMU/KVM virtualization with virt-manager
# on a fresh Ubuntu installation.
#
# Usage: sudo ./install-qemu-kvm.sh
################################################################################

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
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

# Function to check if running as root
check_root() {
    if [[ $EUID -ne 0 ]]; then
        print_error "This script must be run as root (use sudo)"
        exit 1
    fi
}

# Function to check if running on Ubuntu
check_ubuntu() {
    if [[ ! -f /etc/os-release ]]; then
        print_error "Cannot determine OS. /etc/os-release not found."
        exit 1
    fi
    
    # shellcheck source=/dev/null
    source /etc/os-release
    if [[ "$ID" != "ubuntu" ]]; then
        print_error "This script is designed for Ubuntu. Detected OS: $ID"
        exit 1
    fi
    
    print_info "Detected Ubuntu $VERSION"
}

# Function to check CPU virtualization support
check_virtualization() {
    print_info "Checking for hardware virtualization support..."
    
    if grep -E -q '(vmx|svm)' /proc/cpuinfo; then
        print_info "Hardware virtualization is supported"
        
        if grep -q 'vmx' /proc/cpuinfo; then
            print_info "Intel VT-x detected"
        elif grep -q 'svm' /proc/cpuinfo; then
            print_info "AMD-V detected"
        fi
    else
        print_error "Hardware virtualization is NOT supported or not enabled in BIOS"
        print_warning "Please enable VT-x (Intel) or AMD-V in your BIOS/UEFI settings"
        exit 1
    fi
}

# Function to update package list
update_packages() {
    print_info "Updating package list..."
    apt-get update -qq
}

# Function to install QEMU/KVM packages
install_qemu_kvm() {
    print_info "Installing QEMU/KVM packages..."
    
    # Core packages
    PACKAGES=(
        qemu-kvm
        libvirt-daemon-system
        libvirt-clients
        bridge-utils
        virt-manager
        virtinst
        libvirt-daemon
        qemu-system-x86
        qemu-utils
    )
    
    apt-get install -y "${PACKAGES[@]}"
    
    print_info "QEMU/KVM packages installed successfully"
}

# Function to add user to libvirt group
setup_user_permissions() {
    print_info "Setting up user permissions..."
    
    # Get the actual user who invoked sudo
    if [[ -n "$SUDO_USER" ]]; then
        ACTUAL_USER="$SUDO_USER"
    else
        print_warning "Could not determine the user who invoked sudo"
        read -r -p "Enter the username to add to libvirt group: " ACTUAL_USER
    fi
    
    if id "$ACTUAL_USER" &>/dev/null; then
        usermod -aG libvirt "$ACTUAL_USER"
        usermod -aG kvm "$ACTUAL_USER"
        print_info "User '$ACTUAL_USER' added to libvirt and kvm groups"
        print_warning "User '$ACTUAL_USER' needs to log out and log back in for group changes to take effect"
    else
        print_error "User '$ACTUAL_USER' does not exist"
        exit 1
    fi
}

# Function to enable and start libvirt service
start_services() {
    print_info "Enabling and starting libvirtd service..."
    
    systemctl enable libvirtd
    systemctl start libvirtd
    
    # Check service status
    if systemctl is-active --quiet libvirtd; then
        print_info "libvirtd service is running"
    else
        print_error "Failed to start libvirtd service"
        exit 1
    fi
}

# Function to verify installation
verify_installation() {
    print_info "Verifying installation..."
    
    # Check if KVM module is loaded
    if lsmod | grep -q kvm; then
        print_info "KVM kernel module is loaded"
    else
        print_warning "KVM kernel module is not loaded"
    fi
    
    # Check libvirt connection
    if virsh list --all &>/dev/null; then
        print_info "libvirt connection successful"
    else
        print_warning "libvirt connection test failed"
    fi
    
    # Check if virt-manager is installed
    if command -v virt-manager &>/dev/null; then
        print_info "virt-manager is installed"
    else
        print_warning "virt-manager command not found"
    fi
}

# Function to display post-installation information
display_info() {
    echo ""
    echo "=============================================="
    print_info "Installation completed successfully!"
    echo "=============================================="
    echo ""
    echo "Next steps:"
    echo "  1. Log out and log back in for group changes to take effect"
    echo "  2. Launch virt-manager by running: virt-manager"
    echo "  3. Or use virsh command-line tool: virsh list --all"
    echo ""
    echo "Useful commands:"
    echo "  - Start virt-manager: virt-manager"
    echo "  - List VMs: virsh list --all"
    echo "  - Check service status: systemctl status libvirtd"
    echo "  - View libvirt version: virsh --version"
    echo ""
    echo "Default network 'default' should be available."
    echo "You can verify with: virsh net-list --all"
    echo ""
    echo "To start the default network if not active:"
    echo "  virsh net-start default"
    echo "  virsh net-autostart default"
    echo ""
}

# Main execution
main() {
    echo "=============================================="
    echo "  QEMU/KVM + virt-manager Installation"
    echo "  for Ubuntu"
    echo "=============================================="
    echo ""
    
    check_root
    check_ubuntu
    check_virtualization
    update_packages
    install_qemu_kvm
    setup_user_permissions
    start_services
    verify_installation
    display_info
}

# Run main function
main
