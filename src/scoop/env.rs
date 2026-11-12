use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoopEnv {
    pub root: PathBuf,
}

impl ScoopEnv {
    pub fn detect() -> Self {
        let root = if let Some(scoop_env) = std::env::var_os("SCOOP") {
            PathBuf::from(scoop_env)
        } else if let Some(home) = dirs::home_dir() {
            home.join("scoop")
        } else {
            PathBuf::from(r"C:\scoop")
        };

        Self { root }
    }

    pub fn is_installed(&self) -> bool {
        self.root.exists() && (self.apps_dir().exists() || self.buckets_dir().exists())
    }

    pub fn buckets_dir(&self) -> PathBuf {
        self.root.join("buckets")
    }

    pub fn apps_dir(&self) -> PathBuf {
        self.root.join("apps")
    }

    pub fn shims_dir(&self) -> PathBuf {
        self.root.join("shims")
    }
}
