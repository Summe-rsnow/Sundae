use crate::scoop::env::ScoopEnv;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

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

        let mut installed = Vec::new();

        for entry in entries.flatten() {
            let app_path = entry.path();
            if !app_path.is_dir() {
                continue;
            }

            let app_name = match app_path.file_name().and_then(|n| n.to_str()) {
                Some(name) => {
                    if name.eq_ignore_ascii_case("scoop") {
                        continue; // Skip scoop self package
                    }
                    name.to_string()
                }
                None => continue,
            };

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

            // Check hold status: check scoop-install.json and install.json
            let mut is_held = app_path.join(".hold").exists()
                || current_dir.join(".hold").exists();

            for dir in &target_dirs {
                for name in &["scoop-install.json", "install.json"] {
                    let p = dir.join(name);
                    if p.exists() {
                        if let Ok(content) = fs::read_to_string(&p) {
                            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                                if val.get("hold").and_then(|h| h.as_bool()) == Some(true) {
                                    is_held = true;
                                    break;
                                }
                            }
                        }
                    }
                }
                if is_held {
                    break;
                }
            }

            // Check failed status: if current does not exist and no manifest was found, or version directory is empty
            let is_failed = (!current_dir.exists() && found_version == "Unknown")
                || (!current_dir.exists() && Self::is_dir_empty(&app_path));

            let status = if is_failed {
                AppInstallStatus::Failed
            } else if is_held {
                AppInstallStatus::Held
            } else {
                AppInstallStatus::Installed
            };

            let version = if is_failed && found_version == "Unknown" {
                "安装失败".to_string()
            } else {
                found_version
            };

            let manifest_path = found_manifest;
            installed.push(InstalledApp {
                name: app_name,
                version,
                status,
                path: if current_dir.exists() { current_dir } else { app_path },
                manifest_path,
            });
        }

        installed.sort_by(|a, b| a.name.to_ascii_lowercase().cmp(&b.name.to_ascii_lowercase()));
        installed
    }

    fn is_dir_empty(path: &PathBuf) -> bool {
        if let Ok(mut entries) = fs::read_dir(path) {
            entries.next().is_none()
        } else {
            true
        }
    }

    fn extract_version(manifest_path: &PathBuf) -> Option<String> {
        let content = fs::read_to_string(manifest_path).ok()?;
        let parsed: MinimalManifest = serde_json::from_str(&content).ok()?;
        parsed.version
    }
}
