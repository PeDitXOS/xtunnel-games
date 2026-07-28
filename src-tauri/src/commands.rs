use std::process::Stdio;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};
use tauri_plugin_store::StoreExt;
use tokio::process::Command;
use tokio::sync::Mutex;

use crate::aether::AetherManager;
use crate::windivert::WinDivertManager;

#[tauri::command]
pub async fn scan_apps() -> Result<Vec<AppInfo>, String> {
    apps::scan_apps().await
}

#[tauri::command]
pub async fn aether_connect(
    apps: Vec<String>,
    config: AetherConfig,
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<(), String> {
    let mut aether_mgr = state.aether_manager.lock().await;
    aether_mgr.connect(apps, config, app).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn aether_disconnect(
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut aether_mgr = state.aether_manager.lock().await;
    aether_mgr.disconnect().await.map_err(|e| e.to_string())
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

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInfo {
    pub name: String,
    pub exe_name: String,
    pub exe_path: String,
    pub icon_path: Option<String>,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AetherConfig {
    pub protocol: String,
    pub scan_mode: String,
    pub ip_version: String,
    pub quick_reconnect: bool,
}

impl Default for AetherConfig {
    fn default() -> Self {
        Self {
            protocol: "auto".into(),
            scan_mode: "balanced".into(),
            ip_version: "v4".into(),
            quick_reconnect: true,
        }
    }
}

pub struct AppState {
    pub aether_manager: Arc<Mutex<AetherManager>>,
    pub windivert_manager: Arc<Mutex<WinDivertManager>>,
}

mod aether;
mod windivert;
mod apps;
mod config;