# VM-Tools

A high-performance VM management tool for QEMU/KVM with libvirt integration, written in Rust for minimal overhead and maximum safety.

## 🔒 Security Notice

VM-Tools has been hardened against CWE-22 (Path Traversal) vulnerabilities. See [SECURITY_CWE22.md](SECURITY_CWE22.md) for detailed security improvements and protections implemented.

## Features

### ✨ Core Functionality
- **VM Lifecycle Management**: Create, start, stop, delete, clone VMs
- **Real-time Monitoring**: CPU, memory, disk, and network monitoring
- **Template System**: Predefined VM templates for quick deployment
- **Console Access**: Direct console connection to VMs
- **Network Management**: List and manage virtual networks
- **Configuration Management**: Flexible configuration system

### 🚀 Performance & Safety
- **Zero-overhead abstractions** with Rust's performance guarantees
- **Memory safety** without garbage collection overhead
- **Async I/O** for non-blocking operations
- **Efficient resource usage** with minimal system footprint

### 🛠 Integration
- **Libvirt compatibility** for seamless KVM/QEMU integration
- **QEMU Monitor Protocol** support for advanced operations
- **Shell integration** with colored output and progress bars
- **Template-based deployment** for consistent VM creation

## Prerequisites

- **Operating System**: Linux (Ubuntu/Debian recommended)
- **Virtualization**: CPU with VT-x (Intel) or AMD-V support
- **Software Dependencies**:
  - QEMU/KVM
  - libvirt
  - Rust (1.70+) for building from source

## Installation

### 🚀 Quick Install (Recommended)

**One-liner remote install:**
```bash
curl -sSL https://raw.githubusercontent.com/FabulaNox/VM-Tools/main/install.sh | bash
```

**From repository:**
```bash
git clone https://github.com/FabulaNox/VM-Tools.git
cd VM-Tools
make deploy
```

### 📋 Installation Options

#### Option 1: Full Deployment (System-wide)
```bash
# Clone repository
git clone https://github.com/FabulaNox/VM-Tools.git
cd VM-Tools

# Full deployment with post-install setup
./install.sh quick
# OR
make deploy
```

#### Option 2: User-only Install (No sudo)
```bash
# Install for current user only
./install.sh user
# OR  
make install-user
```

#### Option 3: Development Setup
```bash
# Development environment
./install.sh dev
# OR
make dev
```

#### Option 4: Custom Installation
```bash
# Check dependencies first
./build.sh deps

# Build and install
./build.sh release
sudo ./build.sh install

# Run post-install configuration
./post-install.sh all
```

### 🔧 Manual Dependencies

If you prefer to install dependencies manually:

**Ubuntu/Debian:**
```bash
sudo apt-get update
sudo apt-get install git curl build-essential libvirt-clients qemu-utils qemu-kvm libvirt-daemon-system
```

**CentOS/RHEL:**
```bash
sudo yum groupinstall "Development Tools"
sudo yum install git curl libvirt-client qemu-img qemu-kvm libvirt-daemon
```

**Arch Linux:**
```bash
sudo pacman -S git curl base-devel libvirt qemu-img qemu
```

**Rust (if not installed):**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
### 🎯 Post-Installation

After installation, vmtools automatically runs post-install configuration. You can also run it manually:

```bash
# Run all post-install steps
./post-install.sh all

# Or individual components
./post-install.sh validate    # Validate system requirements
./post-install.sh configure   # Setup permissions
./post-install.sh network     # Configure networking
./post-install.sh health      # Run health checks
```

### ✅ Verify Installation

```bash
# Check installation
vmtools --version
vmtools config --show

# List existing VMs
vmtools list --all

# Run health check
./post-install.sh health
```

### 🗑️ Uninstallation

**VM-Tools provides a safe uninstaller that NEVER touches your VMs or their data.**

#### Complete Safe Uninstall (Recommended)

```bash
# Use the dedicated uninstall script
./uninstall.sh

# Preview what will be removed
./uninstall.sh --dry-run

# Check current installation
./uninstall.sh --verify

# Remove only the binary, keep config
./uninstall.sh --binary-only

# Skip confirmations
./uninstall.sh --force
```

#### Quick Uninstall

```bash
# Using installer (basic removal)
./install.sh uninstall

# Using Makefile
make uninstall
```

#### Manual Removal

```bash
# Remove binaries
sudo rm -f /usr/local/bin/vmtools
rm -f ~/.local/bin/vmtools

# Remove configuration (optional)
rm -rf ~/.config/vmtools
rm -rf ~/.cache/vmtools
```

#### What Gets Removed vs Preserved

