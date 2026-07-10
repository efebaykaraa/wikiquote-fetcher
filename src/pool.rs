use engyls::config::ConfigManager;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct QuotePool {
    pub position_hash: String,
    pub quotes: Vec<String>,
}

impl QuotePool {
    pub fn pool_path(author: &str) -> PathBuf {
        let dir = ConfigManager::config_dir().join("pools");
        let _ = std::fs::create_dir_all(&dir);
        dir.join(format!("{}.json", author.replace(' ', "_")))
    }

    pub fn load(author: &str) -> Option<Self> {
        let path = Self::pool_path(author);
        let content = std::fs::read_to_string(path).ok()?;
        serde_json::from_str(&content).ok()
    }

    pub fn save(&self, author: &str) -> anyhow::Result<()> {
        let path = Self::pool_path(author);
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
