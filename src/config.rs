use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub watchlist: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            watchlist: vec![
                "sh600519".to_string(), // 贵州茅台
                "sz000858".to_string(), // 五粮液
                "sh601318".to_string(), // 中国平安
            ],
        }
    }
}

impl Config {
    pub fn load() -> Self {
        if let Some(path) = Self::get_config_path() {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(config) = serde_json::from_str(&content) {
                        return config;
                    }
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<()> {
        if let Some(path) = Self::get_config_path() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let content = serde_json::to_string_pretty(self)?;
            fs::write(path, content)?;
        }
        Ok(())
    }

    fn get_config_path() -> Option<PathBuf> {
        ProjectDirs::from("com", "stock-tui", "stock-tui")
            .map(|proj_dirs| proj_dirs.config_dir().join("config.json"))
    }
}
