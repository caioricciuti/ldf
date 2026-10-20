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
            .env("LDF_FAKE_CLAUDE", "1")
            .env_remove("CLAUDE_CONFIG_DIR");
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

/// A user settings.json as people have them: keys in no particular order,
/// other hooks on the same events, 2-space pretty JSON.
const SETTINGS: &str = r#"{
  "permissions": {
    "deny": [
      "Read(./.env)"
    ],
    "defaultMode": "auto"
  },
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "python3 \"/home/me/.claude/hooks/vet-deps.py\""
          }
        ]
      }
    ],
    "PostToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "rtk hook claude-post"
          }
        ]
      }
    ]
  },
  "theme": "dark",
  "includeCoAuthoredBy": false
}
"#;

impl World {
    fn settings_path(&self) -> PathBuf {
        self.home.join(".claude").join("settings.json")
    }

    fn write_settings(&self, text: &str) {
        std::fs::create_dir_all(self.home.join(".claude")).unwrap();
        std::fs::write(self.settings_path(), text).unwrap();
    }

    fn settings_text(&self) -> String {
        std::fs::read_to_string(self.settings_path()).unwrap()
    }

    fn settings(&self) -> Json {
        serde_json::from_str(&self.settings_text()).unwrap()
    }
}

fn top_keys(text: &str) -> Vec<String> {
    let v: ldf_data::Value = serde_json::from_str(text).unwrap();
    v.as_object()
        .unwrap()
        .iter()
        .map(|(k, _)| k.clone())
        .collect()
}

#[test]
fn hooks_plan_alone_changes_nothing() {
    let w = World::new("hooks-plan");
    w.write_settings(SETTINGS);
    let text = w.ok(&["install", "claude-code", "--hooks"]);
    assert!(text.contains("Would add 3 hook group(s)"), "{text}");
    assert!(
        text.contains("PostToolUse: keeps 1 existing handler(s): rtk"),
        "{text}"
    );
    assert!(
        !text.contains("rows (local)"),
        "--hooks alone leaves MCP out"
    );
    assert_eq!(w.settings_text(), SETTINGS);
    assert!(!w.root.join("state/installed.json").exists());
}

#[test]
fn hooks_install_keeps_everything_else_runs_and_uninstall_restores_the_file() {
    let w = World::new("hooks-apply");
    w.write_settings(SETTINGS);
    let text = w.ok(&["install", "claude-code", "--hooks", "--apply"]);
    assert!(text.contains("done."), "{text}");
    let after = w.settings_text();
    assert_eq!(
        top_keys(&after),
        ["permissions", "hooks", "theme", "includeCoAuthoredBy"]
    );
    let s = w.settings();
    assert_eq!(s["permissions"]["defaultMode"], "auto");
    assert_eq!(s["hooks"]["PreToolUse"].as_array().unwrap().len(), 1);
    let post = s["hooks"]["PostToolUse"].as_array().unwrap();
    assert_eq!(post.len(), 2);
    assert_eq!(post[0]["hooks"][0]["command"], "rtk hook claude-post");
    assert_eq!(post[1]["matcher"], "Bash");
    let handler = post[1]["hooks"][0].clone();
    assert_eq!(handler["args"], json!(["hook", "claude-code"]));
    assert_eq!(handler["timeout"], 10);
    assert!(s["hooks"]["SessionStart"][0]["matcher"].is_null());
    assert_eq!(s["hooks"]["PostModelSwitch"].as_array().unwrap().len(), 1);
    // The original went to the state directory first.
    let copies: Vec<_> = std::fs::read_dir(w.root.join("state"))
        .unwrap()
        .flatten()
        .filter(|e| {
            e.file_name()
                .to_string_lossy()
                .starts_with("settings.json.before-")
        })
        .collect();
    assert_eq!(copies.len(), 1);
    assert_eq!(std::fs::read_to_string(copies[0].path()).unwrap(), SETTINGS);

    // The installed handler, run the way Claude Code runs it.
    let rows: Vec<Json> = (0..50)
        .map(|i| json!({"id": i, "name": format!("n{i}"), "ok": i % 2 == 0}))
        .collect();
    let input = json!({"session_id": "s", "hook_event_name": "PostToolUse", "tool_name": "Bash",
        "tool_input": {"command": "cat x.json"},
        "tool_response": {"stdout": serde_json::to_string_pretty(&rows).unwrap(), "stderr": "", "interrupted": false, "isImage": false}});
    let mut child = Command::new(handler["command"].as_str().unwrap())
        .args(["hook", "claude-code"])
        .env("LDF_STATE_DIR", w.root.join("state"))
        .env("LDF_STATS", "0")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.to_string().as_bytes())
        .unwrap();
    let out = child.wait_with_output().unwrap();
    let reply: Json = serde_json::from_slice(&out.stdout).unwrap();
    assert!(
        reply["hookSpecificOutput"]["updatedToolOutput"]["stdout"]
            .as_str()
            .unwrap()
            .starts_with("@records id:i,name:s,ok:b\n")
    );

    let again = w.ok(&["install", "claude-code", "--hooks", "--apply"]);
    assert!(again.contains("Nothing to change"), "{again}");
    assert_eq!(w.settings_text(), after, "idempotent");
    let doctor = w.ok(&["doctor"]);
    assert!(doctor.contains("Bash output hook: installed"), "{doctor}");

    let plan = w.ok(&["uninstall", "claude-code"]);
    assert!(plan.contains("hook PostToolUse: remove"), "{plan}");
    assert!(plan.contains("Nothing changed"), "{plan}");
    assert_eq!(w.settings_text(), after);
    w.ok(&["uninstall", "claude-code", "--apply"]);
    assert_eq!(w.settings_text(), SETTINGS, "byte for byte");
    let rest = w.ok(&["uninstall", "claude-code", "--apply"]);
    assert!(rest.contains("Nothing installed"), "{rest}");
}

