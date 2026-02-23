use std::io::{Read, Write};
use std::process::{Command, Stdio};

fn hyper_bin() -> String {
    env!("CARGO_BIN_EXE_hyper").to_string()
}

// ========================================================================
// --stdin mode
// ========================================================================

#[test]
fn stdin_valid_source_produces_python() {
    let mut child = Command::new(hyper_bin())
        .args(["generate", "--stdin"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start hyper");

    child.stdin.take().unwrap().write_all(b"<div>Hello</div>").unwrap();
    let output = child.wait_with_output().unwrap();

    assert!(output.status.success(), "Should exit 0 for valid source");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("def Render"), "Output should contain compiled function");
    assert!(stdout.contains("@html"), "Output should contain @html decorator");
}

#[test]
fn stdin_invalid_source_exits_nonzero() {
    let mut child = Command::new(hyper_bin())
        .args(["generate", "--stdin"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start hyper");

    child.stdin.take().unwrap().write_all(b"<div>unclosed").unwrap();
    let output = child.wait_with_output().unwrap();

    assert!(!output.status.success(), "Should exit non-zero for invalid source");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("error"), "Stderr should contain error message");
}

// ========================================================================
// --json output
// ========================================================================

#[test]
fn json_output_has_compiled_and_mappings() {
    let mut child = Command::new(hyper_bin())
        .args(["generate", "--stdin", "--json"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start hyper");

    child.stdin.take().unwrap().write_all(b"<div>Hello</div>").unwrap();
    let output = child.wait_with_output().unwrap();

    assert!(output.status.success(), "Should exit 0");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .expect("Output should be valid JSON");

    assert!(json.get("compiled").is_some(), "JSON should have 'compiled' field");
    assert!(json.get("mappings").is_some(), "JSON should have 'mappings' field");

    let compiled = json["compiled"].as_str().unwrap();
    assert!(compiled.contains("def Render"), "Compiled code should contain function");
}

#[test]
fn json_error_output_has_error_field() {
    let mut child = Command::new(hyper_bin())
        .args(["generate", "--stdin", "--json"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start hyper");

    child.stdin.take().unwrap().write_all(b"<div>unclosed").unwrap();
    let output = child.wait_with_output().unwrap();

    // JSON error mode still exits non-zero but writes JSON to stdout
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .expect("Error output should be valid JSON");

    assert!(json.get("error").is_some(), "JSON error should have 'error' field");
}

// ========================================================================
// --daemon protocol
// ========================================================================

#[test]
fn daemon_ready_and_compile() {
    let mut child = Command::new(hyper_bin())
        .args(["generate", "--daemon"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to start daemon");

    let mut stdout = child.stdout.take().unwrap();
    let mut stdin = child.stdin.take().unwrap();

    // Read ready message: 4-byte length + JSON
    let mut len_buf = [0u8; 4];
    stdout.read_exact(&mut len_buf).expect("Should read ready length");
    let ready_len = u32::from_be_bytes(len_buf) as usize;
    assert!(ready_len > 0 && ready_len < 1000, "Ready message length should be reasonable");

    let mut ready_buf = vec![0u8; ready_len];
    stdout.read_exact(&mut ready_buf).expect("Should read ready payload");
    let ready_str = String::from_utf8_lossy(&ready_buf);
    assert!(ready_str.contains("ready"), "Ready message should contain 'ready'");

    // Send a compile request
    let request = r#"{"content": "<div>Hello</div>"}"#;
    let request_bytes = request.as_bytes();
    let request_len = (request_bytes.len() as u32).to_be_bytes();
    stdin.write_all(&request_len).unwrap();
    stdin.write_all(request_bytes).unwrap();
    stdin.flush().unwrap();

    // Read response: 4-byte length + JSON
    let mut resp_len_buf = [0u8; 4];
    stdout.read_exact(&mut resp_len_buf).expect("Should read response length");
    let resp_len = u32::from_be_bytes(resp_len_buf) as usize;
    assert!(resp_len > 0 && resp_len < 100_000, "Response length should be reasonable");

    let mut resp_buf = vec![0u8; resp_len];
    stdout.read_exact(&mut resp_buf).expect("Should read response payload");
    let resp_str = String::from_utf8_lossy(&resp_buf);
    let json: serde_json::Value = serde_json::from_str(&resp_str)
        .expect("Response should be valid JSON");

    assert!(json.get("compiled").is_some(), "Response should have 'compiled' field");
    let compiled = json["compiled"].as_str().unwrap();
    assert!(compiled.contains("def Render"), "Compiled code should contain function");

    // Close stdin to trigger clean shutdown
    drop(stdin);
    let status = child.wait().unwrap();
    assert!(status.success(), "Daemon should exit cleanly when stdin closes");
}
