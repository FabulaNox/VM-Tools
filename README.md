# VM-Tools
Small set of scripts to manage VMs

## Available Scripts

### install-qemu-kvm.sh
Automated installation script for QEMU/KVM virtualization with virt-manager on Ubuntu.

**Features:**
- Checks for hardware virtualization support (Intel VT-x / AMD-V)
- Installs QEMU, KVM, libvirt, and virt-manager
- Configures user permissions for libvirt group
- Enables and starts libvirtd service
- Verifies installation

**Usage:**
```bash
sudo ./install-qemu-kvm.sh
```

**Requirements:**
- Fresh Ubuntu installation
- CPU with virtualization support (VT-x/AMD-V) enabled in BIOS
- Root/sudo privileges

**Post-Installation:**
After running the script, log out and log back in for group changes to take effect, then launch virt-manager:
```bash
virt-manager
```

Or use the command-line tool:
```bash
virsh list --all
```
