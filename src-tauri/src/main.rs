// Main entry point
use crate::{
    apps,
    config::ConfigManager,
    error::XtunnelError,
    models::*,
    providers::*,
    tunnel::TunnelManager,
};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![
            apps::scan_apps,
            apps::resolve_pids,
            get_available_providers,
            connect_provider,
            disconnect,
            get_status,
            get_config,
            set_config,
            get_selected_apps,
            set_selected_apps,
        ])
        .setup(|app| {
            // Initialize config
            let config_mgr = ConfigManager::new(app.handle())?;
            app.manage(config_mgr);
            
            // Load saved config
            let config = ConfigManager::get(app.state::<ConfigManager>());
            
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn main() {
    run()
}