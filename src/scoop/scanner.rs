use crate::scoop::env::ScoopEnv;
use crate::scoop::manifest::PackageManifest;
use rayon::prelude::*;
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct SearchIndex {
    packages: Vec<PackageManifest>,
    buckets: Vec<String>,
}

impl SearchIndex {
    pub fn new(packages: Vec<PackageManifest>) -> Self {
        let mut buckets_set = HashSet::new();
        for p in &packages {
            buckets_set.insert(p.bucket.clone());
        }
        let mut buckets: Vec<String> = buckets_set.into_iter().collect();
        buckets.sort();

        Self { packages, buckets }
    }

    pub fn total_packages(&self) -> usize {
        self.packages.len()
    }

    pub fn buckets(&self) -> &[String] {
        &self.buckets
    }

    pub fn all_packages(&self) -> &[PackageManifest] {
        &self.packages
    }

    pub fn find(&self, name: &str) -> Option<&PackageManifest> {
        self.packages.iter().find(|p| p.name.eq_ignore_ascii_case(name))
    }

    pub fn search<'a>(&'a self, query: &str, bucket_filter: Option<&str>) -> Vec<&'a PackageManifest> {
        let q = query.trim().to_ascii_lowercase();

        let mut matches: Vec<(&'a PackageManifest, u8)> = self
            .packages
            .iter()
            .filter(|p| {
                if let Some(b) = bucket_filter {
                    if !b.is_empty() && !p.bucket.eq_ignore_ascii_case(b) {
                        return false;
                    }
                }
                true
            })
            .filter_map(|p| {
                if q.is_empty() {
                    return Some((p, 0));
                }

                let name_lower = p.name.to_ascii_lowercase();
                if name_lower == q {
                    Some((p, 3)) // exact name match
                } else if name_lower.starts_with(&q) {
                    Some((p, 2)) // name prefix match
                } else if name_lower.contains(&q) {
                    Some((p, 1)) // name substring match
                } else if p.description.to_ascii_lowercase().contains(&q) {
                    Some((p, 0)) // description match
                } else {
                    None
                }
            })
            .collect();

        // Sort by score descending, then by name alphabetically
        matches.sort_by(|a, b| {
            b.1.cmp(&a.1).then_with(|| a.0.name.cmp(&b.0.name))
        });

        matches.into_iter().map(|(p, _)| p).collect()
    }
}

pub struct BucketScanner;

impl BucketScanner {
    pub fn scan_buckets(env: &ScoopEnv) -> SearchIndex {
        let buckets_dir = env.buckets_dir();
        if !buckets_dir.exists() {
            return SearchIndex::default();
        }

        let bucket_entries = match fs::read_dir(&buckets_dir) {
            Ok(entries) => entries,
            Err(e) => {
                log::warn!("Failed to read buckets dir {:?}: {}", buckets_dir, e);
                return SearchIndex::default();
            }
        };

        let mut files_to_scan: Vec<(String, String, PathBuf)> = Vec::new();

        for entry in bucket_entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let bucket_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                // Scoop manifests live in <bucket>/bucket/*.json or root *.json in simple buckets
                let manifest_dir = path.join("bucket");
                let target_dir = if manifest_dir.exists() {
                    manifest_dir
                } else {
                    path
                };

                if let Ok(manifest_entries) = fs::read_dir(target_dir) {
                    for m_entry in manifest_entries.flatten() {
                        let m_path = m_entry.path();
                        if m_path.extension().and_then(|ext| ext.to_str()) == Some("json") {
                            if let Some(stem) = m_path.file_stem().and_then(|s| s.to_str()) {
                                files_to_scan.push((stem.to_string(), bucket_name.clone(), m_path));
                            }
                        }
                    }
                }
            }
        }

        // Parallel parse JSON manifests
        let packages: Vec<PackageManifest> = files_to_scan
            .into_par_iter()
            .filter_map(|(name, bucket, path)| {
                match fs::read_to_string(&path) {
                    Ok(content) => match PackageManifest::from_json(&name, &bucket, &content) {
                        Ok(pkg) => Some(pkg),
                        Err(e) => {
                            log::debug!("Skipping malformed manifest at {:?}: {}", path, e);
                            None
                        }
                    },
                    Err(e) => {
                        log::debug!("Failed reading manifest {:?}: {}", path, e);
                        None
                    }
                }
            })
            .collect();

        SearchIndex::new(packages)
    }
}
