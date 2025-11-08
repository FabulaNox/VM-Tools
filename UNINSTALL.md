# VM-Tools Uninstall Quick Reference

## 🛡️ Safe Uninstallation

**VM-Tools uninstaller NEVER touches your VMs, disk images, or VM data.**

### 🎯 Recommended Method

```bash
# Complete safe uninstall
./uninstall.sh

# Check what's installed first
./uninstall.sh --verify

# Preview what will be removed
./uninstall.sh --dry-run
```

### 🎛️ Uninstall Options

| Command | Description |
|---------|-------------|
| `./uninstall.sh` | Interactive complete uninstall |
| `./uninstall.sh --complete` | Remove everything except VMs (default) |
| `./uninstall.sh --binary-only` | Remove only vmtools binary |
| `./uninstall.sh --config-only` | Remove only config files |
| `./uninstall.sh --preserve-config` | Keep configuration files |
| `./uninstall.sh --dry-run` | Show what would be removed |
| `./uninstall.sh --force` | Skip confirmations |
| `./uninstall.sh --verify` | Check current installation |

### 🚀 Quick Methods

```bash
# Basic uninstall via installer
./install.sh uninstall

# Using Makefile
make uninstall-safe    # Complete safe removal
make uninstall         # Basic removal
```

### 📂 What Gets Removed

✅ **Removed:**
- `/usr/local/bin/vmtools` (system binary)
- `~/.local/bin/vmtools` (user binary)
- `~/.config/vmtools/` (configuration)
- `~/.cache/vmtools/` (cache files)
- Repository clones (with confirmation)
- PATH modifications (detected, manual cleanup suggested)

🚫 **NEVER Removed:**
- Virtual machines and their disk images
- VM snapshots and configurations
- libvirt/QEMU system settings
- VM networks and storage pools
- Any VM-related data

### 🔄 After Uninstallation

Your VMs remain fully functional and can be managed with:
- `virsh` (command line)
- `virt-manager` (GUI)
- `virt-viewer` (console)
- Any other libvirt-compatible tools

### 🆘 Emergency Manual Removal

If scripts fail, manual cleanup:

```bash
# Remove binaries
sudo rm -f /usr/local/bin/vmtools
rm -f ~/.local/bin/vmtools

# Remove configuration
rm -rf ~/.config/vmtools
rm -rf ~/.cache/vmtools

# Check for repository clones
find ~ -name "VM-Tools" -type d 2>/dev/null

# Check PATH modifications
grep -n vmtools ~/.bashrc ~/.zshrc ~/.profile 2>/dev/null
```

### ✅ Verification

After uninstall, verify removal:

```bash
# These should fail
which vmtools
vmtools --version

# These should still work (if libvirt installed)
virsh list --all
virt-manager
```

---

**Remember: VM-Tools uninstallation is designed to be 100% safe for your VMs and VM data.**