use nsh_app_sundae::scoop::config::ScoopConfig;

#[test]
fn test_scoop_config_parse_and_serialize() {
    let json = r#"{
        "proxy": "127.0.0.1:7890",
        "gh_token": "ghp_secret123",
        "aria2-enabled": true,
        "aria2-split": 8,
        "scoop_repo": "https://gitee.com/scoop-installer/scoop",
        "scoop_branch": "develop",
        "use_sqlite_cache": true
    }"#;

    let cfg = ScoopConfig::from_json(json).expect("parse config");
    assert_eq!(cfg.proxy, Some("127.0.0.1:7890".to_string()));
    assert_eq!(cfg.gh_token, Some("ghp_secret123".to_string()));
    assert!(cfg.aria2_enabled);
    assert_eq!(cfg.aria2_split, 8);
    assert_eq!(cfg.scoop_branch, "develop");
    assert!(cfg.use_sqlite_cache);

    let serialized = cfg.to_json_pretty().expect("serialize config");
    assert!(serialized.contains("127.0.0.1:7890"));
    assert!(serialized.contains("aria2-enabled"));
}
