use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::fmt;

use crate::lib::error::{VmError, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub libvirt: LibvirtConfig,
    pub storage: StorageConfig,
    pub network: NetworkConfig,
    pub templates: HashMap<String, VmTemplate>,
    pub defaults: DefaultsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibvirtConfig {
    pub uri: String,
    pub socket_path: Option<String>,
    pub timeout: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub default_pool: String,
    pub vm_images_path: PathBuf,
    pub iso_path: PathBuf,
    pub backup_path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub default_network: String,
    pub bridge_interface: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmTemplate {
    pub memory: u64,
    pub cpus: u32,
    pub disk_size: u64,
    pub os_type: String,
    pub arch: String,
    pub machine_type: String,
    pub boot_order: Vec<String>,
    pub features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultsConfig {
    pub memory: u64,
    pub cpus: u32,
    pub disk_size: u64,
    pub disk_format: String,
    pub network: String,
    pub graphics: String,
}

impl Default for Config {
    fn default() -> Self {
        let mut templates = HashMap::new();
        
        // Ubuntu template
        templates.insert("ubuntu".to_string(), VmTemplate {
            memory: 2048,
            cpus: 2,
            disk_size: 20,
            os_type: "linux".to_string(),
            arch: "x86_64".to_string(),
            machine_type: "pc-q35-7.0".to_string(),
            boot_order: vec!["hd".to_string(), "cdrom".to_string()],
            features: vec!["acpi".to_string(), "apic".to_string(), "pae".to_string()],
        });
        
        // Windows template
        templates.insert("windows".to_string(), VmTemplate {
            memory: 4096,
            cpus: 2,
            disk_size: 40,
            os_type: "windows".to_string(),
            arch: "x86_64".to_string(),
            machine_type: "pc-q35-7.0".to_string(),
            boot_order: vec!["hd".to_string(), "cdrom".to_string()],
            features: vec!["acpi".to_string(), "apic".to_string(), "hyperv".to_string()],
        });
        
        Self {
            libvirt: LibvirtConfig {
                uri: "qemu:///system".to_string(),
                socket_path: Some("/var/run/libvirt/libvirt-sock".to_string()),
                timeout: 30,
            },
            storage: StorageConfig {
                default_pool: "default".to_string(),
                vm_images_path: PathBuf::from("/var/lib/libvirt/images"),
                iso_path: PathBuf::from("/var/lib/libvirt/images/iso"),
                backup_path: PathBuf::from("/var/lib/libvirt/backup"),
            },
            network: NetworkConfig {
                default_network: "default".to_string(),
                bridge_interface: "virbr0".to_string(),
            },
            templates,
            defaults: DefaultsConfig {
                memory: 2048,
                cpus: 2,
                disk_size: 20,
                disk_format: "qcow2".to_string(),
                network: "default".to_string(),
                graphics: "spice".to_string(),
            },
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;
        
        if config_path.exists() {
            let content = fs::read_to_string(&config_path)
                .map_err(|e| VmError::ConfigError(format!("Failed to read config file: {}", e)))?;
            
            let config: Config = toml::from_str(&content)
                .map_err(|e| VmError::ConfigError(format!("Failed to parse config: {}", e)))?;
            
            Ok(config)
        } else {
            let config = Config::default();
            config.save()?;
            Ok(config)
        }
    }
    
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path()?;
        
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| VmError::ConfigError(format!("Failed to create config directory: {}", e)))?;
        }
        
        let content = toml::to_string_pretty(self)
            .map_err(|e| VmError::ConfigError(format!("Failed to serialize config: {}", e)))?;
        
        fs::write(&config_path, content)
            .map_err(|e| VmError::ConfigError(format!("Failed to write config file: {}", e)))?;
        
        Ok(())
    }
    
    fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| VmError::ConfigError("Cannot determine config directory".to_string()))?;
        
        Ok(config_dir.join("vmtools").join("config.toml"))
    }
    
    pub fn get_template(&self, name: &str) -> Option<&VmTemplate> {
        self.templates.get(name)
    }
    
    pub fn set_value(&mut self, key: &str, value: &str) -> Result<()> {
        match key {
            "libvirt.uri" => self.libvirt.uri = value.to_string(),
            "libvirt.timeout" => {
                self.libvirt.timeout = value.parse()
                    .map_err(|_| VmError::InvalidInput(format!("Invalid timeout value: {}", value)))?;
            }
            "storage.default_pool" => self.storage.default_pool = value.to_string(),
            "network.default_network" => self.network.default_network = value.to_string(),
            "defaults.memory" => {
                self.defaults.memory = value.parse()
                    .map_err(|_| VmError::InvalidInput(format!("Invalid memory value: {}", value)))?;
            }
            "defaults.cpus" => {
                self.defaults.cpus = value.parse()
                    .map_err(|_| VmError::InvalidInput(format!("Invalid CPU count: {}", value)))?;
            }
            _ => return Err(VmError::InvalidInput(format!("Unknown config key: {}", key))),
        }
        Ok(())
    }
    
    pub fn get_value(&self, key: &str) -> Result<String> {
        match key {
            "libvirt.uri" => Ok(self.libvirt.uri.clone()),
            "libvirt.timeout" => Ok(self.libvirt.timeout.to_string()),
            "storage.default_pool" => Ok(self.storage.default_pool.clone()),
            "network.default_network" => Ok(self.network.default_network.clone()),
            "defaults.memory" => Ok(self.defaults.memory.to_string()),
            "defaults.cpus" => Ok(self.defaults.cpus.to_string()),
            _ => Err(VmError::InvalidInput(format!("Unknown config key: {}", key))),
        }
    }
}

impl fmt::Display for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "VM Tools Configuration:")?;
        writeln!(f, "=======================")?;
        writeln!(f, "Libvirt URI: {}", self.libvirt.uri)?;
        writeln!(f, "Timeout: {}s", self.libvirt.timeout)?;
        writeln!(f, "Default Pool: {}", self.storage.default_pool)?;
        writeln!(f, "VM Images: {}", self.storage.vm_images_path.display())?;
        writeln!(f, "ISO Path: {}", self.storage.iso_path.display())?;
        writeln!(f, "Default Network: {}", self.network.default_network)?;
        writeln!(f, "Default Memory: {}MB", self.defaults.memory)?;
        writeln!(f, "Default CPUs: {}", self.defaults.cpus)?;
        writeln!(f, "Default Disk: {}GB", self.defaults.disk_size)?;
        writeln!(f, "\nAvailable Templates:")?;
        for (name, template) in &self.templates {
            writeln!(f, "  - {}: {}MB, {} CPUs, {}GB disk", name, template.memory, template.cpus, template.disk_size)?;
        }
        Ok(())
    }
}