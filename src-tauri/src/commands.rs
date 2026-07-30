use crate::models::*;
use crate::AppState;
use tauri::{AppHandle, Emitter, State};

#[tauri::command]
pub async fn scan_apps() -> Result<Vec<AppInfo>, String> {
    crate::apps::scan_apps()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn aether_connect(
    apps: Vec<String>,
    config: AetherConfig,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), String> {
    // Kill existing connections
    let _ = crate::providers::disconnect(state.clone()).await;

    // Resolve PIDs for selected apps
    let pids = crate::apps::resolve_pids(&apps)
        .await
        .map_err(|e| e.to_string())?;
    if pids.is_empty() {
        return Err(
            "No running processes found for selected apps. Start the apps first.".into(),
        );
    }

    // Start aether.exe
    let aether_path = crate::resolve_binary(&app, "aether.exe");
    let mut cmd = tokio::process::Command::new(aether_path);
    cmd.args(config.to_cli_args())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .kill_on_drop(true);

    let child = cmd
        .spawn()
        .map_err(|e| format!("Failed to start Aether: {}", e))?;
    *state.provider_process.lock().await = Some(child);

    // Wait for SOCKS5 to be ready
    wait_for_socks(1819)
        .await
        .map_err(|e| format!("Aether SOCKS5 not ready: {}", e))?;

    // Start sing-box TUN + WinDivert tunnel
    crate::tunnel::start_tunnel(pids, 1819, &state, app.clone())
        .await
        .map_err(|e| format!("Tunnel error: {}", e))?;

    // Update state
    *state.provider_socks_port.lock() = 1819;
    *state.current_provider.lock() = "aether".into();
    *state.provider_config.lock() =
        Some(serde_json::to_value(&config).unwrap_or_default());

    // Emit connected status
    let _ = app.emit(
        "aether://status",
        ConnectionStatus {
            state: "connected".into(),
            message: "Aether connected".into(),
            socks_port: Some(1819),
            connected_at: Some(chrono::Utc::now().timestamp_millis() as u64),
            protocol: Some(config.protocol),
            scan_mode: Some(config.scan_mode),
        },
    );

    Ok(())
}

#[tauri::command]
pub async fn aether_disconnect(
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), String> {
    let _ = crate::providers::disconnect(state).await;

    let _ = app.emit(
        "aether://status",
        ConnectionStatus {
            state: "idle".into(),
            message: "Disconnected".into(),
            socks_port: None,
            connected_at: None,
            protocol: None,
            scan_mode: None,
        },
    );

    Ok(())
}

#[tauri::command]
pub async fn get_status(state: State<'_, AppState>) -> Result<ConnectionStatus, String> {
    let pids = state.connected_pids.lock().clone();
    if pids.is_empty() {
        return Ok(ConnectionStatus {
            state: "idle".into(),
            message: "Disconnected".into(),
            socks_port: None,
            connected_at: None,
            protocol: None,
            scan_mode: None,
        });
    }
    Ok(ConnectionStatus {
        state: "connected".into(),
        message: format!("Connected - {} apps tunneled", pids.len()),
        socks_port: Some(*state.provider_socks_port.lock()),
        connected_at: Some(chrono::Utc::now().timestamp_millis() as u64),
        protocol: Some(state.current_provider.lock().clone()),
        scan_mode: None,
    })
}

#[tauri::command]
pub async fn get_available_providers() -> Result<Vec<String>, String> {
    Ok(crate::providers::get_available_providers()
        .await
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|p| p.id)
        .collect())
}

#[tauri::command]
pub async fn get_selected_apps() -> Result<Vec<String>, String> {
    Ok(vec![])
}

#[tauri::command]
pub async fn set_selected_apps(_apps: Vec<String>) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub async fn get_config() -> Result<AetherConfig, String> {
    Ok(AetherConfig::default())
}

#[tauri::command]
pub async fn set_config(_config: AetherConfig) -> Result<(), String> {
    Ok(())
}

async fn wait_for_socks(port: u16) -> crate::error::Result<()> {
    use crate::error::XtunnelError;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;
    use tokio::time::{sleep, timeout, Duration};

    for _ in 0..60 {
        if let Ok(mut stream) =
            timeout(Duration::from_secs(1), TcpStream::connect(("127.0.0.1", port))).await
        {
            if let Ok(mut stream) = stream {
                let _ = stream.write_all(&[0x05, 0x01, 0x00]).await;
                let _ = stream.flush().await;
                let mut buf = [0u8; 2];
                if timeout(Duration::from_secs(2), stream.read_exact(&mut buf))
                    .await
                    .is_ok()
                {
                    if buf[0] == 0x05 && buf[1] == 0x00 {
                        return Ok(());
                    }
                }
            }
        }
        sleep(Duration::from_secs(1)).await;
    }
    Err(XtunnelError::SocksHandshakeFailed)
}
