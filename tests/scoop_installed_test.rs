use nsh_app_sundae::scoop::env::ScoopEnv;
use nsh_app_sundae::scoop::installed::{AppInstallStatus, InstalledManager};
use nsh_app_sundae::scoop::manifest::PackageManifest;
use nsh_app_sundae::scoop::scanner::SearchIndex;
use nsh_app_sundae::scoop::updates::UpdatesManager;
use std::fs;

#[test]
fn test_installed_scanning_and_update_detection() {
    let temp_dir = std::env::temp_dir().join("sundae_test_installed");
    let _ = fs::remove_dir_all(&temp_dir);

    let apps_dir = temp_dir.join("apps");
    fs::create_dir_all(&apps_dir).unwrap();

    // App 1: git (installed v2.40.0, current manifest + install.json with bucket)
    let git_dir = apps_dir.join("git");
    let git_current = git_dir.join("current");
    fs::create_dir_all(&git_current).unwrap();
    fs::write(
        git_current.join("manifest.json"),
        r#"{"version":"2.40.0","description":"Git SCM"}"#,
    )
    .unwrap();
    fs::write(
        git_current.join("install.json"),
        r#"{"bucket":"main","architecture":"64bit"}"#,
    )
    .unwrap();

    // App 2: neovim (installed v0.9.0, is held in scoop-install.json)
    let nvim_dir = apps_dir.join("neovim");
    let nvim_current = nvim_dir.join("current");
    fs::create_dir_all(&nvim_current).unwrap();
    fs::write(
        nvim_current.join("manifest.json"),
        r#"{"version":"0.9.0","description":"Vim fork"}"#,
    )
    .unwrap();
    fs::write(
        nvim_current.join("scoop-install.json"),
        r#"{"hold":true,"bucket":"main"}"#,
    )
    .unwrap();

    // App 3: special dir "scoop" should be skipped
    let scoop_dir = apps_dir.join("scoop");
    fs::create_dir_all(&scoop_dir).unwrap();

    // App 4: broken install without current junction
    let broken_dir = apps_dir.join("broken-tool");
    fs::create_dir_all(&broken_dir).unwrap();

    let env = ScoopEnv::from_root(temp_dir.clone());
    let installed = InstalledManager::scan_installed(&env);

    assert_eq!(installed.len(), 3);

    let git_app = installed
        .iter()
        .find(|a| a.name == "git")
        .expect("git should be found");
    assert_eq!(git_app.version, "2.40.0");
    assert_eq!(git_app.bucket.as_deref(), Some("main"));
    assert_eq!(git_app.status, AppInstallStatus::Installed);
    assert!(!git_app.is_held());
    assert!(!git_app.is_failed());

    let nvim_app = installed
        .iter()
        .find(|a| a.name == "neovim")
        .expect("neovim should be found");
    assert_eq!(nvim_app.version, "0.9.0");
    assert_eq!(nvim_app.bucket.as_deref(), Some("main"));
    assert_eq!(nvim_app.status, AppInstallStatus::Held);
    assert!(nvim_app.is_held());
    assert!(!nvim_app.is_failed());

    let broken_app = installed
        .iter()
        .find(|a| a.name == "broken-tool")
        .expect("broken should be found");
    assert_eq!(broken_app.status, AppInstallStatus::Failed);
    assert!(broken_app.is_failed());

    // Toggle hold flag via pure Rust helper
    assert!(InstalledManager::set_hold_flag(&nvim_current, false).unwrap());
    let rescanned = InstalledManager::scan_installed(&env);
    assert!(!rescanned.iter().find(|a| a.name == "neovim").unwrap().is_held());
    assert!(InstalledManager::set_hold_flag(&nvim_current, true).unwrap());

    // Create a SearchIndex where git has newer version 2.44.0 and neovim has 0.10.0
    let index = SearchIndex::new(vec![
        PackageManifest {
            name: "git".to_string(),
            bucket: "main".to_string(),
            version: "2.44.0".to_string(),
            description: "Git SCM".to_string(),
            homepage: String::new(),
            license: String::new(),
            license_url: None,
            bins: vec![],
            depends: vec![],
            notes: vec![],
        },
        PackageManifest {
            name: "neovim".to_string(),
            bucket: "main".to_string(),
            version: "0.10.0".to_string(),
            description: "Vim fork".to_string(),
            homepage: String::new(),
            license: String::new(),
            license_url: None,
            bins: vec![],
            depends: vec![],
            notes: vec![],
        },
    ]);

    let updates = UpdatesManager::check_updates(&installed, &index);

    // git has update: 2.40.0 -> 2.44.0
    // neovim is held: should NOT be in updates list
    // broken-tool is failed: should NOT be in updates list
    assert_eq!(updates.len(), 1);
    assert_eq!(updates[0].name, "git");
    assert_eq!(updates[0].current_version, "2.40.0");
    assert_eq!(updates[0].latest_version, "2.44.0");
    assert_eq!(updates[0].bucket, "main");

    let _ = fs::remove_dir_all(&temp_dir);
}
