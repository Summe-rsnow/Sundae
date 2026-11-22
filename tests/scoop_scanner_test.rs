use nsh_app_sundae::scoop::env::ScoopEnv;
use nsh_app_sundae::scoop::scanner::BucketScanner;
use std::fs;

#[test]
fn test_search_index_query_and_filtering() {
    let temp_dir = std::env::temp_dir().join("sundae_test_scanner");
    let _ = fs::remove_dir_all(&temp_dir);

    // Create fake bucket structure including an empty bucket
    let main_bucket = temp_dir.join("buckets").join("main").join("bucket");
    let extras_bucket = temp_dir.join("buckets").join("extras").join("bucket");
    let empty_bucket = temp_dir.join("buckets").join("empty-custom").join("bucket");
    fs::create_dir_all(&main_bucket).unwrap();
    fs::create_dir_all(&extras_bucket).unwrap();
    fs::create_dir_all(&empty_bucket).unwrap();

    // Package 1 in main: git
    fs::write(
        main_bucket.join("git.json"),
        r#"{"version":"2.44.0","description":"Fast distributed version control","bin":"git.exe"}"#,
    )
    .unwrap();

    // Package 2 in main: github-cli
    fs::write(
        main_bucket.join("gh.json"),
        r#"{"version":"2.45.0","description":"GitHub CLI tool","bin":"gh.exe"}"#,
    )
    .unwrap();

    // Package 3 in extras: vscode
    fs::write(
        extras_bucket.join("vscode.json"),
        r#"{"version":"1.87.0","description":"Visual Studio Code IDE","bin":"code.cmd"}"#,
    )
    .unwrap();

    // Package 4: invalid JSON (must be skipped without crashing)
    fs::write(extras_bucket.join("broken.json"), r#"{"version": INVALID"#).unwrap();

    let env = ScoopEnv::from_root(temp_dir.clone());
    let index = BucketScanner::scan_buckets(&env);

    assert_eq!(index.total_packages(), 3);
    assert_eq!(index.buckets(), &["empty-custom", "extras", "main"]);
    assert_eq!(index.bucket_package_count("main"), 2);
    assert_eq!(index.bucket_package_count("extras"), 1);
    assert_eq!(index.bucket_package_count("empty-custom"), 0);

    // Search "git" (exact match "git" ranks ahead of description match "gh")
    let results = index.search("git", None);
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].name, "git");
    assert_eq!(results[1].name, "gh");

    // Search "code" with bucket filter "extras"
    let results = index.search("code", Some("extras"));
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "vscode");

    // Search with bucket filter "main" should not return vscode
    let results = index.search("code", Some("main"));
    assert!(results.is_empty());

    // find_in_bucket matches exact bucket first
    assert_eq!(
        index.find_in_bucket("vscode", Some("extras")).map(|p| p.version.as_str()),
        Some("1.87.0")
    );

    let _ = fs::remove_dir_all(&temp_dir);
}
