use crate::scoop::config::ScoopConfig;
use crate::scoop::env::ScoopEnv;
use crate::scoop::installed::{InstalledApp, InstalledManager};
use crate::scoop::process::{LogLine, ScoopOp, TaskQueue};
use crate::scoop::scanner::{BucketScanner, SearchIndex};
use crate::scoop::updates::{AppUpdate, UpdatesManager};
use nsh_core::prelude::*;
use std::sync::mpsc::Receiver;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Route {
    Discover,
    Installed,
    Updates,
    Buckets,
    Config,
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
    pub scoop_config: Signal<ScoopConfig>,
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
            scoop_config: signal(ScoopConfig::load()),
            is_missing_scoop: signal(is_missing),
        };

        if !is_missing {
            state.refresh_all();
        }

        state
    }

    pub fn poll_tasks(
        &self,
        status_rx: &Receiver<crate::scoop::process::TaskStatus>,
        log_rx: &Receiver<LogLine>,
    ) {
        while let Ok(status) = status_rx.try_recv() {
            match status {
                crate::scoop::process::TaskStatus::Running(_) => {
                    self.is_task_running.set(true);
                }
                crate::scoop::process::TaskStatus::Success(_) => {
                    self.is_task_running.set(false);
                    self.refresh_all();
                }
                crate::scoop::process::TaskStatus::Failed { .. }
                | crate::scoop::process::TaskStatus::Idle => {
                    self.is_task_running.set(false);
                }
            }
        }

        let mut new_logs = Vec::new();
        while let Ok(line) = log_rx.try_recv() {
            new_logs.push(line);
        }
        if !new_logs.is_empty() {
            self.task_logs.update(|logs| {
                if logs.len() > 2000 {
                    logs.drain(0..500);
                }
                logs.extend(new_logs);
            });
            if !self.terminal_drawer_open.get_untracked() {
                self.terminal_drawer_open.set(true);
            }
        }
    }

    pub fn refresh_all(&self) {
        if !self.env.is_installed() {
            self.is_missing_scoop.set(true);
            self.is_indexing.set(false);
            return;
        }

        self.is_missing_scoop.set(false);
        self.is_indexing.set(true);

        // Scan local manifests via Rayon parallel threads
        let index = BucketScanner::scan_buckets(&self.env);
        // Scan installed applications
        let installed = InstalledManager::scan_installed(&self.env);
        // Check for updates
        let updates = UpdatesManager::check_updates(&installed, &index);

        self.search_index.set(index);
        self.installed_apps.set(installed);
        self.updates_list.set(updates);
        self.is_indexing.set(false);
    }

    pub fn enqueue_op(&self, op: ScoopOp) {
        self.terminal_drawer_open.set(true);
        self.task_queue.enqueue(op);
    }
}
