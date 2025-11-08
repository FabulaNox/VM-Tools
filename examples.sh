#!/bin/bash

################################################################################
# VM-Tools Usage Examples
################################################################################
# This script demonstrates common usage patterns for vmtools
#
# Usage: ./examples.sh
################################################################################

set -e

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo_example() {
    echo -e "${BLUE}[EXAMPLE]${NC} $1"
}

echo_note() {
    echo -e "${YELLOW}[NOTE]${NC} $1"
}

echo_result() {
    echo -e "${GREEN}[RESULT]${NC} $1"
}

echo "=============================================="
echo "  VM-Tools Usage Examples"
echo "=============================================="
echo ""

echo_example "1. List all virtual machines"
echo "$ vmtools list --all"
echo ""

echo_example "2. Show current configuration"
echo "$ vmtools config --show"
echo ""

echo_example "3. Create a new Ubuntu VM"
echo "$ vmtools create ubuntu-test --memory 4096 --cpus 2 --disk-size 30 --template ubuntu"
echo_note "This creates a VM with 4GB RAM, 2 CPUs, 30GB disk using Ubuntu template"
echo ""

echo_example "4. Create a VM with ISO for installation"
echo "$ vmtools create new-vm --iso-path /path/to/ubuntu-22.04.iso --memory 2048"
echo_note "This creates a VM and attaches an ISO for OS installation"
echo ""

echo_example "5. Start a virtual machine"
echo "$ vmtools start ubuntu-test"
echo_note "This boots the VM and shows progress"
echo ""

echo_example "6. Check VM status and resources"
echo "$ vmtools status ubuntu-test"
echo_note "Shows detailed information about CPU, memory, disk, and network"
echo ""

echo_example "7. Monitor VM performance in real-time"
echo "$ vmtools monitor ubuntu-test"
echo_note "Shows live updates of resource usage (Ctrl+C to exit)"
echo ""

echo_example "8. Connect to VM console"
echo "$ vmtools console ubuntu-test"
echo_note "Connects directly to VM console (Ctrl+] to disconnect)"
echo ""

echo_example "9. Clone an existing VM"
echo "$ vmtools clone ubuntu-test ubuntu-clone"
echo_note "Creates an exact copy of the VM with new name"
echo ""

echo_example "10. Stop a virtual machine"
echo "$ vmtools stop ubuntu-test"
echo_note "Gracefully shuts down the VM"
echo ""

echo_example "11. Force stop a VM"
echo "$ vmtools stop ubuntu-test --force"
echo_note "Immediately stops the VM (like pulling power cord)"
echo ""

echo_example "12. List available networks"
echo "$ vmtools networks"
echo_note "Shows all virtual networks and their status"
echo ""

echo_example "13. Delete a virtual machine"
echo "$ vmtools delete ubuntu-test"
echo_note "Removes the VM and its disk files (asks for confirmation)"
echo ""

echo_example "14. Force delete without confirmation"
echo "$ vmtools delete ubuntu-test --force"
echo_note "Immediately deletes VM without asking"
echo ""

echo_example "15. Configure default settings"
echo "$ vmtools config --set defaults.memory=4096"
echo "$ vmtools config --set defaults.cpus=4"
echo_note "Changes default values for new VMs"
echo ""

echo_example "16. Get specific configuration value"
echo "$ vmtools config --get defaults.memory"
echo_note "Shows current value of a configuration option"
echo ""

echo ""
echo "=============================================="
echo "  Advanced Usage Patterns"
echo "=============================================="
echo ""

echo_example "Batch operations with shell scripting:"
echo 'for vm in vm1 vm2 vm3; do'
echo '    vmtools create $vm --template ubuntu'
echo '    vmtools start $vm'
echo 'done'
echo ""

echo_example "Monitor multiple VMs:"
echo 'watch -n 2 "vmtools list --running"'
echo_note "Updates every 2 seconds showing only running VMs"
echo ""

echo_example "Automated VM lifecycle:"
echo 'vmtools create test-vm --template ubuntu'
echo 'vmtools start test-vm'
echo 'sleep 30  # Wait for boot'
echo 'vmtools status test-vm'
echo 'vmtools stop test-vm'
echo 'vmtools delete test-vm --force'
echo ""

echo_example "Debug mode with detailed logging:"
echo 'RUST_LOG=debug vmtools list --all'
echo_note "Shows detailed internal operations for troubleshooting"
echo ""

echo ""
echo "=============================================="
echo "  Templates and Configuration"
echo "=============================================="
echo ""

echo_note "Available templates: ubuntu, windows"
echo_note "Config file location: ~/.config/vmtools/config.toml"
echo_note "You can create custom templates by editing the config file"
echo ""

echo_example "Custom template in config.toml:"
echo '[templates.development]'
echo 'memory = 8192'
echo 'cpus = 4'
echo 'disk_size = 100'
echo 'os_type = "linux"'
echo 'arch = "x86_64"'
echo ""

echo_example "Using custom template:"
echo '$ vmtools create dev-vm --template development'
echo ""

echo ""
echo "=============================================="
echo "  Performance and Resource Management"
echo "=============================================="
echo ""

echo_note "vmtools is designed for minimal overhead:"
echo_note "- Memory usage: ~2-5 MB resident"
echo_note "- CPU usage: <0.1% when idle"
echo_note "- Written in Rust for memory safety and performance"
echo ""

echo_example "Check system resources before creating VMs:"
echo 'cat /proc/meminfo | grep MemAvailable'
echo 'nproc  # Number of CPU cores'
echo 'df -h /var/lib/libvirt/images  # Available disk space'
echo ""

echo ""
echo "=============================================="
echo "  Troubleshooting"
echo "=============================================="
echo ""

echo_example "If libvirt connection fails:"
echo '$ sudo systemctl status libvirtd'
echo '$ sudo systemctl start libvirtd'
echo '$ sudo usermod -aG libvirt $USER'
echo_note "Log out and back in for group changes to take effect"
echo ""

echo_example "If KVM is not available:"
echo '$ lsmod | grep kvm'
echo '$ ls -la /dev/kvm'
echo '$ grep -E "(vmx|svm)" /proc/cpuinfo'
echo ""

echo_example "Check vmtools installation:"
echo '$ vmtools --version'
echo '$ which vmtools'
echo ""

echo_result "For more help, use: vmtools <command> --help"
echo_result "Full documentation: README.md"
echo ""