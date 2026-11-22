use crate::scoop::env::ScoopEnv;
use rayon::prelude::*;
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppInstallStatus {
    Installed,
    Held,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstalledApp {
    pub name: String,
    pub version: String,
    pub bucket: Option<String>,
    pub status: AppInstallStatus,
    pub path: PathBuf,
    pub manifest_path: PathBuf,
}

impl InstalledApp {
    pub fn is_held(&self) -> bool {
        self.status == AppInstallStatus::Held
    }

    pub fn is_failed(&self) -> bool {
        self.status == AppInstallStatus::Failed
    }
}

#[derive(Deserialize)]
struct MinimalManifest {
    version: Option<String>,
}

pub struct InstalledManager;

impl InstalledManager {
    pub fn scan_installed(env: &ScoopEnv) -> Vec<InstalledApp> {
        let apps_dir = env.apps_dir();
        if !apps_dir.exists() {
            return Vec::new();
        }

        let entries = match fs::read_dir(&apps_dir) {
            Ok(e) => e,
            Err(_) => return Vec::new(),
        };

        let app_paths: Vec<(String, PathBuf)> = entries
            .flatten()
            .filter_map(|entry| {
                let app_path = entry.path();
                if !app_path.is_dir() {
                    return None;
                }
                let name = app_path.file_name().and_then(|n| n.to_str())?;
                if name.eq_ignore_ascii_case("scoop") || name.starts_with('.') {
                    return None;
                }
                Some((name.to_string(), app_path))
            })
            .collect();

        let mut installed: Vec<InstalledApp> = app_paths
            .into_par_iter()
            .map(|(app_name, app_path)| Self::inspect_app(app_name, app_path))
            .collect();

        installed.sort_by(|a, b| {
            a.name
                .to_ascii_lowercase()
                .cmp(&b.name.to_ascii_lowercase())
        });
        installed
    }

    fn inspect_app(app_name: String, app_path: PathBuf) -> InstalledApp {
        let current_dir = app_path.join("current");
        let target_dirs = if current_dir.exists() {
            let mut dirs = vec![current_dir.clone()];
            if let Ok(entries) = fs::read_dir(&app_path) {
                for e in entries.flatten() {
                    let p = e.path();
                    if p.is_dir() && p != current_dir {
                        dirs.push(p);
                    }
                }
            }
            dirs
        } else {
            let mut dirs = Vec::new();
            if let Ok(entries) = fs::read_dir(&app_path) {
                for e in entries.flatten() {
                    let p = e.path();
                    if p.is_dir() {
                        dirs.push(p);
                    }
                }
            }
            if dirs.is_empty() {
                dirs.push(app_path.clone());
            }
            dirs
        };

        let manifest_names = [
            "manifest.json",
            "scoop-manifest.json",
            "install.json",
            "scoop-install.json",
        ];

        let mut found_version = "Unknown".to_string();
        let mut found_manifest = current_dir.join("manifest.json");

        for dir in &target_dirs {
            for m_name in &manifest_names {
                let m_path = dir.join(m_name);
                if m_path.exists() {
                    found_manifest = m_path.clone();
                    if let Some(v) = Self::extract_version(&m_path) {
                        found_version = v;
                        break;
                    }
                }
            }
            if found_version != "Unknown" {
                break;
            }
        }

        // Fallback: directory name of version folder (e.g. apps/ghidra/12.2-20261016)
        if found_version == "Unknown" {
            for dir in &target_dirs {
                if let Some(dirname) = dir.file_name().and_then(|n| n.to_str()) {
                    let lower = dirname.to_ascii_lowercase();
                    if lower != "current" && lower != "cache" && lower != ".git" {
                        found_version = dirname.to_string();
                        break;
                    }
                }
            }
        }

        // Check hold status & bucket from scoop-install.json / install.json
        let mut is_held = app_path.join(".hold").exists() || current_dir.join(".hold").exists();
        let mut bucket: Option<String> = None;

        for dir in &target_dirs {
            for name in &["scoop-install.json", "install.json"] {
                let p = dir.join(name);
                if p.exists() {
                    if let Ok(content) = fs::read_to_string(&p) {
                        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                            if val.get("hold").and_then(|h| h.as_bool()) == Some(true) {
                                is_held = true;
                            }
                            if bucket.is_none() {
                                bucket = val
                                    .get("bucket")
                                    .and_then(|b| b.as_str())
                                    .filter(|s| !s.is_empty())
                                    .map(str::to_owned);
                            }
                        }
                    }
                }
            }
        }

        let has_valid_manifest = target_dirs
            .iter()
            .any(|d| manifest_names.iter().any(|m| d.join(m).exists()));

        let is_failed = !current_dir.exists() || !has_valid_manifest;

        let status = if is_failed {
            AppInstallStatus::Failed
        } else if is_held {
            AppInstallStatus::Held
        } else {
            AppInstallStatus::Installed
        };

        let version = if is_failed {
            if found_version != "Unknown" {
                format!("{} (安装未完成)", found_version)
            } else {
                "安装未完成".to_string()
            }
        } else {
            found_version
        };

        InstalledApp {
            name: app_name,
            version,
            bucket,
            status,
            path: if current_dir.exists() {
                current_dir
            } else {
                app_path
            },
            manifest_path: found_manifest,
        }
    }

    fn extract_version(manifest_path: &Path) -> Option<String> {
        let content = fs::read_to_string(manifest_path).ok()?;
        let parsed: MinimalManifest = serde_json::from_str(&content).ok()?;
        parsed.version
    }

    /// Updates `hold` inside local `scoop-install.json` / `install.json` for immediate feedback.
    pub fn set_hold_flag(app_path: &Path, hold: bool) -> std::io::Result<bool> {
        let mut updated = false;
        for fname in &["scoop-install.json", "install.json"] {
            let p = app_path.join(fname);
            if p.exists() {
                if let Ok(content) = fs::read_to_string(&p) {
                    if let Ok(mut val) = serde_json::from_str::<serde_json::Value>(&content) {
                        if let Some(obj) = val.as_object_mut() {
                            if hold {
                                obj.insert("hold".to_string(), serde_json::Value::Bool(true));
                            } else {
                                obj.remove("hold");
                            }
                            if let Ok(serialized) = serde_json::to_string_pretty(&obj) {
                                fs::write(&p, serialized)?;
                                updated = true;
                            }
                        }
                    }
                }
            }
        }
        Ok(updated)
    }
}

/// Opens a local directory in the system file manager.
/// Uses OS commands instead of `nsh_core::open_url` (which rejects Windows drive-letter paths in NSH 0.6.1).
pub fn open_directory(path: &Path) -> std::io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        Command::new("explorer").arg(path).spawn().map(|_| ())
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(path).spawn().map(|_| ())
    }
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open").arg(path).spawn().map(|_| ())
    }
}

/// Opens a local file (e.g. `manifest.json`) in the default editor/viewer.
pub fn open_local_file(path: &Path) -> std::io::Result<()> {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        Command::new("cmd")
            .args(["/c", "start", "", &path.to_string_lossy()])
            .creation_flags(0x0800_0000)
            .spawn()
            .map(|_| ())
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open").arg(path).spawn().map(|_| ())
    }
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open").arg(path).spawn().map(|_| ())
    }
}
