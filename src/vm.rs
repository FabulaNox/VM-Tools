use serde::{Deserialize, Serialize};
use colored::*;
use tokio::time::{sleep, Duration};
use indicatif::{ProgressBar, ProgressStyle};

use crate::{
    config::{Config, VmTemplate},
    error::{VmError, Result},
    libvirt::LibvirtClient,
    utils,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum VmState {
    Running,
    Stopped,
    Paused,
    Suspended,
    Unknown,
}

impl std::fmt::Display for VmState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state_str = match self {
            VmState::Running => "RUNNING".green(),
            VmState::Stopped => "STOPPED".red(),
            VmState::Paused => "PAUSED".yellow(),
            VmState::Suspended => "SUSPENDED".blue(),
            VmState::Unknown => "UNKNOWN".bright_black(),
        };
        write!(f, "{}", state_str)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmInfo {
    pub name: String,
    pub uuid: String,
    pub state: VmState,
    pub memory: u64,
    pub cpus: u32,
    pub uptime: Option<u64>,
    pub cpu_usage: Option<f64>,
    pub memory_usage: Option<f64>,
    pub disk_usage: Vec<DiskInfo>,
    pub network_info: Vec<NetworkInfo>,
    pub created_at: u64,
    pub last_started: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskInfo {
    pub device: String,
    pub path: String,
    pub size: u64,
    pub used: u64,
    pub format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub interface: String,
    pub network: String,
    pub mac_address: String,
    pub ip_address: Option<String>,
    pub bridge: String,
}

pub struct VmManager {
    config: Config,
    libvirt: LibvirtClient,
}

impl VmManager {
    pub async fn new(config: &Config) -> Result<Self> {
        let libvirt = LibvirtClient::new(
            &config.libvirt.uri, 
            config.system.temp_dir.to_str().unwrap_or("/tmp")
        ).await?;
        
        Ok(Self {
            config: config.clone(),
            libvirt,
        })
    }
    
    pub async fn list_vms(&self, all: bool, running_only: bool) -> Result<()> {
        let vms = self.libvirt.list_domains(all).await?;
        
        if vms.is_empty() {
            println!("{}", "No virtual machines found".yellow());
            return Ok(());
        }
        
        println!("{:<20} {:<12} {:<8} {:<6} {:<8} {:<12}", 
                 "NAME".bold(), "STATE".bold(), "MEMORY".bold(), 
                 "CPUS".bold(), "UPTIME".bold(), "IP ADDRESS".bold());
        println!("{}", "─".repeat(80));
        
        for vm in vms {
            if running_only && vm.state != VmState::Running {
                continue;
            }
            
            let uptime_str = match vm.uptime {
                Some(uptime) => utils::format_duration(uptime),
                None => "-".to_string(),
            };
            
            let ip_str = vm.network_info.first()
                .and_then(|net| net.ip_address.as_ref())
                .map(|ip| ip.as_str())
                .unwrap_or("-");
            
            println!("{:<20} {:<12} {:<8} {:<6} {:<8} {:<12}",
                     vm.name,
                     vm.state,
                     format!("{}MB", vm.memory),
                     vm.cpus,
                     uptime_str,
                     ip_str);
        }
        
        Ok(())
    }
    
    pub async fn start_vm(&self, name: &str) -> Result<()> {
        println!("Starting VM '{}'...", name.green());
        
        // Validate VM name to prevent path traversal attacks (CWE-22)
        utils::validate_vm_name(name)?;
        
        let pb = ProgressBar::new_spinner();
        pb.set_style(ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap());
        pb.set_message("Starting virtual machine...");
        
        self.libvirt.start_domain(name).await?;
        
        // Wait for VM to fully start
        for _ in 0..30 {
            pb.tick();
            sleep(Duration::from_secs(1)).await;
            
            let state = self.libvirt.get_domain_state(name).await?;
            if state == VmState::Running {
                pb.finish_with_message(format!("✓ VM '{}' started successfully", name));
                return Ok(());
            }
        }
        
        pb.finish_with_message(format!("⚠ VM '{}' may still be starting", name));
        Ok(())
    }
    
    pub async fn stop_vm(&self, name: &str, force: bool) -> Result<()> {
        let action = if force { "Force stopping" } else { "Stopping" };
        println!("{} VM '{}'...", action, name.red());
        
        // Validate VM name to prevent path traversal attacks (CWE-22)
        utils::validate_vm_name(name)?;
        
        if force {
            self.libvirt.destroy_domain(name).await?;
        } else {
            self.libvirt.shutdown_domain(name).await?;
        }
        
        println!("✓ VM '{}' stopped successfully", name);
        Ok(())
    }
    
    pub async fn get_vm_status(&self, name: &str) -> Result<()> {
        // Validate VM name to prevent path traversal attacks (CWE-22)
        utils::validate_vm_name(name)?;
        
        let vm_info = self.libvirt.get_domain_info(name).await?;
        
        println!("{}", format!("VM Status: {}", name).bold());
        println!("{}", "═".repeat(40));
        println!("State: {}", vm_info.state);
        println!("UUID: {}", vm_info.uuid);
        println!("Memory: {}MB", vm_info.memory);
        println!("CPUs: {}", vm_info.cpus);
        
        if let Some(uptime) = vm_info.uptime {
            println!("Uptime: {}", utils::format_duration(uptime));
        }
        
        if let Some(cpu_usage) = vm_info.cpu_usage {
            println!("CPU Usage: {:.1}%", cpu_usage);
        }
        
        if let Some(memory_usage) = vm_info.memory_usage {
            println!("Memory Usage: {:.1}%", memory_usage);
        }
        
        if !vm_info.disk_usage.is_empty() {
            println!("\nDisk Information:");
            for disk in &vm_info.disk_usage {
                println!("  {} ({}): {}/{} ({})", 
                         disk.device, 
                         disk.format,
                         utils::format_bytes(disk.used),
                         utils::format_bytes(disk.size),
                         disk.path);
            }
        }
        
        if !vm_info.network_info.is_empty() {
            println!("\nNetwork Information:");
            for net in &vm_info.network_info {
                println!("  {}: {} ({})", 
                         net.interface,
                         net.ip_address.as_deref().unwrap_or("No IP"),
                         net.mac_address);
            }
        }
        
        Ok(())
    }
    
    pub async fn create_vm(
        &self,
        name: &str,
        memory: u64,
        cpus: u32,
        disk_size: u64,
        iso_path: Option<&str>,
        template_name: Option<&str>,
    ) -> Result<()> {
        println!("Creating VM '{}'...", name.green());
        
        // Validate VM name to prevent path traversal attacks (CWE-22)
        utils::validate_vm_name(name)?;
        
        // Check if VM already exists
        if self.libvirt.domain_exists(name).await? {
            return Err(VmError::VmAlreadyExists(name.to_string()));
        }

        // Check available networks and select the best one
        let available_networks = self.libvirt.list_networks().await?;
        let active_networks: Vec<String> = available_networks.iter()
            .filter(|(_, active, _, _)| *active)
            .map(|(name, _, _, _)| name.clone())
            .collect();
        
        let selected_network = if active_networks.contains(&self.config.network.default_network) {
            println!("{} Using default network: {}", 
                     "Network:".cyan(), self.config.network.default_network.green());
            self.config.network.default_network.clone()
        } else if let Some(first_network) = active_networks.first() {
            println!("{} Default network '{}' not available, using: {}", 
                     "Network:".yellow(), 
                     self.config.network.default_network,
                     first_network.green());
            first_network.clone()
        } else {
            return Err(VmError::NetworkError(
                "No active virtual networks found. Please start a network first:\n  virsh net-start default\n  or create a new network.".to_string()
            ));
        };
        
        if !active_networks.is_empty() {
            println!("{} Available networks: {}", 
                     "Info:".cyan(), 
                     active_networks.join(", "));
        }
        
        // Get template or use defaults
        let template = if let Some(template_name) = template_name {
            self.config.get_template(template_name)
                .ok_or_else(|| VmError::InvalidInput(format!("Template '{}' not found", template_name)))?
                .clone()
        } else {
            VmTemplate {
                memory,
                cpus,
                disk_size,
                os_type: "linux".to_string(),
                arch: "x86_64".to_string(),
                machine_type: "pc-q35-7.0".to_string(),
                boot_order: vec!["hd".to_string(), "cdrom".to_string()],
                features: vec!["acpi".to_string(), "apic".to_string()],
            }
        };
        
        let pb = ProgressBar::new(100);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos:>7}/{len:7} {msg}")
            .unwrap());
        
        pb.set_message("Creating disk image...");
        pb.set_position(10);
        
        // Create disk image
        let disk_path = self.config.storage.vm_images_path.join(format!("{}.qcow2", name));
        utils::create_qcow2_image(&disk_path, disk_size * 1024 * 1024 * 1024).await?;
        
        pb.set_message("Generating VM configuration...");
        pb.set_position(40);
        
        // Generate XML configuration
        let xml_config = self.generate_vm_xml(name, &template, &disk_path, iso_path, &selected_network)?;
        
        pb.set_message("Registering VM with libvirt...");
        pb.set_position(70);
        
        // Define the domain
        self.libvirt.define_domain(&xml_config).await?;
        
        pb.set_message("VM created successfully");
        pb.finish_with_message(format!("✓ VM '{}' created successfully", name));
        
        println!("VM Configuration:");
        println!("  Memory: {}MB", template.memory);
        println!("  CPUs: {}", template.cpus);
        println!("  Disk: {}GB", template.disk_size);
        println!("  Disk Path: {}", disk_path.display());
        
        if let Some(iso) = iso_path {
            println!("  ISO: {}", iso);
        }
        
        Ok(())
    }
    
    pub async fn delete_vm(&self, name: &str, force: bool) -> Result<()> {
        // Validate VM name to prevent path traversal attacks (CWE-22)
        utils::validate_vm_name(name)?;
        
        if !force {
            print!("Are you sure you want to delete VM '{}'? [y/N]: ", name);
            use std::io::{self, Write};
            io::stdout().flush().unwrap();
            
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            
            if !input.trim().to_lowercase().starts_with('y') {
                println!("Operation cancelled");
                return Ok(());
            }
        }
        
        println!("Deleting VM '{}'...", name.red());
        
        // Stop VM if running
        let state = self.libvirt.get_domain_state(name).await?;
        if state == VmState::Running {
            self.libvirt.destroy_domain(name).await?;
        }
        
        // Get VM info to find disk files
        let vm_info = self.libvirt.get_domain_info(name).await?;
        
        // Undefine the domain
        self.libvirt.undefine_domain(name).await?;
        
        // Delete disk files
        for disk in &vm_info.disk_usage {
            if let Err(e) = tokio::fs::remove_file(&disk.path).await {
                eprintln!("Warning: Failed to delete disk {}: {}", disk.path, e);
            }
        }
        
        println!("✓ VM '{}' deleted successfully", name);
        Ok(())
    }
    
    pub async fn clone_vm(&self, source: &str, target: &str) -> Result<()> {
        println!("Cloning VM '{}' to '{}'...", source.blue(), target.green());
        
        // Validate VM names to prevent path traversal attacks (CWE-22)
        utils::validate_vm_name(source)?;
        utils::validate_vm_name(target)?;
        
        if self.libvirt.domain_exists(target).await? {
            return Err(VmError::VmAlreadyExists(target.to_string()));
        }
        
        let pb = ProgressBar::new(100);
        pb.set_style(ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos:>7}/{len:7} {msg}")
            .unwrap());
        
        pb.set_message("Reading source VM configuration...");
        pb.set_position(20);
        
        let source_info = self.libvirt.get_domain_info(source).await?;
        
        pb.set_message("Cloning disk images...");
        pb.set_position(60);
        
        // Clone disk images
        for disk in &source_info.disk_usage {
            let target_path_str = self.config.storage.vm_images_path.join(format!("{}.qcow2", target));
            utils::clone_qcow2_image(disk.path.clone(), target_path_str.to_string_lossy().to_string()).await?;
        }
        
        pb.set_message("Creating new VM configuration...");
        pb.set_position(80);
        
        // Detect available networks
        let networks = self.libvirt.list_networks().await?;
        let active_networks: Vec<String> = networks.iter()
            .filter(|(_, active, _, _)| *active)
            .map(|(name, _, _, _)| name.clone())
            .collect();
            
        let selected_network = if active_networks.contains(&self.config.network.default_network) {
            println!("📡 Using configured network: {}", self.config.network.default_network.green());
            self.config.network.default_network.clone()
        } else if let Some(first_network) = active_networks.first() {
            println!("⚠️  Configured network '{}' not available, using: {}", 
                     self.config.network.default_network,
                     first_network.green());
            first_network.clone()
        } else {
            return Err(VmError::NetworkError(
                "No active networks available for VM creation".to_string()
            ));
        };
        
        // Create new XML with updated paths and UUID
        let target_disk_path = self.config.storage.vm_images_path.join(format!("{}.qcow2", target));
        let template = VmTemplate {
            memory: source_info.memory,
            cpus: source_info.cpus,
            disk_size: source_info.disk_usage.first().map(|d| d.size / (1024 * 1024 * 1024)).unwrap_or(20),
            os_type: "linux".to_string(),
            arch: "x86_64".to_string(),
            machine_type: "pc-q35-7.0".to_string(),
            boot_order: vec!["hd".to_string()],
            features: vec!["acpi".to_string(), "apic".to_string()],
        };
        
        let xml_config = self.generate_vm_xml(target, &template, &target_disk_path, None, &selected_network)?;
        self.libvirt.define_domain(&xml_config).await?;
        
        pb.finish_with_message(format!("✓ VM '{}' cloned successfully", target));
        Ok(())
    }
    
    pub async fn monitor_vm(&self, name: &str) -> Result<()> {
        // Validate VM name to prevent path traversal attacks (CWE-22)
        utils::validate_vm_name(name)?;
        
        println!("Monitoring VM '{}' (Press Ctrl+C to exit)...", name.cyan());
        
        loop {
            let vm_info = self.libvirt.get_domain_info(name).await?;
            
            print!("\x1B[2J\x1B[1;1H"); // Clear screen
            println!("{}", format!("VM Monitor: {} | {}", name, chrono::Local::now().format("%Y-%m-%d %H:%M:%S")).bold());
            println!("{}", "═".repeat(60));
            println!("State: {}", vm_info.state);
            
            if let Some(cpu_usage) = vm_info.cpu_usage {
                println!("CPU Usage: {:.1}%", cpu_usage);
            }
            
            if let Some(memory_usage) = vm_info.memory_usage {
                println!("Memory Usage: {:.1}% ({}/{}MB)", 
                         memory_usage,
                         (vm_info.memory as f64 * memory_usage / 100.0) as u64,
                         vm_info.memory);
            }
            
            if let Some(uptime) = vm_info.uptime {
                println!("Uptime: {}", utils::format_duration(uptime));
            }
            
            sleep(Duration::from_secs(2)).await;
        }
    }
    
    pub async fn connect_console(&self, name: &str) -> Result<()> {
        // Validate VM name to prevent path traversal attacks (CWE-22)
        utils::validate_vm_name(name)?;
        
        println!("Connecting to console of VM '{}'...", name.cyan());
        self.libvirt.connect_console(name).await
    }
    
    pub async fn list_networks(&self) -> Result<()> {
        let networks = self.libvirt.list_networks().await?;
        
        println!("{:<20} {:<12} {:<15} {:<10}", 
                 "NAME".bold(), "STATE".bold(), "BRIDGE".bold(), "AUTOSTART".bold());
        println!("{}", "─".repeat(60));
        
        for (name, active, bridge, autostart) in networks {
            let state = if active { "ACTIVE".green() } else { "INACTIVE".red() };
            let autostart_str = if autostart { "Yes".green() } else { "No".red() };
            
            println!("{:<20} {:<12} {:<15} {:<10}",
                     name, state, bridge, autostart_str);
        }
        
        Ok(())
    }
    
    pub async fn set_config(&self, key: &str, value: &str) -> Result<()> {
        let mut config = self.config.clone();
        config.set_value(key, value)?;
        config.save()?;
        println!("✓ Configuration updated: {} = {}", key, value);
        Ok(())
    }
    
    pub async fn get_config(&self, key: &str) -> Result<()> {
        let value = self.config.get_value(key)?;
        println!("{} = {}", key, value);
        Ok(())
    }
    
    fn generate_vm_xml(
        &self,
        name: &str,
        template: &VmTemplate,
        disk_path: &std::path::Path,
        iso_path: Option<&str>,
        network: &str,
    ) -> Result<String> {
        let uuid = uuid::Uuid::new_v4();
        
        let mut xml = format!(r#"<domain type='kvm'>
  <name>{}</name>
  <uuid>{}</uuid>
  <memory unit='MiB'>{}</memory>
  <currentMemory unit='MiB'>{}</currentMemory>
  <vcpu placement='static'>{}</vcpu>
  <os>
    <type arch='{}' machine='{}'>{}</type>
    <boot dev='hd'/>
    <boot dev='cdrom'/>
  </os>
  <features>
    <acpi/>
    <apic/>
  </features>
  <cpu mode='host-passthrough' check='none'/>
  <clock offset='utc'>
    <timer name='rtc' tickpolicy='catchup'/>
    <timer name='pit' tickpolicy='delay'/>
    <timer name='hpet' present='no'/>
  </clock>
  <on_poweroff>destroy</on_poweroff>
  <on_reboot>restart</on_reboot>
  <on_crash>destroy</on_crash>
  <devices>
    <emulator>/usr/bin/qemu-system-x86_64</emulator>
    <disk type='file' device='disk'>
      <driver name='qemu' type='qcow2'/>
      <source file='{}'/>
      <target dev='vda' bus='virtio'/>
      <address type='pci' domain='0x0000' bus='0x04' slot='0x00' function='0x0'/>
    </disk>"#,
            name,
            uuid,
            template.memory,
            template.memory,
            template.cpus,
            template.arch,
            template.machine_type,
            template.os_type,
            disk_path.display()
        );
        
        if let Some(iso) = iso_path {
            xml.push_str(&format!(r#"
    <disk type='file' device='cdrom'>
      <driver name='qemu' type='raw'/>
      <source file='{}'/>
      <target dev='sda' bus='sata'/>
      <readonly/>
      <address type='drive' controller='0' bus='0' target='0' unit='0'/>
    </disk>"#, iso));
        }
        
        xml.push_str(&format!(r#"
    <controller type='usb' index='0' model='qemu-xhci' ports='15'>
      <address type='pci' domain='0x0000' bus='0x02' slot='0x00' function='0x0'/>
    </controller>
    <controller type='sata' index='0'>
      <address type='pci' domain='0x0000' bus='0x00' slot='0x1f' function='0x2'/>
    </controller>
    <controller type='pci' index='0' model='pcie-root'/>
    <controller type='pci' index='1' model='pcie-root-port'>
      <model name='pcie-root-port'/>
      <target chassis='1' port='0x10'/>
      <address type='pci' domain='0x0000' bus='0x00' slot='0x02' function='0x0' multifunction='on'/>
    </controller>
    <interface type='network'>
      <mac address='{}'/>
      <source network='{}'/>
      <model type='virtio'/>
      <address type='pci' domain='0x0000' bus='0x01' slot='0x00' function='0x0'/>
    </interface>
    <serial type='pty'>
      <target type='isa-serial' port='0'>
        <model name='isa-serial'/>
      </target>
    </serial>
    <console type='pty'>
      <target type='serial' port='0'/>
    </console>
    <input type='tablet' bus='usb'>
      <address type='usb' bus='0' port='1'/>
    </input>
    <input type='mouse' bus='ps2'/>
    <input type='keyboard' bus='ps2'/>
    <graphics type='spice' autoport='yes'>
      <listen type='address'/>
      <image compression='off'/>
    </graphics>
    <sound model='ich9'>
      <address type='pci' domain='0x0000' bus='0x00' slot='0x1b' function='0x0'/>
    </sound>
    <video>
      <model type='qxl' ram='65536' vram='65536' vgamem='16384' heads='1' primary='yes'/>
      <address type='pci' domain='0x0000' bus='0x00' slot='0x01' function='0x0'/>
    </video>
    <memballoon model='virtio'>
      <address type='pci' domain='0x0000' bus='0x05' slot='0x00' function='0x0'/>
    </memballoon>
    <rng model='virtio'>
      <backend model='random'>/dev/urandom</backend>
      <address type='pci' domain='0x0000' bus='0x06' slot='0x00' function='0x0'/>
    </rng>
  </devices>
</domain>"#,
            utils::generate_mac_address(),
            network
        ));
        
        Ok(xml)
    }
    
    /// Detects and fixes network mismatches for a VM
    pub async fn fix_network_issues(&self, name: &str, auto_fix: bool) -> Result<()> {
        println!("🔍 Analyzing network configuration for VM '{}'...", name.cyan());
        
        // Validate VM name to prevent path traversal attacks (CWE-22)
        utils::validate_vm_name(name)?;
        
        // Detect network mismatches
        let mismatches = utils::detect_network_mismatches(name).await?;
        
        if mismatches.is_empty() {
            println!("✅ No network issues detected for VM '{}'", name.green());
            return Ok(());
        }
        
        println!("⚠️  Found {} network issue(s):", mismatches.len());
        for (i, mismatch) in mismatches.iter().enumerate() {
            println!("  {}. {} on interface '{}'", 
                     i + 1, 
                     mismatch.issue_type, 
                     mismatch.interface_name);
            
            if let Some(current) = &mismatch.current_config {
                println!("     Current: Network={}, MAC={}, Active={}", 
                         current.network, 
                         current.mac_address, 
                         current.is_active);
            }
            
            println!("     Suggested: Network={}, MAC={}, Active={}", 
                     mismatch.suggested_config.network, 
                     mismatch.suggested_config.mac_address, 
                     mismatch.suggested_config.is_active);
        }
        
        if auto_fix {
            println!("\n🔧 Attempting to auto-fix network issues...");
            let fixes = utils::auto_fix_network_mismatches(name, &mismatches).await?;
            
            if fixes.is_empty() {
                println!("❌ No automatic fixes could be applied");
            } else {
                println!("✅ Applied {} fix(es):", fixes.len());
                for fix in fixes {
                    println!("  • {}", fix);
                }
                
                // Suggest restarting the VM
                println!("\n💡 Recommendation: Restart the VM to apply network changes:");
                println!("   vmtools stop {} && vmtools start {}", name, name);
            }
        } else {
            println!("\n💡 To automatically fix these issues, run:");
            println!("   vmtools fix-network {} --auto", name);
            
            println!("\n📝 Manual fixes you can apply:");
            for mismatch in &mismatches {
                match mismatch.issue_type {
                    utils::NetworkIssueType::DuplicateMacAddress => {
                        println!("  • Generate new MAC: virsh edit {} (update <mac address='...'/>)", name);
                    },
                    utils::NetworkIssueType::InactiveNetwork => {
                        println!("  • Start network: virsh net-start {}", mismatch.suggested_config.network);
                    },
                    utils::NetworkIssueType::InvalidNetworkReference => {
                        println!("  • Update network: virsh edit {} (change <source network='...'/>)", name);
                    },
                    _ => {
                        println!("  • Check libvirt documentation for {}", mismatch.issue_type);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Optimizes VM configuration based on libvirt environment
    pub async fn optimize_vm_config(&self, name: &str) -> Result<()> {
        println!("🚀 Optimizing VM configuration for '{}'...", name.cyan());
        
        // Validate VM name to prevent path traversal attacks (CWE-22)
        utils::validate_vm_name(name)?;
        
        // Check if VM is running (can't optimize running VM)
        let state = self.libvirt.get_domain_state(name).await?;
        if state == VmState::Running {
            return Err(VmError::InvalidVmState(
                "Cannot optimize running VM. Please stop the VM first.".to_string()
            ));
        }
        
        // Get current VM configuration
        let vm_info = self.libvirt.get_domain_info(name).await?;
        
        // Check network configuration
        self.fix_network_issues(name, false).await?;
        
        // Check for excessive network interfaces
        if vm_info.network_info.len() > 2 {
            println!("⚠️  VM has {} network interfaces. Consider simplifying:", vm_info.network_info.len());
            for (i, net) in vm_info.network_info.iter().enumerate() {
                println!("  {}. {} on {} ({})", i + 1, net.interface, net.network, net.mac_address);
            }
            println!("💡 Recommendation: Use only necessary network interfaces for better performance");
        }
        
        // Check available networks and suggest optimization
        let networks = self.libvirt.list_networks().await?;
        let active_networks: Vec<String> = networks.iter()
            .filter(|(_, active, _, _)| *active)
            .map(|(name, _, _, _)| name.clone())
            .collect();
            
        if active_networks.len() > 1 {
            println!("📡 Available networks for optimization:");
            for network in &active_networks {
                println!("  • {}", network);
            }
            
            if !active_networks.contains(&self.config.network.default_network) {
                println!("⚠️  Configured default network '{}' is not active", self.config.network.default_network);
                if let Some(first_active) = active_networks.first() {
                    println!("💡 Consider updating config to use: {}", first_active);
                }
            }
        }
        
        println!("✅ VM configuration analysis complete");
        Ok(())
    }
}