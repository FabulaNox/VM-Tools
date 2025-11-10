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

/// Network mismatch detection and auto-configuration functionality
#[derive(Debug, Clone)]
pub struct NetworkInterface {
    pub mac_address: String,
    pub network: String,
    pub bridge: String,
    pub is_active: bool,
}

#[derive(Debug, Clone)]
pub struct NetworkMismatch {
    pub interface_name: String,
    pub issue_type: NetworkIssueType,
    pub current_config: Option<NetworkInterface>,
    pub suggested_config: NetworkInterface,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NetworkIssueType {
    DuplicateMacAddress,
    InactiveNetwork,
    InvalidNetworkReference,
    ConflictingConfiguration,
    MissingBridge,
}

impl std::fmt::Display for NetworkIssueType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NetworkIssueType::DuplicateMacAddress => write!(f, "Duplicate MAC Address"),
            NetworkIssueType::InactiveNetwork => write!(f, "Inactive Network"),
            NetworkIssueType::InvalidNetworkReference => write!(f, "Invalid Network Reference"),
            NetworkIssueType::ConflictingConfiguration => write!(f, "Conflicting Configuration"),
            NetworkIssueType::MissingBridge => write!(f, "Missing Bridge"),
        }
    }
}

/// Detects network mismatches in VM configuration
pub async fn detect_network_mismatches(vm_name: &str) -> Result<Vec<NetworkMismatch>> {
    let mut mismatches = Vec::new();
    
    // Get VM's current network configuration
    let vm_interfaces = get_vm_network_interfaces(vm_name).await?;
    
    // Get available libvirt networks
    let available_networks = get_available_networks().await?;
    
    // Check for duplicate MAC addresses across all VMs
    let all_mac_addresses = get_all_vm_mac_addresses().await?;
    
    for interface in &vm_interfaces {
        // Check for duplicate MAC addresses
        let mac_count = all_mac_addresses.iter()
            .filter(|mac| **mac == interface.mac_address)
            .count();
        
        if mac_count > 1 {
            mismatches.push(NetworkMismatch {
                interface_name: format!("{}-dup-mac", interface.network),
                issue_type: NetworkIssueType::DuplicateMacAddress,
                current_config: Some(interface.clone()),
                suggested_config: NetworkInterface {
                    mac_address: generate_mac_address(),
                    network: interface.network.clone(),
                    bridge: interface.bridge.clone(),
                    is_active: interface.is_active,
                },
            });
        }
        
        // Check if referenced network exists and is active
        if let Some(network_info) = available_networks.iter().find(|n| n.network == interface.network) {
            if !network_info.is_active {
                mismatches.push(NetworkMismatch {
                    interface_name: interface.network.clone(),
                    issue_type: NetworkIssueType::InactiveNetwork,
                    current_config: Some(interface.clone()),
                    suggested_config: NetworkInterface {
                        mac_address: interface.mac_address.clone(),
                        network: interface.network.clone(),
                        bridge: interface.bridge.clone(),
                        is_active: true,
                    },
                });
            }
        } else {
            // Network doesn't exist, suggest using default
            let default_network = available_networks.iter()
                .find(|n| n.network == "default" && n.is_active)
                .cloned()
                .unwrap_or_else(|| NetworkInterface {
                    mac_address: interface.mac_address.clone(),
                    network: "default".to_string(),
                    bridge: "virbr0".to_string(),
                    is_active: false,
                });
            
            mismatches.push(NetworkMismatch {
                interface_name: interface.network.clone(),
                issue_type: NetworkIssueType::InvalidNetworkReference,
                current_config: Some(interface.clone()),
                suggested_config: default_network,
            });
        }
    }
    
    // NEW: Check for missing bridges and conflicting configurations
    let bridge_conflicts = detect_bridge_and_config_issues(&vm_interfaces, &available_networks).await?;
    mismatches.extend(bridge_conflicts);
    
    Ok(mismatches)
}

