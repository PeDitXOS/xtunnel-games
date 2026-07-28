use crate::error::Result;
use crate::models::SocksConfig;
use crate::providers::Provider;
use parking_lot::Mutex;
use std::process::Stdio;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::process::Command;
use tokio::sync::Mutex as TokioMutex;
use tokio::time::{sleep, Duration};

pub struct SocksProvider {
    process: Arc<TokioMutex<Option<tokio::process::Child>>>,
    config: Arc<Mutex<Option<SocksConfig>>>,
    socks_port: u16,
    mixed_port: u16,
}

impl SocksProvider {
    pub fn new() -> Self {
        Self {
            process: Arc::new(TokioMutex::new(None)),
            config: Arc::new(Mutex::new(None)),
            socks_port: 1080,
            mixed_port: 2080,
        }
    }
}

#[async_trait::async_trait]
impl Provider for SocksProvider {
    fn id(&self) -> &str { "socks" }
    fn name(&self) -> &str { "SOCKS5 / HTTP Proxy" }
    fn description(&self) -> &str { "External SOCKS5/HTTP proxy with sing-box TUN bridge" }
    fn requires_server(&self) -> bool { true }

    async fn connect(
        &self,
        pids: Vec<u32>,
        config: crate::providers::ProviderConfig,
        state: tauri::State<'_, crate::AppState>,
        app: AppHandle,
    ) -> Result<()> {
        let cfg = match config {
            crate::providers::ProviderConfig::Socks(c) => c,
            _ => return Err(crate::error::XtunnelError::Config("Invalid config for SOCKS".into())),
        };

        self.socks_port = cfg.port;
        self.mixed_port = 2080;

        // Build sing-box config with SOCKS outbound
        let singbox_config = self.build_singbox_config(&cfg)?;
        let config_path = std::env::temp_dir().join("xtunnel-socks.json");
        tokio::fs::write(&config_path, singbox_config).await?;

        // Start sing-box
        let mut cmd = Command::new("sing-box.exe")
            .args(["run", "-c", config_path.to_str().unwrap()])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| crate::error::XtunnelError::Process(e.to_string()))?;

        *self.process.lock().await = Some(cmd);

        // Wait for TUN
        sleep(Duration::from_secs(3)).await;

        // Update state
        *state.provider_socks_port.lock().await = self.mixed_port;
        *state.current_provider.lock().await = "socks".into();

        // Start tunnel
        crate::tunnel::start_tunnel(pids, self.mixed_port, state.clone(), app.clone()).await?;

        Ok(())
    }

    async fn disconnect(&self, _state: tauri::State<'_, crate::AppState>) -> Result<()> {
        if let Some(mut p) = self.process.lock().await.take() {
            let _ = p.kill().await;
        }
        Ok(())
    }

    fn status(&self) -> crate::models::ConnectionStatus {
        crate::models::ConnectionStatus {
            state: "idle".into(),
            message: "SOCKS Proxy ready".into(),
            socks_port: Some(self.mixed_port),
            connected_at: None,
            protocol: None,
            scan_mode: None,
        }
    }
}

impl SocksProvider {
    fn build_singbox_config(&self, cfg: &crate::models::SocksConfig) -> Result<String> {
        let outbound = serde_json::json!({
            "type": if cfg.protocol == "http" { "http" } else { "socks" },
            "tag": "proxy-out",
            "server": cfg.server,
            "server_port": cfg.port,
            "version": "5",
            "username": cfg.username,
            "password": cfg.password,
            "tls": {
                "enabled": false
            }
        });

        let config = serde_json::json!({
            "log": { "level": "warn", "timestamp": false },
            "inbounds": [{
                "type": "tun",
                "tag": "tun-in",
                "interface_name": "Xtunnel-Socks",
                "address": ["172.22.0.1/30"],
                "mtu": 1500,
                "auto_route": false,
                "strict_route": false,
                "stack": "gvisor",
                "sniff": true
            }, {
                "type": "mixed",
                "tag": "mixed-in",
                "listen": "127.0.0.1",
                "listen_port": self.mixed_port
            }],
            "outbounds": [outbound, { "type": "direct", "tag": "direct" }],
            "route": {
                "rules": [
                    { "ip_cidr": ["127.0.0.0/8"], "outbound": "direct" },
                    { "inbound": ["tun-in"], "outbound": "proxy-out" },
                    { "inbound": ["mixed-in"], "outbound": "proxy-out" }
                ]
            }
        });

        Ok(serde_json::to_string_pretty(&config)?)
    }
}