use crate::error::Result;
use crate::models::OpenVpnConfig;
use crate::providers::Provider;
use parking_lot::Mutex;
use std::process::Stdio;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::process::Command;
use tokio::sync::Mutex as TokioMutex;
use tokio::time::{sleep, Duration};

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
    fn id(&self) -> &str { "openvpn" }
    fn name(&self) -> &str { "OpenVPN" }
    fn description(&self) -> &str { "OpenVPN Community" }
    fn requires_server(&self) -> bool { true }

    async fn connect(
        &self,
        pids: Vec<u32>,
        config: crate::providers::ProviderConfig,
        state: tauri::State<'_, crate::AppState>,
        app: AppHandle,
    ) -> Result<()> {
        let cfg = match config {
            crate::providers::ProviderConfig::OpenVpn(c) => c,
            _ => return Err(crate::error::XtunnelError::Config("Invalid config for OpenVPN".into())),
        };

        let openvpn_exe = find_openvpn_exe()?;
        
        let mut cmd = Command::new(&openvpn_exe)
            .args([
                "--config", &cfg.config_path,
                "--auth-user-pass", &format!("{}", cfg.username),
                "--auth-retry", "nointeract",
                "--route-nopull",
                "--pull-filter", "ignore", "redirect-gateway",
                "--script-security", "2",
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| crate::error::XtunnelError::Process(e.to_string()))?;

        *self.process.lock().await = Some(cmd);

        // Wait for TUN interface
        let _ = wait_for_openvpn_interface().await?;

        // Get SOCKS port from openvpn management interface or config
        let socks_port = 2080;

        // Start WinDivert tunnel
        crate::tunnel::start_tunnel(pids, socks_port, state.clone(), app.clone()).await?;

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
    Err(crate::error::XtunnelError::BinaryNotFound("OpenVPN not found".into()))
}

async fn wait_for_openvpn_interface() -> Result<u32> {
    for _ in 0..30 {
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
    Ok(0)
}