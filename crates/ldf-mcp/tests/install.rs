//! `install`, `uninstall` and `doctor` against a stand-in Claude Code: a
//! temporary HOME with a `.claude.json`, and this binary playing the
//! `claude mcp remove|add-json` CLI (`LDF_FAKE_CLAUDE=1`).

use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

use serde_json::{Value as Json, json};

fn exe() -> &'static str {
    env!("CARGO_BIN_EXE_ldf-mcp")
}

struct World {
    root: PathBuf,
    home: PathBuf,
    project: PathBuf,
    server: PathBuf,
}

impl World {
    fn new(tag: &str) -> World {
        let root = std::env::temp_dir().join(format!("ldf-install-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let home = root.join("home");
        let project = root.join("project");
        std::fs::create_dir_all(&home).unwrap();
        std::fs::create_dir_all(&project).unwrap();
        let project = project.canonicalize().unwrap();
        // A downstream server under a name that is not ldf-mcp.
        let server = root.join(if cfg!(windows) {
            "rows-server.exe"
        } else {
            "rows-server"
        });
        std::fs::copy(exe(), &server).unwrap();
        let w = World {
            root,
            home,
            project,
            server,
        };
        w.write_config(&w.original());
        w
    }

    fn original(&self) -> Json {
        json!({
            "numStartups": 3,
            "mcpServers": {"user-rows": {"type": "stdio", "command": self.server.to_string_lossy(), "args": ["fake-server"]}},
            "projects": {self.project.to_string_lossy(): {
                "allowedTools": [],
                "mcpServers": {
                    "rows": {"type": "stdio", "command": self.server.to_string_lossy(), "args": ["fake-server"], "env": {}},
                    "secret": {"type": "stdio", "command": "srv", "args": [], "env": {"API_KEY": "abc"}},
                    "remote": {"type": "http", "url": "https://example.invalid/mcp"}
                }
            }}
        })
    }

    fn config(&self) -> Json {
        serde_json::from_slice(&std::fs::read(self.home.join(".claude.json")).unwrap()).unwrap()
    }

    fn write_config(&self, v: &Json) {
        std::fs::write(self.home.join(".claude.json"), v.to_string()).unwrap();
    }

    fn run(&self, args: &[&str], fail_on: Option<&str>) -> Output {
        let mut c = Command::new(exe());
        c.args(args)
            .current_dir(&self.project)
            .env("HOME", &self.home)
            .env("USERPROFILE", &self.home)
            .env("LDF_STATE_DIR", self.root.join("state"))
            .env("LDF_STATS", "0")
            .env("LDF_CLAUDE", exe())
            .env("LDF_FAKE_CLAUDE", "1");
        if let Some(f) = fail_on {
            c.env("LDF_FAKE_CLAUDE_FAIL", f);
        }
        c.output().unwrap()
    }

    fn ok(&self, args: &[&str]) -> String {
        let out = self.run(args, None);
        let text = String::from_utf8_lossy(&out.stdout).into_owned();
        assert!(
            out.status.success(),
            "{args:?}: {text}{}",
            String::from_utf8_lossy(&out.stderr)
        );
        text
    }

    fn local(&self, name: &str) -> Json {
        self.config()["projects"][self.project.to_string_lossy().as_ref()]["mcpServers"][name]
            .clone()
    }
}

impl Drop for World {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

/// Starts an MCP entry the way an agent would and asks for 100 rows.
fn call_through(entry: &Json, dir: &Path) -> String {
    let args: Vec<String> = entry["args"]
        .as_array()
        .unwrap()
        .iter()
        .map(|a| a.as_str().unwrap().to_string())
        .collect();
    let mut child = Command::new(entry["command"].as_str().unwrap())
        .args(&args)
        .current_dir(dir)
        .env("LDF_STATS", "0")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let mut stdin = child.stdin.take().unwrap();
    let mut out = BufReader::new(child.stdout.take().unwrap());
    let call = json!({"jsonrpc": "2.0", "id": 1, "method": "tools/call", "params": {"name": "query", "arguments": {"rows": 100}}});
    writeln!(stdin, "{call}").unwrap();
    let mut line = String::new();
    out.read_line(&mut line).unwrap();
    drop(stdin);
    child.wait().unwrap();
    let reply: Json = serde_json::from_str(&line).unwrap();
    reply["result"]["content"][0]["text"]
        .as_str()
        .unwrap()
        .to_string()
}

#[test]
fn the_plan_alone_changes_nothing() {
    let w = World::new("plan");
    let text = w.ok(&["install", "claude-code", "--wrap-all", "--add-files", "."]);
    assert!(text.contains("rows (local): can be wrapped"), "{text}");
    assert!(
        text.contains("secret (local): left alone: has env variables"),
        "{text}"
    );
    assert!(
        text.contains("remote (local): left alone: http server"),
        "{text}"
    );
    assert!(text.contains("Would make 3 change(s)"), "{text}");
    assert!(!text.contains("abc"), "env values are never printed");
    assert_eq!(w.config(), w.original());
    assert!(!w.root.join("state/installed.json").exists());
}

#[test]
fn install_wraps_serves_ldf_is_idempotent_and_uninstall_restores_exactly() {
    let w = World::new("apply");
    let text = w.ok(&[
        "install",
        "claude-code",
        "--wrap-all",
        "--add-files",
        ".",
        "--apply",
    ]);
    assert!(
        text.contains("done: rows") && text.contains("done: ldf-files"),
        "{text}"
    );
    let rows = w.local("rows");
    assert_eq!(rows["args"][0], "proxy");
    assert_eq!(rows["args"][2], "rows");
    assert_eq!(w.config()["mcpServers"]["user-rows"]["args"][0], "proxy");
    assert_eq!(w.local("ldf-files")["args"][0], "files");
    // Untouched: the server with env, the remote one, other settings.
    assert_eq!(
        w.local("secret"),
        w.original()["projects"][w.project.to_string_lossy().as_ref()]["mcpServers"]["secret"]
    );
    assert_eq!(w.config()["numStartups"], 3);

    // The wrapped entry, started as an agent would, answers in LDF.
    assert!(call_through(&rows, &w.project).starts_with("@records "));

    let again = w.ok(&[
        "install",
        "claude-code",
        "--wrap-all",
        "--add-files",
        ".",
        "--apply",
    ]);
    assert!(again.contains("Nothing to change"), "{again}");
    let doctor = w.ok(&["doctor"]);
    assert!(
        doctor.contains("rows (local): behind ldf-mcp proxy"),
        "{doctor}"
    );
    assert!(
        doctor.contains("install record:") && doctor.contains("(3 changes)"),
        "{doctor}"
    );

    let plan = w.ok(&["uninstall", "claude-code"]);
    assert!(plan.contains("Nothing changed"), "{plan}");
    let done = w.ok(&["uninstall", "claude-code", "--apply"]);
    assert!(done.contains("undone: rows"), "{done}");
    assert_eq!(w.config(), w.original());
    let rest = w.ok(&["uninstall", "claude-code", "--apply"]);
    assert!(rest.contains("Nothing installed"), "{rest}");
}

#[test]
fn a_failed_add_puts_the_original_back() {
    let w = World::new("fail");
    let out = w.run(
        &["install", "claude-code", "--wrap", "rows", "--apply"],
        Some("proxy"),
    );
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("the original entry was put back"), "{err}");
    assert_eq!(w.config(), w.original());
}

#[test]
fn uninstall_leaves_an_entry_changed_by_hand_alone() {
    let w = World::new("edited");
    w.ok(&["install", "claude-code", "--wrap", "rows", "--apply"]);
    let mut cfg = w.config();
    cfg["projects"][w.project.to_string_lossy().as_ref()]["mcpServers"]["rows"]["args"]
        .as_array_mut()
        .unwrap()
        .push(json!("--extra"));
    w.write_config(&cfg);
    let text = w.ok(&["uninstall", "claude-code", "--apply"]);
    assert!(text.contains("changed since install; left alone"), "{text}");
    assert_eq!(w.config(), cfg);
}

#[test]
fn wrapping_a_server_that_cannot_be_wrapped_is_refused() {
    let w = World::new("refuse");
    let out = w.run(
        &["install", "claude-code", "--wrap", "secret", "--apply"],
        None,
    );
    assert!(!out.status.success());
    let out = w.run(&["install", "claude-code", "--wrap", "nope"], None);
    assert!(!out.status.success());
    assert_eq!(w.config(), w.original());
}
