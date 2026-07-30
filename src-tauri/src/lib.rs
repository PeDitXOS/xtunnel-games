//! Xtunnel Games - Split tunneling for games and apps

pub mod apps;
pub mod commands;
pub mod config;
pub mod error;
pub mod models;
pub mod providers;
pub mod tunnel;
pub mod updater;

pub use error::XtunnelError;
pub use models::*;
pub use providers::*;

/// Resolve a bundled binary path from the resources directory.
/// Falls back to bare name (for dev mode or PATH).
pub fn resolve_binary(app: &tauri::AppHandle, name: &str) -> std::path::PathBuf {
    use tauri::Manager;
    if let Ok(res_dir) = app.path().resource_dir() {
        let bundled = res_dir.join(name);
        if bundled.exists() {
            return bundled;
        }
    }
    std::path::PathBuf::from(name)
}

pub struct AppState {
    pub provider_process: tokio::sync::Mutex<Option<tokio::process::Child>>,
    pub singbox_process: tokio::sync::Mutex<Option<tokio::process::Child>>,
    pub connected_pids: parking_lot::Mutex<Vec<u32>>,
    pub provider_socks_port: parking_lot::Mutex<u16>,
    pub current_provider: parking_lot::Mutex<String>,
    pub provider_config: parking_lot::Mutex<Option<serde_json::Value>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            provider_process: tokio::sync::Mutex::new(None),
            singbox_process: tokio::sync::Mutex::new(None),
            connected_pids: parking_lot::Mutex::new(Vec::new()),
            provider_socks_port: parking_lot::Mutex::new(0),
            current_provider: parking_lot::Mutex::new("aether".into()),
            provider_config: parking_lot::Mutex::new(None),
        }
    }
}
