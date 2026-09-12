use std::{
    io::Write,
    process::{Command, Stdio},
};

#[test]
fn cli_and_mcp_start_without_desktop_and_deny_unapproved_reads() {
    let root = tempfile::tempdir().unwrap();
    let binary = env!("CARGO_BIN_EXE_mobius-connect");
    let init = Command::new(binary)
        .args(["--data-root", root.path().to_str().unwrap(), "init"])
        .output()
        .unwrap();
    assert!(
        init.status.success(),
        "{}",
        String::from_utf8_lossy(&init.stderr)
    );
    let mut server = Command::new(binary)
        .args(["--data-root", root.path().to_str().unwrap(), "mcp", "serve"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let requests = [
        serde_json::json!({"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18"}}),
        serde_json::json!({"jsonrpc":"2.0","method":"notifications/initialized"}),
        serde_json::json!({"jsonrpc":"2.0","id":2,"method":"tools/list"}),
        serde_json::json!({"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"list_sessions","arguments":{}}}),
        serde_json::json!({"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"read_session_range","arguments":{"session_id":"not-approved","offset":0,"bytes":100}}}),
    ];
    let mut input = server.stdin.take().unwrap();
    for request in requests {
        writeln!(input, "{request}").unwrap();
    }
    drop(input);
    let response = server.wait_with_output().unwrap();
    assert!(
        response.status.success(),
        "{}",
        String::from_utf8_lossy(&response.stderr)
    );
    let messages: Vec<serde_json::Value> = String::from_utf8(response.stdout)
        .unwrap()
        .lines()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    assert_eq!(messages.len(), 4);
    assert_eq!(messages[0]["result"]["protocolVersion"], "2025-06-18");
    assert_eq!(
        messages[2]["result"]["structuredContent"]["result"],
        serde_json::json!([])
    );
    assert_eq!(messages[3]["result"]["isError"], true);
}