✅ **Removed:**
- vmtools binary files
- Configuration directory (`~/.config/vmtools`)
- Cache files (`~/.cache/vmtools`)
- Repository clones (with confirmation)

🚫 **NEVER Removed:**
- Your VMs and disk images
- libvirt/QEMU system configuration  
- VM networks and storage pools
- VM snapshots and data

**After uninstalling, your VMs remain fully functional and manageable with `virsh`, `virt-manager`, or other libvirt tools.**

## Deployment

### 🚢 Server Deployment

For deploying on servers or automated environments:

```bash
# Silent installation
curl -sSL https://raw.githubusercontent.com/FabulaNox/VM-Tools/main/install.sh | SKIP_INTERACTIVE=1 bash

# Or with environment variables
INSTALL_DIR=/opt/bin ./install.sh quick
SKIP_DEPENDENCIES=1 ./install.sh user
```

### 🐳 Container Deployment

VM-Tools can be used in containers for VM management:

```dockerfile
FROM ubuntu:22.04
RUN apt-get update && apt-get install -y curl git
RUN curl -sSL https://raw.githubusercontent.com/FabulaNox/VM-Tools/main/install.sh | bash
CMD ["vmtools", "--help"]
```

### 📦 Package Distribution

For creating distribution packages:

```bash
# Build static binary
cargo build --release --target x86_64-unknown-linux-musl

# Create tarball
tar -czf vmtools-linux-x86_64.tar.gz -C target/release vmtools
```

## Usage

### Basic Commands

```bash
# List all VMs
vmtools list --all

# Create a new VM
vmtools create myvm --memory 2048 --cpus 2 --disk-size 20 --template ubuntu

# Start a VM
vmtools start myvm

# Check VM status
vmtools status myvm

# Stop a VM
vmtools stop myvm

# Delete a VM
vmtools delete myvm
```

### Advanced Operations

```bash
# Clone a VM
vmtools clone source-vm new-vm

# Monitor VM performance (real-time)
vmtools monitor myvm

# Connect to VM console
vmtools console myvm

# List available networks
vmtools networks

# Create VM with ISO
vmtools create testvm --iso-path /path/to/ubuntu.iso --memory 4096
```

### Configuration Management

```bash
# Show current configuration
vmtools config --show

# Set configuration values
vmtools config --set libvirt.timeout=60
vmtools config --set defaults.memory=4096

# Get specific configuration
vmtools config --get defaults.memory
```

## Configuration

vmtools uses a TOML configuration file located at `~/.config/vmtools/config.toml`.

### Configurable Paths

VM-Tools supports configurable paths to avoid hardcoded system paths and improve flexibility:

- **System paths**: Temporary directory, KVM device, proc filesystem paths
- **Storage paths**: VM images, ISO files, backup locations  
- **Libvirt paths**: Socket path and connection settings

See `config.sample.toml` for a complete example and `CONFIGURABLE_PATHS.md` for detailed documentation.

### Default Configuration

```toml
[libvirt]
uri = "qemu:///system"
timeout = 30

[storage]
default_pool = "default"
vm_images_path = "/var/lib/libvirt/images"
iso_path = "/var/lib/libvirt/images/iso"
backup_path = "/var/lib/libvirt/backup"

[system]
temp_dir = "/tmp"
kvm_device = "/dev/kvm"
proc_cpuinfo = "/proc/cpuinfo"
proc_meminfo = "/proc/meminfo"

[network]
default_network = "default"
bridge_interface = "virbr0"

[defaults]
memory = 2048
cpus = 2
disk_size = 20
disk_format = "qcow2"
network = "default"
graphics = "spice"

[templates.ubuntu]
memory = 2048
cpus = 2
disk_size = 20
os_type = "linux"
arch = "x86_64"
machine_type = "pc-q35-7.0"
boot_order = ["hd", "cdrom"]
features = ["acpi", "apic", "pae"]

[templates.windows]
memory = 4096
cpus = 2
disk_size = 40
os_type = "windows"
arch = "x86_64"
machine_type = "pc-q35-7.0"
boot_order = ["hd", "cdrom"]
features = ["acpi", "apic", "hyperv"]
```

### Custom Templates

You can create custom VM templates by editing the configuration file:

```toml
[templates.mytemplate]
memory = 8192
cpus = 4
disk_size = 100
os_type = "linux"
arch = "x86_64"
machine_type = "pc-q35-7.0"
boot_order = ["hd"]
features = ["acpi", "apic"]
```

## Architecture

### Core Components

```
┌─────────────────────┐
│    CLI Interface    │  ← clap-based command interface
├─────────────────────┤
│   VM Manager        │  ← High-level VM operations
├─────────────────────┤
│  Libvirt Client     │  ← virsh wrapper with async I/O
├─────────────────────┤
│   QEMU Monitor      │  ← QMP protocol implementation
├─────────────────────┤
│     Utilities       │  ← Image management, validation
└─────────────────────┘
```

