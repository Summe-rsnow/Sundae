use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScoopEnv {
    pub root: PathBuf,
}

impl ScoopEnv {
    pub fn detect() -> Self {
        let root = if let Some(scoop_env) = std::env::var_os("SCOOP") {
            let p = PathBuf::from(scoop_env);
            if !p.as_os_str().is_empty() {
                p
            } else {
                Self::default_root()
            }
        } else {
            Self::default_root()
        };

        Self { root }
    }

    pub fn from_root(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    fn default_root() -> PathBuf {
        if let Some(home) = dirs::home_dir() {
            home.join("scoop")
        } else {
            PathBuf::from(r"C:\scoop")
        }
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

    pub fn persist_dir(&self) -> PathBuf {
        self.root.join("persist")
    }
}