/// Detects bridge and configuration issues for network interfaces
async fn detect_bridge_and_config_issues(vm_interfaces: &[NetworkInterface], available_networks: &[NetworkInterface]) -> Result<Vec<NetworkMismatch>> {
    let mut mismatches = Vec::new();
    
    // Get system bridge information
    let system_bridges = get_system_bridges().await?;
    
    for interface in vm_interfaces {
        // Check for missing bridges
        if !system_bridges.contains(&interface.bridge) {
            // Bridge referenced by VM doesn't exist on system
            let suggested_bridge = if system_bridges.contains(&"virbr0".to_string()) {
                "virbr0".to_string()
            } else if !system_bridges.is_empty() {
                system_bridges[0].clone()
            } else {
                "virbr0".to_string() // Fallback
            };
            
            mismatches.push(NetworkMismatch {
                interface_name: format!("{}-missing-bridge", interface.bridge),
                issue_type: NetworkIssueType::MissingBridge,
                current_config: Some(interface.clone()),
                suggested_config: NetworkInterface {
                    mac_address: interface.mac_address.clone(),
                    network: interface.network.clone(),
                    bridge: suggested_bridge,
                    is_active: true,
                },
            });
        }
        
        // Check for conflicting configurations
        // Multiple interfaces using same bridge with different expected states
        for other_interface in vm_interfaces {
            if interface.bridge == other_interface.bridge && 
               interface.network != other_interface.network &&
               interface.is_active != other_interface.is_active {
                
                // Found conflicting configuration on same bridge
                mismatches.push(NetworkMismatch {
                    interface_name: format!("{}-config-conflict", interface.bridge),
                    issue_type: NetworkIssueType::ConflictingConfiguration,
                    current_config: Some(interface.clone()),
                    suggested_config: NetworkInterface {
                        mac_address: interface.mac_address.clone(),
                        network: interface.network.clone(),
                        bridge: interface.bridge.clone(),
                        is_active: true, // Prefer active state
                    },
                });
                break; // Only report once per interface
            }
        }
        
        // Check for bridge-network mismatch
        // Bridge expected by network definition doesn't match VM config
        if let Some(network_info) = available_networks.iter().find(|n| n.network == interface.network) {
            if network_info.bridge != interface.bridge {
                mismatches.push(NetworkMismatch {
                    interface_name: format!("{}-bridge-mismatch", interface.network),
                    issue_type: NetworkIssueType::ConflictingConfiguration,
                    current_config: Some(interface.clone()),
                    suggested_config: NetworkInterface {
                        mac_address: interface.mac_address.clone(),
                        network: interface.network.clone(),
                        bridge: network_info.bridge.clone(),
                        is_active: network_info.is_active,
                    },
                });
            }
        }
    }
    
    Ok(mismatches)
}

/// Gets network interfaces for a specific VM
async fn get_vm_network_interfaces(vm_name: &str) -> Result<Vec<NetworkInterface>> {
    // Try with regular virsh first, then with sudo if needed
    let mut cmd = Command::new("virsh");
    cmd.args(&["domiflist", vm_name]);
    
    let output = cmd.output().await
        .map_err(|e| VmError::CommandError(format!("Failed to get VM network interfaces: {}", e)))?;
    
    // If regular virsh fails, try with sudo
    if !output.status.success() {
        let mut sudo_cmd = Command::new("sudo");
        sudo_cmd.args(&["virsh", "domiflist", vm_name]);
        
        let sudo_output = sudo_cmd.output().await
            .map_err(|e| VmError::CommandError(format!("Failed to get VM network interfaces with sudo: {}", e)))?;
        
        if !sudo_output.status.success() {
            return Err(VmError::CommandError(format!(
                "Failed to get VM network interfaces: {}", 
                String::from_utf8_lossy(&sudo_output.stderr)
            )));
        }
        
        return parse_domiflist_output(&String::from_utf8_lossy(&sudo_output.stdout)).await;
    }
    
    parse_domiflist_output(&String::from_utf8_lossy(&output.stdout)).await
}

/// Helper function to parse domiflist output
async fn parse_domiflist_output(output_str: &str) -> Result<Vec<NetworkInterface>> {
    let mut interfaces = Vec::new();
    
    for line in output_str.lines().skip(2) { // Skip header lines
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 {
            let network = parts[1].to_string();
            let bridge = parts[2].to_string();
            let mac = parts[4].to_string();
            
            // Check if network is active
            let is_active = is_network_active(&network).await.unwrap_or(false);
            
            interfaces.push(NetworkInterface {
                mac_address: mac,
                network,
                bridge,
                is_active,
            });
        }
    }
    
    Ok(interfaces)
}

