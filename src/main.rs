use clap::Parser;
use log::error;
use std::process;
use tokio;

mod cli;
mod config;
mod vm;
mod libvirt;
mod error;
mod qemu;
mod utils;

use cli::Cli;
use config::Config;
use vm::VmManager;
use error::VmError;

#[tokio::main]
async fn main() {
    env_logger::init();
    
    let cli = Cli::parse();
    
    let config = match Config::load() {
        Ok(config) => config,
        Err(e) => {
            error!("Failed to load configuration: {}", e);
            process::exit(1);
        }
    };
    
    let vm_manager = match VmManager::new(&config).await {
        Ok(manager) => manager,
        Err(e) => {
            error!("Failed to initialize VM manager: {}", e);
            process::exit(1);
        }
    };
    
    let result = match cli.command {
        cli::Commands::List { all, running } => {
            vm_manager.list_vms(all, running).await
        }
        cli::Commands::Start { name } => {
            vm_manager.start_vm(&name).await
        }
        cli::Commands::Stop { name, force } => {
            vm_manager.stop_vm(&name, force).await
        }
        cli::Commands::Status { name } => {
            vm_manager.get_vm_status(&name).await
        }
        cli::Commands::Create { 
            name, 
            memory, 
            cpus, 
            disk_size, 
            iso_path,
            template 
        } => {
            vm_manager.create_vm(&name, memory, cpus, disk_size, iso_path.as_deref(), template.as_deref()).await
        }
        cli::Commands::Delete { name, force } => {
            vm_manager.delete_vm(&name, force).await
        }
        cli::Commands::Clone { source, target } => {
            vm_manager.clone_vm(&source, &target).await
        }
        cli::Commands::Monitor { name } => {
            vm_manager.monitor_vm(&name).await
        }
        cli::Commands::Console { name } => {
            vm_manager.connect_console(&name).await
        }
        cli::Commands::Networks => {
            vm_manager.list_networks().await
        }
        cli::Commands::Config { show, set, get } => {
            if show {
                println!("{}", config);
                Ok(())
            } else if let Some((key, value)) = set {
                vm_manager.set_config(&key, &value).await
            } else if let Some(key) = get {
                vm_manager.get_config(&key).await
            } else {
                Err(VmError::InvalidInput("No config action specified".to_string()))
            }
        }
        cli::Commands::FixNetwork { name, auto } => {
            vm_manager.fix_network_issues(&name, auto).await
        }
        cli::Commands::Optimize { name } => {
            vm_manager.optimize_vm_config(&name).await
        }
        cli::Commands::FixClipboard { name } => {
            vm_manager.fix_clipboard_integration(&name).await
        }
    };
    
    if let Err(e) = result {
        error!("Command failed: {}", e);
        process::exit(1);
    }
}