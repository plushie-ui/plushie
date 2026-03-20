//! Integration test: verify session multiplexing in mock mode.
//!
//! Spawns `toddy --mock --max-sessions 4 --json` as a subprocess,
//! sends interleaved messages with different session IDs, and verifies
//! that responses come back tagged with the correct session.

use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::time::Duration;

fn send(stdin: &mut impl Write, msg: &serde_json::Value) {
    let line = serde_json::to_string(msg).unwrap();
    writeln!(stdin, "{line}").unwrap();
    stdin.flush().unwrap();
}

fn recv(reader: &mut impl BufRead) -> serde_json::Value {
    let mut line = String::new();
    reader.read_line(&mut line).unwrap();
    serde_json::from_str(line.trim()).unwrap()
}

fn toddy_binary() -> String {
    // The integration test binary is in target/debug/deps. The toddy
    // binary is in target/debug.
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // remove test binary name
    path.pop(); // remove deps/
    path.push("toddy");
    path.to_string_lossy().to_string()
}

#[test]
fn hello_message_has_empty_session() {
    let mut child = Command::new(toddy_binary())
        .args(["--mock", "--json"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn toddy");

    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());

    // Send initial settings to trigger hello.
    send(
        &mut stdin,
        &serde_json::json!({"session": "s1", "type": "settings", "settings": {}}),
    );

    let hello = recv(&mut stdout);
    assert_eq!(hello["type"], "hello");
    assert_eq!(hello["session"], "");

    drop(stdin);
    child.wait().unwrap();
}

#[test]
fn single_session_echoes_session_id() {
    let mut child = Command::new(toddy_binary())
        .args(["--mock", "--json"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn toddy");

    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());

    send(
        &mut stdin,
        &serde_json::json!({"session": "test_1", "type": "settings", "settings": {}}),
    );
    let _hello = recv(&mut stdout);

    // Send a reset and verify session is echoed.
    send(
        &mut stdin,
        &serde_json::json!({"session": "test_1", "type": "reset", "id": "r1"}),
    );
    let resp = recv(&mut stdout);
    assert_eq!(resp["type"], "reset_response");
    assert_eq!(resp["session"], "test_1");
    assert_eq!(resp["id"], "r1");

    drop(stdin);
    child.wait().unwrap();
}

#[test]
fn multiplexed_sessions_are_isolated() {
    let mut child = Command::new(toddy_binary())
        .args(["--mock", "--max-sessions", "4", "--json"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn toddy");

    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());

    // Consume hello.
    send(
        &mut stdin,
        &serde_json::json!({"session": "s1", "type": "settings", "settings": {}}),
    );
    let _hello = recv(&mut stdout);

    // Send snapshots to two different sessions with different trees.
    send(
        &mut stdin,
        &serde_json::json!({
            "session": "s1",
            "type": "snapshot",
            "tree": {"id": "root", "type": "text", "props": {"content": "session one"}, "children": []}
        }),
    );
    send(
        &mut stdin,
        &serde_json::json!({
            "session": "s2",
            "type": "snapshot",
            "tree": {"id": "root", "type": "text", "props": {"content": "session two"}, "children": []}
        }),
    );

    // Query tree from each session -- they should have different content.
    send(
        &mut stdin,
        &serde_json::json!({
            "session": "s1",
            "type": "query",
            "id": "q1",
            "target": "tree",
            "selector": {}
        }),
    );
    send(
        &mut stdin,
        &serde_json::json!({
            "session": "s2",
            "type": "query",
            "id": "q2",
            "target": "tree",
            "selector": {}
        }),
    );

    // Collect both responses (order may vary due to threading).
    let r1 = recv(&mut stdout);
    let r2 = recv(&mut stdout);

    let mut responses: std::collections::HashMap<String, serde_json::Value> =
        std::collections::HashMap::new();
    responses.insert(r1["session"].as_str().unwrap().to_string(), r1);
    responses.insert(r2["session"].as_str().unwrap().to_string(), r2);

    let s1_tree = &responses["s1"];
    assert_eq!(s1_tree["type"], "query_response");
    assert_eq!(s1_tree["id"], "q1");
    assert_eq!(s1_tree["data"]["props"]["content"], "session one");

    let s2_tree = &responses["s2"];
    assert_eq!(s2_tree["type"], "query_response");
    assert_eq!(s2_tree["id"], "q2");
    assert_eq!(s2_tree["data"]["props"]["content"], "session two");

    drop(stdin);
    child.wait().unwrap();
}

#[test]
fn reset_tears_down_session() {
    let mut child = Command::new(toddy_binary())
        .args(["--mock", "--max-sessions", "4", "--json"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn toddy");

    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());

    send(
        &mut stdin,
        &serde_json::json!({"session": "s1", "type": "settings", "settings": {}}),
    );
    let _hello = recv(&mut stdout);

    // Create a session, send a tree, reset it.
    send(
        &mut stdin,
        &serde_json::json!({
            "session": "s1",
            "type": "snapshot",
            "tree": {"id": "root", "type": "text", "props": {"content": "before"}, "children": []}
        }),
    );
    send(
        &mut stdin,
        &serde_json::json!({"session": "s1", "type": "reset", "id": "r1"}),
    );

    let reset_resp = recv(&mut stdout);
    assert_eq!(reset_resp["type"], "reset_response");
    assert_eq!(reset_resp["session"], "s1");

    // Reuse the same session ID -- should get a fresh session.
    send(
        &mut stdin,
        &serde_json::json!({
            "session": "s1",
            "type": "query",
            "id": "q1",
            "target": "tree",
            "selector": {}
        }),
    );

    let tree_resp = recv(&mut stdout);
    assert_eq!(tree_resp["session"], "s1");
    // Tree should be null (fresh session, no snapshot sent).
    assert!(tree_resp["data"].is_null());

    drop(stdin);
    child.wait().unwrap();
}

#[test]
fn headless_interact_step_round_trip() {
    let mut child = Command::new(toddy_binary())
        .args(["--headless", "--json"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn toddy");

    let mut stdin = child.stdin.take().unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());

    // Bootstrap: settings + hello.
    send(
        &mut stdin,
        &serde_json::json!({"session": "s1", "type": "settings", "settings": {}}),
    );
    let _hello = recv(&mut stdout);

    // Send a tree with a button.
    send(
        &mut stdin,
        &serde_json::json!({
            "session": "s1",
            "type": "snapshot",
            "tree": {
                "id": "root", "type": "column", "props": {}, "children": [
                    {"id": "btn1", "type": "button", "props": {"label": "Click me"}, "children": []}
                ]
            }
        }),
    );

    // Click the button. In headless mode, this injects CursorMoved +
    // ButtonPressed + ButtonReleased. The ButtonReleased should
    // produce a Click message, emitted as an interact_step.
    send(
        &mut stdin,
        &serde_json::json!({
            "session": "s1",
            "type": "interact",
            "id": "i1",
            "action": "click",
            "selector": {"by": "id", "value": "btn1"},
            "payload": {}
        }),
    );

    // We should receive an interact_step with the click event.
    let step = recv(&mut stdout);
    assert_eq!(step["type"], "interact_step");
    assert_eq!(step["session"], "s1");
    assert_eq!(step["id"], "i1");
    assert!(step["events"].is_array());
    let events = step["events"].as_array().unwrap();
    assert!(!events.is_empty(), "interact_step should have events");
    assert_eq!(events[0]["family"], "click");
    assert_eq!(events[0]["id"], "btn1");

    // Send the snapshot back (the renderer is blocked waiting for it).
    send(
        &mut stdin,
        &serde_json::json!({
            "session": "s1",
            "type": "snapshot",
            "tree": {
                "id": "root", "type": "column", "props": {}, "children": [
                    {"id": "btn1", "type": "button", "props": {"label": "Clicked!"}, "children": []}
                ]
            }
        }),
    );

    // The final interact_response should arrive with empty events
    // (the click was already delivered via the step).
    let resp = recv(&mut stdout);
    assert_eq!(resp["type"], "interact_response");
    assert_eq!(resp["session"], "s1");
    assert_eq!(resp["id"], "i1");
    assert!(resp["events"].as_array().unwrap().is_empty());

    drop(stdin);
    child.wait().unwrap();
}

// ---------------------------------------------------------------------------
// LineReceiver: background thread reads stdout lines into an mpsc channel,
// enabling recv_timeout so tests don't hang forever on protocol bugs.
// ---------------------------------------------------------------------------

struct LineReceiver {
    rx: mpsc::Receiver<serde_json::Value>,
}

impl LineReceiver {
    fn new(reader: BufReader<std::process::ChildStdout>) -> Self {
        let (tx, rx) = mpsc::channel();
        std::thread::spawn(move || {
            let mut reader = reader;
            loop {
                let mut line = String::new();
                match reader.read_line(&mut line) {
                    Ok(0) | Err(_) => break, // EOF or error
                    Ok(_) => {
                        if let Ok(val) = serde_json::from_str::<serde_json::Value>(line.trim())
                            && tx.send(val).is_err()
                        {
                            break;
                        }
                    }
                }
            }
        });
        Self { rx }
    }

    fn recv_timeout(&self, timeout: Duration) -> serde_json::Value {
        self.rx
            .recv_timeout(timeout)
            .expect("timed out waiting for response from toddy")
    }
}

// ---------------------------------------------------------------------------
// Item 4+17: Widget render structural verification via mock interact tests
// ---------------------------------------------------------------------------

#[test]
fn mock_text_input_emits_input_event() {
    let mut child = Command::new(toddy_binary())
        .args(["--mock", "--json"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn toddy");

    let mut stdin = child.stdin.take().unwrap();
    let receiver = LineReceiver::new(BufReader::new(child.stdout.take().unwrap()));
    let timeout = Duration::from_secs(10);

    // Bootstrap.
    send(
        &mut stdin,
        &serde_json::json!({"session": "s1", "type": "settings", "settings": {}}),
    );
    let hello = receiver.recv_timeout(timeout);
    assert_eq!(hello["type"], "hello");

    // Send a tree with a text_input widget.
    send(
        &mut stdin,
        &serde_json::json!({
            "session": "s1",
            "type": "snapshot",
            "tree": {
                "id": "root", "type": "column", "props": {}, "children": [
                    {"id": "inp1", "type": "text_input", "props": {"value": "", "placeholder": "Type here"}, "children": []}
                ]
            }
        }),
    );

    // Interact: type_text on the text_input.
    send(
        &mut stdin,
        &serde_json::json!({
            "session": "s1",
            "type": "interact",
            "id": "i1",
            "action": "type_text",
            "selector": {"by": "id", "value": "inp1"},
            "payload": {"text": "hello"}
        }),
    );

    let resp = receiver.recv_timeout(timeout);
    assert_eq!(resp["type"], "interact_response");
    assert_eq!(resp["session"], "s1");
    assert_eq!(resp["id"], "i1");

    let events = resp["events"]
        .as_array()
        .expect("events should be an array");
    assert_eq!(events.len(), 1, "expected exactly one event");
    assert_eq!(events[0]["family"], "input");
    assert_eq!(events[0]["id"], "inp1");
    assert_eq!(events[0]["value"], "hello");

    drop(stdin);
    child.wait().unwrap();
}

#[test]
fn mock_checkbox_emits_toggle_event() {
    let mut child = Command::new(toddy_binary())
        .args(["--mock", "--json"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn toddy");

    let mut stdin = child.stdin.take().unwrap();
    let receiver = LineReceiver::new(BufReader::new(child.stdout.take().unwrap()));
    let timeout = Duration::from_secs(10);

    send(
        &mut stdin,
        &serde_json::json!({"session": "s1", "type": "settings", "settings": {}}),
    );
    let hello = receiver.recv_timeout(timeout);
    assert_eq!(hello["type"], "hello");

    // Send a tree with a checkbox widget.
    send(
        &mut stdin,
        &serde_json::json!({
            "session": "s1",
            "type": "snapshot",
            "tree": {
                "id": "root", "type": "column", "props": {}, "children": [
                    {"id": "chk1", "type": "checkbox", "props": {"label": "Accept", "checked": false}, "children": []}
                ]
            }
        }),
    );

    // Interact: toggle the checkbox.
    send(
        &mut stdin,
        &serde_json::json!({
            "session": "s1",
            "type": "interact",
            "id": "i1",
            "action": "toggle",
            "selector": {"by": "id", "value": "chk1"},
            "payload": {"value": true}
        }),
    );

    let resp = receiver.recv_timeout(timeout);
    assert_eq!(resp["type"], "interact_response");
    assert_eq!(resp["session"], "s1");
    assert_eq!(resp["id"], "i1");

    let events = resp["events"]
        .as_array()
        .expect("events should be an array");
    assert_eq!(events.len(), 1, "expected exactly one event");
    assert_eq!(events[0]["family"], "toggle");
    assert_eq!(events[0]["id"], "chk1");
    assert_eq!(events[0]["value"], true);

    drop(stdin);
    child.wait().unwrap();
}

#[test]
fn mock_slider_emits_slide_event() {
    let mut child = Command::new(toddy_binary())
        .args(["--mock", "--json"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn toddy");

    let mut stdin = child.stdin.take().unwrap();
    let receiver = LineReceiver::new(BufReader::new(child.stdout.take().unwrap()));
    let timeout = Duration::from_secs(10);

    send(
        &mut stdin,
        &serde_json::json!({"session": "s1", "type": "settings", "settings": {}}),
    );
    let hello = receiver.recv_timeout(timeout);
    assert_eq!(hello["type"], "hello");

    // Send a tree with a slider widget.
    send(
        &mut stdin,
        &serde_json::json!({
            "session": "s1",
            "type": "snapshot",
            "tree": {
                "id": "root", "type": "column", "props": {}, "children": [
                    {"id": "sld1", "type": "slider", "props": {"value": 50, "range": [0, 100]}, "children": []}
                ]
            }
        }),
    );

    // Interact: slide to a new value.
    send(
        &mut stdin,
        &serde_json::json!({
            "session": "s1",
            "type": "interact",
            "id": "i1",
            "action": "slide",
            "selector": {"by": "id", "value": "sld1"},
            "payload": {"value": 75.0}
        }),
    );

    let resp = receiver.recv_timeout(timeout);
    assert_eq!(resp["type"], "interact_response");
    assert_eq!(resp["session"], "s1");
    assert_eq!(resp["id"], "i1");

    let events = resp["events"]
        .as_array()
        .expect("events should be an array");
    assert_eq!(events.len(), 1, "expected exactly one event");
    assert_eq!(events[0]["family"], "slide");
    assert_eq!(events[0]["id"], "sld1");
    assert_eq!(events[0]["value"], 75.0);

    drop(stdin);
    child.wait().unwrap();
}

// ---------------------------------------------------------------------------
// Item 6: Concurrent session stress test
// ---------------------------------------------------------------------------

#[test]
fn concurrent_sessions_interleaved() {
    let mut child = Command::new(toddy_binary())
        .args(["--mock", "--max-sessions", "4", "--json"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn toddy");

    let mut stdin = child.stdin.take().unwrap();
    let receiver = LineReceiver::new(BufReader::new(child.stdout.take().unwrap()));
    let timeout = Duration::from_secs(10);

    // Bootstrap.
    send(
        &mut stdin,
        &serde_json::json!({"session": "s1", "type": "settings", "settings": {}}),
    );
    let hello = receiver.recv_timeout(timeout);
    assert_eq!(hello["type"], "hello");

    let session_ids = ["s1", "s2", "s3", "s4"];

    // Send snapshots to all 4 sessions interleaved.
    for &sid in &session_ids {
        send(
            &mut stdin,
            &serde_json::json!({
                "session": sid,
                "type": "snapshot",
                "tree": {
                    "id": "root",
                    "type": "text",
                    "props": {"content": format!("content-{sid}")},
                    "children": []
                }
            }),
        );
    }

    // Query each session's tree.
    for (i, &sid) in session_ids.iter().enumerate() {
        send(
            &mut stdin,
            &serde_json::json!({
                "session": sid,
                "type": "query",
                "id": format!("q{}", i + 1),
                "target": "tree",
                "selector": {}
            }),
        );
    }

    // Collect all 4 responses (order may vary due to threading).
    let mut responses: std::collections::HashMap<String, serde_json::Value> =
        std::collections::HashMap::new();
    for _ in 0..4 {
        let resp = receiver.recv_timeout(timeout);
        let session = resp["session"].as_str().unwrap().to_string();
        responses.insert(session, resp);
    }

    // Verify each response has the correct session ID and content.
    for &sid in &session_ids {
        let resp = responses
            .get(sid)
            .unwrap_or_else(|| panic!("missing response for session {sid}"));
        assert_eq!(resp["type"], "query_response");
        assert_eq!(
            resp["data"]["props"]["content"],
            format!("content-{sid}"),
            "session {sid} should have its own tree content"
        );
    }

    drop(stdin);
    child.wait().unwrap();
}
