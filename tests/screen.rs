//! End-to-end headless UI tests for Sundae using `nsh_testing::Headless`.

use std::path::PathBuf;
use std::time::Duration;

use nsh_app_sundae::scoop::config::ScoopConfig;
use nsh_app_sundae::scoop::installed::{AppInstallStatus, InstalledApp};
use nsh_app_sundae::scoop::manifest::PackageManifest;
use nsh_app_sundae::scoop::scanner::SearchIndex;
use nsh_app_sundae::{shell, store};
use nsh_base::theme;
use nsh_core::prelude::*;
use nsh_engine::{EngineConfig, Points, Root, Size};
use nsh_testing::Headless;

fn sample_index() -> SearchIndex {
    SearchIndex::new(vec![
        PackageManifest {
            name: "git".to_owned(),
            bucket: "main".to_owned(),
            version: "2.46.0".to_owned(),
            description: "Distributed version control system".to_owned(),
            homepage: "https://git-scm.com".to_owned(),
            license: "GPL-2.0-only".to_owned(),
            license_url: None,
            bins: vec!["git.exe".to_owned()],
            depends: vec![],
            notes: vec!["Add git to PATH if needed.".to_owned()],
        },
        PackageManifest {
            name: "ripgrep".to_owned(),
            bucket: "main".to_owned(),
            version: "14.1.1".to_owned(),
            description: "Fast line-oriented regex search tool".to_owned(),
            homepage: "https://github.com/BurntSushi/ripgrep".to_owned(),
            license: "MIT".to_owned(),
            license_url: None,
            bins: vec!["rg.exe".to_owned()],
            depends: vec![],
            notes: vec![],
        },
        PackageManifest {
            name: "vscode".to_owned(),
            bucket: "extras".to_owned(),
            version: "1.95.0".to_owned(),
            description: "Visual Studio Code editor".to_owned(),
            homepage: "https://code.visualstudio.com".to_owned(),
            license: "Freeware".to_owned(),
            license_url: None,
            bins: vec!["code.cmd".to_owned()],
            depends: vec![],
            notes: vec![],
        },
    ])
}

fn sample_installed() -> Vec<InstalledApp> {
    vec![
        InstalledApp {
            name: "git".to_owned(),
            version: "2.44.0".to_owned(),
            bucket: Some("main".to_owned()),
            status: AppInstallStatus::Installed,
            path: PathBuf::from(r"C:\scoop\apps\git\current"),
            manifest_path: PathBuf::from(r"C:\scoop\apps\git\current\manifest.json"),
        },
        InstalledApp {
            name: "neovim".to_owned(),
            version: "0.10.0".to_owned(),
            bucket: Some("main".to_owned()),
            status: AppInstallStatus::Held,
            path: PathBuf::from(r"C:\scoop\apps\neovim\current"),
            manifest_path: PathBuf::from(r"C:\scoop\apps\neovim\current\manifest.json"),
        },
    ]
}

#[component]
fn SeededApp() -> impl IntoElement {
    store::seed_for_test(sample_index(), sample_installed(), ScoopConfig::default());
    view()
        .flex_1()
        .bg(move || theme().background)
        .text_color(move || theme().foreground)
        .child(shell::screen())
}

fn start_seeded() -> Headless {
    Headless::start(EngineConfig {
        viewport: Size::new(Points(1120.0), Points(720.0)),
        ..EngineConfig::new(Root::of::<SeededApp>())
    })
    .expect("start seeded Sundae headless")
}

#[test]
fn real_app_root_mounts_and_opens_about_and_config() -> Result<(), String> {
    let mut app = Headless::start(EngineConfig {
        viewport: Size::new(Points(1080.0), Points(720.0)),
        ..EngineConfig::new(Root::of::<nsh_app_sundae::App>())
    })
    .map_err(|e| e.to_string())?;

    app.wait_for("Sundae")?;
    assert!(app.screen().shows("v1.0.1"));

    // Verify title bar right controls: Refresh and Terminal sit left of the theme switcher.
    let layout = app.screen().layout_nodes();
    let refresh_id = app.screen().id("role:button[刷新索引]")?;
    let terminal_id = app.screen().id("role:button[切换终端抽屉]")?;
    let theme_id = app.screen().id("role:button[跟随系统]")?;
    let x_of = |id| {
        layout
            .iter()
            .find(|n| n.id == id)
            .map(|n| n.bounds.origin.x.0)
            .ok_or_else(|| format!("missing layout bounds for {id:?}"))
    };
    let refresh_x = x_of(refresh_id)?;
    let terminal_x = x_of(terminal_id)?;
    let theme_x = x_of(theme_id)?;
    assert!(
        refresh_x < terminal_x && terminal_x < theme_x,
        "expected refresh ({refresh_x}) < terminal ({terminal_x}) < theme ({theme_x})"
    );

    app.press("label:关于 Sundae")?;
    app.wait_for("版本更新")?;
    app.wait_for("Summe-rsnow/Sundae")?;
    assert!(app.screen().shows("v1.0.1"));
    app.press("关闭")?;
    app.wait_for_gone("版本更新", Duration::from_secs(2))?;

    app.press("label:选项配置")?;
    app.wait_for("Scoop 选项配置")?;
    app.wait_for("保存配置")?;
    Ok(())
}

