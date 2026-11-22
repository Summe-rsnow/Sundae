use nsh_app_sundae::scoop::process::{ScoopOp, TaskQueue, TaskStatus};
use std::time::Duration;

#[test]
fn test_scoop_op_args() {
    assert_eq!(ScoopOp::Install("git".into()).to_args(), vec!["install", "git"]);
    assert_eq!(
        ScoopOp::uninstall("git", false).to_args(),
        vec!["uninstall", "git"]
    );
    assert_eq!(
        ScoopOp::uninstall("git", true).to_args(),
        vec!["uninstall", "git", "--purge"]
    );
    assert_eq!(ScoopOp::Update("curl".into()).to_args(), vec!["update", "curl"]);
    assert_eq!(ScoopOp::UpdateAll.to_args(), vec!["update", "*"]);
    assert_eq!(ScoopOp::Hold("neovim".into()).to_args(), vec!["hold", "neovim"]);
    assert_eq!(ScoopOp::Unhold("neovim".into()).to_args(), vec!["unhold", "neovim"]);
    assert_eq!(
        ScoopOp::BucketAdd {
            name: "extras".into(),
            url: None
        }
        .to_args(),
        vec!["bucket", "add", "extras"]
    );
    assert_eq!(
        ScoopOp::BucketAdd {
            name: "custom".into(),
            url: Some("https://example.com/b".into())
        }
        .to_args(),
        vec!["bucket", "add", "custom", "https://example.com/b"]
    );
    assert_eq!(ScoopOp::BucketRm("extras".into()).to_args(), vec!["bucket", "rm", "extras"]);
    assert_eq!(ScoopOp::BucketUpdate.to_args(), vec!["update"]);
}

#[test]
fn test_task_queue_execution_and_status() {
    let queue = TaskQueue::new();
    assert_eq!(queue.current_status(), TaskStatus::Idle);
    assert_eq!(queue.pending_count(), 0);

    let log_rx = queue.subscribe_logs();
    let status_rx = queue.subscribe_status();
    queue.enqueue(ScoopOp::Custom(vec![
        "cmd.exe".into(),
        "/c".into(),
        "echo".into(),
        "hello_sundae".into(),
    ]));

    let mut received_hello = false;
    let mut received_success = false;
    for _ in 0..60 {
        while let Ok(line) = log_rx.try_recv() {
            assert!(line.id > 0);
            if line.text.contains("hello_sundae") {
                received_hello = true;
            }
        }
        while let Ok(st) = status_rx.try_recv() {
            if matches!(st, TaskStatus::Success(_)) {
                received_success = true;
            }
        }
        if received_hello && received_success {
            break;
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    assert!(received_hello, "Should have received hello_sundae in log stream");
    assert!(received_success, "Should have received TaskStatus::Success");
    assert_eq!(queue.pending_count(), 0);
}
