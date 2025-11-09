# Security Improvements: CWE-22 Path Traversal Prevention

## Overview

This document outlines the security improvements implemented to prevent CWE-22 (Path Traversal) vulnerabilities in VM-Tools.

## Vulnerabilities Addressed

### 1. Path Traversal in System File Access (CWE-22)

**Location**: `src/utils.rs` lines 271 and 279 (original)
**Issue**: The application was reading system files using configurable paths without proper validation, potentially allowing path traversal attacks.

**Risk**: An attacker could manipulate configuration paths to:
- Access sensitive files outside intended directories
- Read configuration files, user data, or system information
- Potentially escalate privileges or gain unauthorized access

### 2. VM Name Path Traversal (CWE-22)

**Location**: Multiple VM operations in `src/vm.rs`
**Issue**: VM names were used to construct file paths without validation, allowing potential path traversal.

**Risk**: Malicious VM names could:
- Create files outside intended directories
- Overwrite critical system files
- Access or modify files in parent directories

## Security Implementations

### 1. System File Path Validation

Added `validate_system_file_path()` function that:
- **Canonicalizes paths** to resolve symbolic links and relative components
- **Validates path prefixes** to ensure files are within expected directories (e.g., `/proc/`, `/dev/`)
- **Detects path traversal sequences** like `..` and `./`
- **Returns SecurityError** for any suspicious path manipulation attempts

```rust
fn validate_system_file_path(path: &Path, expected_prefix: &str) -> Result<PathBuf> {
    let canonical_path = path.canonicalize()
        .map_err(|_| VmError::SecurityError(format!("Invalid path: {}", path.display())))?;
    
    if !canonical_path.starts_with(expected_prefix) {
        return Err(VmError::SecurityError("Path traversal attempt detected".to_string()));
    }
    
    // Additional security checks...
}
```

### 2. VM Name Validation Enhancement

Enhanced `validate_vm_name()` function with security features:
- **Path traversal detection** for `..`, `/`, and `\\` characters
- **Character restriction** to alphanumeric, hyphens, and underscores only
- **Length limits** to prevent buffer overflow attempts
- **Prefix restrictions** to prevent hidden files or invalid names

```rust
pub fn validate_vm_name(name: &str) -> Result<()> {
    // Check for path traversal sequences - SECURITY: Prevent CWE-22
    if name.contains("..") || name.contains('/') || name.contains('\\') {
        return Err(VmError::SecurityError(format!(
            "VM name contains prohibited characters: {}", name
        )));
    }
    // Additional validation...
}
```

### 3. Comprehensive VM Function Protection

Added validation calls to all VM management functions:
- `create_vm()` - Validates VM name before creating files
- `clone_vm()` - Validates both source and target VM names
- `start_vm()` - Validates VM name before operations
- `stop_vm()` - Validates VM name before shutdown
- `delete_vm()` - Validates VM name before deletion
- `get_vm_status()` - Validates VM name before status queries
- `monitor_vm()` - Validates VM name before monitoring
- `connect_console()` - Validates VM name before console connection

### 4. Security Error Handling

Added new `SecurityError` variant to `VmError` enum:
```rust
#[error("Security error: {0}")]
SecurityError(String),
```

This allows for proper categorization and handling of security-related errors.

## Protected Operations

### System File Operations

1. **KVM Device Access**: `/dev/kvm` path validation
2. **CPU Info Reading**: `/proc/cpuinfo` path validation  
3. **Memory Info Reading**: `/proc/meminfo` path validation

### VM File Operations

1. **Disk Image Creation**: VM name validation before path construction
2. **Backup Operations**: Path validation for backup locations
3. **Configuration Files**: Secure handling of XML and config files

## Security Best Practices Implemented

1. **Input Validation**: All user-provided names and paths are validated
2. **Path Canonicalization**: Resolves symbolic links and relative paths
3. **Whitelist Approach**: Only allowed characters and patterns are permitted
4. **Fail-Safe Defaults**: Security errors cause operations to fail safely
5. **Comprehensive Coverage**: Protection applied to all VM operations
6. **Clear Error Messages**: Security violations provide informative feedback

## Testing Security Improvements

### Test Cases for Path Traversal Prevention

```bash
# These should all fail with SecurityError:
vmtools create ../../../etc/passwd 2048 2 20
vmtools create "vm..name" 2048 2 20  
vmtools create "vm/with/slash" 2048 2 20
vmtools create "vm\\with\\backslash" 2048 2 20
vmtools clone source ../target
```

### Test Cases for System File Protection

Configuration attempts with malicious paths should fail:
```toml
[system]
kvm_device = "../../../etc/shadow"
proc_cpuinfo = "/etc/passwd"
proc_meminfo = "../../home/user/.ssh/id_rsa"
```

## Compliance

These improvements address:
- **CWE-22**: Improper Limitation of a Pathname to a Restricted Directory ('Path Traversal')
- **OWASP Top 10**: A01:2021 – Broken Access Control
- **Security Best Practices**: Input validation and path sanitization

## Backwards Compatibility

All security improvements maintain backwards compatibility:
- Valid VM names continue to work unchanged
- Standard system paths function normally
- Default configurations remain secure
- No breaking changes to existing functionality

## Monitoring and Logging

Security violations are logged with clear error messages:
- Path traversal attempts are detected and reported
- Invalid VM names are rejected with explanations
- System file access violations are documented
- All security errors include contextual information

This comprehensive approach ensures VM-Tools is protected against path traversal attacks while maintaining usability and backwards compatibility.