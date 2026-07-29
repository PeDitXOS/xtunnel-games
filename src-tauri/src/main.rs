use xtunnel_games::{
    commands, config::ConfigManager, providers, updater, AppState,
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
            commands::scan_apps,
            commands::aether_connect,
            commands::aether_disconnect,
            commands::get_status,
            commands::get_available_providers,
            commands::get_selected_apps,
            commands::set_selected_apps,
            commands::get_config,
            commands::set_config,
            providers::get_available_providers,
            providers::get_status,
            updater::check_updates,
        ])
        .setup(|app| {
            let config_mgr = ConfigManager::new(app.handle())?;
            app.manage(config_mgr);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn main() {
    run();
}