#[test]
fn uninstall_keeps_edits_made_after_install() {
    let w = World::new("hooks-edits");
    w.write_settings(SETTINGS);
    w.ok(&["install", "claude-code", "--hooks", "--apply"]);
    let mut s = w.settings();
    s["permissions"]["deny"]
        .as_array_mut()
        .unwrap()
        .push(json!("Read(~/.ssh/**)"));
    s["hooks"]["SessionStart"]
        .as_array_mut()
        .unwrap()
        .push(json!({"hooks": [{"type": "command", "command": "echo hi"}]}));
    // The user tuned the PostModelSwitch handler: that one is theirs now.
    s["hooks"]["PostModelSwitch"][0]["hooks"][0]["timeout"] = json!(30);
    s["model"] = json!("opus");
    w.write_settings(&serde_json::to_string_pretty(&s).unwrap());

    let text = w.ok(&["uninstall", "claude-code", "--apply"]);
    assert!(
        text.contains("hook PostModelSwitch: changed since install; left alone"),
        "{text}"
    );
    let s = w.settings();
    assert_eq!(s["model"], "opus");
    assert_eq!(s["permissions"]["deny"].as_array().unwrap().len(), 2);
    assert_eq!(s["hooks"]["PostToolUse"].as_array().unwrap().len(), 1);
    assert_eq!(
        s["hooks"]["SessionStart"][0]["hooks"][0]["command"],
        "echo hi"
    );
    assert_eq!(s["hooks"]["SessionStart"].as_array().unwrap().len(), 1);
    assert_eq!(s["hooks"]["PostModelSwitch"][0]["hooks"][0]["timeout"], 30);
    // Its record stays, so a later uninstall still knows about it.
    let manifest: Json =
        serde_json::from_slice(&std::fs::read(w.root.join("state/installed.json")).unwrap())
            .unwrap();
    assert_eq!(manifest["changes"].as_array().unwrap().len(), 1);
}

#[test]
fn hooks_and_wrapping_are_independent_and_undo_together() {
    let w = World::new("hooks-both");
    // No settings.json yet: install creates one with only the hooks.
    w.ok(&["install", "claude-code", "--wrap", "rows", "--apply"]);
    assert!(
        !w.settings_path().exists(),
        "wrapping alone writes no hooks"
    );
    w.ok(&[
        "install",
        "claude-code",
        "--hooks",
        "--model",
        "claude-opus-5-5",
        "--apply",
    ]);
    let s = w.settings();
    assert_eq!(
        s["hooks"]["PostToolUse"][0]["hooks"][0]["args"],
        json!(["hook", "claude-code", "--model", "claude-opus-5-5"])
    );
    assert_eq!(w.local("rows")["args"][0], "proxy");
    let done = w.ok(&["uninstall", "claude-code", "--apply"]);
    assert!(
        done.contains("undone: rows") && done.contains("hook SessionStart: remove"),
        "{done}"
    );
    assert_eq!(w.config(), w.original());
    assert_eq!(w.settings(), json!({}));
}

#[test]
fn unreadable_settings_are_refused_untouched() {
    let w = World::new("hooks-bad");
    w.write_settings("{ \"theme\": \"dark\", }");
    let out = w.run(&["install", "claude-code", "--hooks", "--apply"], None);
    assert!(!out.status.success());
    assert_eq!(w.settings_text(), "{ \"theme\": \"dark\", }");
    w.write_settings(r#"{"hooks": {"PostToolUse": {"matcher": "Bash"}}}"#);
    let out = w.run(&["install", "claude-code", "--hooks", "--apply"], None);
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("not a list"));
}
