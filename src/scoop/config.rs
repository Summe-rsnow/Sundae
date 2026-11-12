use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScoopConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxy: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub gh_token: Option<String>,

    #[serde(rename = "aria2-enabled", default)]
    pub aria2_enabled: bool,

    #[serde(rename = "aria2-split", default = "default_split")]
    pub aria2_split: u32,

    #[serde(rename = "aria2-max-connection-per-server", default = "default_split")]
    pub aria2_max_conn: u32,

    #[serde(rename = "aria2-retry-wait", default = "default_retry_wait")]
    pub aria2_retry_wait: u32,

    #[serde(default = "default_repo")]
    pub scoop_repo: String,

    #[serde(default = "default_branch")]
    pub scoop_branch: String,

    #[serde(default)]
    pub use_sqlite_cache: bool,

    #[serde(default)]
    pub ignore_running_processes: bool,
}

fn default_split() -> u32 { 5 }
fn default_retry_wait() -> u32 { 2 }
fn default_repo() -> String { "https://github.com/ScoopInstaller/Scoop".to_string() }
fn default_branch() -> String { "master".to_string() }

impl Default for ScoopConfig {
    fn default() -> Self {
        Self {
            proxy: None,
            gh_token: None,
            aria2_enabled: true,
            aria2_split: 5,
            aria2_max_conn: 5,
            aria2_retry_wait: 2,
            scoop_repo: default_repo(),
            scoop_branch: default_branch(),
            use_sqlite_cache: false,
            ignore_running_processes: false,
        }
    }
}

impl ScoopConfig {
    pub fn config_path() -> Option<PathBuf> {
        let home = dirs::home_dir()?;
        Some(home.join(".config").join("scoop").join("config.json"))
    }

    pub fn load() -> Self {
        if let Some(path) = Self::config_path() {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(cfg) = Self::from_json(&content) {
                        return cfg;
                    }
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        if let Some(path) = Self::config_path() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let json = self.to_json_pretty().map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
            fs::write(path, json)?;
        }
        Ok(())
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}
