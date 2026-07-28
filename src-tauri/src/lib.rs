//! Xtunnel Games - Split tunneling for games and apps

mod error;
mod models;
mod apps;
mod tunnel;
mod providers;
mod config;
mod updater;

pub use error::XtunnelError;
pub use models::*;
pub use providers::*;
pub use updater::*;

pub struct AppState {
    pub provider_process: tokio::sync::Mutex<Option<tokio::process::Child>>,
    pub singbox_process: tokio::sync::Mutex<Option<tokio::process::Child>>,
    pub windivert_manager: tokio::sync::Mutex<Option<crate::tunnel::WinDivert>>,
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
            windivert_manager: tokio::sync::Mutex::new(None),
            connected_pids: parking_lot::Mutex::new(Vec::new()),
            provider_socks_port: parking_lot::Mutex::new(0),
            current_provider: parking_lot::Mutex::new("aether".into()),
            provider_config: parking_lot::Mutex::new(None),
        }
    }
}