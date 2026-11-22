use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::fs;
use std::path::{Path, PathBuf};

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

    /// Preserves all other Scoop-managed keys (e.g. `last_update`, `scoop_branch`, `root_path`)
    /// so saving from Sundae never discards user or CLI configuration.
    #[serde(flatten, default)]
    pub extra: Map<String, Value>,
}

fn default_split() -> u32 {
    5
}
fn default_retry_wait() -> u32 {
    2
}
fn default_repo() -> String {
    "https://github.com/ScoopInstaller/Scoop".to_string()
}
fn default_branch() -> String {
    "master".to_string()
}

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
            extra: Map::new(),
        }
    }
}

impl ScoopConfig {
    pub fn config_path() -> Option<PathBuf> {
        let home = dirs::home_dir()?;
        Some(home.join(".config").join("scoop").join("config.json"))
    }

    pub fn load() -> Self {
        Self::config_path()
            .map(|p| Self::load_from(&p))
            .unwrap_or_default()
    }

    pub fn load_from(path: &Path) -> Self {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(cfg) = Self::from_json(&content) {
                    return cfg;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        if let Some(path) = Self::config_path() {
            self.save_to(&path)?;
        }
        Ok(())
    }

    pub fn save_to(&self, path: &Path) -> Result<(), std::io::Error> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = self
            .to_json_pretty()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        fs::write(path, json)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

/// Sundae desktop UI preferences stored separately from Scoop's own `config.json`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SundaePrefs {
    /// `"system"` | `"light"` | `"dark"`
    #[serde(default = "default_theme_mode")]
    pub theme_mode: String,

    /// `"ask"` | `"tray"` | `"quit"`
    #[serde(default = "default_close_behavior")]
    pub close_behavior: String,
}

fn default_theme_mode() -> String {
    "system".to_string()
}

fn default_close_behavior() -> String {
    "ask".to_string()
}

impl Default for SundaePrefs {
    fn default() -> Self {
        Self {
            theme_mode: default_theme_mode(),
            close_behavior: default_close_behavior(),
        }
    }
}

impl SundaePrefs {
    pub fn prefs_path() -> Option<PathBuf> {
        let home = dirs::home_dir()?;
        Some(home.join(".config").join("sundae").join("prefs.json"))
    }

    pub fn load() -> Self {
        Self::prefs_path()
            .map(|p| Self::load_from(&p))
            .unwrap_or_default()
    }

    pub fn load_from(path: &Path) -> Self {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(path) {
                if let Ok(prefs) = Self::from_json(&content) {
                    return prefs;
                }
            }
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        if let Some(path) = Self::prefs_path() {
            self.save_to(&path)?;
        }
        Ok(())
    }

    pub fn save_to(&self, path: &Path) -> Result<(), std::io::Error> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let json = self
            .to_json_pretty()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        fs::write(path, json)
    }

    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    pub fn to_json_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}
