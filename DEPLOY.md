# 🚀 VM-Tools Quick Deployment Guide

## One-Line Installation

```bash
# Remote install (easiest)
curl -sSL https://raw.githubusercontent.com/FabulaNox/VM-Tools/main/install.sh | bash

# From repository
git clone https://github.com/FabulaNox/VM-Tools.git && cd VM-Tools && make deploy
```

## Installation Methods

| Method | Command | Requirements | Target Users |
|--------|---------|--------------|--------------|
| **Remote** | `curl ... \| bash` | Internet | Quick setup |
| **System** | `make deploy` | sudo | System admins |
| **User** | `make install-user` | None | Regular users |
| **Dev** | `make dev` | None | Developers |

## Quick Commands

```bash
# Build options
make help                    # Show all options
make deploy                  # Full deployment (recommended)
make install                 # System install
make install-user            # User install
make dev                     # Development setup

# Using scripts directly
./install.sh quick          # System install
./install.sh user           # User install  
./install.sh dev             # Development
./post-install.sh all       # Post-install config

# Traditional build
./build.sh deps              # Check dependencies
./build.sh release           # Build release
./build.sh install           # Install system
```

## Environment Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `INSTALL_DIR` | Custom install location | `/opt/bin` |
| `SKIP_DEPENDENCIES` | Skip system deps | `1` |
| `SKIP_RUST` | Skip Rust install | `1` |

## Verification

```bash
vmtools --version            # Check installation
vmtools config --show       # Show configuration
vmtools list --all          # List VMs
./post-install.sh health    # Health check
```

## Troubleshooting

| Issue | Solution |
|-------|----------|
| `vmtools: command not found` | `export PATH="$HOME/.local/bin:$PATH"` |
| Permission denied | `sudo usermod -aG libvirt $USER && logout` |
| KVM not available | Enable VT-x/AMD-V in BIOS |
| Build failed | `rustup update && ./build.sh clean` |

## Uninstall

```bash
./install.sh uninstall      # Complete removal
make uninstall               # Remove binary only
```

## Quick Start After Install

```bash
# Create and start a VM
vmtools create test-vm --template ubuntu-server --memory 2048
vmtools start test-vm
vmtools status test-vm

# Monitor and manage
vmtools monitor test-vm      # Real-time monitoring
vmtools console test-vm      # Console access
vmtools stop test-vm         # Graceful shutdown
vmtools delete test-vm       # Remove VM
```

## Advanced Deployment

```bash
# Silent install
SKIP_INTERACTIVE=1 ./install.sh quick

# Custom location
INSTALL_DIR=/opt/vmtools ./install.sh quick

# Skip dependencies (if already installed)
SKIP_DEPENDENCIES=1 ./install.sh user

# Development with custom settings
SKIP_RUST=1 ./install.sh dev
```

---

**Need help?** Run any command with `--help` or check `README.md` for full documentation.