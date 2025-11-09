use std::path::{Path, PathBuf};
use tokio::process::Command;
use rand::Rng;

use crate::{
    error::{VmError, Result},
    config::Config,
};

/// Validates and sanitizes a file path to prevent path traversal attacks (CWE-22)
/// This function ensures the path is safe to read from and doesn't contain path traversal sequences
/// Validates system file paths to prevent CWE-22 path traversal attacks
/// This function provides comprehensive path validation including:
/// - Path canonicalization to resolve symbolic links and relative components
/// - Prefix validation to ensure paths stay within expected directories
/// - Path traversal sequence detection to block malicious patterns
fn validate_system_file_path(path: &Path, expected_prefix: &str) -> Result<PathBuf> {
    // SECURITY: Convert to canonical path to resolve any symbolic links and relative components
    // This prevents path traversal attacks using symbolic links or ".." sequences
    let canonical_path = path.canonicalize()
        .map_err(|_| VmError::SecurityError(format!("Invalid or inaccessible path: {}", path.display())))?;
    
    // SECURITY: Ensure the path starts with the expected system prefix (e.g., "/proc/", "/dev/")
    // This implements a whitelist approach to prevent access to unauthorized directories
    if !canonical_path.starts_with(expected_prefix) {
        return Err(VmError::SecurityError(format!(
            "Path traversal attempt detected: {} does not start with expected prefix {}", 
            canonical_path.display(), 
            expected_prefix
        )));
    }
    
    // SECURITY: Additional defense-in-depth check for path traversal sequences
    // Even after canonicalization, ensure no suspicious patterns remain
    let path_str = canonical_path.to_string_lossy();
    if path_str.contains("..") || path_str.contains("./") {
        return Err(VmError::SecurityError(format!(
            "Path traversal sequences detected in: {}", 
            path_str
        )));
    }
    
    // Path has been validated and is safe to use
    Ok(canonical_path)
}

/// Secure file reader that only reads validated system files
/// This function encapsulates the security validation and file reading
/// to prevent path traversal vulnerabilities (CWE-22)
async fn read_validated_system_file(file_path: &Path, expected_prefix: &str) -> Result<String> {
    // SECURITY: First validate the path to prevent path traversal
    let validated_path = validate_system_file_path(file_path, expected_prefix)?;
    
    // SECURITY: Use explicit hardcoded paths for known-safe system files
    // This completely eliminates any possibility of path traversal
    let canonical_str = validated_path.to_string_lossy();
    
    let content = match expected_prefix {
        "/proc/" => {
            if canonical_str == "/proc/cpuinfo" {
                tokio::fs::read_to_string("/proc/cpuinfo").await
            } else if canonical_str == "/proc/meminfo" {
                tokio::fs::read_to_string("/proc/meminfo").await
            } else {
                return Err(VmError::SecurityError("Unauthorized proc file access".to_string()));
            }
        },
        "/dev/" => {
            if canonical_str == "/dev/kvm" {
                tokio::fs::read_to_string("/dev/kvm").await
            } else {
                return Err(VmError::SecurityError("Unauthorized dev file access".to_string()));
            }
        },
        _ => return Err(VmError::SecurityError("Unauthorized file access".to_string()))
    };
    
    content.map_err(|e| VmError::IoError(e))
}

pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

pub fn format_duration(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    if days > 0 {
        format!("{}d {}h {}m", days, hours, minutes)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else if minutes > 0 {
        format!("{}m {}s", minutes, secs)
    } else {
        format!("{}s", secs)
    }
}

pub fn generate_mac_address() -> String {
    let mut rng = rand::thread_rng();
    format!(
        "52:54:00:{:02x}:{:02x}:{:02x}",
        rng.gen::<u8>(),
        rng.gen::<u8>(),
        rng.gen::<u8>()
    )
}

pub async fn create_qcow2_image<P: AsRef<Path>>(path: P, size_bytes: u64) -> Result<()> {
    let size_str = format!("{}G", size_bytes / (1024 * 1024 * 1024));
    
    let output = Command::new("qemu-img")
        .args(&[
            "create",
            "-f", "qcow2",
            path.as_ref().to_str().unwrap(),
            &size_str
        ])
        .output()
        .await
        .map_err(|e| VmError::IoError(e))?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(VmError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to create qcow2 image: {}", error)
        )));
    }

    Ok(())
}

pub async fn clone_qcow2_image<P: AsRef<Path>>(source: P, target: P) -> Result<()> {
    let output = Command::new("qemu-img")
        .args(&[
            "convert",
            "-f", "qcow2",
            "-O", "qcow2",
            source.as_ref().to_str().unwrap(),
            target.as_ref().to_str().unwrap()
        ])
        .output()
        .await
        .map_err(|e| VmError::IoError(e))?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(VmError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to clone qcow2 image: {}", error)
        )));
    }

    Ok(())
}

#[allow(dead_code)]
pub async fn get_image_info<P: AsRef<Path>>(path: P) -> Result<ImageInfo> {
    let output = Command::new("qemu-img")
        .args(&["info", "--output=json", path.as_ref().to_str().unwrap()])
        .output()
        .await
        .map_err(|e| VmError::IoError(e))?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(VmError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to get image info: {}", error)
        )));
    }

    let json_str = String::from_utf8_lossy(&output.stdout);
    let info: serde_json::Value = serde_json::from_str(&json_str)
        .map_err(|e| VmError::SerdeError(e))?;

    Ok(ImageInfo {
        format: info["format"].as_str().unwrap_or("unknown").to_string(),
        virtual_size: info["virtual-size"].as_u64().unwrap_or(0),
        actual_size: info["actual-size"].as_u64().unwrap_or(0),
        filename: info["filename"].as_str().unwrap_or("").to_string(),
    })
}

