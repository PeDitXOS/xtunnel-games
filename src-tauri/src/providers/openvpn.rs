use crate::error::Result;
use crate::models::{ConnectionStatus, OpenVpnConfig};
use crate::providers::Provider;
use parking_lot::Mutex;
use std::process::Stdio;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::process::Command;
use tokio::sync::Mutex as TokioMutex;

pub struct OpenVpnProvider {
    process: Arc<TokioMutex<Option<tokio::process::Child>>>,
    config: Arc<Mutex<Option<OpenVpnConfig>>>,
}

impl OpenVpnProvider {
    pub fn new() -> Self {
        Self {
            process: Arc::new(TokioMutex::new(None)),
            config: Arc::new(Mutex::new(None)),
        }
    }
}

#[async_trait::async_trait]
impl Provider for OpenVpnProvider {
    fn id(&self) -> &str {
        "openvpn"
    }
    fn name(&self) -> &str {
        "OpenVPN"
    }
    fn description(&self) -> &str {
        "OpenVPN Community"
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
            crate::models::ProviderConfig::OpenVpn(c) => c,
            _ => {
                return Err(crate::error::XtunnelError::Config(
                    "Invalid config for OpenVPN".into(),
                ))
            }
        };

        let openvpn_exe = find_openvpn_exe()?;

        let mut cmd = Command::new(&openvpn_exe)
            .args([
                "--config",
                &cfg.config_path,
                "--auth-user-pass",
                &cfg.username,
                "--auth-retry",
                "nointeract",
                "--route-nopull",
                "--pull-filter",
                "ignore",
                "redirect-gateway",
                "--script-security",
                "2",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| crate::error::XtunnelError::Process(e.to_string()))?;

        *self.process.lock().await = Some(cmd);

        // Wait for TUN interface
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        let socks_port = 2080;

        *state.provider_socks_port.lock() = socks_port;
        *state.current_provider.lock() = "openvpn".into();

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
            message: "OpenVPN ready".into(),
            socks_port: None,
            connected_at: None,
            protocol: None,
            scan_mode: None,
        }
    }
}

fn find_openvpn_exe() -> Result<String> {
    let paths = [
        r"C:\Program Files\OpenVPN\bin\openvpn.exe",
        r"C:\Program Files (x86)\OpenVPN\bin\openvpn.exe",
    ];
    for p in &paths {
        if std::path::Path::new(p).exists() {
            return Ok(p.to_string());
        }
    }
    Err(crate::error::XtunnelError::BinaryNotFound(
        "OpenVPN not found".into(),
    ))
}
