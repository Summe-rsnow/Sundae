use crate::scoop::installed::InstalledApp;
use crate::scoop::scanner::SearchIndex;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppUpdate {
    pub name: String,
    pub current_version: String,
    pub latest_version: String,
    pub bucket: String,
}

pub struct UpdatesManager;

impl UpdatesManager {
    pub fn check_updates(installed: &[InstalledApp], index: &SearchIndex) -> Vec<AppUpdate> {
        let mut updates = Vec::new();

        for app in installed {
            if app.is_held {
                continue;
            }

            if let Some(manifest) = index.find(&app.name) {
                if manifest.version != "Unknown"
                    && app.version != "Unknown"
                    && manifest.version != app.version
                {
                    updates.push(AppUpdate {
                        name: app.name.clone(),
                        current_version: app.version.clone(),
                        latest_version: manifest.version.clone(),
                        bucket: manifest.bucket.clone(),
                    });
                }
            }
        }

        updates.sort_by(|a, b| a.name.to_ascii_lowercase().cmp(&b.name.to_ascii_lowercase()));
        updates
    }
}