#[allow(dead_code)]
pub async fn resize_image<P: AsRef<Path>>(path: P, new_size: u64) -> Result<()> {
    let size_str = format!("{}G", new_size / (1024 * 1024 * 1024));
    
    let output = Command::new("qemu-img")
        .args(&[
            "resize",
            path.as_ref().to_str().unwrap(),
            &size_str
        ])
        .output()
        .await
        .map_err(|e| VmError::IoError(e))?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(VmError::IoError(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to resize image: {}", error)
        )));
    }

    Ok(())
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ImageInfo {
    pub format: String,
    pub virtual_size: u64,
    pub actual_size: u64,
    pub filename: String,
}

#[allow(dead_code)]
pub fn validate_vm_name(name: &str) -> Result<()> {
    // Check for empty or whitespace-only names
    if name.trim().is_empty() {
        return Err(VmError::InvalidInput("VM name cannot be empty".to_string()));
    }

    // Check length (reasonable limit)
    if name.len() > 64 {
        return Err(VmError::InvalidInput("VM name too long (max 64 characters)".to_string()));
    }

    // Check for path traversal sequences - SECURITY: Prevent CWE-22 path traversal
    if name.contains("..") || name.contains('/') || name.contains('\\') {
        return Err(VmError::SecurityError(format!(
            "VM name contains prohibited characters that could lead to path traversal: {}", 
            name
        )));
    }

    // Only allow alphanumeric characters, hyphens, and underscores (no dots to prevent hidden files)
    if !name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err(VmError::InvalidInput(
            "VM name can only contain alphanumeric characters, hyphens, and underscores".to_string()
        ));
    }

    // Prevent names that start or end with hyphens
    if name.starts_with('-') || name.ends_with('-') {
        return Err(VmError::InvalidInput("VM name cannot start or end with a hyphen".to_string()));
    }

    Ok(())
}

#[allow(dead_code)]
pub fn validate_memory(memory_mb: u64) -> Result<()> {
    if memory_mb < 128 {
        return Err(VmError::InvalidInput("Memory must be at least 128MB".to_string()));
    }

    if memory_mb > 1024 * 1024 { // 1TB
        return Err(VmError::InvalidInput("Memory cannot exceed 1TB".to_string()));
    }

    Ok(())
}

#[allow(dead_code)]
pub fn validate_cpus(cpus: u32) -> Result<()> {
    if cpus == 0 {
        return Err(VmError::InvalidInput("CPU count must be at least 1".to_string()));
    }

    if cpus > 256 {
        return Err(VmError::InvalidInput("CPU count cannot exceed 256".to_string()));
    }

    Ok(())
}

#[allow(dead_code)]
pub fn validate_disk_size(size_gb: u64) -> Result<()> {
    if size_gb == 0 {
        return Err(VmError::InvalidInput("Disk size must be at least 1GB".to_string()));
    }

    if size_gb > 10240 { // 10TB
        return Err(VmError::InvalidInput("Disk size cannot exceed 10TB".to_string()));
    }

    Ok(())
}

#[allow(dead_code)]
pub async fn check_libvirt_running() -> Result<()> {
    let output = Command::new("systemctl")
        .args(&["is-active", "libvirtd"])
        .output()
        .await
        .map_err(|e| VmError::LibvirtError(format!("Failed to check libvirtd status: {}", e)))?;

    if !output.status.success() {
        return Err(VmError::LibvirtError("libvirtd service is not running".to_string()));
    }

    let status = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if status != "active" {
        return Err(VmError::LibvirtError(format!("libvirtd service status: {}", status)));
    }

    Ok(())
}

#[allow(dead_code)]
pub async fn check_kvm_support(config: &Config) -> Result<()> {
    // Check if KVM module is loaded
    let output = Command::new("lsmod")
        .output()
        .await
        .map_err(|e| VmError::IoError(e))?;

    let lsmod_output = String::from_utf8_lossy(&output.stdout);
    if !lsmod_output.contains("kvm") {
        return Err(VmError::ResourceUnavailable("KVM module is not loaded".to_string()));
    }

    // Validate and check if /dev/kvm exists and is accessible using configurable path
    let validated_kvm_path = validate_system_file_path(&config.system.kvm_device, "/dev/")?;
    if !tokio::fs::try_exists(&validated_kvm_path).await.unwrap_or(false) {
        return Err(VmError::ResourceUnavailable(format!("{} device not found", validated_kvm_path.display())));
    }

    Ok(())
}

#[allow(dead_code)]
pub async fn get_host_info(config: &Config) -> Result<HostInfo> {
    // SECURITY: Use secure file reader to prevent CWE-22 path traversal
    let cpuinfo = read_validated_system_file(&config.system.proc_cpuinfo, "/proc/").await?;
    
    let cpu_count = cpuinfo.lines()
        .filter(|line| line.starts_with("processor"))
        .count() as u32;

    // SECURITY: Use secure file reader to prevent CWE-22 path traversal
    let meminfo = read_validated_system_file(&config.system.proc_meminfo, "/proc/").await?;
    
    let mut total_memory = 0;
    for line in meminfo.lines() {
        if line.starts_with("MemTotal:") {
            if let Some(kb_str) = line.split_whitespace().nth(1) {
                if let Ok(kb) = kb_str.parse::<u64>() {
                    total_memory = kb / 1024; // Convert to MB
                }
            }
            break;
        }
    }

    Ok(HostInfo {
        cpu_count,
        total_memory,
        architecture: std::env::consts::ARCH.to_string(),
        os: "Linux".to_string(),
    })
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct HostInfo {
    pub cpu_count: u32,
    pub total_memory: u64, // in MB
    pub architecture: String,
    pub os: String,
}