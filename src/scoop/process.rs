use parking_lot::Mutex;
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScoopOp {
    Install(String),
    Uninstall { name: String, purge: bool },
    Update(String),
    UpdateAll,
    Hold(String),
    Unhold(String),
    BucketAdd { name: String, url: Option<String> },
    BucketRm(String),
    BucketUpdate,
    Custom(Vec<String>),
}

impl ScoopOp {
    pub fn uninstall(name: impl Into<String>, purge: bool) -> Self {
        Self::Uninstall {
            name: name.into(),
            purge,
        }
    }

    pub fn to_args(&self) -> Vec<String> {
        match self {
            ScoopOp::Install(name) => vec!["install".into(), name.clone()],
            ScoopOp::Uninstall { name, purge } => {
                let mut args = vec!["uninstall".into(), name.clone()];
                if *purge {
                    args.push("--purge".into());
                }
                args
            }
            ScoopOp::Update(name) => vec!["update".into(), name.clone()],
            ScoopOp::UpdateAll => vec!["update".into(), "*".into()],
            ScoopOp::Hold(name) => vec!["hold".into(), name.clone()],
            ScoopOp::Unhold(name) => vec!["unhold".into(), name.clone()],
            ScoopOp::BucketAdd { name, url } => {
                let mut args = vec!["bucket".into(), "add".into(), name.clone()];
                if let Some(u) = url {
                    args.push(u.clone());
                }
                args
            }
            ScoopOp::BucketRm(name) => vec!["bucket".into(), "rm".into(), name.clone()],
            ScoopOp::BucketUpdate => vec!["update".into()],
            ScoopOp::Custom(args) => args.clone(),
        }
    }