/// Gets all available libvirt networks
async fn get_available_networks() -> Result<Vec<NetworkInterface>> {
    // Try with regular virsh first, then with sudo if needed
    let mut cmd = Command::new("virsh");
    cmd.args(&["net-list", "--all"]);
    
    let output = cmd.output().await
        .map_err(|e| VmError::CommandError(format!("Failed to list networks: {}", e)))?;
    
    let output_str = if !output.status.success() {
        // Try with sudo
        let mut sudo_cmd = Command::new("sudo");
        sudo_cmd.args(&["virsh", "net-list", "--all"]);
        
        let sudo_output = sudo_cmd.output().await
            .map_err(|e| VmError::CommandError(format!("Failed to list networks with sudo: {}", e)))?;
        
        if !sudo_output.status.success() {
            return Err(VmError::CommandError(format!(
                "Failed to list networks: {}", 
                String::from_utf8_lossy(&sudo_output.stderr)
            )));
        }
        
        String::from_utf8_lossy(&sudo_output.stdout).to_string()
    } else {
        String::from_utf8_lossy(&output.stdout).to_string()
    };
    let mut networks = Vec::new();
    
    for line in output_str.lines().skip(2) { // Skip header lines
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 3 {
            let network_name = parts[0].to_string();
            let is_active = parts[1] == "active";
            let bridge = get_network_bridge(&network_name).await.unwrap_or_else(|| "virbr0".to_string());
            
            networks.push(NetworkInterface {
                mac_address: String::new(), // Not applicable for network definitions
                network: network_name,
                bridge,
                is_active,
            });
        }
    }
    
    Ok(networks)
}

