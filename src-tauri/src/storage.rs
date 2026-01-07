use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::AppHandle;
use tauri::Manager;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StorageData {
    pub config: serde_json::Value,
    pub play_records: serde_json::Value,
    pub favorites: serde_json::Value,
    pub search_history: serde_json::Value,
    pub skip_configs: serde_json::Value,
}

impl Default for StorageData {
    fn default() -> Self {
        Self {
            config: serde_json::json!({}),
            play_records: serde_json::json!({}),
            favorites: serde_json::json!({}),
            search_history: serde_json::json!([]),
            skip_configs: serde_json::json!({}),
        }
    }
}

pub struct StorageManager {
    data_path: PathBuf,
    data: Mutex<StorageData>,
}

impl StorageManager {
    pub fn new(app_handle: &AppHandle) -> Self {
        let app_dir = app_handle
            .path()
            .app_data_dir()
            .expect("failed to get app data dir");
        if !app_dir.exists() {
            fs::create_dir_all(&app_dir).expect("failed to create app data dir");
        }
        let data_path = app_dir.join("data.json");
        let data = if data_path.exists() {
            let content = fs::read_to_string(&data_path).expect("failed to read storage file");
            serde_json::from_str(&content).unwrap_or_default()
        } else {
            StorageData::default()
        };

        Self {
            data_path,
            data: Mutex::new(data),
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let data = self.data.lock().map_err(|e| e.to_string())?;
        let content = serde_json::to_string_pretty(&*data).map_err(|e| e.to_string())?;
        fs::write(&self.data_path, content).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn get_data(&self) -> Result<StorageData, String> {
        let data = self.data.lock().map_err(|e| e.to_string())?;
        Ok(data.clone())
    }

    pub fn update_config(&self, new_config: serde_json::Value) -> Result<(), String> {
        let mut data = self.data.lock().map_err(|e| e.to_string())?;
        data.config = new_config;
        drop(data);
        self.save()
    }

    pub fn reset_config(&self) -> Result<(), String> {
        let mut data = self.data.lock().map_err(|e| e.to_string())?;
        data.config = serde_json::json!({});
        drop(data);
        self.save()
    }

    pub fn update_play_records(&self, new_records: serde_json::Value) -> Result<(), String> {
        let mut data = self.data.lock().map_err(|e| e.to_string())?;
        data.play_records = new_records;
        drop(data);
        self.save()
    }

    pub fn update_favorites(&self, new_favorites: serde_json::Value) -> Result<(), String> {
        let mut data = self.data.lock().map_err(|e| e.to_string())?;
        data.favorites = new_favorites;
        drop(data);
        self.save()
    }

    pub fn update_search_history(&self, new_history: serde_json::Value) -> Result<(), String> {
        let mut data = self.data.lock().map_err(|e| e.to_string())?;
        data.search_history = new_history;
        drop(data);
        self.save()
    }

    pub fn update_skip_configs(&self, new_configs: serde_json::Value) -> Result<(), String> {
        let mut data = self.data.lock().map_err(|e| e.to_string())?;
        data.skip_configs = new_configs;
        drop(data);
        self.save()
    }
}
