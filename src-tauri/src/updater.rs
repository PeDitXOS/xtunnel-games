use serde::{Deserialize, Serialize};
use tauri::AppHandle;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub version: String,
    pub current_version: String,
    pub has_update: bool,
    pub notes: String,
    pub download_url: String,
    pub published_at: String,
}

#[tauri::command]
pub async fn check_updates(app: AppHandle) -> std::result::Result<UpdateInfo, String> {
    let current_version = env!("CARGO_PKG_VERSION");
    
    let client = reqwest::Client::new();
    let response = client
        .get("https://api.github.com/repos/peditxos/xtunnel-games/releases/latest")
        .header("User-Agent", "XtunnelGames")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Ok(UpdateInfo {
            version: current_version.to_string(),
            current_version: current_version.to_string(),
            has_update: false,
            notes: "Could not check for updates".into(),
            download_url: "https://github.com/peditxos/xtunnel-games/releases".into(),
            published_at: chrono::Utc::now().to_rfc3339(),
        });
    }

    #[derive(Deserialize)]
    struct GitHubRelease {
        tag_name: String,
        body: Option<String>,
        html_url: String,
        published_at: String,
        assets: Vec<GitHubAsset>,
    }

    #[derive(Deserialize)]
    struct GitHubAsset {
        name: String,
        browser_download_url: String,
    }

    let release: GitHubRelease = response.json().await
        .map_err(|e| e.to_string())?;

    let latest_version = release.tag_name.trim_start_matches('v');
    let has_update = version_compare(latest_version, current_version).unwrap_or(false);

    let download_url = release.assets.iter()
        .find(|a| a.name.ends_with(".exe") || a.name.ends_with(".msi"))
        .map(|a| a.browser_download_url.clone())
        .unwrap_or(release.html_url.clone());

    Ok(UpdateInfo {
        version: latest_version.to_string(),
        current_version: current_version.to_string(),
        has_update,
        notes: release.body.unwrap_or("No release notes".into()),
        download_url,
        published_at: release.published_at,
    })
}

fn version_compare(a: &str, b: &str) -> std::result::Result<bool, Box<dyn std::error::Error>> {
    let a_parts: Vec<u32> = a.split('.').map(|s| s.parse().unwrap_or(0)).collect();
    let b_parts: Vec<u32> = b.split('.').map(|s| s.parse().unwrap_or(0)).collect();
    
    let max_len = a_parts.len().max(b_parts.len());
    for i in 0..max_len {
        let a_val = a_parts.get(i).copied().unwrap_or(0);
        let b_val = b_parts.get(i).copied().unwrap_or(0);
        if a_val > b_val {
            return Ok(true);
        } else if a_val < b_val {
            return Ok(false);
        }
    }
    Ok(false)
}
