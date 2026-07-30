use crate::error::Result;
use crate::models::*;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use tauri::{AppHandle, State};

mod aether;
mod openvpn;
mod socks;
mod v2ray;
mod wireguard;

pub use aether::AetherProvider;
pub use openvpn::OpenVpnProvider;
pub use socks::SocksProvider;
pub use v2ray::V2RayProvider;
pub use wireguard::WireGuardProvider;

#[async_trait::async_trait]
pub trait Provider: Send + Sync {
    fn id(&self) -> &str;
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn requires_server(&self) -> bool;

    async fn connect(
        &self,
        pids: Vec<u32>,
        config: ProviderConfig,
        state: State<'_, crate::AppState>,
        app: AppHandle,
    ) -> Result<()>;

    async fn disconnect(&self, state: State<'_, crate::AppState>) -> Result<()>;

    fn status(&self) -> ConnectionStatus;
}

pub type ProviderRegistry = Arc<Mutex<HashMap<String, Box<dyn Provider>>>>;

pub fn create_provider_registry() -> ProviderRegistry {
    let mut registry: HashMap<String, Box<dyn Provider>> = HashMap::new();
    registry.insert("aether".into(), Box::new(AetherProvider::new()));
    registry.insert("v2ray".into(), Box::new(V2RayProvider::new()));
    registry.insert("wireguard".into(), Box::new(WireGuardProvider::new()));
    registry.insert("openvpn".into(), Box::new(OpenVpnProvider::new()));
    registry.insert("socks".into(), Box::new(SocksProvider::new()));
    Arc::new(Mutex::new(registry))
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProviderInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub requires_server: bool,
}

impl ProviderInfo {
    pub fn for_provider(id: &str) -> Self {
        match id {
            "aether" => Self {
                id: "aether".into(),
                name: "Aether".into(),
                description: "Serverless anti-censorship tunnel (MASQUE/WireGuard/gool)".into(),
                requires_server: false,
            },
            "v2ray" => Self {
                id: "v2ray".into(),
                name: "V2Ray / Xray".into(),
                description: "VLESS/VMess/Trojan/Shadowsocks via Xray or sing-box".into(),
                requires_server: true,
            },
            "wireguard" => Self {
                id: "wireguard".into(),
                name: "WireGuard".into(),
                description: "WireGuard for Windows adapter".into(),
                requires_server: true,
            },
            "openvpn" => Self {
                id: "openvpn".into(),
                name: "OpenVPN".into(),
                description: "OpenVPN Community".into(),
                requires_server: true,
            },
            "socks" => Self {
                id: "socks".into(),
                name: "SOCKS5 / HTTP Proxy".into(),
                description: "External SOCKS5 or HTTP proxy".into(),
                requires_server: true,
            },
            _ => Self {
                id: "unknown".into(),
                name: "Unknown".into(),
                description: "".into(),
                requires_server: true,
            },
        }
    }
}

pub async fn get_available_providers() -> Result<Vec<ProviderInfo>> {
    Ok(["aether", "v2ray", "wireguard", "openvpn", "socks"]
        .iter()
        .map(|id| ProviderInfo::for_provider(id))
        .collect())
}

pub async fn disconnect(state: State<'_, crate::AppState>) -> Result<()> {
    // Stop sing-box
    if let Some(mut proc) = state.singbox_process.lock().await.take() {
        let _ = proc.kill().await;
    }
    // Stop provider process
    if let Some(mut proc) = state.provider_process.lock().await.take() {
        let _ = proc.kill().await;
    }
    *state.connected_pids.lock() = Vec::new();
    Ok(())
}

pub async fn get_status(state: State<'_, crate::AppState>) -> Result<ConnectionStatus> {
    let pids = state.connected_pids.lock().clone();
    if pids.is_empty() {
        return Ok(ConnectionStatus {
            state: "idle".into(),
            message: "Disconnected".into(),
            socks_port: None,
            connected_at: None,
            protocol: None,
            scan_mode: None,
        });
    }
    Ok(ConnectionStatus {
        state: "connected".into(),
        message: format!("Connected - {} apps tunneled", pids.len()),
        socks_port: Some(*state.provider_socks_port.lock()),
        connected_at: Some(chrono::Utc::now().timestamp_millis() as u64),
        protocol: Some(state.current_provider.lock().clone()),
        scan_mode: None,
    })
}
