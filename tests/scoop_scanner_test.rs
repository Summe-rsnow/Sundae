use nsh_app_sundae::scoop::env::ScoopEnv;
use nsh_app_sundae::scoop::scanner::BucketScanner;
use std::fs;

#[test]
fn test_search_index_query_and_filtering() {
    let temp_dir = std::env::temp_dir().join("sundae_test_scanner");
    let _ = fs::remove_dir_all(&temp_dir);

    // Create fake bucket structure
    let main_bucket = temp_dir.join("buckets").join("main").join("bucket");
    let extras_bucket = temp_dir.join("buckets").join("extras").join("bucket");
    fs::create_dir_all(&main_bucket).unwrap();
    fs::create_dir_all(&extras_bucket).unwrap();

    // Package 1 in main: git
    fs::write(
        main_bucket.join("git.json"),
        r#"{"version":"2.44.0","description":"Fast distributed version control"}"#,
    ).unwrap();

    // Package 2 in main: github-cli
    fs::write(
        main_bucket.join("gh.json"),
        r#"{"version":"2.45.0","description":"GitHub CLI tool"}"#,
    ).unwrap();

    // Package 3 in extras: vscode
    fs::write(
        extras_bucket.join("vscode.json"),
        r#"{"version":"1.87.0","description":"Visual Studio Code IDE"}"#,
    ).unwrap();

    // Package 4: invalid JSON (must be skipped without crashing)
    fs::write(
        extras_bucket.join("broken.json"),
        r#"{"version": INVALID"#,
    ).unwrap();

    let env = ScoopEnv { root: temp_dir.clone() };
    let index = BucketScanner::scan_buckets(&env);

    assert_eq!(index.total_packages(), 3);

    // Search "git" (matches git and github-cli)
    let results = index.search("git", None);
    assert_eq!(results.len(), 2);
    assert!(results.iter().any(|p| p.name == "git"));
    assert!(results.iter().any(|p| p.name == "gh"));

    // Search "code" with bucket filter "extras"
    let results = index.search("code", Some("extras"));
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "vscode");

    // Search with bucket filter "main" should not return vscode
    let results = index.search("code", Some("main"));
    assert!(results.is_empty());

    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);
}
