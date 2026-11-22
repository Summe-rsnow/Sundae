//! Store actions: background Scoop scanning via Rayon, serialized Scoop CLI
//! task queue polling/dispatch, configuration saving, and GitHub release checks.

use std::time::Duration;

use bento::updater::{check_latest_release, UpdateStatus};
use nsh_core::prelude::*;

use super::{
    append_logs, begin_scan, commit_scan, configured_proxy_for_updater, current_queue,
    ensure_queue, finish_scan_with_error, is_scan_seq_current, is_update_checking, notices,
    set_env_missing, set_pending_count, set_scoop_config, set_task_state, set_terminal_open,
    set_update_status,
};
use crate::scoop::config::ScoopConfig;
use crate::scoop::env::ScoopEnv;
use crate::scoop::installed::{InstalledApp, InstalledManager};
use crate::scoop::process::{ScoopOp, TaskStatus};
use crate::scoop::scanner::BucketScanner;
use crate::scoop::updates::UpdatesManager;

/// Starts the lightweight 100ms task queue poller for the current shell owner.
pub fn start_task_poller() {
    let queue = ensure_queue();
    let status_rx = queue.subscribe_status();
    let log_rx = queue.subscribe_logs();

    set_interval(Duration::from_millis(100), move || {
        let mut needs_refresh = false;
        while let Ok(status) = status_rx.try_recv() {
            match status {
                TaskStatus::Running(op) => {
                    set_task_state(true, Some(op.display_name()), queue.pending_count());
                }
                TaskStatus::Success(op) => {
                    set_task_state(false, None, queue.pending_count());
                    notices().notify_ok(format!("已完成：{}", op.display_name()));
                    needs_refresh = true;
                }
                TaskStatus::Failed { op, error } => {
                    set_task_state(false, None, queue.pending_count());
                    notices().notify_bad(format!("{}失败：{}", op.display_name(), error));
                }
                TaskStatus::Idle => {
                    set_task_state(false, None, 0);
                }
            }
        }

        let mut new_logs = Vec::new();
        while let Ok(line) = log_rx.try_recv() {
            new_logs.push(line);
        }
        append_logs(new_logs);

        if needs_refresh {
            refresh_all();
        }
    });
}

/// Scans Scoop buckets, installed apps, and pending updates on a background thread
/// using Rayon, then commits the snapshot atomically to UI signals.
pub fn refresh_all() {
    let env = ScoopEnv::detect();
    if !env.is_installed() {
        set_env_missing(env);
        return;
    }

    let next_seq = begin_scan(env.clone());

    spawn(async move {
        let scanned = blocking(move || {
            let index = BucketScanner::scan_buckets(&env);
            let installed = InstalledManager::scan_installed(&env);
            let updates = UpdatesManager::check_updates(&installed, &index);
            (index, installed, updates)
        })
        .await;

        if !is_scan_seq_current(next_seq) {
            return;
        }

        match scanned {
            Ok((index, installed, updates)) => commit_scan(index, installed, updates),
            Err(err) => finish_scan_with_error(err),
        }
    });
}

pub fn enqueue_op(op: ScoopOp) {
    let queue = ensure_queue();
    set_terminal_open(true);
    queue.enqueue(op);
    set_pending_count(queue.pending_count());
}

pub fn cancel_current_task() {
    if let Some(queue) = current_queue() {
        queue.cancel_current();
        notices().notify_warn("已请求终止当前任务");
    }
}

pub fn toggle_hold(app: &InstalledApp) {
    let next_hold = !app.is_held();
    let _ = InstalledManager::set_hold_flag(&app.path, next_hold);
    let op = if next_hold {
        ScoopOp::Hold(app.name.clone())
    } else {
        ScoopOp::Unhold(app.name.clone())
    };
    enqueue_op(op);
    refresh_all();
}

pub fn save_scoop_config(cfg: ScoopConfig) {
    match cfg.save() {
        Ok(()) => {
            set_scoop_config(cfg);
            notices().notify_ok("Scoop 配置已保存并生效");
        }
        Err(err) => {
            notices().notify_bad(format!("保存配置失败：{err}"));
        }
    }
}

pub fn check_for_updates() {
    if is_update_checking() {
        return;
    }
    set_update_status(UpdateStatus::Checking);
    let proxy = configured_proxy_for_updater();

    spawn(async move {
        let result = check_latest_release(crate::APP_META, proxy).await;
        set_update_status(UpdateStatus::from(result));
    });
}