#[test]
fn navigation_switches_between_all_five_views() -> Result<(), String> {
    let mut app = start_seeded();

    // Starts on Discover
    app.wait_for("探索软件")?;
    app.wait_for("ripgrep")?;

    // Switch to Installed
    app.press("label:已安装")?;
    app.wait_for("已安装应用")?;
    app.wait_for("neovim")?;
    app.wait_for("已锁定版本")?;

    // Switch to Updates (git 2.44.0 -> 2.46.0)
    app.press("label:更新中心")?;
    app.wait_for("更新中心")?;
    app.wait_for("当前 v2.44.0")?;
    app.wait_for("最新 v2.46.0")?;

    // Switch to Buckets
    app.press("label:仓库源")?;
    app.wait_for("Bucket 仓库源")?;
    app.wait_for("已添加的仓库源")?;
    app.wait_for("官方与热门推荐源")?;

    // Switch to Config
    app.press("label:选项配置")?;
    app.wait_for("Scoop 选项配置")?;
    app.wait_for("Aria2 多线程下载加速")?;
    app.wait_for("Sundae 窗口与托盘行为")?;

    Ok(())
}

#[test]
fn discover_search_and_bucket_filtering_work() -> Result<(), String> {
    let mut app = start_seeded();
    app.wait_for("探索软件")?;
    app.wait_for("3 项结果")?;

    // Filter by query "ripgrep"
    app.type_text("label:搜索软件", "ripgrep")?;
    app.wait_for("1 项结果")?;
    app.wait_for("ripgrep")?;

    // Clear search
    app.press("label:清空搜索")?;
    app.wait_for("3 项结果")?;

    // Filter by bucket "extras"
    app.press("id:bucket-filters >> extras")?;
    app.wait_for("1 项结果")?;
    app.wait_for("vscode")?;

    // Back to all buckets
    app.press("id:bucket-filters >> 全部源")?;
    app.wait_for("3 项结果")?;
    Ok(())
}

#[component]
fn LargeListApp() -> impl IntoElement {
    let packages = (0..120)
        .map(|i| PackageManifest {
            name: format!("pkg-{i:03}"),
            bucket: "main".to_owned(),
            version: format!("1.0.{i}"),
            description: format!("Package number {i}"),
            homepage: String::new(),
            license: "MIT".to_owned(),
            license_url: None,
            bins: vec![],
            depends: vec![],
            notes: vec![],
        })
        .collect();
    store::seed_for_test(SearchIndex::new(packages), vec![], ScoopConfig::default());
    view()
        .flex_1()
        .bg(move || theme().background)
        .text_color(move || theme().foreground)
        .child(shell::screen())
}

#[test]
fn discover_virtual_list_recycles_cells_on_scroll() -> Result<(), String> {
    let mut app = Headless::start(EngineConfig {
        viewport: Size::new(Points(1120.0), Points(640.0)),
        ..EngineConfig::new(Root::of::<LargeListApp>())
    })
    .map_err(|e| e.to_string())?;

    app.wait_for("120 项结果")?;
    app.wait_for("pkg-000")?;
    app.scroll("id:discover-list", Points(4500.0))?;
    app.wait_for("pkg-060")?;
    Ok(())
}

#[test]
fn package_detail_and_uninstall_confirm_dialogs_and_terminal_drawer() -> Result<(), String> {
    let mut app = start_seeded();
    app.wait_for("探索软件")?;

    // Open Package Detail Dialog for git
    app.press("label:查看 git 详情")?;
    app.wait_for("scoop install main/git")?;
    app.wait_for("安装说明 (Notes)")?;
    app.press("复制命令")?;
    assert_eq!(app.screen().clipboard(), Some("scoop install main/git"));
    app.press("关闭")?;
    app.wait_for_gone("scoop install main/git", Duration::from_secs(2))?;

    // Switch to Installed and open Confirm Uninstall dialog
    app.press("label:已安装")?;
    app.wait_for("已安装应用")?;
    app.press("label:卸载应用 >> nth=0")?;
    app.wait_for("确认卸载 git")?;
    app.wait_for("同时清除持久化数据（追加 --purge 参数，不可恢复）")?;
    app.press("取消")?;
    app.wait_for_gone("确认卸载 git", Duration::from_secs(2))?;

    // Toggle Terminal Drawer via top-bar button
    app.press("label:切换终端抽屉")?;
    app.wait_for("任务执行终端")?;
    app.press("收起")?;
    app.wait_for_gone("任务执行终端", Duration::from_secs(2))?;

    Ok(())
}
