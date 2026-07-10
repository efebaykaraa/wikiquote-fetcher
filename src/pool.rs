use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct QuotePool {
    pub key: String,
    pub quotes: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct QuotePoolStore {
    dir: PathBuf,
}

impl QuotePoolStore {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self { dir: dir.into() }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn pool_path(&self, key: &str) -> PathBuf {
        let _ = std::fs::create_dir_all(&self.dir);
        self.dir.join(format!("{}.json", sanitize_key(key)))
    }

    pub fn load(&self, key: &str) -> Option<QuotePool> {
        let content = std::fs::read_to_string(self.pool_path(key)).ok()?;
        serde_json::from_str(&content).ok()
    }

    pub fn save(&self, pool: &QuotePool) -> anyhow::Result<()> {
        let path = self.pool_path(&pool.key);
        let content = serde_json::to_string_pretty(pool)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    pub fn clear(&self, key: &str) -> anyhow::Result<()> {
        match std::fs::remove_file(self.pool_path(key)) {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(err.into()),
        }
    }
}

fn sanitize_key(key: &str) -> String {
    key.chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            } else {
                '_'
            }
        })
        .collect()
}
