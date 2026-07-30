use crate::error::Result;
use crate::models::ConnectionStatus;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, State};
use tokio::time::{sleep, Duration};
use windivert::{WinDivert, WinDivertFlags, WinDivertLayer};

pub struct WinDivertTunnel {
    handle: WinDivert,
    pids: Vec<u32>,
    proxy_port: u16,
    running: Arc<AtomicBool>,
}

impl WinDivertTunnel {
    pub fn new(pids: Vec<u32>, proxy_port: u16) -> Result<Self> {
        let pids_str = pids
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(",");

        let filter = format!(
            "outbound && (tcp || udp) && (pid in {{{}}}) && !(dstPort == {})",
            pids_str, proxy_port
        );

        let handle = WinDivert::new(
            &filter,
            WinDivertLayer::Network,
            0,
            WinDivertFlags::empty(),
        )
        .map_err(|e| crate::error::XtunnelError::WinDivert(e.to_string()))?;

        Ok(Self {
            handle,
            pids,
            proxy_port,
            running: Arc::new(AtomicBool::new(true)),
        })
    }

    pub fn start(&self) -> Result<()> {
        let running = self.running.clone();

        std::thread::spawn(move || {
            // ponytail: WinDivert recv needs buffer_size param, not raw buffer+addr
            loop {
                if !running.load(Ordering::Relaxed) {
                    break;
                }
                match self.handle.recv(65535) {
                    Ok(packet) => {
                        if !packet.data.is_empty() {
                            let _ = self.handle.send(packet);
                        }
                    }
                    Err(e) => {
                        if running.load(Ordering::Relaxed) {
                            eprintln!("WinDivert recv error: {}", e);
                        }
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        let _ = self.handle.close(windivert::CloseAction::Nothing);
    }
}

fn build_singbox_config(proxy_port: u16) -> Result<String> {
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

pub async fn start_tunnel(
    pids: Vec<u32>,
    proxy_port: u16,
    state: &State<'_, crate::AppState>,
    app: AppHandle,
) -> Result<()> {
    // 1. Start sing-box with TUN inbound
    let singbox_config = build_singbox_config(proxy_port)?;
    let config_path = std::env::temp_dir().join("xtunnel-tunnel.json");
    tokio::fs::write(&config_path, &singbox_config).await?;

    let mut cmd = tokio::process::Command::new("sing-box.exe")
        .args(["run", "-c", config_path.to_str().unwrap()])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true)
        .spawn()
        .map_err(|e| crate::error::XtunnelError::Process(e.to_string()))?;

    *state.singbox_process.lock().await = Some(cmd);

    // 2. Wait for TUN interface to be ready
    sleep(Duration::from_secs(3)).await;

    // 3. Start WinDivert with PID filter
    let mut windivert_tunnel = WinDivertTunnel::new(pids.clone(), proxy_port)?;
    windivert_tunnel.start()?;
    // ponytail: WinDivert handle needs to live as long as the tunnel.
    // Currently it's dropped here which closes the handle.
    // Upgrade: store in AppState.windivert field.
    std::mem::forget(windivert_tunnel); // leaked intentionally — will be cleaned up when process dies

    *state.connected_pids.lock() = pids.clone();

    // 4. Emit connected status
    let _ = app.emit(
        "aether://status",
        &ConnectionStatus {
            state: "connected".into(),
            message: format!("Connected - {} apps tunneled", pids.len()),
            socks_port: Some(proxy_port),
            connected_at: Some(chrono::Utc::now().timestamp_millis() as u64),
            protocol: Some(state.current_provider.lock().clone()),
            scan_mode: None,
        },
    );

    // 5. Start monitor: watch sing-box process
    let singbox_process = state.singbox_process.clone();
    let app_handle = app.clone();
    tokio::spawn(async move {
        sleep(Duration::from_secs(5)).await;
        loop {
            sleep(Duration::from_secs(3)).await;
            let mut proc = singbox_process.lock().await;
            if let Some(p) = proc.as_mut() {
                if let Ok(Some(_status)) = p.try_wait() {
                    let _ = app_handle.emit(
                        "aether://status",
                        &ConnectionStatus {
                            state: "error".into(),
                            message: "sing-box process exited".into(),
                            socks_port: None,
                            connected_at: None,
                            protocol: None,
                            scan_mode: None,
                        },
                    );
                    break;
                }
            } else {
                break;
            }
        }
    });

    Ok(())
}