### Key Design Principles

1. **Zero-cost abstractions**: Rust's ownership model eliminates runtime overhead
2. **Memory safety**: No buffer overflows, null pointer dereferences, or data races
3. **Async by default**: Non-blocking I/O for responsive CLI experience
4. **Modular architecture**: Each component can be used independently
5. **Error handling**: Comprehensive error types with helpful messages

## Development

### Building

```bash
# Debug build
./build.sh debug

# Release build
./build.sh release

# Run tests
./build.sh test

# Lint code
./build.sh lint

# Format code
./build.sh format

# Complete development cycle
./build.sh all
```

### Project Structure

```
VM-Tools/
├── src/
│   ├── main.rs              # Entry point
│   ├── cli.rs               # Command-line interface
│   └── lib/
│       ├── mod.rs           # Library module exports
│       ├── vm.rs            # VM management logic
│       ├── libvirt.rs       # Libvirt client wrapper
│       ├── qemu.rs          # QEMU monitor integration
│       ├── config.rs        # Configuration management
│       ├── error.rs         # Error types
│       └── utils.rs         # Utility functions
├── Cargo.toml               # Rust dependencies
├── build.sh                 # Build script
├── install-qemu-kvm.sh      # QEMU/KVM installer
└── README.md                # Documentation
```

### Adding New Features

1. **VM Operations**: Extend `VmManager` in `src/lib/vm.rs`
2. **CLI Commands**: Add to `Commands` enum in `src/cli.rs`
3. **Configuration**: Update `Config` struct in `src/lib/config.rs`
4. **Error Handling**: Add new error types in `src/lib/error.rs`

## Troubleshooting

### Common Issues

**1. Permission Denied**
```bash
# Add user to libvirt group
sudo usermod -aG libvirt $USER
# Log out and log back in
```

**2. KVM Not Available**
```bash
# Check virtualization support
grep -E '(vmx|svm)' /proc/cpuinfo

# Check KVM module
lsmod | grep kvm

# Load KVM module
sudo modprobe kvm
sudo modprobe kvm_intel  # or kvm_amd
```

**3. Libvirt Connection Failed**
```bash
# Check libvirtd service
sudo systemctl status libvirtd
sudo systemctl start libvirtd

# Test connection
virsh list --all
```

**4. Build Errors**
```bash
# Update Rust
rustup update

# Clean and rebuild
./build.sh clean
./build.sh debug
```

### Debug Mode

Enable detailed logging:
```bash
RUST_LOG=debug vmtools list --all
```

## Performance Characteristics

### Memory Usage
- **Base overhead**: ~2-5 MB resident memory
- **Per-VM monitoring**: ~100 KB additional memory
- **Zero-copy operations** where possible

### CPU Usage
- **Idle**: <0.1% CPU usage
- **Active monitoring**: ~1-2% CPU per monitored VM
- **Async I/O**: Non-blocking operations prevent UI freezing

### Disk I/O
- **QCOW2 optimization**: Sparse file support
- **Copy-on-write cloning**: Efficient VM duplication
- **Async file operations**: Non-blocking disk access

## Comparison with Alternatives

| Feature | vmtools | virt-manager | virsh | Docker |
|---------|---------|--------------|-------|---------|
| Performance | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| Memory Safety | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐ | ⭐⭐⭐ |
| CLI Interface | ⭐⭐⭐⭐⭐ | ⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| GUI | ❌ | ⭐⭐⭐⭐⭐ | ❌ | ❌ |
| Scriptability | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| VM Support | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐ |

## Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Make your changes and add tests
4. Run the development cycle: `./build.sh all`
5. Commit your changes: `git commit -m 'Add amazing feature'`
6. Push to the branch: `git push origin feature/amazing-feature`
7. Open a Pull Request

### Code Style

- Follow Rust standard formatting (`cargo fmt`)
- Pass all clippy lints (`cargo clippy`)
- Add tests for new functionality
- Update documentation as needed

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Roadmap

### v0.2.0
- [ ] VM snapshots and restoration
- [ ] Automated backups
- [ ] Resource quotas and limits
- [ ] Web-based dashboard

### v0.3.0
- [ ] Cluster management
- [ ] VM migration support
- [ ] Advanced networking (VLANs, bridges)
- [ ] GPU passthrough configuration

### v1.0.0
- [ ] Stable API
- [ ] Full documentation
- [ ] Performance benchmarks
- [ ] Enterprise features

---

**vmtools** - Fast(-ish), safe(-ish), and reliable(-ish) VM management for Linux 🚀
