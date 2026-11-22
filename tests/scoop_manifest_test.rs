use nsh_app_sundae::scoop::env::ScoopEnv;
use nsh_app_sundae::scoop::manifest::PackageManifest;

#[test]
fn test_scoop_env_detect() {
    let env = ScoopEnv::detect();
    assert!(!env.root.as_os_str().is_empty());
    assert_eq!(env.buckets_dir(), env.root.join("buckets"));
    assert_eq!(env.apps_dir(), env.root.join("apps"));
    assert_eq!(env.shims_dir(), env.root.join("shims"));
    assert_eq!(env.persist_dir(), env.root.join("persist"));
}

#[test]
fn test_manifest_parse_full() {
    let json = r#"{
        "version": "2.4.0",
        "description": "High performance tool",
        "homepage": "https://example.com/tool",
        "license": {
            "identifier": "MIT",
            "url": "https://opensource.org/licenses/MIT"
        },
        "bin": "tool.exe",
        "depends": ["curl", "git"],
        "notes": ["Run tool --init first", "Restart terminal after install"],
        "suggest": { "optional": "tool-opt" }
    }"#;

    let manifest = PackageManifest::from_json("my-tool", "main", json).expect("parse failed");
    assert_eq!(manifest.name, "my-tool");
    assert_eq!(manifest.bucket, "main");
    assert_eq!(manifest.version, "2.4.0");
    assert_eq!(manifest.description, "High performance tool");
    assert_eq!(manifest.homepage, "https://example.com/tool");
    assert_eq!(manifest.license, "MIT");
    assert_eq!(
        manifest.license_url.as_deref(),
        Some("https://opensource.org/licenses/MIT")
    );
    assert_eq!(manifest.bins, vec!["tool.exe"]);
    assert_eq!(manifest.depends, vec!["curl", "git"]);
    assert_eq!(
        manifest.notes,
        vec!["Run tool --init first", "Restart terminal after install"]
    );
    assert_eq!(manifest.install_command(), "scoop install main/my-tool");
}

#[test]
fn test_manifest_parse_array_bins_and_missing_optional() {
    let json = r#"{
        "version": "1.0",
        "bin": [
            "tool.exe",
            ["tool.exe", "alias-tool"]
        ],
        "notes": "Single line note"
    }"#;

    let manifest = PackageManifest::from_json("minimal", "extras", json).expect("parse failed");
    assert_eq!(manifest.name, "minimal");
    assert_eq!(manifest.bucket, "extras");
    assert_eq!(manifest.version, "1.0");
    assert_eq!(manifest.description, "");
    assert_eq!(manifest.display_description(), "暂无描述");
    assert_eq!(manifest.homepage, "");
    assert_eq!(manifest.license, "Unknown");
    assert_eq!(manifest.license_url, None);
    assert_eq!(manifest.bins, vec!["tool.exe", "alias-tool"]);
    assert!(manifest.depends.is_empty());
    assert_eq!(manifest.notes, vec!["Single line note"]);
}

#[test]
fn test_manifest_parse_invalid_json() {
    let result = PackageManifest::from_json("bad", "main", "{ not valid json");
    assert!(result.is_err());
}
