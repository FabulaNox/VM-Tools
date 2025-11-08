# Configurable Paths in VM-Tools

VM-Tools now supports configurable paths through the configuration file to avoid hardcoded paths and improve flexibility.

## Configuration File Location

The main configuration file is located at `~/.config/vmtools/config.toml`. Use the provided `config.sample.toml` as a template.

## Configurable System Paths

### 1. Temporary Directory (`system.temp_dir`)
- **Default**: `/tmp`
- **Purpose**: Used for temporary XML files during VM operations
- **Customization**: Can be changed to any writable directory
- **Example**: `/var/tmp/vmtools` or `~/.cache/vmtools/tmp`

### 2. KVM Device Path (`system.kvm_device`)
- **Default**: `/dev/kvm`
- **Purpose**: Hardware virtualization device validation
- **Customization**: Should only be changed for testing or non-standard setups
- **Note**: This path is usually standard on Linux systems

### 3. System Information Paths
- **CPU Info** (`system.proc_cpuinfo`): Default `/proc/cpuinfo`
- **Memory Info** (`system.proc_meminfo`): Default `/proc/meminfo`
- **Purpose**: Host system resource detection
- **Customization**: Useful for testing or alternative system layouts

### 4. Storage Paths
- **VM Images** (`storage.vm_images_path`): Default `/var/lib/libvirt/images`
- **ISO Files** (`storage.iso_path`): Default `/var/lib/libvirt/images/iso`
- **Backups** (`storage.backup_path`): Default `/var/lib/libvirt/backup`

### 5. Libvirt Configuration
- **Socket Path** (`libvirt.socket_path`): Default `/var/run/libvirt/libvirt-sock`
- **URI** (`libvirt.uri`): Default `qemu:///system`

## Benefits of Configurable Paths

1. **Flexibility**: Adapt to different system layouts and requirements
2. **Testing**: Use alternative paths for development and testing
3. **Security**: Store sensitive data in secure locations
4. **Performance**: Use faster storage for temporary files
5. **Multi-tenant**: Support different configurations per user

## Migration from Hardcoded Paths

The following hardcoded paths have been made configurable:

| Component | Old Hardcoded Path | New Config Key |
|-----------|-------------------|----------------|
| Temp files | `/tmp/vmtools_domain_*.xml` | `system.temp_dir` |
| KVM device | `/dev/kvm` | `system.kvm_device` |
| CPU info | `/proc/cpuinfo` | `system.proc_cpuinfo` |
| Memory info | `/proc/meminfo` | `system.proc_meminfo` |
| VM images | `/var/lib/libvirt/images` | `storage.vm_images_path` |
| ISO files | `/var/lib/libvirt/images/iso` | `storage.iso_path` |
| Backups | `/var/lib/libvirt/backup` | `storage.backup_path` |
| Libvirt socket | `/var/run/libvirt/libvirt-sock` | `libvirt.socket_path` |

## Example Custom Configuration

```toml
[system]
# Use a custom temp directory with better performance
temp_dir = "/dev/shm/vmtools"
# Standard paths for system files
kvm_device = "/dev/kvm"
proc_cpuinfo = "/proc/cpuinfo"
proc_meminfo = "/proc/meminfo"

[storage]
# Use custom storage locations
vm_images_path = "/storage/vms/images"
iso_path = "/storage/vms/iso"
backup_path = "/backup/vms"

[libvirt]
# Custom libvirt configuration
uri = "qemu:///system"
socket_path = "/var/run/libvirt/libvirt-sock"
timeout = 60
```

## Security Considerations

1. Ensure all configured directories have proper permissions
2. Temporary directories should be writable only by the user
3. Storage paths should be on secure filesystems
4. Backup paths should have adequate security and access controls

## Backwards Compatibility

All default values maintain backwards compatibility with existing installations. No configuration changes are required for current users unless custom paths are desired.