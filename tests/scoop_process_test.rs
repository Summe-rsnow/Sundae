use nsh_app_sundae::scoop::process::{ScoopOp, TaskQueue, TaskStatus};
use std::time::Duration;

#[test]
fn test_scoop_op_args() {
    assert_eq!(ScoopOp::Install("git".into()).to_args(), vec!["install", "git"]);
    assert_eq!(ScoopOp::Uninstall("git".into()).to_args(), vec!["uninstall", "git"]);
    assert_eq!(ScoopOp::Update("curl".into()).to_args(), vec!["update", "curl"]);
    assert_eq!(ScoopOp::UpdateAll.to_args(), vec!["update", "*"]);
    assert_eq!(ScoopOp::Hold("neovim".into()).to_args(), vec!["hold", "neovim"]);
    assert_eq!(ScoopOp::Unhold("neovim".into()).to_args(), vec!["unhold", "neovim"]);
    assert_eq!(
        ScoopOp::BucketAdd { name: "extras".into(), url: None }.to_args(),
        vec!["bucket", "add", "extras"]
    );
    assert_eq!(
        ScoopOp::BucketAdd { name: "custom".into(), url: Some("https://example.com/b".into()) }.to_args(),
        vec!["bucket", "add", "custom", "https://example.com/b"]
    );
    assert_eq!(ScoopOp::BucketUpdate.to_args(), vec!["update"]);
}

#[test]
fn test_task_queue_execution_and_status() {
    let queue = TaskQueue::new();
    assert_eq!(queue.current_status(), TaskStatus::Idle);

    // Enqueue a simulated custom test op or echo
    let receiver = queue.subscribe_logs();
    queue.enqueue(ScoopOp::Custom(vec![
        "cmd.exe".into(),
        "/c".into(),
        "echo".into(),
        "hello_sundae".into(),
    ]));

    // Wait briefly for worker to pick up and finish
    let mut received_hello = false;
    for _ in 0..50 {
        if let Ok(line) = receiver.try_recv() {
            if line.text.contains("hello_sundae") {
                received_hello = true;
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(50));
    }

    assert!(received_hello, "Should have received hello_sundae in log stream");
}
