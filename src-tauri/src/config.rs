use crate::error::Result;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_store::StoreExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub selected_apps: Vec<String>,
    pub provider: String,
    pub provider_config: serde_json::Value,
    pub auto_connect: bool,
    pub minimize_to_tray: bool,
    pub start_minimized: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            selected_apps: Vec::new(),
            provider: "aether".into(),
            provider_config: serde_json::json!({}),
            auto_connect: false,
            minimize_to_tray: true,
            start_minimized: false,
        }
    }
}

pub struct ConfigManager {
    store: Arc<tauri_plugin_store::Store<tauri::Wry>>,
    cache: Arc<Mutex<AppConfig>>,
}

impl ConfigManager {
    pub fn new(app: &AppHandle) -> Result<Self> {
        let store = app.store("config.json")?;
        let cache = store.get("config")
            .and_then(|v| serde_json::from_value(v).ok())
            .unwrap_or_default();
        
        Ok(Self {
            store,
            cache: Arc::new(Mutex::new(cache)),
        })
    }

    pub fn get(&self) -> AppConfig {
        self.cache.lock().clone()
    }

    pub fn set(&self, config: AppConfig) -> Result<()> {
        *self.cache.lock() = config.clone();
        self.store.set("config", serde_json::to_value(config)?);
        self.store.save()?;
        Ok(())
    }

    pub fn update_provider(&self, provider: String, config: serde_json::Value) -> Result<()> {
        let mut cfg = self.get();
        cfg.provider = provider;
        cfg.provider_config = config;
        self.set(cfg)
    }

    pub fn update_selected_apps(&self, apps: Vec<String>) -> Result<()> {
        let mut cfg = self.get();
        cfg.selected_apps = apps;
        self.set(cfg)
    }
}