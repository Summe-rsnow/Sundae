use parking_lot::Mutex;
use std::io::{BufRead, BufReader};
use std::process::{Child, Command, Stdio};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScoopOp {
    Install(String),
    Uninstall(String),
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
    pub fn to_args(&self) -> Vec<String> {
        match self {
            ScoopOp::Install(name) => vec!["install".into(), name.clone()],
            ScoopOp::Uninstall(name) => vec!["uninstall".into(), name.clone()],
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
            ScoopOp::Install(n) => format!("Install {}", n),
            ScoopOp::Uninstall(n) => format!("Uninstall {}", n),
            ScoopOp::Update(n) => format!("Update {}", n),
            ScoopOp::UpdateAll => "Update All Packages".to_string(),
            ScoopOp::Hold(n) => format!("Hold {}", n),
            ScoopOp::Unhold(n) => format!("Unhold {}", n),
            ScoopOp::BucketAdd { name, .. } => format!("Add Bucket {}", name),
            ScoopOp::BucketRm(name) => format!("Remove Bucket {}", name),
            ScoopOp::BucketUpdate => "Sync Buckets".to_string(),
            ScoopOp::Custom(args) => args.join(" "),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogLine {
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
}

impl Default for TaskQueue {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskQueue {
    pub fn new() -> Self {
        let (sender, receiver) = channel::<ScoopOp>();
        let log_subscribers = Arc::new(Mutex::new(Vec::<Sender<LogLine>>::new()));
        let status_subscribers = Arc::new(Mutex::new(Vec::<Sender<TaskStatus>>::new()));
        let status = Arc::new(Mutex::new(TaskStatus::Idle));
        let active_child = Arc::new(Mutex::new(None));

        let worker_subscribers = Arc::clone(&log_subscribers);
        let worker_status_subs = Arc::clone(&status_subscribers);
        let worker_status = Arc::clone(&status);
        let worker_child = Arc::clone(&active_child);
        thread::spawn(move || {
            while let Ok(op) = receiver.recv() {
                let broadcast_status = |st: TaskStatus| {
                    *worker_status.lock() = st.clone();
                    let mut subs = worker_status_subs.lock();
                    subs.retain(|sub| sub.send(st.clone()).is_ok());
                };

                broadcast_status(TaskStatus::Running(op.clone()));

                let broadcast = |text: String, is_err: bool| {
                    let mut subs = worker_subscribers.lock();
                    subs.retain(|sub| sub.send(LogLine { text: text.clone(), is_err }).is_ok());
                };
                broadcast(format!(">>> Starting: {}", op.display_name()), false);

                let mut cmd = match &op {
                    ScoopOp::Custom(args) => {
                        if args.is_empty() {
                            *worker_status.lock() = TaskStatus::Success(op);
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
                        c.args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", "scoop"]);
                        c.args(op.to_args());
                        c
                    }
                };

                cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

                let mut child = match cmd.spawn() {
                    Ok(c) => c,
                    Err(e) => {
                        let err_msg = format!("Failed to spawn process: {}", e);
                        broadcast(err_msg.clone(), true);
                        *worker_status.lock() = TaskStatus::Failed { op, error: err_msg };
                        continue;
                    }
                };

                let stdout = child.stdout.take();
                let stderr = child.stderr.take();

                *worker_child.lock() = Some(child);

                let log_subs_out = Arc::clone(&worker_subscribers);
                let stdout_thread = thread::spawn(move || {
                    if let Some(out) = stdout {
                        let reader = BufReader::new(out);
                        for line in reader.lines().map_while(Result::ok) {
                            let mut subs = log_subs_out.lock();
                            subs.retain(|sub| sub.send(LogLine { text: line.clone(), is_err: false }).is_ok());
                        }
                    }
                });

                let log_subs_err = Arc::clone(&worker_subscribers);
                let stderr_thread = thread::spawn(move || {
                    if let Some(err) = stderr {
                        let reader = BufReader::new(err);
                        for line in reader.lines().map_while(Result::ok) {
                            let mut subs = log_subs_err.lock();
                            subs.retain(|sub| sub.send(LogLine { text: line.clone(), is_err: true }).is_ok());
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

                match exit_status {
                    Some(s) if s.success() => {
                        broadcast(format!("✓ Completed: {}", op.display_name()), false);
                        broadcast_status(TaskStatus::Success(op));
                    }
                    Some(s) => {
                        let msg = format!("✖ Failed with exit code: {}", s.code().unwrap_or(-1));
                        broadcast(msg.clone(), true);
                        broadcast_status(TaskStatus::Failed { op, error: msg });
                    }
                    None => {
                        let msg = "Process terminated or cancelled".to_string();
                        broadcast(msg.clone(), true);
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
        }
    }

    pub fn enqueue(&self, op: ScoopOp) {
        let _ = self.sender.send(op);
    }

    pub fn cancel_current(&self) {
        let mut lock = self.active_child.lock();
        if let Some(mut child) = lock.take() {
            #[cfg(target_os = "windows")]
            {
                let pid = child.id();
                let _ = Command::new("taskkill")
                    .args(["/F", "/T", "/PID", &pid.to_string()])
                    .spawn()
                    .and_then(|mut k| k.wait());
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
}
