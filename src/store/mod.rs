//! Centralized reactive store for Sundae.
//!
//! All signals are strictly private to this module (zero `pub Signal` exposure).
//! Views and dialogs read state via tracked functions and request mutations via
//! semantic actions in [`actions`].

mod actions;

pub use actions::{
    cancel_current_task, check_for_updates, enqueue_op, refresh_all, save_scoop_config,
    start_task_poller, toggle_hold,
};

use std::sync::Arc;
use std::time::Duration;

use bento::shell::{CloseBehavior, NoticeCenter, ThemeMode};
use bento::updater::UpdateStatus;
use nsh_core::prelude::*;

use crate::scoop::config::{ScoopConfig, SundaePrefs};
use crate::scoop::env::ScoopEnv;
use crate::scoop::installed::InstalledApp;
use crate::scoop::manifest::PackageManifest;
use crate::scoop::process::{LogLine, TaskQueue};
use crate::scoop::scanner::SearchIndex;
use crate::scoop::updates::{AppUpdate, UpdatesManager};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Route {
    #[default]
    Discover,
    Installed,
    Updates,
    Buckets,
    Config,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum TaskbarProgress {
    #[default]
    None,
    Indeterminate,
    Normal(f32),
    Error,
}

#[derive(Clone)]
struct EnvState(ScoopEnv);

impl Default for EnvState {
    fn default() -> Self {
        Self(ScoopEnv::detect())
    }
}

impl PartialEq for EnvState {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

#[derive(Default, Clone, PartialEq)]
struct ActiveRoute(Route);

#[derive(Default, Clone, PartialEq)]
struct DiscoverQuery(String);

#[derive(Default, Clone, PartialEq)]
struct BucketFilter(Option<String>);

#[derive(Default, Clone, PartialEq)]
struct InstalledQuery(String);

#[derive(Default, Clone, PartialEq)]
struct IndexState(SearchIndex);

#[derive(Default, Clone, PartialEq)]
struct InstalledState(Vec<InstalledApp>);

#[derive(Default, Clone, PartialEq)]
struct UpdatesState(Vec<AppUpdate>);

#[derive(Default, Clone, PartialEq)]
struct IndexingState(bool);

#[derive(Default, Clone, PartialEq)]
struct ScanSeq(u64);

#[derive(Default, Clone, PartialEq)]
struct MissingScoopState(bool);

#[derive(Default, Clone, PartialEq)]
struct TaskRunningState(bool);

#[derive(Default, Clone, PartialEq)]
struct RunningTaskLabel(Option<String>);

#[derive(Default, Clone, PartialEq)]
struct PendingCountState(usize);

#[derive(Default, Clone, PartialEq)]
struct TaskBatchTotal(usize);

#[derive(Default, Clone, PartialEq)]
struct TaskBatchCompleted(usize);

#[derive(Default, Clone, PartialEq)]
struct TaskErrorFlash(bool);

#[derive(Default, Clone, PartialEq)]
struct TaskErrorSeq(u64);

#[derive(Default, Clone, PartialEq)]
struct TerminalOpenState(bool);

#[derive(Default, Clone, PartialEq)]
struct LogListState(Vec<LogLine>);

#[derive(Default, Clone, PartialEq)]
struct ScoopConfigState(ScoopConfig);

#[derive(Default, Clone, PartialEq)]
struct ThemeModeState(ThemeMode);

#[derive(Default, Clone, PartialEq)]
struct CloseBehaviorState(CloseBehavior);

#[derive(Default, Clone, PartialEq)]
struct UpdateCheckState(UpdateStatus);

#[derive(Default, Clone, PartialEq)]
struct AboutOpenState(bool);

#[derive(Default, Clone, PartialEq)]
struct DetailTargetState(Option<PackageManifest>);

#[derive(Default, Clone, PartialEq)]
struct UninstallTargetState(Option<String>);

#[derive(Default, Clone, PartialEq)]
struct RemoveBucketTargetState(Option<String>);

#[derive(Clone, Default)]
struct QueueHandle(Option<Arc<TaskQueue>>);

impl PartialEq for QueueHandle {
    fn eq(&self, other: &Self) -> bool {
        match (&self.0, &other.0) {
            (Some(a), Some(b)) => Arc::ptr_eq(a, b),
            (None, None) => true,
            _ => false,
        }
    }
}

// --- Private signal accessors ---

fn env_sig() -> Signal<EnvState> {
    global::<EnvState>()
}
fn route_sig() -> Signal<ActiveRoute> {
    global::<ActiveRoute>()
}
fn discover_query_sig() -> Signal<DiscoverQuery> {
    global::<DiscoverQuery>()
}
fn bucket_filter_sig() -> Signal<BucketFilter> {
    global::<BucketFilter>()
}
fn installed_query_sig() -> Signal<InstalledQuery> {
    global::<InstalledQuery>()
}
fn index_sig() -> Signal<IndexState> {
    global::<IndexState>()
}
fn installed_sig() -> Signal<InstalledState> {
    global::<InstalledState>()
}
fn updates_sig() -> Signal<UpdatesState> {
    global::<UpdatesState>()
}
fn indexing_sig() -> Signal<IndexingState> {
    global::<IndexingState>()
}
fn scan_seq_sig() -> Signal<ScanSeq> {
    global::<ScanSeq>()
}
fn missing_scoop_sig() -> Signal<MissingScoopState> {
    global::<MissingScoopState>()
}
fn task_running_sig() -> Signal<TaskRunningState> {
    global::<TaskRunningState>()
}
fn running_label_sig() -> Signal<RunningTaskLabel> {
    global::<RunningTaskLabel>()
}
fn pending_count_sig() -> Signal<PendingCountState> {
    global::<PendingCountState>()
}
fn task_batch_total_sig() -> Signal<TaskBatchTotal> {
    global::<TaskBatchTotal>()
}
fn task_batch_completed_sig() -> Signal<TaskBatchCompleted> {
    global::<TaskBatchCompleted>()
}
fn task_error_flash_sig() -> Signal<TaskErrorFlash> {
    global::<TaskErrorFlash>()
}
fn task_error_seq_sig() -> Signal<TaskErrorSeq> {
    global::<TaskErrorSeq>()
}
fn terminal_open_sig() -> Signal<TerminalOpenState> {
    global::<TerminalOpenState>()
}
fn logs_sig() -> Signal<LogListState> {
    global::<LogListState>()
}
fn scoop_config_sig() -> Signal<ScoopConfigState> {
    global::<ScoopConfigState>()
}
fn theme_mode_sig() -> Signal<ThemeModeState> {
    global::<ThemeModeState>()
}
fn close_behavior_sig() -> Signal<CloseBehaviorState> {
    global::<CloseBehaviorState>()
}
fn update_status_sig() -> Signal<UpdateCheckState> {
    global::<UpdateCheckState>()
}
fn about_open_sig() -> Signal<AboutOpenState> {
    global::<AboutOpenState>()
}
fn detail_target_sig() -> Signal<DetailTargetState> {
    global::<DetailTargetState>()
}
fn uninstall_target_sig() -> Signal<UninstallTargetState> {
    global::<UninstallTargetState>()
}
fn remove_bucket_target_sig() -> Signal<RemoveBucketTargetState> {
    global::<RemoveBucketTargetState>()
}
fn queue_sig() -> Signal<QueueHandle> {
    global::<QueueHandle>()
}

// --- Internal helpers shared with `actions` ---

fn ensure_queue() -> Arc<TaskQueue> {
    let existing = queue_sig().with_untracked(|h| h.0.clone());
    if let Some(q) = existing {
        return q;
    }
    let q = Arc::new(TaskQueue::new());
    queue_sig().set(QueueHandle(Some(Arc::clone(&q))));
    q
}

fn current_queue() -> Option<Arc<TaskQueue>> {
    queue_sig().with_untracked(|h| h.0.clone())
}

fn clear_task_error() {
    task_error_flash_sig().set(TaskErrorFlash(false));
}

fn flash_task_error() {
    let next_seq = task_error_seq_sig().get_untracked().0.wrapping_add(1);
    batch(|| {
        task_error_seq_sig().set(TaskErrorSeq(next_seq));
        task_error_flash_sig().set(TaskErrorFlash(true));
    });
    set_timeout(Duration::from_secs(4), move || {
        if task_error_seq_sig().get_untracked().0 == next_seq {
            task_error_flash_sig().set(TaskErrorFlash(false));
        }
    });
}

fn note_op_enqueued(pending: usize) {
    let prev_total = task_batch_total_sig().get_untracked().0;
    batch(|| {
        task_error_flash_sig().set(TaskErrorFlash(false));
        pending_count_sig().set(PendingCountState(pending));
        if prev_total == 0 {
            task_batch_total_sig().set(TaskBatchTotal(pending.max(1)));
            task_batch_completed_sig().set(TaskBatchCompleted(0));
        } else {
            task_batch_total_sig().set(TaskBatchTotal(prev_total.saturating_add(1)));
        }
    });
}

fn note_task_running(label: String, pending: usize) {
    let prev_total = task_batch_total_sig().get_untracked().0;
    batch(|| {
        task_error_flash_sig().set(TaskErrorFlash(false));
        task_running_sig().set(TaskRunningState(true));
        running_label_sig().set(RunningTaskLabel(Some(label)));
        pending_count_sig().set(PendingCountState(pending));
        if prev_total < pending.max(1) {
            task_batch_total_sig().set(TaskBatchTotal(pending.max(1)));
        }
    });
}

fn note_task_finished(succeeded: bool, pending: usize) {
    let next_completed = task_batch_completed_sig()
        .get_untracked()
        .0
        .saturating_add(1);
    batch(|| {
        task_running_sig().set(TaskRunningState(false));
        running_label_sig().set(RunningTaskLabel(None));
        pending_count_sig().set(PendingCountState(pending));
        if pending == 0 {
            task_batch_total_sig().set(TaskBatchTotal(0));
            task_batch_completed_sig().set(TaskBatchCompleted(0));
        } else {
            task_batch_completed_sig().set(TaskBatchCompleted(next_completed));
        }
    });
    if !succeeded && pending == 0 {
        flash_task_error();
    }
}

fn note_task_idle() {
    batch(|| {
        task_running_sig().set(TaskRunningState(false));
        running_label_sig().set(RunningTaskLabel(None));
        pending_count_sig().set(PendingCountState(0));
        task_batch_total_sig().set(TaskBatchTotal(0));
        task_batch_completed_sig().set(TaskBatchCompleted(0));
    });
}

fn append_logs(new_logs: Vec<LogLine>) {
    if new_logs.is_empty() {
        return;
    }
    logs_sig().update(|state| {
        if state.0.len() + new_logs.len() > 2500 {
            let drop_count = (state.0.len() + new_logs.len()).saturating_sub(2000);
            let drain_to = drop_count.min(state.0.len());
            state.0.drain(0..drain_to);
        }
        state.0.extend(new_logs);
    });
    if !terminal_open_sig().get_untracked().0 {
        terminal_open_sig().set(TerminalOpenState(true));
    }
}

fn set_env_missing(env: ScoopEnv) {
    env_sig().set(EnvState(env));
    batch(|| {
        missing_scoop_sig().set(MissingScoopState(true));
        indexing_sig().set(IndexingState(false));
    });
}

fn begin_scan(env: ScoopEnv) -> u64 {
    env_sig().set(EnvState(env));
    let next_seq = scan_seq_sig().get_untracked().0.wrapping_add(1);
    batch(|| {
        missing_scoop_sig().set(MissingScoopState(false));
        scan_seq_sig().set(ScanSeq(next_seq));
        indexing_sig().set(IndexingState(true));
        clear_task_error();
    });
    next_seq
}

fn is_scan_seq_current(seq: u64) -> bool {
    scan_seq_sig().get_untracked().0 == seq
}

fn commit_scan(index: SearchIndex, installed: Vec<InstalledApp>, updates: Vec<AppUpdate>) {
    batch(|| {
        index_sig().set(IndexState(index));
        installed_sig().set(InstalledState(installed));
        updates_sig().set(UpdatesState(updates));
        indexing_sig().set(IndexingState(false));
    });
}

fn finish_scan_with_error(err: impl std::fmt::Display) {
    indexing_sig().set(IndexingState(false));
    flash_task_error();
    notices().notify_bad(format!("后台扫描 Scoop 索引失败：{err}"));
}

fn set_scoop_config(cfg: ScoopConfig) {
    scoop_config_sig().set(ScoopConfigState(cfg));
}

fn configured_proxy_for_updater() -> Option<String> {
    scoop_config_sig()
        .get_untracked()
        .0
        .proxy
        .filter(|p| {
            let trimmed = p.trim();
            !trimmed.is_empty()
                && !trimmed.eq_ignore_ascii_case("none")
                && !trimmed.eq_ignore_ascii_case("default")
        })
}

fn is_update_checking() -> bool {
    matches!(
        update_status_sig().get_untracked().0,
        UpdateStatus::Checking
    )
}

fn set_update_status(status: UpdateStatus) {
    update_status_sig().set(UpdateCheckState(status));
}

fn persist_prefs() {
    let prefs = SundaePrefs {
        theme_mode: theme_mode_sig().get_untracked().0.as_key().to_owned(),
        close_behavior: close_behavior_sig().get_untracked().0.as_key().to_owned(),
    };
    if let Err(err) = prefs.save() {
        log::warn!("Failed to persist Sundae prefs: {err}");
    }
}

/// Initializes store state on app startup without blocking the UI thread.
pub fn load() {
    let env = ScoopEnv::detect();
    let cfg = ScoopConfig::load();
    let prefs = SundaePrefs::load();
    let theme_mode = ThemeMode::from_key(&prefs.theme_mode);
    let close_behavior = CloseBehavior::from_key(&prefs.close_behavior);
    let is_missing = !env.is_installed();

    bento::shell::init_autostart("com.xcmm.sundae", Some("Sundae"));

    batch(|| {
        env_sig().set(EnvState(env));
        scoop_config_sig().set(ScoopConfigState(cfg));
        theme_mode_sig().set(ThemeModeState(theme_mode));
        close_behavior_sig().set(CloseBehaviorState(close_behavior));
        missing_scoop_sig().set(MissingScoopState(is_missing));
    });

    theme_mode.apply();

    if !is_missing {
        refresh_all();
    }
}

/// Seeds the store with deterministic data for headless UI tests.
pub fn seed_for_test(index: SearchIndex, installed: Vec<InstalledApp>, config: ScoopConfig) {
    let updates = UpdatesManager::check_updates(&installed, &index);
    bento::shell::init_autostart_for_test(false);
    batch(|| {
        missing_scoop_sig().set(MissingScoopState(false));
        indexing_sig().set(IndexingState(false));
        task_running_sig().set(TaskRunningState(false));
        running_label_sig().set(RunningTaskLabel(None));
        pending_count_sig().set(PendingCountState(0));
        task_batch_total_sig().set(TaskBatchTotal(0));
        task_batch_completed_sig().set(TaskBatchCompleted(0));
        task_error_flash_sig().set(TaskErrorFlash(false));
        index_sig().set(IndexState(index));
        installed_sig().set(InstalledState(installed));
        updates_sig().set(UpdatesState(updates));
        scoop_config_sig().set(ScoopConfigState(config));
    });
}

// --- Navigation & Filters ---

pub fn active_route() -> Route {
    route_sig().get().0
}

pub fn set_route(route: Route) {
    route_sig().set(ActiveRoute(route));
}

pub fn search_query() -> String {
    discover_query_sig().get().0
}

pub fn set_search_query(query: impl Into<String>) {
    discover_query_sig().set(DiscoverQuery(query.into()));
}

pub fn bucket_filter() -> Option<String> {
    bucket_filter_sig().get().0
}

pub fn set_bucket_filter(bucket: Option<String>) {
    bucket_filter_sig().set(BucketFilter(bucket));
}

pub fn installed_filter() -> String {
    installed_query_sig().get().0
}

pub fn set_installed_filter(query: impl Into<String>) {
    installed_query_sig().set(InstalledQuery(query.into()));
}

// --- Domain Queries (tracked) ---

pub fn is_indexing() -> bool {
    indexing_sig().get().0
}

pub fn is_missing_scoop() -> bool {
    missing_scoop_sig().get().0
}

pub fn scoop_root_display() -> String {
    env_sig().with(|e| e.0.root.display().to_string())
}

pub fn total_packages_count() -> usize {
    index_sig().with(|idx| idx.0.total_packages())
}

pub fn buckets() -> Vec<String> {
    index_sig().with(|idx| idx.0.buckets().to_vec())
}

pub fn bucket_count() -> usize {
    index_sig().with(|idx| idx.0.buckets().len())
}

pub fn bucket_package_count(bucket: &str) -> usize {
    index_sig().with(|idx| idx.0.bucket_package_count(bucket))
}

pub fn filtered_discover_packages() -> Vec<PackageManifest> {
    let query = discover_query_sig().get().0;
    let bucket = bucket_filter_sig().get().0;
    index_sig().with(|idx| {
        idx.0
            .search(&query, bucket.as_deref())
            .into_iter()
            .cloned()
            .collect()
    })
}

pub fn installed_count() -> usize {
    installed_sig().with(|apps| apps.0.len())
}

pub fn filtered_installed_apps() -> Vec<InstalledApp> {
    let filter = installed_query_sig().get().0.trim().to_ascii_lowercase();
    installed_sig().with(|apps| {
        apps.0
            .iter()
            .filter(|a| filter.is_empty() || a.name.to_ascii_lowercase().contains(&filter))
            .cloned()
            .collect()
    })
}

pub fn is_installed_package(name: &str) -> bool {
    installed_sig().with(|apps| apps.0.iter().any(|a| a.name.eq_ignore_ascii_case(name)))
}

pub fn find_manifest(name: &str, bucket: Option<&str>) -> Option<PackageManifest> {
    index_sig().with(|idx| idx.0.find_in_bucket(name, bucket).cloned())
}

pub fn updates() -> Vec<AppUpdate> {
    updates_sig().get().0
}

pub fn updates_count() -> usize {
    updates_sig().with(|u| u.0.len())
}

// --- Process Queue & Terminal ---

pub fn is_task_running() -> bool {
    task_running_sig().get().0
}

pub fn running_task_label() -> Option<String> {
    running_label_sig().get().0
}

pub fn pending_task_count() -> usize {
    pending_count_sig().get().0
}

fn compute_taskbar_progress(
    task_running: bool,
    pending: usize,
    batch_total: usize,
    batch_completed: usize,
    indexing: bool,
    error_flash: bool,
) -> TaskbarProgress {
    if task_running || pending > 0 {
        if batch_total > 1 && batch_completed > 0 {
            TaskbarProgress::Normal((batch_completed as f32 / batch_total as f32).clamp(0.0, 1.0))
        } else {
            TaskbarProgress::Indeterminate
        }
    } else if indexing {
        TaskbarProgress::Indeterminate
    } else if error_flash {
        TaskbarProgress::Error
    } else {
        TaskbarProgress::None
    }
}

pub fn taskbar_progress() -> TaskbarProgress {
    compute_taskbar_progress(
        is_task_running(),
        pending_task_count(),
        task_batch_total_sig().get().0,
        task_batch_completed_sig().get().0,
        is_indexing(),
        task_error_flash_sig().get().0,
    )
}

pub fn terminal_open() -> bool {
    terminal_open_sig().get().0
}

pub fn set_terminal_open(open: bool) {
    terminal_open_sig().set(TerminalOpenState(open));
}

pub fn toggle_terminal() {
    let current = terminal_open_sig().get_untracked().0;
    terminal_open_sig().set(TerminalOpenState(!current));
}

pub fn log_count() -> usize {
    logs_sig().with(|l| l.0.len())
}

pub fn log_ids() -> Vec<u64> {
    logs_sig().with(|l| l.0.iter().map(|line| line.id).collect())
}

pub fn log_at(index: usize) -> Option<LogLine> {
    logs_sig().with(|l| l.0.get(index).cloned())
}

pub fn clear_logs() {
    logs_sig().set(LogListState(Vec::new()));
}

// --- Configuration & Preferences ---

pub fn scoop_config() -> ScoopConfig {
    scoop_config_sig().get().0
}

pub fn theme_mode() -> ThemeMode {
    theme_mode_sig().get().0
}

pub fn set_theme_mode(mode: ThemeMode) {
    theme_mode_sig().set(ThemeModeState(mode));
    mode.apply();
    persist_prefs();
}

pub fn close_behavior() -> CloseBehavior {
    close_behavior_sig().get().0
}

pub fn close_behavior_untracked() -> CloseBehavior {
    close_behavior_sig().get_untracked().0
}

pub fn set_close_behavior(behavior: CloseBehavior) {
    close_behavior_sig().set(CloseBehaviorState(behavior));
    persist_prefs();
}

// --- Notices & Updater ---

pub fn notices() -> NoticeCenter {
    NoticeCenter::global()
}

pub fn update_status() -> UpdateStatus {
    update_status_sig().get().0
}

// --- Modal Dialog States ---

pub fn about_open() -> bool {
    about_open_sig().get().0
}

pub fn set_about_open(open: bool) {
    about_open_sig().set(AboutOpenState(open));
}

pub fn detail_target() -> Option<PackageManifest> {
    detail_target_sig().get().0
}

pub fn open_package_detail(pkg: PackageManifest) {
    detail_target_sig().set(DetailTargetState(Some(pkg)));
}

pub fn close_package_detail() {
    detail_target_sig().set(DetailTargetState(None));
}

pub fn uninstall_target() -> Option<String> {
    uninstall_target_sig().get().0
}

pub fn request_uninstall(name: impl Into<String>) {
    uninstall_target_sig().set(UninstallTargetState(Some(name.into())));
}

pub fn close_uninstall_confirm() {
    uninstall_target_sig().set(UninstallTargetState(None));
}

pub fn remove_bucket_target() -> Option<String> {
    remove_bucket_target_sig().get().0
}

pub fn request_remove_bucket(name: impl Into<String>) {
    remove_bucket_target_sig().set(RemoveBucketTargetState(Some(name.into())));
}

pub fn close_remove_bucket_confirm() {
    remove_bucket_target_sig().set(RemoveBucketTargetState(None));
}

/// Closes whichever modal dialog is currently open (for Escape shortcut).
pub fn dismiss_top_dialog() -> bool {
    if remove_bucket_target_sig().get_untracked().0.is_some() {
        close_remove_bucket_confirm();
        return true;
    }
    if uninstall_target_sig().get_untracked().0.is_some() {
        close_uninstall_confirm();
        return true;
    }
    if detail_target_sig().get_untracked().0.is_some() {
        close_package_detail();
        return true;
    }
    if about_open_sig().get_untracked().0 {
        set_about_open(false);
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::{compute_taskbar_progress, TaskbarProgress};

    #[test]
    fn taskbar_progress_covers_idle_indeterminate_normal_and_error_states() {
        assert_eq!(
            compute_taskbar_progress(false, 0, 0, 0, false, false),
            TaskbarProgress::None
        );
        assert_eq!(
            compute_taskbar_progress(false, 0, 0, 0, true, false),
            TaskbarProgress::Indeterminate
        );
        assert_eq!(
            compute_taskbar_progress(true, 1, 1, 0, false, false),
            TaskbarProgress::Indeterminate
        );
        assert_eq!(
            compute_taskbar_progress(true, 2, 4, 2, false, false),
            TaskbarProgress::Normal(0.5)
        );
        assert_eq!(
            compute_taskbar_progress(false, 0, 0, 0, false, true),
            TaskbarProgress::Error
        );
        // Active task takes precedence over stale error flash.
        assert_eq!(
            compute_taskbar_progress(true, 1, 2, 1, false, true),
            TaskbarProgress::Normal(0.5)
        );
    }
}
