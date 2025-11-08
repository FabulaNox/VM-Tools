use std::str;
use tokio::process::Command as AsyncCommand;

use crate::{
    error::{VmError, Result},
    vm::{VmInfo, VmState, DiskInfo, NetworkInfo},
};

pub struct LibvirtClient {
    uri: String,
}

impl LibvirtClient {
    pub async fn new(uri: &str) -> Result<Self> {
        // Test connection
        let output = AsyncCommand::new("virsh")
            .args(&["-c", uri, "version"])
            .output()
            .await
            .map_err(|e| VmError::LibvirtError(format!("Failed to execute virsh: {}", e)))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(VmError::LibvirtError(format!("Failed to connect to libvirt: {}", error)));
        }

        Ok(Self {
            uri: uri.to_string(),
        })
    }

    pub async fn list_domains(&self, all: bool) -> Result<Vec<VmInfo>> {
        let args = if all {
            vec!["-c", &self.uri, "list", "--all"]
        } else {
            vec!["-c", &self.uri, "list"]
        };

        let output = AsyncCommand::new("virsh")
            .args(&args)
            .output()
            .await
            .map_err(|e| VmError::LibvirtError(format!("Failed to list domains: {}", e)))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(VmError::LibvirtError(format!("Failed to list domains: {}", error)));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut vms = Vec::new();

        for line in stdout.lines().skip(2) {
            let line = line.trim();
            if line.is_empty() || line.starts_with("---") {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let name = parts[1].to_string();
                let state_str = parts[2];
                
                let state = match state_str {
                    "running" => VmState::Running,
                    "shut" => VmState::Stopped,
                    "paused" => VmState::Paused,
                    "in" => VmState::Stopped, // "in shutdown"
                    _ => VmState::Unknown,
                };

                // Get detailed info for each VM
                if let Ok(vm_info) = self.get_domain_info(&name).await {
                    vms.push(vm_info);
                } else {
                    // Fallback with basic info
                    vms.push(VmInfo {
                        name,
                        uuid: "unknown".to_string(),
                        state,
                        memory: 0,
                        cpus: 0,
                        uptime: None,
                        cpu_usage: None,
                        memory_usage: None,
                        disk_usage: Vec::new(),
                        network_info: Vec::new(),
                        created_at: 0,
                        last_started: None,
                    });
                }
            }
        }

        Ok(vms)
    }

    pub async fn get_domain_info(&self, name: &str) -> Result<VmInfo> {
        // Get basic domain info
        let dominfo_output = AsyncCommand::new("virsh")
            .args(&["-c", &self.uri, "dominfo", name])
            .output()
            .await
            .map_err(|e| VmError::LibvirtError(format!("Failed to get domain info: {}", e)))?;

        if !dominfo_output.status.success() {
            let error = String::from_utf8_lossy(&dominfo_output.stderr);
            if error.contains("not found") {
                return Err(VmError::VmNotFound(name.to_string()));
            }
            return Err(VmError::LibvirtError(format!("Failed to get domain info: {}", error)));
        }

        let dominfo = String::from_utf8_lossy(&dominfo_output.stdout);
        let mut vm_info = VmInfo {
            name: name.to_string(),
            uuid: String::new(),
            state: VmState::Unknown,
            memory: 0,
            cpus: 0,
            uptime: None,
            cpu_usage: None,
            memory_usage: None,
            disk_usage: Vec::new(),
            network_info: Vec::new(),
            created_at: 0,
            last_started: None,
        };

        // Parse dominfo output
        for line in dominfo.lines() {
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() == 2 {
                let key = parts[0].trim();
                let value = parts[1].trim();

                match key {
                    "UUID" => vm_info.uuid = value.to_string(),
                    "State" => {
                        vm_info.state = match value {
                            "running" => VmState::Running,
                            "shut off" => VmState::Stopped,
                            "paused" => VmState::Paused,
                            "suspended" => VmState::Suspended,
                            _ => VmState::Unknown,
                        };
                    }
                    "Max memory" => {
                        if let Ok(memory_kb) = value.split_whitespace().next().unwrap_or("0").parse::<u64>() {
                            vm_info.memory = memory_kb / 1024; // Convert to MB
                        }
                    }
                    "CPU(s)" => {
                        if let Ok(cpus) = value.parse::<u32>() {
                            vm_info.cpus = cpus;
                        }
                    }
                    _ => {}
                }
            }
        }

        // Get additional info if VM is running
        if vm_info.state == VmState::Running {
            // Get CPU and memory stats
            if let Ok(stats) = self.get_domain_stats(name).await {
                vm_info.cpu_usage = stats.0;
                vm_info.memory_usage = stats.1;
            }

            // Get uptime
            vm_info.uptime = self.get_domain_uptime(name).await.ok();
        }

        // Get disk info
        vm_info.disk_usage = self.get_domain_disks(name).await.unwrap_or_default();

        // Get network info
        vm_info.network_info = self.get_domain_interfaces(name).await.unwrap_or_default();

        Ok(vm_info)
    }

    pub async fn get_domain_state(&self, name: &str) -> Result<VmState> {
        let output = AsyncCommand::new("virsh")
            .args(&["-c", &self.uri, "domstate", name])
            .output()
            .await
            .map_err(|e| VmError::LibvirtError(format!("Failed to get domain state: {}", e)))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            if error.contains("not found") {
                return Err(VmError::VmNotFound(name.to_string()));
            }
            return Err(VmError::LibvirtError(format!("Failed to get domain state: {}", error)));
        }

        let state_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let state = match state_str.as_str() {
            "running" => VmState::Running,
            "shut off" => VmState::Stopped,
            "paused" => VmState::Paused,
            "suspended" => VmState::Suspended,
            _ => VmState::Unknown,
        };

        Ok(state)
    }

    pub async fn start_domain(&self, name: &str) -> Result<()> {
        let output = AsyncCommand::new("virsh")
            .args(&["-c", &self.uri, "start", name])
            .output()
            .await
            .map_err(|e| VmError::LibvirtError(format!("Failed to start domain: {}", e)))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            if error.contains("not found") {
                return Err(VmError::VmNotFound(name.to_string()));
            } else if error.contains("already active") {
                return Err(VmError::VmAlreadyRunning(name.to_string()));
            }
            return Err(VmError::LibvirtError(format!("Failed to start domain: {}", error)));
        }

        Ok(())
    }

    pub async fn shutdown_domain(&self, name: &str) -> Result<()> {
        let output = AsyncCommand::new("virsh")
            .args(&["-c", &self.uri, "shutdown", name])
            .output()
            .await
            .map_err(|e| VmError::LibvirtError(format!("Failed to shutdown domain: {}", e)))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            if error.contains("not found") {
                return Err(VmError::VmNotFound(name.to_string()));
            } else if error.contains("not running") {
                return Err(VmError::VmNotRunning(name.to_string()));
            }
            return Err(VmError::LibvirtError(format!("Failed to shutdown domain: {}", error)));
        }

        Ok(())
    }

    pub async fn destroy_domain(&self, name: &str) -> Result<()> {
        let output = AsyncCommand::new("virsh")
            .args(&["-c", &self.uri, "destroy", name])
            .output()
            .await
            .map_err(|e| VmError::LibvirtError(format!("Failed to destroy domain: {}", e)))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            if error.contains("not found") {
                return Err(VmError::VmNotFound(name.to_string()));
            }
            return Err(VmError::LibvirtError(format!("Failed to destroy domain: {}", error)));
        }

        Ok(())
    }

    pub async fn define_domain(&self, xml: &str) -> Result<()> {
        // Write XML to temporary file
        let temp_file = format!("/tmp/vmtools_domain_{}.xml", uuid::Uuid::new_v4());
        tokio::fs::write(&temp_file, xml).await
            .map_err(|e| VmError::IoError(e))?;

        let output = AsyncCommand::new("virsh")
            .args(&["-c", &self.uri, "define", &temp_file])
            .output()
            .await
            .map_err(|e| VmError::LibvirtError(format!("Failed to define domain: {}", e)))?;

        // Clean up temp file
        let _ = tokio::fs::remove_file(&temp_file).await;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(VmError::LibvirtError(format!("Failed to define domain: {}", error)));
        }

        Ok(())
    }

    pub async fn undefine_domain(&self, name: &str) -> Result<()> {
        let output = AsyncCommand::new("virsh")
            .args(&["-c", &self.uri, "undefine", name])
            .output()
            .await
            .map_err(|e| VmError::LibvirtError(format!("Failed to undefine domain: {}", e)))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            if error.contains("not found") {
                return Err(VmError::VmNotFound(name.to_string()));
            }
            return Err(VmError::LibvirtError(format!("Failed to undefine domain: {}", error)));
        }

        Ok(())
    }

    pub async fn domain_exists(&self, name: &str) -> Result<bool> {
        let output = AsyncCommand::new("virsh")
            .args(&["-c", &self.uri, "dominfo", name])
            .output()
            .await
            .map_err(|e| VmError::LibvirtError(format!("Failed to check domain existence: {}", e)))?;

        Ok(output.status.success())
    }

    pub async fn connect_console(&self, name: &str) -> Result<()> {
        let status = AsyncCommand::new("virsh")
            .args(&["-c", &self.uri, "console", name])
            .status()
            .await
            .map_err(|e| VmError::LibvirtError(format!("Failed to connect to console: {}", e)))?;

        if !status.success() {
            return Err(VmError::LibvirtError("Failed to connect to console".to_string()));
        }

        Ok(())
    }

    pub async fn list_networks(&self) -> Result<Vec<(String, bool, String, bool)>> {
        let output = AsyncCommand::new("virsh")
            .args(&["-c", &self.uri, "net-list", "--all"])
            .output()
            .await
            .map_err(|e| VmError::LibvirtError(format!("Failed to list networks: {}", e)))?;

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(VmError::LibvirtError(format!("Failed to list networks: {}", error)));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut networks = Vec::new();

        for line in stdout.lines().skip(2) {
            let line = line.trim();
            if line.is_empty() || line.starts_with("---") {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let name = parts[0].to_string();
                let active = parts[1] == "active";
                let autostart = parts[2] == "yes";
                let bridge = if parts.len() > 3 { parts[3].to_string() } else { "-".to_string() };

                networks.push((name, active, bridge, autostart));
            }
        }

        Ok(networks)
    }

    async fn get_domain_stats(&self, _name: &str) -> Result<(Option<f64>, Option<f64>)> {
        // This is a simplified implementation - in a real scenario you'd parse domstats output
        Ok((None, None))
    }

    async fn get_domain_uptime(&self, _name: &str) -> Result<u64> {
        // This would require parsing more detailed libvirt output
        Ok(0)
    }

    async fn get_domain_disks(&self, name: &str) -> Result<Vec<DiskInfo>> {
        let output = AsyncCommand::new("virsh")
            .args(&["-c", &self.uri, "domblklist", name, "--details"])
            .output()
            .await
            .map_err(|e| VmError::LibvirtError(format!("Failed to get domain disks: {}", e)))?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut disks = Vec::new();

        for line in stdout.lines().skip(2) {
            let line = line.trim();
            if line.is_empty() || line.starts_with("---") {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let device = parts[2].to_string();
                let path = parts[3].to_string();

                // Get disk size (simplified)
                disks.push(DiskInfo {
                    device,
                    path: path.clone(),
                    size: 0, // Would need to query actual size
                    used: 0, // Would need to query actual usage
                    format: "qcow2".to_string(), // Default assumption
                });
            }
        }

        Ok(disks)
    }

    async fn get_domain_interfaces(&self, name: &str) -> Result<Vec<NetworkInfo>> {
        let output = AsyncCommand::new("virsh")
            .args(&["-c", &self.uri, "domiflist", name])
            .output()
            .await
            .map_err(|e| VmError::LibvirtError(format!("Failed to get domain interfaces: {}", e)))?;

        if !output.status.success() {
            return Ok(Vec::new());
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut interfaces = Vec::new();

        for line in stdout.lines().skip(2) {
            let line = line.trim();
            if line.is_empty() || line.starts_with("---") {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                let interface = parts[0].to_string();
                let network = parts[2].to_string();
                let mac = parts[4].to_string();

                interfaces.push(NetworkInfo {
                    interface,
                    network,
                    mac_address: mac,
                    ip_address: None, // Would need additional query
                    bridge: "virbr0".to_string(), // Default assumption
                });
            }
        }

        Ok(interfaces)
    }
}