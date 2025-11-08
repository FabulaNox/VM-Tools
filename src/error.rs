use thiserror::Error;

#[derive(Error, Debug)]
#[allow(dead_code)]
pub enum VmError {
    #[error("VM not found: {0}")]
    VmNotFound(String),
    
    #[error("VM already exists: {0}")]
    VmAlreadyExists(String),
    
    #[error("VM is already running: {0}")]
    VmAlreadyRunning(String),
    
    #[error("VM is not running: {0}")]
    VmNotRunning(String),
    
    #[error("Invalid VM state: {0}")]
    InvalidVmState(String),
    
    #[error("Libvirt error: {0}")]
    LibvirtError(String),
    
    #[error("QEMU error: {0}")]
    QemuError(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    SerdeError(#[from] serde_json::Error),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Resource not available: {0}")]
    ResourceUnavailable(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Timeout: {0}")]
    Timeout(String),
}

pub type Result<T> = std::result::Result<T, VmError>;