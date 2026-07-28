use crate::error::Result;
use crate::models::AetherConfig;
use crate::providers::Provider;
use parking_lot::Mutex;
use std::process::Stdio;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::process::Command;
use tokio::sync::Mutex as TokioMutex;
use tokio::time::{sleep, Duration};

pub struct V2RayProvider {
    process: Arc<TokioMutex<Option<tokio::process::Child>>>,
    singbox_process: Arc<TokioMutex<Option<tokio::process::Child>>>,
    socks_port: u16,
    config: Arc<Mutex<Option<V2RayConfig>>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct V2RayConfig {
    pub v2ray_config: String,
    pub auto_tune_mtu: bool,
    pub enable_dns_optimization: bool,
    pub protocol: String, // "xray" or "sing-box"
}

impl V2RayProvider {
    pub fn new() -> Self {
        Self {
            process: Arc::new(TokioMutex::new(None)),
            singbox_process: Arc::new(TokioMutex::new(None)),
            socks_port: 2080,
            config: Arc::new(Mutex::new(None)),
        }
    }

    fn build_singbox_config(&self, config: &V2RayConfig, socks_port: u16, mixed_port: u16) -> Result<String> {
        // Build sing-box config with TUN inbound + SOCKS outbound
        let outbound = serde_json::json!({
            "type": "socks",
            "tag": "v2ray-socks",
            "server": "127.0.0.1",
            "server_port": socks_port,
            "version": "5"
        });

        let config_json = serde_json::json!({
            "log": { "level": "warn", "timestamp": false },
            "inbounds": [{
                "type": "tun",
                "tag": "tun-in",
                "interface_name": "Xtunnel-V2Ray",
                "address": ["172.18.0.1/30"],
                "mtu": 1500,
                "auto_route": false,
                "strict_route": false,
                "stack": "gvisor",
                "sniff": true,
                "sniff_override_destination": true
            }, {
                "type": "mixed",
                "tag": "mixed-in",
                "listen": "127.0.0.1",
                "listen_port": mixed_port
            }],
            "outbounds": [outbound, {
                "type": "direct",
                "tag": "direct"
            }],
            "route": {
                "rules": [
                    {"ip_cidr": ["127.0.0.0/8"], "outbound": "direct"},
                    {"inbound": ["tun-in"], "outbound": "v2ray-socks"},
                    {"inbound": ["mixed-in"], "outbound": "v2ray-socks"}
                ]
            }
        });

        Ok(serde_json::to_string_pretty(&config_json)?)
    }
}

#[async_trait::async_trait]
impl Provider for V2RayProvider {
    fn id(&self) -> &str { "v2ray" }
    fn name(&self) -> &str { "V2Ray / Xray" }
    fn description(&self) -> &str { "V2Ray/Xray with sing-box TUN bridge" }
    fn requires_server(&self) -> bool { true }

    async fn connect(
        &self,
        pids: Vec<u32>,
        config: crate::providers::ProviderConfig,
        state: tauri::State<'_, crate::AppState>,
        app: AppHandle,
    ) -> Result<()> {
        let cfg = match config {
            crate::providers::ProviderConfig::V2Ray(c) => c,
            _ => return Err(crate::error::XtunnelError::Config("Invalid config for V2Ray".into())),
        };

        // Build sing-box config
        let mixed_port = 2080;
        let singbox_config = self.build_singbox_config(&cfg, self.socks_port, mixed_port)?;
        let config_path = std::env::temp_dir().join("xtunnel-v2ray.json");
        tokio::fs::write(&config_path, singbox_config).await?;

        // Start sing-box with TUN
        let mut cmd = tokio::process::Command::new("sing-box.exe")
            .args(["run", "-c", config_path.to_str().unwrap()])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| crate::error::XtunnelError::Process(e.to_string()))?;
        
        *self.singbox_process.lock().await = Some(cmd);

        // Wait for TUN interface
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

        // Update state
        *state.provider_socks_port.lock().await = mixed_port;
        *state.current_provider.lock().await = "v2ray".into();

        // Start WinDivert tunnel
        crate::tunnel::start_tunnel(pids, mixed_port, state.clone(), app.clone()).await?;

        Ok(())
    }

    async fn disconnect(&self, _state: tauri::State<'_, crate::AppState>) -> Result<()> {
        if let Some(mut p) = self.process.lock().await.take() {
            let _ = p.kill().await;
        }
        if let Some(mut p) = self.singbox_process.lock().await.take() {
            let _ = p.kill().await;
        }
        Ok(())
    }

    fn status(&self) -> crate::models::ConnectionStatus {
        crate::models::ConnectionStatus {
            state: "idle".into(),
            message: "V2Ray ready".into(),
            socks_port: Some(self.socks_port),
            connected_at: None,
            protocol: None,
            scan_mode: None,
        }
    }
}