//! End-to-end: the proxy in front of the fake server, driven over stdio.

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use serde_json::{Value as Json, json};

struct Session {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
}

impl Session {
    fn start(extra: &[&str]) -> Session {
        let exe = env!("CARGO_BIN_EXE_ldf-mcp");
        let mut child = Command::new(exe)
            .arg("proxy")
            .args(extra)
            .arg("--")
            .arg(exe)
            .arg("fake-server")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn proxy");
        let stdin = child.stdin.take().unwrap();
        let stdout = BufReader::new(child.stdout.take().unwrap());
        Session {
            child,
            stdin,
            stdout,
        }
    }

    fn send(&mut self, msg: Json) {
        let mut s = serde_json::to_string(&msg).unwrap();
        s.push('\n');
        self.stdin.write_all(s.as_bytes()).unwrap();
        self.stdin.flush().unwrap();
    }

    fn recv(&mut self) -> Json {
        let mut line = String::new();
        self.stdout.read_line(&mut line).unwrap();
        assert!(!line.is_empty(), "proxy closed its stdout");
        serde_json::from_str(line.trim_end()).expect("valid JSON line")
    }

    fn call(&mut self, id: u64, method: &str, params: Json) -> Json {
        self.send(json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}));
        let reply = self.recv();
        assert_eq!(reply["id"], id);
        reply["result"].clone()
    }

    fn handshake(&mut self) -> Json {
        let init = self.call(
            1,
            "initialize",
            json!({"protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "t", "version": "0"}}),
        );
        self.send(json!({"jsonrpc": "2.0", "method": "notifications/initialized"}));
        init
    }

    fn finish(mut self) {
        drop(self.stdin);
        let status = self.child.wait().unwrap();
        assert!(status.success(), "proxy exit {status}");
    }
}

#[test]
fn rewrites_initialize_tools_list_and_large_results() {
    let mut s = Session::start(&[]);
    let init = s.handshake();
    assert_eq!(init["protocolVersion"], "2025-06-18");
    let instructions = init["instructions"].as_str().unwrap();
    assert!(instructions.starts_with("Fake row server.\n\nTool results"));

    let list = s.call(2, "tools/list", json!({}));
    let desc = list["tools"][0]["description"].as_str().unwrap();
    assert!(desc.starts_with("Returns rows as a JSON array."));
    assert!(desc.contains("LDF"));

    let big = s.call(
        3,
        "tools/call",
        json!({"name": "query", "arguments": {"rows": 100}}),
    );
    let text = big["content"][0]["text"].as_str().unwrap();
    // serde_json's `json!` orders keys alphabetically; the proxy keeps
    // whatever order the downstream server emits.
    assert!(
        text.starts_with("@records active:b,email:s,id:i,name:s,note:s?,score:f\n"),
        "{text}"
    );
    let records = ldf::decode(text).unwrap();
    assert_eq!(records.len(), 100);
    assert_eq!(records[6].get("note"), Some(&ldf_data::Value::Null));
    assert_eq!(
        records[0].get("note").and_then(|v| v.as_str()),
        Some("row, with comma")
    );

    // Two rows are under the 1024 byte floor: still JSON.
    let small = s.call(
        4,
        "tools/call",
        json!({"name": "query", "arguments": {"rows": 2}}),
    );
    assert!(
        small["content"][0]["text"]
            .as_str()
            .unwrap()
            .starts_with("[{")
    );

    // An object holding the rows becomes a one-row root with a child table.
    let nested = s.call(
        5,
        "tools/call",
        json!({"name": "query", "arguments": {"rows": 40, "nested": true}}),
    );
    let text = nested["content"][0]["text"].as_str().unwrap();
    assert!(
        text.starts_with("@records meta.elapsed_ms:i,meta.rows:i,rows:t\n3,40,40\n@records.rows "),
        "{text}"
    );
    let back = ldf::decode(text).unwrap();
    assert_eq!(
        back[0].get("rows").and_then(|r| r.as_array()).map(Vec::len),
        Some(40)
    );

    // Unknown method: the downstream error passes through untouched.
    s.send(json!({"jsonrpc": "2.0", "id": 6, "method": "nope"}));
    let err = s.recv();
    assert_eq!(err["error"]["code"], -32601);
    s.finish();
}

#[test]
fn passthrough_and_plain_leave_everything_alone() {
    let mut s = Session::start(&["--passthrough", "--plain"]);
    let init = s.handshake();
    assert_eq!(init["instructions"], "Fake row server.");
    let list = s.call(2, "tools/list", json!({}));
    assert_eq!(
        list["tools"][0]["description"],
        "Returns rows as a JSON array."
    );
    let big = s.call(
        3,
        "tools/call",
        json!({"name": "query", "arguments": {"rows": 100}}),
    );
    assert!(
        big["content"][0]["text"]
            .as_str()
            .unwrap()
            .starts_with("[{")
    );
    s.finish();
}

#[test]
fn min_bytes_zero_converts_small_results_too() {
    let mut s = Session::start(&["--min-bytes", "0", "--quiet"]);
    s.handshake();
    let small = s.call(
        2,
        "tools/call",
        json!({"name": "query", "arguments": {"rows": 2}}),
    );
    assert!(
        small["content"][0]["text"]
            .as_str()
            .unwrap()
            .starts_with("@records ")
    );
    s.finish();
}

#[test]
fn model_without_evidence_gets_json_and_no_primer() {
    let mut s = Session::start(&["--model", "no-such-model", "--quiet"]);
    let init = s.handshake();
    assert_eq!(init["instructions"], "Fake row server.");
    let list = s.call(2, "tools/list", json!({}));
    assert_eq!(
        list["tools"][0]["description"],
        "Returns rows as a JSON array."
    );
    let big = s.call(
        3,
        "tools/call",
        json!({"name": "query", "arguments": {"rows": 100}}),
    );
    assert!(
        big["content"][0]["text"]
            .as_str()
            .unwrap()
            .starts_with("[{")
    );
    s.finish();
}

#[test]
fn measured_model_gets_ldf_on_flat_rows() {
    // Built-in evidence: Haiku 5.5 is measured on flat data at 200 records
    // with no loss against JSON.
    let mut s = Session::start(&["--model", "claude-haiku-5-5", "--quiet"]);
    let init = s.handshake();
    assert!(init["instructions"].as_str().unwrap().contains("LDF"));
    let big = s.call(
        2,
        "tools/call",
        json!({"name": "query", "arguments": {"rows": 100}}),
    );
    assert!(
        big["content"][0]["text"]
            .as_str()
            .unwrap()
            .starts_with("@records ")
    );
    s.finish();
}
