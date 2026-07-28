use thiserror::Error;

#[derive(Error, Debug)]
pub enum XtunnelError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("WinDivert error: {0}")]
    WinDivert(String),
    
    #[error("Provider error: {0}")]
    Provider(String),
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("Process error: {0}")]
    Process(String),
    
    #[error("WinDivert driver not installed")]
    DriverNotInstalled,
    
    #[error("Admin privileges required")]
    AdminRequired,
    
    #[error("No process found for: {0}")]
    ProcessNotFound(String),
    
    #[error("SOCKS5 handshake failed")]
    SocksHandshakeFailed,
    
    #[error("Binary not found: {0}")]
    BinaryNotFound(String),
    
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),
    
    #[error("Tauri error: {0}")]
    Tauri(#[from] tauri::Error),
    
    #[error("Store error: {0}")]
    Store(#[from] tauri_plugin_store::Error),
    
    #[error("Other: {0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, XtunnelError>;