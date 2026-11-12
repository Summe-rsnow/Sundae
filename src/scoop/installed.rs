use crate::scoop::env::ScoopEnv;
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstalledApp {
    pub name: String,
    pub version: String,
    pub is_held: bool,
    pub path: PathBuf,
    pub manifest_path: PathBuf,
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

            // Check hold status: .hold file in app dir or current dir
            let is_held = app_path.join(".hold").exists()
                || app_path.join("current").join(".hold").exists();

            // Try current/manifest.json
            let current_dir = app_path.join("current");
            let manifest_path = current_dir.join("manifest.json");

            let version = if manifest_path.exists() {
                Self::extract_version(&manifest_path).unwrap_or_else(|| "Unknown".to_string())
            } else {
                // Fallback: search any version directory under app_path
                let mut found_ver = "Unknown".to_string();
                if let Ok(ver_entries) = fs::read_dir(&app_path) {
                    for v_entry in ver_entries.flatten() {
                        let v_path = v_entry.path();
                        if v_path.is_dir() {
                            let v_manifest = v_path.join("manifest.json");
                            if v_manifest.exists() {
                                if let Some(v) = Self::extract_version(&v_manifest) {
                                    found_ver = v;
                                    break;
                                }
                            }
                        }
                    }
                }
                found_ver
            };

            installed.push(InstalledApp {
                name: app_name,
                version,
                is_held,
                path: if current_dir.exists() { current_dir } else { app_path },
                manifest_path,
            });
        }

        installed.sort_by(|a, b| a.name.to_ascii_lowercase().cmp(&b.name.to_ascii_lowercase()));
        installed
    }

    fn extract_version(manifest_path: &PathBuf) -> Option<String> {
        let content = fs::read_to_string(manifest_path).ok()?;
        let parsed: MinimalManifest = serde_json::from_str(&content).ok()?;
        parsed.version
    }
}