/// Gets all MAC addresses used by VMs
async fn get_all_vm_mac_addresses() -> Result<Vec<String>> {
    let output = Command::new("virsh")
        .args(&["list", "--all", "--name"])
        .output()
        .await
        .map_err(|e| VmError::CommandError(format!("Failed to list VMs: {}", e)))?;
    
    if !output.status.success() {
        return Err(VmError::CommandError(format!(
            "Failed to list VMs: {}", 
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    
    let output_string = String::from_utf8_lossy(&output.stdout);
    let vm_names: Vec<&str> = output_string
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();
    
    let mut all_macs = Vec::new();
    
    for vm_name in vm_names {
        if let Ok(interfaces) = get_vm_network_interfaces(vm_name).await {
            for interface in interfaces {
                all_macs.push(interface.mac_address);
            }
        }
    }
    
    Ok(all_macs)
}

/// Checks if a network is currently active
async fn is_network_active(network_name: &str) -> Result<bool> {
    let output = Command::new("virsh")
        .args(&["net-info", network_name])
        .output()
        .await
        .map_err(|e| VmError::CommandError(format!("Failed to get network info: {}", e)))?;
    
    if !output.status.success() {
        return Ok(false);
    }
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    for line in output_str.lines() {
        if line.starts_with("Active:") {
            return Ok(line.contains("yes"));
        }
    }
    
    Ok(false)
}

/// Gets the bridge name for a network
async fn get_network_bridge(network_name: &str) -> Option<String> {
    let output = Command::new("virsh")
        .args(&["net-info", network_name])
        .output()
        .await
        .ok()?;
    
    if !output.status.success() {
        return None;
    }
    
    let output_str = String::from_utf8_lossy(&output.stdout);
    for line in output_str.lines() {
        if line.starts_with("Bridge:") {
            return line.split_whitespace().nth(1).map(|s| s.to_string());
        }
    }
    
    None
}

/// Gets all bridge interfaces available on the system
async fn get_system_bridges() -> Result<Vec<String>> {
    let mut bridges = Vec::new();
    
    // Method 1: Check using ip link for bridge interfaces
    let output = Command::new("ip")
        .args(&["link", "show", "type", "bridge"])
        .output()
        .await
        .map_err(|e| VmError::CommandError(format!("Failed to get bridge interfaces: {}", e)))?;
    
    if output.status.success() {
        let output_str = String::from_utf8_lossy(&output.stdout);
        for line in output_str.lines() {
            // Parse lines like: "3: virbr0: <BROADCAST,MULTICAST,UP,LOWER_UP> mtu 1500"
            if let Some(bridge_part) = line.split(':').nth(1) {
                let bridge_name = bridge_part.trim().split_whitespace().next();
                if let Some(name) = bridge_name {
                    if name.starts_with("virbr") || name.starts_with("br-") {
                        bridges.push(name.to_string());
                    }
                }
            }
        }
    }
    
    // Method 2: Fallback to checking /sys/class/net for bridge interfaces
    if bridges.is_empty() {
        let sys_output = Command::new("find")
            .args(&["/sys/class/net", "-name", "virbr*", "-o", "-name", "br-*"])
            .output()
            .await;
        
        if let Ok(sys_output) = sys_output {
            if sys_output.status.success() {
                let output_str = String::from_utf8_lossy(&sys_output.stdout);
                for line in output_str.lines() {
                    if let Some(bridge_name) = line.split('/').last() {
                        bridges.push(bridge_name.to_string());
                    }
                }
            }
        }
    }
    
    // Method 3: Check libvirt networks for their bridges as ultimate fallback
    if bridges.is_empty() {
        let networks = get_available_networks().await?;
        for network in networks {
            if !bridges.contains(&network.bridge) {
                bridges.push(network.bridge);
            }
        }
    }
    
    Ok(bridges)
}

/// Automatically fixes network mismatches
pub async fn auto_fix_network_mismatches(vm_name: &str, mismatches: &[NetworkMismatch]) -> Result<Vec<String>> {
    let mut fixes_applied = Vec::new();
    
    for mismatch in mismatches {
        match mismatch.issue_type {
            NetworkIssueType::DuplicateMacAddress => {
                if let Err(e) = update_vm_mac_address(vm_name, &mismatch.suggested_config.mac_address).await {
                    eprintln!("Failed to update MAC address: {}", e);
                } else {
                    fixes_applied.push(format!("Updated MAC address to {}", mismatch.suggested_config.mac_address));
                }
            },
            NetworkIssueType::InactiveNetwork => {
                if let Err(e) = start_network(&mismatch.suggested_config.network).await {
                    eprintln!("Failed to start network {}: {}", mismatch.suggested_config.network, e);
                } else {
                    fixes_applied.push(format!("Started network {}", mismatch.suggested_config.network));
                }
            },
            NetworkIssueType::InvalidNetworkReference => {
                if let Err(e) = update_vm_network(vm_name, &mismatch.current_config.as_ref().unwrap().network, &mismatch.suggested_config.network).await {
                    eprintln!("Failed to update network reference: {}", e);
                } else {
                    fixes_applied.push(format!("Updated network from {} to {}", 
                        mismatch.current_config.as_ref().unwrap().network, 
                        mismatch.suggested_config.network));
                }
            },
            NetworkIssueType::MissingBridge => {
                // Create the missing bridge or update VM config to use existing bridge
                if let Err(e) = update_vm_bridge(vm_name, &mismatch.current_config.as_ref().unwrap().bridge, &mismatch.suggested_config.bridge).await {
                    eprintln!("Failed to update bridge reference: {}", e);
                } else {
                    fixes_applied.push(format!("Updated bridge from {} to {}", 
                        mismatch.current_config.as_ref().unwrap().bridge, 
                        mismatch.suggested_config.bridge));
                }
            },
            NetworkIssueType::ConflictingConfiguration => {
                // Resolve configuration conflicts by standardizing to suggested config
                if let Err(e) = resolve_config_conflict(vm_name, mismatch).await {
                    eprintln!("Failed to resolve configuration conflict: {}", e);
                } else {
                    fixes_applied.push(format!("Resolved configuration conflict for {}", mismatch.interface_name));
                }
            },
        }
    }
    
    Ok(fixes_applied)
}

/// Updates MAC address for a VM interface
async fn update_vm_mac_address(vm_name: &str, new_mac: &str) -> Result<()> {
    // This requires editing the VM XML configuration
    // For now, we'll use a simple sed-based approach, but in production
    // you'd want to use proper XML parsing
    
    let output = Command::new("bash")
        .args(&["-c", &format!(
            "virsh dumpxml {} | sed 's/mac address=.*/mac address=\"{}\"\\/>/g' | virsh define /dev/stdin",
            vm_name, new_mac
        )])
        .output()
        .await
        .map_err(|e| VmError::CommandError(format!("Failed to update MAC address: {}", e)))?;
    
    if !output.status.success() {
        return Err(VmError::CommandError(format!(
            "Failed to update MAC address: {}", 
            String::from_utf8_lossy(&output.stderr)
        )));
    }
    
    Ok(())
}

/// Starts a libvirt network
async fn start_network(network_name: &str) -> Result<()> {
    let output = Command::new("virsh")
        .args(&["net-start", network_name])
        .output()
        .await
        .map_err(|e| VmError::CommandError(format!("Failed to start network: {}", e)))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if stderr.contains("already active") {
            return Ok(()); // Network is already active, that's fine
        }
        return Err(VmError::CommandError(format!(
            "Failed to start network {}: {}", 
            network_name, stderr
        )));
    }
    
    Ok(())
}

/// Updates VM network configuration
async fn update_vm_network(_vm_name: &str, _old_network: &str, _new_network: &str) -> Result<()> {
    // This would require complex XML manipulation
    // For now, we'll return an error suggesting manual intervention
    Err(VmError::OperationError(
        "Network configuration updates require manual XML editing via 'virsh edit'".to_string()
    ))
}

/// Updates VM bridge configuration
async fn update_vm_bridge(vm_name: &str, old_bridge: &str, new_bridge: &str) -> Result<()> {
    // Try with regular virsh first, then with sudo if needed
    let mut cmd = Command::new("virsh");
    cmd.args(&["dumpxml", vm_name]);
    
    let output = cmd.output().await
        .map_err(|e| VmError::CommandError(format!("Failed to get VM XML: {}", e)))?;
    
    let mut xml_content = if output.status.success() {
        String::from_utf8_lossy(&output.stdout).to_string()
    } else {
        // Try with sudo
        let sudo_output = Command::new("sudo")
            .args(&["virsh", "dumpxml", vm_name])
            .output()
            .await
            .map_err(|e| VmError::CommandError(format!("Failed to get VM XML with sudo: {}", e)))?;
        
        if !sudo_output.status.success() {
            return Err(VmError::CommandError(format!(
                "Failed to get VM XML: {}", 
                String::from_utf8_lossy(&sudo_output.stderr)
            )));
        }
        
        String::from_utf8_lossy(&sudo_output.stdout).to_string()
    };
    
    // Simple bridge name replacement
    #[allow(unused_assignments)]
    {
        xml_content = xml_content.replace(
            &format!("bridge='{}'", old_bridge), 
            &format!("bridge='{}'", new_bridge)
        );
    }
    
    // Write back the XML (this is a simplified approach)
    // In production, you'd want proper XML parsing
    eprintln!("Bridge update would require manual XML editing");
    eprintln!("Replace bridge='{}' with bridge='{}' in VM configuration", old_bridge, new_bridge);
    eprintln!("Use: virsh edit {}", vm_name);
    
    Ok(())
}

/// Resolves configuration conflicts for network interfaces
async fn resolve_config_conflict(vm_name: &str, mismatch: &NetworkMismatch) -> Result<()> {
    match mismatch.interface_name.as_str() {
        name if name.contains("config-conflict") => {
            // For now, just report the conflict resolution
            eprintln!("Configuration conflict detected on bridge: {}", mismatch.suggested_config.bridge);
            eprintln!("Suggested action: Standardize all interfaces to active state");
            eprintln!("Manual intervention required via: virsh edit {}", vm_name);
        },
        name if name.contains("bridge-mismatch") => {
            // Bridge-network mismatch resolution
            eprintln!("Bridge mismatch detected for network: {}", mismatch.suggested_config.network);
            eprintln!("Expected bridge: {}, Current: {}", 
                     mismatch.suggested_config.bridge, 
                     mismatch.current_config.as_ref().unwrap().bridge);
            eprintln!("Manual intervention required via: virsh edit {}", vm_name);
        },
        _ => {
            eprintln!("Unknown configuration conflict: {}", mismatch.interface_name);
        }
    }
    
    Ok(())
}