    pub fn display_name(&self) -> String {
        match self {
            ScoopOp::Install(n) => format!("安装 {}", n),
            ScoopOp::Uninstall { name, purge } => {
                if *purge {
                    format!("卸载并清除数据 {}", name)
                } else {
                    format!("卸载 {}", name)
                }
            }
            ScoopOp::Update(n) => format!("更新 {}", n),
            ScoopOp::UpdateAll => "更新全部软件".to_string(),
            ScoopOp::Hold(n) => format!("锁定版本 {}", n),
            ScoopOp::Unhold(n) => format!("解锁版本 {}", n),
            ScoopOp::BucketAdd { name, .. } => format!("添加源 {}", name),
            ScoopOp::BucketRm(name) => format!("移除源 {}", name),
            ScoopOp::BucketUpdate => "同步全部仓库源".to_string(),
            ScoopOp::Custom(args) => args.join(" "),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogLine {
    pub id: u64,
    pub text: String,
    pub is_err: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaskStatus {
    Idle,
    Running(ScoopOp),
    Success(ScoopOp),
    Failed { op: ScoopOp, error: String },
}

pub struct TaskQueue {
    sender: Sender<ScoopOp>,
    log_subscribers: Arc<Mutex<Vec<Sender<LogLine>>>>,
    status_subscribers: Arc<Mutex<Vec<Sender<TaskStatus>>>>,
    status: Arc<Mutex<TaskStatus>>,
    active_child: Arc<Mutex<Option<Child>>>,
    pending: Arc<AtomicUsize>,
    next_log_id: Arc<AtomicU64>,
}

impl Default for TaskQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(target_os = "windows")]
fn hide_console_window(cmd: &mut Command) {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    cmd.creation_flags(CREATE_NO_WINDOW);
}

#[cfg(not(target_os = "windows"))]
fn hide_console_window(_cmd: &mut Command) {}

impl TaskQueue {
    pub fn new() -> Self {
        let (sender, receiver) = channel::<ScoopOp>();
        let log_subscribers = Arc::new(Mutex::new(Vec::<Sender<LogLine>>::new()));
        let status_subscribers = Arc::new(Mutex::new(Vec::<Sender<TaskStatus>>::new()));
        let status = Arc::new(Mutex::new(TaskStatus::Idle));
        let active_child = Arc::new(Mutex::new(None));
        let pending = Arc::new(AtomicUsize::new(0));
        let next_log_id = Arc::new(AtomicU64::new(1));

        let worker_subscribers = Arc::clone(&log_subscribers);
        let worker_status_subs = Arc::clone(&status_subscribers);
        let worker_status = Arc::clone(&status);
        let worker_child = Arc::clone(&active_child);
        let worker_pending = Arc::clone(&pending);
        let worker_log_id = Arc::clone(&next_log_id);

        thread::spawn(move || {
            let emit_log = |subs_arc: &Arc<Mutex<Vec<Sender<LogLine>>>>,
                            id_counter: &Arc<AtomicU64>,
                            text: String,
                            is_err: bool| {
                let id = id_counter.fetch_add(1, Ordering::Relaxed);
                let line = LogLine { id, text, is_err };
                let mut subs = subs_arc.lock();
                subs.retain(|sub| sub.send(line.clone()).is_ok());
            };

            while let Ok(op) = receiver.recv() {
                let broadcast_status = |st: TaskStatus| {
                    *worker_status.lock() = st.clone();
                    let mut subs = worker_status_subs.lock();
                    subs.retain(|sub| sub.send(st.clone()).is_ok());
                };

                broadcast_status(TaskStatus::Running(op.clone()));
                emit_log(
                    &worker_subscribers,
                    &worker_log_id,
                    format!(">>> 开始执行: {}", op.display_name()),
                    false,
                );

                let mut cmd = match &op {
                    ScoopOp::Custom(args) => {
                        if args.is_empty() {
                            worker_pending.fetch_sub(1, Ordering::SeqCst);
                            broadcast_status(TaskStatus::Success(op));
                            continue;
                        }
                        let mut c = Command::new(&args[0]);
                        if args.len() > 1 {
                            c.args(&args[1..]);
                        }
                        c
                    }
                    _ => {
                        let mut c = Command::new("powershell");
                        c.args([
                            "-NoProfile",
                            "-ExecutionPolicy",
                            "Bypass",
                            "-Command",
                            "scoop",
                        ]);
                        c.args(op.to_args());
                        c
                    }
                };

                hide_console_window(&mut cmd);
                cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

                let mut child = match cmd.spawn() {
                    Ok(c) => c,
                    Err(e) => {
                        let err_msg = format!("无法启动进程: {}", e);
                        emit_log(&worker_subscribers, &worker_log_id, err_msg.clone(), true);
                        worker_pending.fetch_sub(1, Ordering::SeqCst);
                        broadcast_status(TaskStatus::Failed { op, error: err_msg });
                        continue;
                    }
                };

                let stdout = child.stdout.take();
                let stderr = child.stderr.take();

                *worker_child.lock() = Some(child);

                let log_subs_out = Arc::clone(&worker_subscribers);
                let id_out = Arc::clone(&worker_log_id);
                let stdout_thread = thread::spawn(move || {
                    if let Some(out) = stdout {
                        let reader = BufReader::new(out);
                        for line in reader.lines().map_while(Result::ok) {
                            let id = id_out.fetch_add(1, Ordering::Relaxed);
                            let entry = LogLine {
                                id,
                                text: line,
                                is_err: false,
                            };
                            let mut subs = log_subs_out.lock();
                            subs.retain(|sub| sub.send(entry.clone()).is_ok());
                        }
                    }
                });

                let log_subs_err = Arc::clone(&worker_subscribers);
                let id_err = Arc::clone(&worker_log_id);
                let stderr_thread = thread::spawn(move || {
                    if let Some(err) = stderr {
                        let reader = BufReader::new(err);
                        for line in reader.lines().map_while(Result::ok) {
                            let id = id_err.fetch_add(1, Ordering::Relaxed);
                            let entry = LogLine {
                                id,
                                text: line,
                                is_err: true,
                            };
                            let mut subs = log_subs_err.lock();
                            subs.retain(|sub| sub.send(entry.clone()).is_ok());
                        }
                    }
                });

                let _ = stdout_thread.join();
                let _ = stderr_thread.join();

                let exit_status = {
                    let mut lock = worker_child.lock();
                    if let Some(mut c) = lock.take() {
                        c.wait().ok()
                    } else {
                        None
                    }
                };

                worker_pending.fetch_sub(1, Ordering::SeqCst);

                match exit_status {
                    Some(s) if s.success() => {
                        emit_log(
                            &worker_subscribers,
                            &worker_log_id,
                            format!("✓ 执行完成: {}", op.display_name()),
                            false,
                        );
                        broadcast_status(TaskStatus::Success(op));
                    }
                    Some(s) => {
                        let msg = format!("✖ 执行失败，退出码: {}", s.code().unwrap_or(-1));
                        emit_log(&worker_subscribers, &worker_log_id, msg.clone(), true);
                        broadcast_status(TaskStatus::Failed { op, error: msg });
                    }
                    None => {
                        let msg = "任务已被取消或终止".to_string();
                        emit_log(&worker_subscribers, &worker_log_id, msg.clone(), true);
                        broadcast_status(TaskStatus::Failed { op, error: msg });
                    }
                }
            }
        });

        Self {
            sender,
            log_subscribers,
            status_subscribers,
            status,
            active_child,
            pending,
            next_log_id,
        }
    }

    pub fn enqueue(&self, op: ScoopOp) {
        self.pending.fetch_add(1, Ordering::SeqCst);
        if self.sender.send(op).is_err() {
            self.pending.fetch_sub(1, Ordering::SeqCst);
        }
    }

    pub fn pending_count(&self) -> usize {
        self.pending.load(Ordering::SeqCst)
    }

    pub fn cancel_current(&self) {
        let mut lock = self.active_child.lock();
        if let Some(mut child) = lock.take() {
            #[cfg(target_os = "windows")]
            {
                let pid = child.id();
                let mut kill_cmd = Command::new("taskkill");
                kill_cmd.args(["/F", "/T", "/PID", &pid.to_string()]);
                hide_console_window(&mut kill_cmd);
                let _ = kill_cmd.spawn().and_then(|mut k| k.wait());
            }
            let _ = child.kill();
        }
    }

    pub fn subscribe_status(&self) -> Receiver<TaskStatus> {
        let (tx, rx) = channel();
        self.status_subscribers.lock().push(tx);
        rx
    }

    pub fn current_status(&self) -> TaskStatus {
        self.status.lock().clone()
    }

    pub fn is_running(&self) -> bool {
        matches!(self.current_status(), TaskStatus::Running(_))
    }

    pub fn subscribe_logs(&self) -> Receiver<LogLine> {
        let (tx, rx) = channel();
        self.log_subscribers.lock().push(tx);
        rx
    }

    pub fn next_log_id(&self) -> u64 {
        self.next_log_id.fetch_add(1, Ordering::Relaxed)
    }
}
