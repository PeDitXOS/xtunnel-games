use crate::error::Result;
use crate::models::AetherConfig;
use crate::providers::Provider;
use parking_lot::Mutex;
use std::collections::HashSet;
use std::process::Stdio;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::process::Command;
use tokio::sync::Mutex as TokioMutex;
use tokio::time::{sleep, Duration, timeout};

mod aether;
mod v2ray;
mod wireguard;
mod openvpn;
mod socks;

pub use aether::AetherProvider;
pub use v2ray::V2RayProvider;
pub use wireguard::WireGuardProvider;
pub use openvpn::OpenVpnProvider;
pub use socks::SocksProvider;

pub type ProviderRegistry = Arc<Mutex<HashMap<String, Box<dyn Provider + Send + Sync>>>>;

pub fn create_provider_registry() -> ProviderRegistry {
    let mut registry: HashMap<String, Box<dyn Provider + Send + Sync>> = HashMap::new();
    registry.insert("aether".into(), Box::new(AetherProvider::new()));
    registry.insert("v2ray".into(), Box::new(V2RayProvider::new()));
    registry.insert("wireguard".into(), Box::new(WireGuardProvider::new()));
    registry.insert("openvpn".into(), Box::new(OpenVpnProvider::new()));
    registry.insert("socks".into(), Box::new(SocksProvider::new()));
    Arc::new(Mutex::new(registry))
}

#[tauri::command]
pub async fn get_available_providers() -> Result<Vec<ProviderInfo>> {
    let registry = create_provider_registry();
    let providers = registry.lock().keys().cloned().collect();
    Ok(providers)
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

pub async fn connect_provider(
    provider_id: String,
    apps: Vec<String>,
    config: serde_json::Value,
    state: State<'_, crate::AppState>,
    app: AppHandle,
) -> Result<()> {
    let registry = create_provider_registry();
    let providers = registry.lock();
    
    let provider = providers.get(&provider_id)
        .ok_or_else(|| crate::error::XtunnelError::Provider(format!("Unknown provider: {}", provider_id)))?;
    
    let provider_config = match provider_id.as_str() {
        "aether" => {
            let cfg: crate::models::AetherConfig = serde_json::from_value(config)?;
            ProviderConfig::Aether(cfg)
        }
        "v2ray" => {
            let cfg: V2RayConfig = serde_json::from_value(config)?;
            ProviderConfig::V2Ray(cfg)
        }
        "wireguard" => {
            let cfg: WireGuardConfig = serde_json::from_value(config)?;
            ProviderConfig::WireGuard(cfg)
        }
        "openvpn" => {
            let cfg: OpenVpnConfig = serde_json::from_value(config)?;
            ProviderConfig::OpenVpn(cfg)
        }
        "socks" => {
            let cfg: SocksConfig = serde_json::from_value(config)?;
            ProviderConfig::Socks(cfg)
        }
        _ => return Err(crate::error::XtunnelError::Provider("Unknown provider".into())),
    };
    
    let pids = crate::apps::resolve_pids(&apps).await?;
    if pids.is_empty() {
        return Err(crate::error::XtunnelError::ProcessNotFound(
            "No running processes found for selected apps".into()
        ));
    }
    
    provider.connect(pids, provider_config, state, app).await
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type")]
pub enum ProviderConfig {
    Aether(crate::models::AetherConfig),
    V2Ray(V2RayConfig),
    WireGuard(WireGuardConfig),
    OpenVpn(OpenVpnConfig),
    Socks(SocksConfig),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct V2RayConfig {
    pub config: String,
    pub auto_tune_mtu: bool,
    pub enable_dns_optimization: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WireGuardConfig {
    pub config: String,
    pub config_path: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OpenVpnConfig {
    pub config_path: String,
    pub username: String,
    pub password: String,
    pub private_key_password: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SocksConfig {
    pub protocol: String,
    pub server: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

pub async fn disconnect(state: State<'_, crate::AppState>) -> Result<()> {
    // Stop WinDivert
    if let Some(mut wd) = state.windivert_manager.lock().await.take() {
        wd.stop()?;
    }
    *state.connected_pids.lock().await = vec![];
    
    // Kill provider process
    if let Some(mut proc) = state.provider_process.lock().await.take() {
        let _ = proc.kill().await;
    }
    
    // Kill sing-box
    if let Some(mut proc) = state.singbox_process.lock().await.take() {
        let _ = proc.kill().await;
    }
    
    Ok(())
}

#[tauri::command]
pub async fn get_status(state: State<'_, crate::AppState>) -> Result<crate::models::ConnectionStatus> {
    let pids = state.connected_pids.lock().await.clone();
    if pids.is_empty() {
        return Ok(crate::models::ConnectionStatus {
            state: "idle".into(),
            message: "Disconnected".into(),
            socks_port: None,
            connected_at: None,
            protocol: None,
            scan_mode: None,
        });
    }
    
    Ok(crate::models::ConnectionStatus {
        state: "connected".into(),
        message: format!("Connected - {} apps tunneled", pids.len()),
        socks_port: Some(*state.provider_socks_port.lock().await),
        connected_at: Some(chrono::Utc::now().timestamp_millis() as u64),
        protocol: Some(state.current_provider.lock().await.clone()),
        scan_mode: None,
    })
}