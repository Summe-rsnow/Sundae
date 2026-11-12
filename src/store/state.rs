use crate::scoop::env::ScoopEnv;
use crate::scoop::installed::{InstalledApp, InstalledManager};
use crate::scoop::process::{LogLine, ScoopOp, TaskQueue};
use crate::scoop::scanner::{BucketScanner, SearchIndex};
use crate::scoop::updates::{AppUpdate, UpdatesManager};
use nsh_core::prelude::*;
use std::sync::Arc;
use std::thread;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    Discover,
    Installed,
    Updates,
    Buckets,
}

#[derive(Clone)]
pub struct AppState {
    pub env: ScoopEnv,
    pub active_route: Signal<Route>,
    pub search_query: Signal<String>,
    pub selected_bucket_filter: Signal<Option<String>>,
    pub search_index: Signal<SearchIndex>,
    pub installed_apps: Signal<Vec<InstalledApp>>,
    pub updates_list: Signal<Vec<AppUpdate>>,
    pub task_queue: Arc<TaskQueue>,
    pub is_indexing: Signal<bool>,
    pub is_task_running: Signal<bool>,
    pub terminal_drawer_open: Signal<bool>,
    pub task_logs: Signal<Vec<LogLine>>,
    pub is_missing_scoop: Signal<bool>,
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

impl AppState {
    pub fn new() -> Self {
        let env = ScoopEnv::detect();
        let is_missing = !env.is_installed();

        let state = Self {
            env,
            active_route: signal(Route::Discover),
            search_query: signal(String::new()),
            selected_bucket_filter: signal(None),
            search_index: signal(SearchIndex::default()),
            installed_apps: signal(Vec::new()),
            updates_list: signal(Vec::new()),
            task_queue: Arc::new(TaskQueue::new()),
            is_indexing: signal(false),
            is_task_running: signal(false),
            terminal_drawer_open: signal(false),
            task_logs: signal(Vec::new()),
            is_missing_scoop: signal(is_missing),
        };

        // Listen for task queue status events
        let status_rx = state.task_queue.subscribe_status();
        let is_running_sig = state.is_task_running;
        let state_for_status = state.clone();

        thread::spawn(move || {
            while let Ok(status) = status_rx.recv() {
                match status {
                    crate::scoop::process::TaskStatus::Running(_) => {
                        is_running_sig.set(true);
                    }
                    crate::scoop::process::TaskStatus::Success(_) => {
                        is_running_sig.set(false);
                        state_for_status.refresh_all();
                    }
                    crate::scoop::process::TaskStatus::Failed { .. }
                    | crate::scoop::process::TaskStatus::Idle => {
                        is_running_sig.set(false);
                    }
                }
            }
        });

        // Listen for task queue logs in background thread and update signal
        let log_rx = state.task_queue.subscribe_logs();
        let logs_signal = state.task_logs;
        let drawer_open = state.terminal_drawer_open;
        thread::spawn(move || {
            while let Ok(line) = log_rx.recv() {
                logs_signal.update(|logs| {
                    if logs.len() > 2000 {
                        logs.drain(0..500); // Ring buffer
                    }
                    logs.push(line);
                });
                // Auto-open drawer when tasks run
                if !drawer_open.get_untracked() {
                    drawer_open.set(true);
                }
            }
        });

        if !is_missing {
            state.refresh_all();
        }

        state
    }

    pub fn refresh_all(&self) {
        let env_clone = self.env.clone();
        let is_indexing = self.is_indexing;
        let search_index_sig = self.search_index;
        let installed_sig = self.installed_apps;
        let updates_sig = self.updates_list;
        let missing_sig = self.is_missing_scoop;

        is_indexing.set(true);

        thread::spawn(move || {
            let is_installed = env_clone.is_installed();
            missing_sig.set(!is_installed);

            if !is_installed {
                is_indexing.set(false);
                return;
            }

            // Scan local manifests
            let index = BucketScanner::scan_buckets(&env_clone);
            // Scan installed
            let installed = InstalledManager::scan_installed(&env_clone);
            // Check updates
            let updates = UpdatesManager::check_updates(&installed, &index);

            search_index_sig.set(index);
            installed_sig.set(installed);
            updates_sig.set(updates);
            is_indexing.set(false);
        });
    }

    pub fn enqueue_op(&self, op: ScoopOp) {
        self.terminal_drawer_open.set(true);
        self.task_queue.enqueue(op);
    }
}
