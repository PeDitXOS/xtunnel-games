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

pub struct AetherProvider {
    process: Arc<TokioMutex<Option<tokio::process::Child>>>,
    socks_port: u16,
    config: Arc<Mutex<Option<AetherConfig>>>,
}

impl AetherProvider {
    pub fn new() -> Self {
        Self {
            process: Arc::new(TokioMutex::new(None)),
            socks_port: 1819,
            config: Arc::new(Mutex::new(None)),
        }
    }

    fn build_command(&self, config: &AetherConfig) -> Command {
        let mut cmd = Command::new("aether.exe");
        cmd.args(config.to_cli_args())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null())
            .kill_on_drop(true);
        cmd
    }

    async fn wait_for_socks(&self, port: u16) -> Result<()> {
        use tokio::net::TcpStream;
        use tokio::time::{timeout, Duration};
        use tokio::io::{AsyncReadExt, AsyncWriteExt};

        for _ in 0..120 {
            if timeout(Duration::from_secs(1), TcpStream::connect(("127.0.0.1", port))).await.is_ok() {
                // Verify SOCKS5 handshake
                if let Ok(mut stream) = TcpStream::connect(("127.0.0.1", port)).await {
                    let _ = stream.write_all(&[0x05, 0x01, 0x00]).await;
                    let _ = stream.flush().await;
                    let mut buf = [0u8; 2];
                    if timeout(Duration::from_secs(2), stream.read_exact(&mut buf)).await.is_ok() {
                        if buf[0] == 0x05 && buf[1] == 0x00 {
                            return Ok(());
                        }
                    }
                }
            }
            sleep(Duration::from_secs(1)).await;
        }
        Err(crate::error::XtunnelError::SocksHandshakeFailed)
    }
}

#[async_trait::async_trait]
impl Provider for AetherProvider {
    fn id(&self) -> &str { "aether" }
    fn name(&self) -> &str { "Aether" }
    fn description(&self) -> &str { "Serverless anti-censorship tunnel (MASQUE/WireGuard/gool)" }
    fn requires_server(&self) -> bool { false }

    async fn connect(
        &self,
        pids: Vec<u32>,
        config: crate::providers::ProviderConfig,
        state: tauri::State<'_, crate::AppState>,
        app: AppHandle,
    ) -> Result<()> {
        let cfg = match config {
            crate::providers::ProviderConfig::Aether(c) => c,
            _ => return Err(crate::error::XtunnelError::Config("Invalid config for Aether".into())),
        };

        *self.config.lock() = Some(cfg.clone());
        self.socks_port = 1819;

        // Start aether.exe
        let mut cmd = self.build_command(&cfg);
        let mut child = cmd.spawn().map_err(|e| crate::error::XtunnelError::Process(e.to_string()))?;
        
        *self.process.lock().await = Some(child);

        // Wait for SOCKS5
        self.wait_for_socks(self.socks_port).await?;

        // Update state
        *state.provider_socks_port.lock().await = self.socks_port;
        *state.current_provider.lock().await = "aether".into();
        *state.provider_config.lock().await = Some(serde_json::to_value(&cfg)?);

        // Start tunnel orchestration
        crate::tunnel::start_tunnel(pids, self.socks_port, state.clone(), app.clone()).await?;

        // Emit status
        let _ = app.emit("aether://status", &crate::models::ConnectionStatus {
            state: "connected".into(),
            message: "Aether connected".into(),
            socks_port: Some(self.socks_port),
            connected_at: Some(chrono::Utc::now().timestamp_millis() as u64),
            protocol: Some(cfg.protocol.clone()),
            scan_mode: Some(cfg.scan_mode.clone()),
        });

        // Monitor loop
        let process = self.process.clone();
        let app_handle = app.clone();
        tokio::spawn(async move {
            loop {
                sleep(Duration::from_secs(2)).await;
                let mut proc = process.lock().await;
                if let Some(p) = proc.as_mut() {
                    if p.try_wait().ok().flatten().is_some() {
                        let _ = app_handle.emit("aether://status", &crate::models::ConnectionStatus {
                            state: "error".into(),
                            message: "Aether process exited".into(),
                            socks_port: None,
                            connected_at: None,
                            protocol: None,
                            scan_mode: None,
                        });
                        break;
                    }
                } else {
                    break;
                }
            }
        });

        Ok(())
    }

    async fn disconnect(&self, _state: tauri::State<'_, crate::AppState>) -> Result<()> {
        let mut proc = self.process.lock().await;
        if let Some(p) = proc.take() {
            let _ = p.kill().await;
        }
        Ok(())
    }

    fn status(&self) -> crate::models::ConnectionStatus {
        crate::models::ConnectionStatus {
            state: "idle".into(),
            message: "Aether ready".into(),
            socks_port: Some(self.socks_port),
            connected_at: None,
            protocol: None,
            scan_mode: None,
        }
    }
}