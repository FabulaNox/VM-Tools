use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "vmtools")]
#[command(about = "A high-performance VM management tool for QEMU/KVM")]
#[command(version = "0.1.0")]
#[command(author = "VM-Tools Contributors")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// List virtual machines
    List {
        /// Show all VMs (including inactive)
        #[arg(short, long)]
        all: bool,
        
        /// Show only running VMs
        #[arg(short, long)]
        running: bool,
    },
    
    /// Start a virtual machine
    Start {
        /// Name of the VM to start
        name: String,
    },
    
    /// Stop a virtual machine
    Stop {
        /// Name of the VM to stop
        name: String,
        
        /// Force stop (equivalent to pulling power)
        #[arg(short, long)]
        force: bool,
    },
    
    /// Get status of a virtual machine
    Status {
        /// Name of the VM
        name: String,
    },
    
    /// Create a new virtual machine
    Create {
        /// Name of the new VM
        name: String,
        
        /// Memory in MB
        #[arg(short, long, default_value = "2048")]
        memory: u64,
        
        /// Number of CPUs
        #[arg(short, long, default_value = "2")]
        cpus: u32,
        
        /// Disk size in GB
        #[arg(short, long, default_value = "20")]
        disk_size: u64,
        
        /// Path to ISO file for installation
        #[arg(short, long)]
        iso_path: Option<String>,
        
        /// VM template to use
        #[arg(short, long)]
        template: Option<String>,
    },
    
    /// Delete a virtual machine
    Delete {
        /// Name of the VM to delete
        name: String,
        
        /// Force delete without confirmation
        #[arg(short, long)]
        force: bool,
    },
    
    /// Clone a virtual machine
    Clone {
        /// Source VM name
        source: String,
        
        /// Target VM name
        target: String,
    },
    
    /// Monitor VM performance and resources
    Monitor {
        /// Name of the VM to monitor
        name: String,
    },
    
    /// Connect to VM console
    Console {
        /// Name of the VM
        name: String,
    },
    
    /// List available networks
    Networks,
    
    /// Configuration management
    Config {
        /// Show current configuration
        #[arg(long)]
        show: bool,
        
        /// Set a configuration value (key=value)
        #[arg(short = 's', long, value_parser = parse_key_val)]
        set: Option<(String, String)>,
        
        /// Get a configuration value
        #[arg(short, long)]
        get: Option<String>,
    },
    
    /// Fix network configuration issues for a VM
    FixNetwork {
        /// Name of the VM to fix
        name: String,
        
        /// Automatically apply fixes (default: analyze only)
        #[arg(long)]
        auto: bool,
    },
    
    /// Optimize VM configuration based on libvirt environment
    Optimize {
        /// Name of the VM to optimize
        name: String,
    },
    
    /// Fix clipboard and SPICE integration issues
    FixClipboard {
        /// Name of the VM to fix
        name: String,
    },
    
    /// Fix identity issues for cloned VMs (hostname, network identity)
    FixIdentity {
        /// Name of the VM to fix
        name: String,
        
        /// Set new hostname for the VM (optional, defaults to VM name)
        #[arg(long)]
        hostname: Option<String>,
    },
}

fn parse_key_val(s: &str) -> Result<(String, String), String> {
    let parts: Vec<&str> = s.splitn(2, '=').collect();
    if parts.len() != 2 {
        return Err("Invalid format. Use key=value".to_string());
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}