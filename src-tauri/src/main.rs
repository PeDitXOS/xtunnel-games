use xtunnel_games::{
    commands, config::ConfigManager, providers, updater, AppState,
};
use tauri::Manager;

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
    // Add exe directory to DLL search path so Windows finds bundled DLLs
    // (WinDivert.dll, wintun.dll, etc.) without needing them in system PATH
    #[cfg(target_os = "windows")]
    {
        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                // Add exe dir AND resources/ subdirectory to PATH
                // Tauri bundles resources in a subdirectory next to the exe
                if let Ok(current_path) = std::env::var("PATH") {
                    let new_path = format!(
                        "{};{}\\resources;{}",
                        dir.display(),
                        dir.display(),
                        current_path
                    );
                    // SAFETY: setting PATH at process start before any threads
                    unsafe { std::env::set_var("PATH", &new_path) };
                }
            }
        }
    }

    run();
}
