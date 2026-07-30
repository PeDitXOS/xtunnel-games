use crate::error::Result;
use crate::models::{ConnectionStatus, V2RayConfig};
use crate::providers::Provider;
use parking_lot::Mutex;
use std::process::Stdio;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::process::Command;
use tokio::sync::Mutex as TokioMutex;

pub struct V2RayProvider {
    process: Arc<TokioMutex<Option<tokio::process::Child>>>,
    config: Arc<Mutex<Option<V2RayConfig>>>,
}

impl V2RayProvider {
    pub fn new() -> Self {
        Self {
            process: Arc::new(TokioMutex::new(None)),
            config: Arc::new(Mutex::new(None)),
        }
    }

    fn build_singbox_config(&self, socks_port: u16) -> Result<String> {
        let config = serde_json::json!({
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
                "listen_port": 2080
            }],
            "outbounds": [{
                "type": "socks",
                "tag": "v2ray-socks",
                "server": "127.0.0.1",
                "server_port": socks_port,
                "version": "5"
            }, {
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

        Ok(serde_json::to_string_pretty(&config)?)
    }
}

#[async_trait::async_trait]
impl Provider for V2RayProvider {
    fn id(&self) -> &str {
        "v2ray"
    }
    fn name(&self) -> &str {
        "V2Ray / Xray"
    }
    fn description(&self) -> &str {
        "V2Ray/Xray with sing-box TUN bridge"
    }
    fn requires_server(&self) -> bool {
        true
    }

    async fn connect(
        &self,
        pids: Vec<u32>,
        config: crate::models::ProviderConfig,
        state: tauri::State<'_, crate::AppState>,
        app: AppHandle,
    ) -> Result<()> {
        let cfg = match config {
            crate::models::ProviderConfig::V2Ray(c) => c,
            _ => {
                return Err(crate::error::XtunnelError::Config(
                    "Invalid config for V2Ray".into(),
                ))
            }
        };

        *self.config.lock() = Some(cfg);

        let socks_port = 2080;
        let singbox_config = self.build_singbox_config(socks_port)?;
        let config_path = std::env::temp_dir().join("xtunnel-v2ray.json");
        tokio::fs::write(&config_path, singbox_config).await?;

        let singbox_path = crate::resolve_binary(&app, "sing-box.exe");
        let mut cmd = Command::new(singbox_path)
            .args(["run", "-c", config_path.to_str().unwrap()])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| crate::error::XtunnelError::Process(e.to_string()))?;

        *self.process.lock().await = Some(cmd);

        // Wait for TUN
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;

        *state.provider_socks_port.lock() = socks_port;
        *state.current_provider.lock() = "v2ray".into();

        crate::tunnel::start_tunnel(pids, socks_port, &state, app).await?;

        Ok(())
    }

    async fn disconnect(&self, _state: tauri::State<'_, crate::AppState>) -> Result<()> {
        if let Some(mut p) = self.process.lock().await.take() {
            let _ = p.kill().await;
        }
        Ok(())
    }

    fn status(&self) -> ConnectionStatus {
        ConnectionStatus {
            state: "idle".into(),
            message: "V2Ray ready".into(),
            socks_port: Some(2080),
            connected_at: None,
            protocol: None,
            scan_mode: None,
        }
    }
}
