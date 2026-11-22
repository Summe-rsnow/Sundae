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
        Self::with_buckets(packages, Vec::new())
    }

    pub fn with_buckets(mut packages: Vec<PackageManifest>, extra_buckets: Vec<String>) -> Self {
        let mut buckets_set: HashSet<String> = extra_buckets.into_iter().collect();
        for p in &packages {
            if !p.bucket.is_empty() {
                buckets_set.insert(p.bucket.clone());
            }
        }
        let mut buckets: Vec<String> = buckets_set.into_iter().collect();
        buckets.sort_by(|a, b| a.to_ascii_lowercase().cmp(&b.to_ascii_lowercase()));

        packages.sort_by(|a, b| {
            a.name
                .to_ascii_lowercase()
                .cmp(&b.name.to_ascii_lowercase())
                .then_with(|| a.bucket.cmp(&b.bucket))
        });

        Self { packages, buckets }
    }

    pub fn total_packages(&self) -> usize {
        self.packages.len()
    }

    pub fn buckets(&self) -> &[String] {
        &self.buckets
    }

    pub fn bucket_package_count(&self, bucket: &str) -> usize {
        self.packages
            .iter()
            .filter(|p| p.bucket.eq_ignore_ascii_case(bucket))
            .count()
    }

    pub fn all_packages(&self) -> &[PackageManifest] {
        &self.packages
    }

    pub fn find(&self, name: &str) -> Option<&PackageManifest> {
        self.packages
            .iter()
            .find(|p| p.name.eq_ignore_ascii_case(name))
    }

    pub fn find_in_bucket(&self, name: &str, bucket: Option<&str>) -> Option<&PackageManifest> {
        if let Some(b) = bucket.filter(|b| !b.is_empty()) {
            if let Some(found) = self
                .packages
                .iter()
                .find(|p| p.name.eq_ignore_ascii_case(name) && p.bucket.eq_ignore_ascii_case(b))
            {
                return Some(found);
            }
        }
        self.find(name)
    }

    pub fn search<'a>(
        &'a self,
        query: &str,
        bucket_filter: Option<&str>,
    ) -> Vec<&'a PackageManifest> {
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
                    Some((p, 4)) // exact name match
                } else if name_lower.starts_with(&q) {
                    Some((p, 3)) // name prefix match
                } else if name_lower.contains(&q) {
                    Some((p, 2)) // name substring match
                } else if p.bins.iter().any(|b| b.to_ascii_lowercase().contains(&q)) {
                    Some((p, 1)) // executable/shim name match
                } else if p.description.to_ascii_lowercase().contains(&q) {
                    Some((p, 0)) // description match
                } else {
                    None
                }
            })
            .collect();

        // Sort by relevance descending, then by name alphabetically (case-insensitive)
        matches.sort_by(|a, b| {
            b.1.cmp(&a.1).then_with(|| {
                a.0.name
                    .to_ascii_lowercase()
                    .cmp(&b.0.name.to_ascii_lowercase())
            })
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

        let mut bucket_names: Vec<String> = Vec::new();
        let mut files_to_scan: Vec<(String, String, PathBuf)> = Vec::new();

        for entry in bucket_entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let bucket_name = match path.file_name().and_then(|n| n.to_str()) {
                    Some(name) if !name.starts_with('.') => name.to_string(),
                    _ => continue,
                };
                bucket_names.push(bucket_name.clone());

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
                                files_to_scan.push((
                                    stem.to_string(),
                                    bucket_name.clone(),
                                    m_path,
                                ));
                            }
                        }
                    }
                }
            }
        }

        // Parallel parse JSON manifests off the UI thread via Rayon
        let packages: Vec<PackageManifest> = files_to_scan
            .into_par_iter()
            .filter_map(|(name, bucket, path)| match fs::read_to_string(&path) {
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
            })
            .collect();

        SearchIndex::with_buckets(packages, bucket_names)
    }
}
