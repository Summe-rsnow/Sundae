use nsh_app_sundae::scoop::config::{ScoopConfig, SundaePrefs};
use std::fs;

#[test]
fn test_scoop_config_parse_and_serialize_preserves_extra() {
    let json = r#"{
        "proxy": "127.0.0.1:7890",
        "gh_token": "ghp_secret123",
        "aria2-enabled": true,
        "aria2-split": 8,
        "scoop_repo": "https://gitee.com/scoop-installer/scoop",
        "scoop_branch": "develop",
        "use_sqlite_cache": true,
        "last_update": "2026-11-22T00:00:00Z",
        "root_path": "D:\\Scoop"
    }"#;

    let cfg = ScoopConfig::from_json(json).expect("parse config");
    assert_eq!(cfg.proxy, Some("127.0.0.1:7890".to_string()));
    assert_eq!(cfg.gh_token, Some("ghp_secret123".to_string()));
    assert!(cfg.aria2_enabled);
    assert_eq!(cfg.aria2_split, 8);
    assert_eq!(cfg.scoop_branch, "develop");
    assert!(cfg.use_sqlite_cache);
    assert_eq!(
        cfg.extra.get("last_update").and_then(|v| v.as_str()),
        Some("2026-11-22T00:00:00Z")
    );

    let serialized = cfg.to_json_pretty().expect("serialize config");
    assert!(serialized.contains("127.0.0.1:7890"));
    assert!(serialized.contains("aria2-enabled"));
    assert!(serialized.contains("last_update"));
    assert!(serialized.contains("root_path"));
}

#[test]
fn test_sundae_prefs_round_trip() {
    let temp_dir = std::env::temp_dir().join("sundae_test_prefs");
    let _ = fs::remove_dir_all(&temp_dir);
    let path = temp_dir.join("prefs.json");

    // Missing file returns sensible defaults
    let default_prefs = SundaePrefs::load_from(&path);
    assert_eq!(default_prefs.theme_mode, "system");
    assert_eq!(default_prefs.close_behavior, "ask");

    // Save custom theme_mode & close_behavior
    let prefs = SundaePrefs {
        theme_mode: "dark".to_string(),
        close_behavior: "tray".to_string(),
    };
    prefs.save_to(&path).expect("save prefs");

    let reloaded = SundaePrefs::load_from(&path);
    assert_eq!(reloaded, prefs);

    let _ = fs::remove_dir_all(&temp_dir);
}
