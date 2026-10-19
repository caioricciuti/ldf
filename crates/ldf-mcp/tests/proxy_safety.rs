//! The proxy under failure: replies out of order, server requests that
//! reuse a client id, lines that are not JSON or not UTF-8, structured
//! content, multi-megabyte results, a crashing child and a child that
//! ignores end of input. In every case the client gets either the rewrite
//! or the original bytes, never a hang or a mangled message.

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::time::{Duration, Instant};

use serde_json::{Value as Json, json};

struct Session {
    child: Child,
    stdin: Option<ChildStdin>,
    stdout: BufReader<ChildStdout>,
}

impl Session {
    fn start(server_args: &[&str]) -> Session {
        let exe = env!("CARGO_BIN_EXE_ldf-mcp");
        let mut child = Command::new(exe)
            .args(["proxy", "--quiet", "--"])
            .arg(exe)
            .arg("fake-server")
            .args(server_args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn proxy");
        let stdin = child.stdin.take();
        let stdout = BufReader::new(child.stdout.take().unwrap());
        let mut s = Session {
            child,
            stdin,
            stdout,
        };
        s.send(json!({"jsonrpc": "2.0", "id": 0, "method": "initialize", "params": {"protocolVersion": "2025-06-18", "capabilities": {}, "clientInfo": {"name": "t", "version": "0"}}}));
        assert_eq!(s.recv()["id"], 0);
        s.send(json!({"jsonrpc": "2.0", "method": "notifications/initialized"}));
        s
    }

    fn send(&mut self, msg: Json) {
        let mut s = serde_json::to_string(&msg).unwrap();
        s.push('\n');
        let w = self.stdin.as_mut().unwrap();
        w.write_all(s.as_bytes()).unwrap();
        w.flush().unwrap();
    }

    fn call(&mut self, id: u64, args: Json) {
        self.send(json!({"jsonrpc": "2.0", "id": id, "method": "tools/call", "params": {"name": "query", "arguments": args}}));
    }

    /// One line as raw bytes, without the line end; empty at end of stream.
    fn recv_raw(&mut self) -> Vec<u8> {
        let mut line = Vec::new();
        self.stdout.read_until(b'\n', &mut line).unwrap();
        while line.last().is_some_and(|b| *b == b'\n' || *b == b'\r') {
            line.pop();
        }
        line
    }

    fn recv(&mut self) -> Json {
        let line = self.recv_raw();
        assert!(!line.is_empty(), "proxy closed its stdout");
        serde_json::from_slice(&line).expect("valid JSON line")
    }

    fn close_and_wait(mut self) -> (i32, Duration) {
        drop(self.stdin.take());
        let start = Instant::now();
        let status = self.child.wait().unwrap();
        (status.code().unwrap_or(-1), start.elapsed())
    }
}

fn text(reply: &Json) -> &str {
    reply["result"]["content"][0]["text"].as_str().unwrap()
}

#[test]
fn replies_that_overtake_each_other_are_each_rewritten() {
    let mut s = Session::start(&[]);
    s.call(10, json!({"rows": 100, "delay_ms": 400}));
    s.call(11, json!({"rows": 100}));
    s.call(12, json!({"rows": 60, "delay_ms": 150}));
    let ids: Vec<Json> = (0..3)
        .map(|_| {
            let r = s.recv();
            assert!(text(&r).starts_with("@records "), "{r}");
            r["id"].clone()
        })
        .collect();
    assert_eq!(ids, vec![json!(11), json!(12), json!(10)]);
    assert_eq!(s.close_and_wait().0, 0);
}

#[test]
fn a_server_request_reusing_a_client_id_is_forwarded_and_the_reply_still_rewritten() {
    let mut s = Session::start(&[]);
    s.call(20, json!({"rows": 100, "server_request": true}));
    let req = s.recv();
    assert_eq!(
        req,
        json!({"jsonrpc": "2.0", "id": 20, "method": "roots/list"})
    );
    let reply = s.recv();
    assert_eq!(reply["id"], 20);
    assert!(text(&reply).starts_with("@records "), "{reply}");
    assert_eq!(s.close_and_wait().0, 0);
}

#[test]
fn lines_that_are_not_json_or_not_utf8_pass_through_byte_for_byte() {
    let mut s = Session::start(&[]);
    s.call(30, json!({"rows": 100, "garbage": true}));
    assert_eq!(s.recv_raw(), b"not json at all");
    assert_eq!(s.recv_raw(), vec![0xff, 0xfe, b'{', b'}']);
    let reply = s.recv();
    assert!(text(&reply).starts_with("@records "), "{reply}");
    assert_eq!(s.close_and_wait().0, 0);
}

#[test]
fn structured_content_is_left_exactly_as_the_server_sent_it() {
    let mut s = Session::start(&[]);
    s.call(40, json!({"rows": 100, "structured": true}));
    let reply = s.recv();
    assert!(text(&reply).starts_with("@records "), "{reply}");
    let rows = reply["result"]["structuredContent"]["rows"]
        .as_array()
        .unwrap();
    assert_eq!(rows.len(), 100);
    assert_eq!(rows[0]["email"], "user.1@example.com");
    assert_eq!(s.close_and_wait().0, 0);
}

#[test]
fn a_multi_megabyte_result_is_rewritten_and_decodes_back() {
    let mut s = Session::start(&[]);
    s.call(50, json!({"rows": 50_000}));
    let reply = s.recv();
    let t = text(&reply);
    assert!(t.starts_with("@records "));
    assert_eq!(ldf::decode(t).unwrap().len(), 50_000);
    assert_eq!(s.close_and_wait().0, 0);
}

#[test]
fn a_crashing_child_ends_the_proxy_with_its_exit_code() {
    let mut s = Session::start(&[]);
    s.call(60, json!({"crash": true}));
    assert!(s.recv_raw().is_empty(), "no reply after the crash");
    let (code, _) = s.close_and_wait();
    assert_eq!(code, 3);
}

#[test]
fn a_child_that_ignores_end_of_input_is_stopped_after_the_grace_period() {
    let s = Session::start(&["--ignore-eof"]);
    let (_, took) = s.close_and_wait();
    // The fake server would stay up for 60 s; the grace period is 5 s.
    assert!(took < Duration::from_secs(15), "took {took:?}");
}

#[test]
fn the_servers_answer_bad_lines_with_parse_errors_and_keep_going() {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../../fixtures/small");
    let mut child = Command::new(env!("CARGO_BIN_EXE_ldf-mcp"))
        .args(["files", "--root", root, "--quiet"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    stdin
        .write_all(b"not json\n\xff\xfe\n{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"ping\"}\n")
        .unwrap();
    drop(stdin);
    let out = child.wait_with_output().unwrap();
    let lines: Vec<Json> = out
        .stdout
        .split(|b| *b == b'\n')
        .filter(|l| !l.is_empty())
        .map(|l| serde_json::from_slice(l).unwrap())
        .collect();
    assert_eq!(lines.len(), 3, "{lines:?}");
    assert_eq!(lines[0]["error"]["code"], -32700);
    assert_eq!(lines[1]["error"]["code"], -32700);
    assert_eq!(lines[2]["id"], 1);
    assert!(out.status.success());
}
