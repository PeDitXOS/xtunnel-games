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

#[cfg(target_os = "windows")]
fn setup_dll_path() {
    use std::ffi::CString;
    use std::ffi::c_void;

    extern "system" {
        fn AddDllDirectory(lpPathName: *const u16) -> *mut c_void;
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            // Add exe directory
            if let Ok(c_dir) = CString::new(dir.to_string_lossy().as_bytes()) {
                let wide: Vec<u16> = c_dir.to_string_lossy().encode_utf16().chain(std::iter::once(0)).collect();
                unsafe {
                    AddDllDirectory(wide.as_ptr());
                }
            }
            // Also try resources/ subdirectory (Tauri bundles resources there)
            let res_dir = dir.join("resources");
            if res_dir.exists() {
                if let Ok(c_res) = CString::new(res_dir.to_string_lossy().as_bytes()) {
                    let wide: Vec<u16> = c_res.to_string_lossy().encode_utf16().chain(std::iter::once(0)).collect();
                    unsafe {
                        AddDllDirectory(wide.as_ptr());
                    }
                }
            }
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn setup_dll_path() {}

fn main() {
    setup_dll_path();
    run();
}