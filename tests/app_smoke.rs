use nsh_app_sundae::scoop::manifest::PackageManifest;
use nsh_app_sundae::scoop::scanner::SearchIndex;
use nsh_app_sundae::store::{AppState, Route};
use nsh_core::element::Runtime;

#[test]
fn test_app_state_routes_and_queue() {
    let rt = Runtime::default();
    let _entered = rt.enter();

    let state = AppState::new();

    // Verify initial route is Discover
    assert_eq!(state.active_route.get(), Route::Discover);

    // Switch routes
    state.active_route.set(Route::Installed);
    assert_eq!(state.active_route.get(), Route::Installed);

    state.active_route.set(Route::Updates);
    assert_eq!(state.active_route.get(), Route::Updates);

    state.active_route.set(Route::Buckets);
    assert_eq!(state.active_route.get(), Route::Buckets);

    state.active_route.set(Route::Config);
    assert_eq!(state.active_route.get(), Route::Config);
    // Verify drawer toggle
    assert!(!state.terminal_drawer_open.get());
    state.terminal_drawer_open.set(true);
    assert!(state.terminal_drawer_open.get());

    // Verify search query updates
    state.search_query.set("rust".to_string());
    assert_eq!(state.search_query.get(), "rust");
}

#[test]
fn test_end_to_end_search_flow() {
    let index = SearchIndex::new(vec![
        PackageManifest {
            name: "git".to_string(),
            bucket: "main".to_string(),
            version: "2.44.0".to_string(),
            description: "Distributed version control system".to_string(),
            homepage: "https://git-scm.com".to_string(),
            license: "GPL-2.0".to_string(),
            bins: vec!["git.exe".to_string()],
            depends: vec![],
        },
        PackageManifest {
            name: "ripgrep".to_string(),
            bucket: "main".to_string(),
            version: "14.1.0".to_string(),
            description: "Fast line-oriented search tool".to_string(),
            homepage: "https://github.com/BurntSushi/ripgrep".to_string(),
            license: "MIT".to_string(),
            bins: vec!["rg.exe".to_string()],
            depends: vec![],
        },
    ]);

    assert_eq!(index.total_packages(), 2);

    let found = index.search("search", None);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].name, "ripgrep");

    let found_name = index.search("git", None);
    assert_eq!(found_name.len(), 1);
    assert_eq!(found_name[0].name, "git");
}
