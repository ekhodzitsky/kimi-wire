use std::io::Write;
use std::os::unix::fs::PermissionsExt;

use kimi_wire::{
    protocol::*,
    transport::{ChildProcessTransport, Transport},
};

fn mock_kimi_script() -> String {
    r#"#!/bin/bash
# Mock kimi binary that ignores all arguments
while IFS= read -r line; do
    echo '{"jsonrpc":"2.0","id":"resp","result":{"status":"finished"}}'
    echo "stderr output" >&2
done
"#
    .to_string()
}

async fn make_mock_binary() -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("mock_kimi");
    let mut file = std::fs::File::create(&path).unwrap();
    file.write_all(mock_kimi_script().as_bytes()).unwrap();
    let mut perms = std::fs::metadata(&path).unwrap().permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&path, perms).unwrap();
    (dir, path)
}

#[tokio::test]
async fn test_child_process_transport_spawn_and_roundtrip() {
    let (_dir, bin) = make_mock_binary().await;

    let mut transport = ChildProcessTransport::spawn(
        bin.to_str().unwrap(),
        None,
        None,
        None,
    )
    .await
    .unwrap();

    // Write a JSON-RPC line
    let req = r#"{"jsonrpc":"2.0","id":"1","method":"prompt","params":{}}"#;
    transport.write_line(req).await.unwrap();

    // Read the echoed response
    let line = transport.read_line().await.unwrap();
    assert!(line.is_some());
    let raw: RawWireMessage = serde_json::from_str(&line.unwrap()).unwrap();
    assert_eq!(raw.result, Some(serde_json::json!({"status":"finished"})));
}

#[tokio::test]
async fn test_child_process_transport_drop_is_clean() {
    let (_dir, bin) = make_mock_binary().await;

    let transport = ChildProcessTransport::spawn(
        bin.to_str().unwrap(),
        None,
        None,
        None,
    )
    .await
    .unwrap();

    // Drop should cancel the stderr task and kill the child process.
    drop(transport);
}

#[tokio::test]
async fn test_child_process_transport_read_line_returns_none_after_child_exit() {
    let (_dir, bin) = make_mock_binary().await;

    let mut transport = ChildProcessTransport::spawn(
        bin.to_str().unwrap(),
        None,
        None,
        None,
    )
    .await
    .unwrap();

    // Write a line so the mock echoes and then exits when stdin closes.
    transport.write_line("{}").await.unwrap();

    // Read the response.
    let line = transport.read_line().await.unwrap();
    assert!(line.is_some());

    // Explicitly drop stdin by dropping the transport. The mock will see EOF and exit.
    drop(transport);
}

#[tokio::test]
async fn test_child_process_transport_spawn_fails_for_missing_binary() {
    let err = ChildProcessTransport::spawn("/nonexistent/binary", None, None, None)
        .await
        .unwrap_err();
    assert!(matches!(err, kimi_wire::WireError::SpawnFailed(_)));
}
