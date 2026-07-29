use crate::error::Result;
use crate::models::{ConnectionStatus, WireGuardConfig};
use crate::providers::Provider;
use parking_lot::Mutex;
use std::process::Stdio;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::process::Command;
use tokio::sync::Mutex as TokioMutex;

pub struct WireGuardProvider {
    process: Arc<TokioMutex<Option<tokio::process::Child>>>,
    config: Arc<Mutex<Option<WireGuardConfig>>>,
    interface_name: String,
}

impl WireGuardProvider {
    pub fn new() -> Self {
        Self {
            process: Arc::new(TokioMutex::new(None)),
            config: Arc::new(Mutex::new(None)),
            interface_name: "Xtunnel-WireGuard".into(),
        }
    }
}

#[async_trait::async_trait]
impl Provider for WireGuardProvider {
    fn id(&self) -> &str {
        "wireguard"
    }
    fn name(&self) -> &str {
        "WireGuard"
    }
    fn description(&self) -> &str {
        "WireGuard for Windows adapter mode"
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
            crate::models::ProviderConfig::WireGuard(c) => c,
            _ => {
                return Err(crate::error::XtunnelError::Config(
                    "Invalid config for WireGuard".into(),
                ))
            }
        };

        *self.config.lock() = Some(cfg.clone());

        let wg_exe = find_wireguard_exe()?;

        let config_path = std::env::temp_dir().join("Xtunnel-WireGuard.conf");
        tokio::fs::write(&config_path, &cfg.config).await?;

        let mut cmd = Command::new(&wg_exe)
            .args([
                "/installtunnelservice",
                config_path.to_str().unwrap(),
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| crate::error::XtunnelError::Process(e.to_string()))?;

        let status = cmd
            .wait()
            .await
            .map_err(|e| crate::error::XtunnelError::Process(e.to_string()))?;
        if !status.success() {
            return Err(crate::error::XtunnelError::Provider(
                "WireGuard service install failed".into(),
            ));
        }

        *state.provider_socks_port.lock() = 0;
        *state.current_provider.lock() = "wireguard".into();

        // WireGuard uses its own TUN, WinDivert routes PIDs to the WG interface
        crate::tunnel::start_tunnel(pids, 0, &state, app).await?;

        Ok(())
    }

    async fn disconnect(&self, _state: tauri::State<'_, crate::AppState>) -> Result<()> {
        let wg_exe = find_wireguard_exe()?;
        let mut cmd = Command::new(&wg_exe)
            .args(["/uninstalltunnelservice", &self.interface_name])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| crate::error::XtunnelError::Process(e.to_string()))?;

        let _ = cmd.wait().await;
        Ok(())
    }

    fn status(&self) -> ConnectionStatus {
        ConnectionStatus {
            state: "idle".into(),
            message: "WireGuard ready".into(),
            socks_port: None,
            connected_at: None,
            protocol: None,
            scan_mode: None,
        }
    }
}

fn find_wireguard_exe() -> Result<String> {
    let paths = [
        r"C:\Program Files\WireGuard\wireguard.exe",
        r"C:\Program Files (x86)\WireGuard\wireguard.exe",
    ];

    for p in &paths {
        if std::path::Path::new(p).exists() {
            return Ok(p.to_string());
        }
    }
    Err(crate::error::XtunnelError::BinaryNotFound(
        "WireGuard not found. Install from wireguard.com".into(),
    ))
}
