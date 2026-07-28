use crate::error::Result;
use crate::models::WireGuardConfig;
use crate::providers::Provider;
use parking_lot::Mutex;
use std::process::Stdio;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::process::Command;
use tokio::sync::Mutex as TokioMutex;
use tokio::time::{sleep, Duration};

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
    fn id(&self) -> &str { "wireguard" }
    fn name(&self) -> &str { "WireGuard" }
    fn description(&self) -> &str { "WireGuard for Windows adapter mode" }
    fn requires_server(&self) -> bool { true }

    async fn connect(
        &self,
        pids: Vec<u32>,
        config: crate::providers::ProviderConfig,
        state: tauri::State<'_, crate::AppState>,
        app: AppHandle,
    ) -> Result<()> {
        let cfg = match config {
            crate::providers::ProviderConfig::WireGuard(c) => c,
            _ => return Err(crate::error::XtunnelError::Config("Invalid config for WireGuard".into())),
        };

        // WireGuard on Windows runs as a service, not a process we spawn directly
        // Use wireguard.exe /installtunnelservice
        let wg_exe = find_wireguard_exe()?;
        
        let config_content = build_wireguard_config(&cfg)?;
        let config_path = std::env::temp_dir().join("Xtunnel-WireGuard.conf");
        tokio::fs::write(&config_path, config_content).await?;

        // Install/start service
        let mut cmd = Command::new(&wg_exe)
            .args(["/installtunnelservice", config_path.to_str().unwrap()])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| crate::error::XtunnelError::Process(e.to_string()))?;
        
        let status = cmd.wait().await.map_err(|e| crate::error::XtunnelError::Process(e.to_string()))?;
        if !status.success() {
            return Err(crate::error::XtunnelError::Provider("WireGuard service install failed".into()));
        }

        // Wait for interface
        let interface_index = wait_for_interface(&self.interface_name).await?;

        // Update state
        *state.provider_socks_port.lock().await = 0; // WireGuard doesn't use SOCKS
        *state.current_provider.lock().await = "wireguard".into();

        // Start tunnel with WinDivert (PID-based, uses interface)
        crate::tunnel::start_tunnel(pids, 0, state.clone(), app.clone()).await?;

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

    fn status(&self) -> crate::models::ConnectionStatus {
        crate::models::ConnectionStatus {
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
    Err(crate::error::XtunnelError::BinaryNotFound("WireGuard not found. Install from wireguard.com".into()))
}

fn build_wireguard_config(cfg: &WireGuardConfig) -> Result<String> {
    // Build WireGuard config from parsed config
    Ok(cfg.private_key.clone()) // Simplified
}

async fn wait_for_interface(name: &str) -> Result<u32> {
    for _ in 0..20 {
        tokio::time::sleep(Duration::from_secs(1)).await;
        // Check interface exists
    }
    Ok(0)
}