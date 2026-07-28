use crate::error::Result;
use crate::models::ConnectionStatus;
use parking_lot::Mutex;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use tokio::process::Command;
use tokio::sync::Mutex as TokioMutex;
use tokio::time::{sleep, Duration};

pub struct TunnelManager {
    windivert: Arc<TokioMutex<Option<WinDivert>>>,
    singbox: Arc<TokioMutex<Option<tokio::process::Child>>>,
    connected_pids: Arc<Mutex<Vec<u32>>>,
    mixed_port: Arc<Mutex<u16>>,
    running: Arc<Mutex<bool>>,
}

impl TunnelManager {
    pub fn new() -> Self {
        Self {
            windivert: Arc::new(TokioMutex::new(None)),
            singbox: Arc::new(TokioMutex::new(None)),
            connected_pids: Arc::new(Mutex::new(Vec::new())),
            mixed_port: Arc::new(Mutex::new(0)),
            running: Arc::new(Mutex::new(false)),
        }
    }

    pub async fn start_tunnel(
        &self,
        pids: Vec<u32>,
        proxy_port: u16,
        state: State<'_, crate::AppState>,
        app: AppHandle,
    ) -> Result<()> {
        // 1. Start sing-box with TUN inbound + mixed proxy
        let singbox_config = self.build_singbox_config(proxy_port)?;
        let config_path = std::env::temp_dir().join("xtunnel-tunnel.json");
        tokio::fs::write(&config_path, singbox_config).await?;

        let mut cmd = Command::new("sing-box.exe")
            .args(["run", "-c", config_path.to_str().unwrap()])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .map_err(|e| crate::error::XtunnelError::Process(e.to_string()))?;

        *self.singbox.lock().await = Some(cmd);
        *self.mixed_port.lock() = 2080;
        *self.connected_pids.lock() = pids.clone();
        *self.running.lock() = true;

        // Wait for TUN interface
        sleep(Duration::from_secs(3)).await;

        // 2. Start WinDivert with PID filter
        let mut windivert = WinDivert::new(pids, proxy_port)?;
        windivert.start()?;
        *self.windivert.lock().await = Some(windivert);

        // 3. Emit connected status
        let _ = app.emit("aether://status", &ConnectionStatus {
            state: "connected".into(),
            message: format!("Connected - {} apps tunneled", self.connected_pids.lock().len()),
            socks_port: Some(proxy_port),
            connected_at: Some(chrono::Utc::now().timestamp_millis() as u64),
            protocol: Some(*state.current_provider.lock().await),
            scan_mode: None,
        });

        // 4. Start monitor
        self.start_monitor(app).await;

        Ok(())
    }

    fn build_singbox_config(&self, proxy_port: u16) -> Result<String> {
        let config = serde_json::json!({
            "log": { "level": "warn", "timestamp": false },
            "inbounds": [{
                "type": "tun",
                "tag": "tun-in",
                "interface_name": "Xtunnel-Games",
                "address": ["172.25.0.1/30"],
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
                "tag": "provider-socks",
                "server": "127.0.0.1",
                "server_port": proxy_port,
                "version": "5",
                "udp_enabled": true
            }, {
                "type": "direct",
                "tag": "direct"
            }],
            "route": {
                "rules": [
                    { "ip_cidr": ["127.0.0.0/8"], "outbound": "direct" },
                    { "inbound": ["tun-in"], "outbound": "provider-socks" },
                    { "inbound": ["mixed-in"], "outbound": "provider-socks" }
                ]
            }
        });

        Ok(serde_json::to_string_pretty(&config)?)
    }

    pub async fn stop_tunnel(&self) -> Result<()> {
        *self.running.lock() = false;

        // Stop WinDivert
        if let Some(mut wd) = self.windivert.lock().await.take() {
            wd.stop()?;
        }

        // Kill sing-box
        if let Some(mut proc) = self.singbox.lock().await.take() {
            let _ = proc.kill().await;
        }

        *self.connected_pids.lock() = Vec::new();
        Ok(())
    }
}

// WinDivert wrapper
use windivert::WinDivertHandle;

pub struct WinDivert {
    handle: WinDivertHandle,
    pids: Vec<u32>,
    proxy_port: u16,
    running: Arc<std::sync::atomic::AtomicBool>,
}

impl WinDivert {
    pub fn new(pids: Vec<u32>, proxy_port: u16) -> Result<Self> {
        let pids_str = pids.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(",");
        
        // Filter: outbound TCP/UDP from selected PIDs, EXCEPT proxy port itself
        let filter = format!(
            "outbound && (tcp || udp) && (pid in {{{}}}) && !(dstPort == {})",
            pids_str, proxy_port
        );

        let handle = windivert::WinDivertHandle::open(&filter, windivert::Layer::Network, 0, windivert::Flags::empty())
            .map_err(|e| crate::error::XtunnelError::WinDivert(e.to_string()))?;

        Ok(Self {
            handle,
            pids,
            proxy_port,
            running: Arc::new(std::sync::atomic::AtomicBool::new(true)),
        })
    }

    pub fn start(&mut self) -> Result<()> {
        let running = self.running.clone();
        let handle = self.handle.clone();
        let proxy_port = self.proxy_port;

        std::thread::spawn(move || {
            let mut packet = vec![0u8; 65535];
            let mut addr = windivert::WinDivertAddress::default();

            while running.load(std::sync::atomic::Ordering::Relaxed) {
                match handle.recv(&mut packet, &mut addr) {
                    Ok(len) => {
                        if len > 0 {
                            // Reinject packet (it will go through TUN via WinDivert filter)
                            let _ = handle.send(&packet[..len], &addr);
                        }
                    }
                    Err(e) => {
                        if running.load(std::sync::atomic::Ordering::Relaxed) {
                            eprintln!("WinDivert recv error: {}", e);
                        }
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    pub fn stop(&mut self) -> Result<()> {
        self.running.store(false, std::sync::atomic::Ordering::Relaxed);
        self.handle.close().map_err(|e| crate::error::XtunnelError::WinDivert(e.to_string()))
    }
}