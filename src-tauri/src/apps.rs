use crate::error::Result;
use crate::models::AppInfo;
use std::collections::HashSet;
use std::path::Path;
use sysinfo::{Pid, ProcessExt, System, SystemExt};

const GAME_KEYWORDS: &[&str] = &[
    "game", "launcher", "steam", "epic", "battle.net", "origin", "ea", "ubisoft", "gog", 
    "xbox", "riot", "valorant", "league", "cs2", "csgo", "dota", "apex", "fortnite", 
    "pubg", "warzone", "minecraft", "roblox", "wow", "warcraft", "diablo", "overwatch"
];

const SOCIAL_KEYWORDS: &[&str] = &[
    "discord", "telegram", "whatsapp", "signal", "slack", "teams", "skype", "zoom"
];

const MEDIA_KEYWORDS: &[&str] = &[
    "youtube", "twitch", "spotify", "netflix", "vlc", "mpc", "potplayer"
];

pub async fn scan_apps() -> Result<Vec<crate::models::AppInfo>> {
    let mut apps = Vec::new();
    let mut seen_exes = HashSet::new();

    // 1. Scan registry for installed applications
    scan_uninstall_registry(&mut apps, &mut seen_exes).await?;

    // 2. Add known launchers/paths
    add_known_applications(&mut apps, &mut seen_exes).await?;

    // 3. Categorize
    for app in &mut apps {
        app.category = categorize(&app.name, &app.exe_name);
    }

    // 4. Deduplicate by exe_name
    apps.sort_by(|a, b| a.exe_name.cmp(&b.exe_name));
    apps.dedup_by(|a, b| a.exe_name == b.exe_name);

    Ok(apps)
}

async fn scan_uninstall_registry(
    apps: &mut Vec<crate::models::AppInfo>,
    seen: &mut HashSet<String>
) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        use winreg::enums::*;
        use winreg::RegKey;

        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);

        let paths = [
            r"Software\Microsoft\Windows\CurrentVersion\Uninstall",
            r"Software\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall",
        ];

        for root in [&hkcu, &hklm] {
            for path in &paths {
                if let Ok(key) = root.open_subkey(path) {
                    for subkey_name in key.enum_keys().flatten() {
                        if let Ok(subkey) = key.open_subkey(&subkey_name) {
                            let name: String = subkey.get_value("DisplayName").unwrap_or_default();
                            let icon: String = subkey.get_value("DisplayIcon").unwrap_or_default();
                            let exe: String = subkey.get_value("InstallLocation").unwrap_or_default();
                            let publisher: String = subkey.get_value("Publisher").unwrap_or_default();

                            if name.is_empty() {
                                continue;
                            }

                            // Extract exe name from icon path or try to find exe in install location
                            let exe_name = extract_exe_name(&icon, &exe, &name);

                            if exe_name.is_empty() {
                                continue;
                            }

                            let exe_name_lower = exe_name.to_lowercase();
                            if seen.contains(&exe_name_lower) {
                                continue;
                            }
                            seen.insert(exe_name_lower.clone());

                            apps.push(crate::models::AppInfo {
                                name,
                                exe_name: exe_name_lower,
                                exe_path: exe,
                                icon_path: if icon.is_empty() { None } else { Some(icon) },
                                category: String::new(),
                            });
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn extract_exe_name(icon: &str, install_location: &str, name: &str) -> String {
    // Try icon path first
    if !icon.is_empty() {
        if let Some(exe) = Path::new(icon).file_stem() {
            if let Some(s) = exe.to_str() {
                if s.to_lowercase().ends_with(".exe") {
                    return s.to_string();
                }
            }
        }
    }

    // Try install location
    if !install_location.is_empty() {
        if let Ok(entries) = std::fs::read_dir(install_location) {
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension() {
                    if ext == "exe" {
                        if let Some(stem) = entry.path().file_stem() {
                            return stem.to_string_lossy().to_lowercase().to_string();
                        }
                    }
                }
            }
        }
    }

    // Fallback: sanitize name
    name.to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == ' ' || *c == '-' || *c == '_')
        .collect::<String>()
        .replace(' ', "_")
        + ".exe"
}

async fn add_known_applications(
    apps: &mut Vec<crate::models::AppInfo>,
    seen: &mut HashSet<String>
) -> Result<()> {
    let known = [
        ("Steam", "steam.exe", "games"),
        ("Epic Games Launcher", "EpicGamesLauncher.exe", "games"),
        ("Battle.net", "Battle.net.exe", "games"),
        ("EA App", "EALauncher.exe", "games"),
        ("Ubisoft Connect", "UbisoftConnect.exe", "games"),
        ("GOG Galaxy", "GalaxyClient.exe", "games"),
        ("Xbox", "XboxApp.exe", "games"),
        ("Riot Client", "RiotClientServices.exe", "games"),
        ("Discord", "Discord.exe", "social"),
        ("Telegram", "Telegram.exe", "social"),
        ("Chrome", "chrome.exe", "other"),
        ("Firefox", "firefox.exe", "other"),
        ("Edge", "msedge.exe", "other"),
        ("Steam (Alt)", "steamwebhelper.exe", "games"),
    ];

    for (name, exe, cat) in known {
        let exe_lower = exe.to_lowercase();
        if seen.contains(&exe_lower) {
            continue;
        }
        seen.insert(exe_lower.clone());

        apps.push(crate::models::AppInfo {
            name: name.into(),
            exe_name: exe_lower,
            exe_path: String::new(),
            icon_path: None,
            category: cat.into(),
        });
    }
    Ok(())
}

fn categorize(name: &str, exe: &str) -> String {
    let name_lower = name.to_lowercase();
    let exe_lower = exe.to_lowercase();

    for kw in GAME_KEYWORDS {
        if name_lower.contains(kw) || exe_lower.contains(kw) {
            return "games".into();
        }
    }
    for kw in SOCIAL_KEYWORDS {
        if name_lower.contains(kw) || exe_lower.contains(kw) {
            return "social".into();
        }
    }
    for kw in MEDIA_KEYWORDS {
        if name_lower.contains(kw) || exe_lower.contains(kw) {
            return "media".into();
        }
    }
    "other".into()
}

pub async fn resolve_pids(exe_names: &[String]) -> Result<Vec<u32>> {
    let mut sys = System::new_all();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

    let mut pids = Vec::new();
    let target_names: HashSet<String> = exe_names.iter().map(|s| s.to_lowercase()).collect();

    for (pid, proc_) in sys.processes() {
        let exe_name = proc_.name().to_string_lossy().to_lowercase();
        if target_names.contains(&exe_name) {
            pids.push(pid.as_u32());
        }
    }

    Ok(pids)
}