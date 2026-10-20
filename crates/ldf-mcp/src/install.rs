//! `install`, `uninstall` and `doctor` for coding agents. Claude Code only
//! for now.
//!
//! Reading: MCP server entries come from `~/.claude.json` (user scope at
//! the top level, local scope under `projects.<dir>`), read only. Writing
//! goes through Claude Code's own CLI, `claude mcp remove` and
//! `claude mcp add-json`, never by editing its files. `.mcp.json` (project
//! scope, shared through version control) is listed but not changed.
//!
//! What gets wrapped: a stdio server with no `env` and no argument that
//! looks like a credential, since its whole entry has to pass through
//! `add-json` on a command line. Others are reported with the reason.
//!
//! Every change is recorded in `installed.json` in the state directory
//! with the original entry, so `uninstall` puts back exactly what was
//! there, and only where the entry is still the one `install` wrote.
//! Without `--apply` both commands only print what they would do.
//!
//! Hooks (`install claude-code --hooks`) have no `claude` subcommand, so
//! they are the one thing written to a Claude Code file directly: three
//! handler groups appended to `hooks` in the user `settings.json`
//! ([`claude_settings`]), parsed and written with key order kept, every
//! other key and hook left as it was. A copy of the file as it was goes
//! to the state directory first; `uninstall` removes only the groups that
//! are still exactly what `install` added.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{Value as Json, json};

use crate::stats;

pub const AGENT: &str = "claude-code";

/// Words that make an argument look like a credential.
const SECRET_WORDS: [&str; 7] = [
    "key", "token", "secret", "password", "passwd", "bearer", "auth",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    Local,
    User,
}

impl Scope {
    pub fn as_str(self) -> &'static str {
        match self {
            Scope::Local => "local",
            Scope::User => "user",
        }
    }
    fn parse(s: &str) -> Option<Scope> {
        match s {
            "local" => Some(Scope::Local),
            "user" => Some(Scope::User),
            _ => None,
        }
    }
}

/// One configured MCP server.
#[derive(Debug, Clone)]
pub struct Server {
    pub name: String,
    pub scope: Scope,
    pub entry: Json,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    /// Can be wrapped with `ldf-mcp proxy`.
    Wrappable,
    /// Already runs through `ldf-mcp proxy`.
    Wrapped,
    /// Is one of this tool's own servers (`files`, `clickhouse`).
    Ldf,
    /// Left alone, with the reason.
    Skip(String),
}

pub fn home() -> Option<PathBuf> {
    let var = |k: &str| {
        std::env::var_os(k)
            .filter(|v| !v.is_empty())
            .map(PathBuf::from)
    };
    var("HOME").or_else(|| var("USERPROFILE"))
}

pub fn claude_json() -> Option<PathBuf> {
    home().map(|h| h.join(".claude.json"))
}

/// The Claude Code CLI: `LDF_CLAUDE` when set (tests), else `claude` on PATH.
pub fn claude_cli() -> Option<PathBuf> {
    std::env::var_os("LDF_CLAUDE")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| which("claude"))
}

/// A program on PATH, with Windows' executable extensions tried.
pub fn which(name: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    let exts: Vec<String> = if cfg!(windows) {
        std::env::var("PATHEXT")
            .unwrap_or_else(|_| ".EXE;.CMD;.BAT".into())
            .split(';')
            .map(str::to_string)
            .collect()
    } else {
        vec![String::new()]
    };
    std::env::split_paths(&path).find_map(|dir| {
        exts.iter()
            .map(|e| dir.join(format!("{name}{e}")))
            .find(|p| p.is_file())
    })
}

/// `canonicalize` on Windows returns `\\?\C:\...`; configs and humans use
/// `C:\...`. Strips that prefix when a drive letter follows.
pub fn plain(p: &Path) -> PathBuf {
    let s = p.to_string_lossy();
    match s.strip_prefix(r"\\?\") {
        Some(rest) if rest.as_bytes().get(1) == Some(&b':') => PathBuf::from(rest),
        _ => p.to_path_buf(),
    }
}

/// A project path as compared with the keys of `projects`: no verbatim
/// prefix and no trailing separator; on Windows also `/` for `\\` and no
/// case.
fn project_key(p: &str) -> String {
    let s = plain(Path::new(p)).to_string_lossy().into_owned();
    if cfg!(windows) {
        s.replace('\\', "/").trim_end_matches('/').to_lowercase()
    } else {
        s.trim_end_matches('/').to_string()
    }
}

/// The key under `projects` that names `project`, if any.
fn find_project<'a>(config: &'a Json, project: &Path) -> Option<&'a str> {
    let want = project_key(&project.to_string_lossy());
    config
        .get("projects")?
        .as_object()?
        .keys()
        .find(|k| project_key(k) == want)
        .map(String::as_str)
}

/// Servers in user scope and in local scope for `project`, from the parsed
/// `~/.claude.json`.
pub fn servers(config: &Json, project: &Path) -> Vec<Server> {
    let mut out = Vec::new();
    let mut take = |map: Option<&Json>, scope: Scope| {
        if let Some(m) = map.and_then(Json::as_object) {
            for (name, entry) in m {
                out.push(Server {
                    name: name.clone(),
                    scope,
                    entry: entry.clone(),
                });
            }
        }
    };
    take(
        config
            .get("projects")
            .and_then(|p| p.get(find_project(config, project)?))
            .and_then(|p| p.get("mcpServers")),
        Scope::Local,
    );
    take(config.get("mcpServers"), Scope::User);
    out
}

fn is_ldf_mcp(command: &str) -> bool {
    Path::new(command)
        .file_stem()
        .is_some_and(|s| s == "ldf-mcp")
}

fn args_of(entry: &Json) -> Vec<String> {
    entry
        .get("args")
        .and_then(Json::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

pub fn classify(entry: &Json) -> Verdict {
    let kind = entry.get("type").and_then(Json::as_str).unwrap_or("stdio");
    if kind != "stdio" {
        return Verdict::Skip(format!("{kind} server; only stdio servers can be wrapped"));
    }
    let Some(command) = entry.get("command").and_then(Json::as_str) else {
        return Verdict::Skip("no command".into());
    };
    let args = args_of(entry);
    if is_ldf_mcp(command) {
        return match args.first().map(String::as_str) {
            Some("proxy") => Verdict::Wrapped,
            _ => Verdict::Ldf,
        };
    }
    if entry
        .get("env")
        .and_then(Json::as_object)
        .is_some_and(|e| !e.is_empty())
    {
        return Verdict::Skip(
            "has env variables, which would pass through a command line; wrap it by hand".into(),
        );
    }
    if let Some(a) = args.iter().find(|a| {
        let l = a.to_ascii_lowercase();
        SECRET_WORDS.iter().any(|w| l.contains(w))
    }) {
        let word = SECRET_WORDS
            .iter()
            .find(|w| a.to_ascii_lowercase().contains(**w))
            .copied()
            .unwrap_or("");
        return Verdict::Skip(format!(
            "an argument looks like a credential (contains \"{word}\"); wrap it by hand"
        ));
    }
    Verdict::Wrappable
}

/// The entry that runs `original` behind the proxy.
pub fn wrapped(exe: &Path, name: &str, model: Option<&str>, original: &Json) -> Json {
    let mut args = vec!["proxy".to_string(), "--name".into(), name.to_string()];
    if let Some(m) = model {
        args.push("--model".into());
        args.push(m.to_string());
    }
    args.push("--".into());
    args.push(
        original
            .get("command")
            .and_then(Json::as_str)
            .unwrap_or_default()
            .to_string(),
    );
    args.extend(args_of(original));
    json!({"type": "stdio", "command": exe.to_string_lossy(), "args": args})
}

/// The entry for an `ldf-mcp files` server over `root`.
pub fn files_entry(exe: &Path, name: &str, root: &Path, model: Option<&str>) -> Json {
    let mut args = vec![
        "files".to_string(),
        "--root".into(),
        root.to_string_lossy().into_owned(),
        "--name".into(),
        name.to_string(),
    ];
    if let Some(m) = model {
        args.push("--model".into());
        args.push(m.to_string());
    }
    json!({"type": "stdio", "command": exe.to_string_lossy(), "args": args})
}

/// One recorded change.
#[derive(Debug, Clone, PartialEq)]
pub struct Change {
    pub scope: Scope,
    /// The project directory for local scope.
    pub project: Option<PathBuf>,
    pub name: String,
    /// `None` when the server was added rather than wrapped.
    pub original: Option<Json>,
    pub installed: Json,
}

impl Change {
    fn to_json(&self) -> Json {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        json!({
            "agent": AGENT,
            "scope": self.scope.as_str(),
            "project": self.project.as_ref().map(|p| p.to_string_lossy().into_owned()),
            "name": self.name,
            "original": self.original,
            "installed": self.installed,
            "at": ts,
        })
    }

    fn from_json(v: &Json) -> Option<Change> {
        if v.get("agent").and_then(Json::as_str) != Some(AGENT) {
            return None;
        }
        Some(Change {
            scope: Scope::parse(v.get("scope")?.as_str()?)?,
            project: v.get("project").and_then(Json::as_str).map(PathBuf::from),
            name: v.get("name")?.as_str()?.to_string(),
            original: v.get("original").filter(|o| !o.is_null()).cloned(),
            installed: v.get("installed")?.clone(),
        })
    }
}

pub fn manifest_path() -> Option<PathBuf> {
    stats::ldf_state_dir().map(|d| d.join("installed.json"))
}

/// Every record in the manifest, ours and other agents' (kept as JSON).
fn read_manifest(path: &Path) -> Result<Vec<Json>, String> {
    match fs::read(path) {
        Ok(b) => serde_json::from_slice::<Json>(&b)
            .ok()
            .and_then(|v| v.get("changes").and_then(Json::as_array).cloned())
            .ok_or_else(|| {
                format!(
                    "{}: not a manifest; move it aside to continue",
                    path.display()
                )
            }),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
        Err(e) => Err(format!("{}: {e}", path.display())),
    }
}

fn write_manifest(path: &Path, changes: &[Json]) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    let tmp = path.with_extension("json.tmp");
    let text = serde_json::to_string_pretty(&json!({"version": 1, "changes": changes}))
        .map_err(|e| e.to_string())?;
    fs::write(&tmp, text + "\n").map_err(|e| format!("{}: {e}", tmp.display()))?;
    fs::rename(&tmp, path).map_err(|e| format!("{}: {e}", path.display()))
}

/// Runs `claude mcp ARGS` in `dir`; `Err` carries its stderr.
fn claude(cli: &Path, dir: &Path, args: &[&str]) -> Result<(), String> {
    let out = Command::new(cli)
        .arg("mcp")
        .args(args)
        .current_dir(dir)
        .output()
        .map_err(|e| format!("cannot run {}: {e}", cli.display()))?;
    if out.status.success() {
        Ok(())
    } else {
        Err(format!(
            "claude mcp {} failed: {}",
            args.first().copied().unwrap_or(""),
            String::from_utf8_lossy(&out.stderr).trim()
        ))
    }
}

fn add_json(cli: &Path, dir: &Path, name: &str, entry: &Json, scope: Scope) -> Result<(), String> {
    claude(
        cli,
        dir,
        &["add-json", name, &entry.to_string(), "-s", scope.as_str()],
    )
}

fn remove(cli: &Path, dir: &Path, name: &str, scope: Scope) -> Result<(), String> {
    claude(cli, dir, &["remove", name, "-s", scope.as_str()])
}

/// Replaces `from` by `to` for one server, restoring `from` when the add
/// fails. With `from` `None` only adds; with `to` `None` only removes.
fn swap(
    cli: &Path,
    dir: &Path,
    name: &str,
    scope: Scope,
    from: Option<&Json>,
    to: Option<&Json>,
) -> Result<(), String> {
    if from.is_some() {
        remove(cli, dir, name, scope)?;
    }
    let Some(to) = to else { return Ok(()) };
    if let Err(e) = add_json(cli, dir, name, to, scope) {
        if let Some(from) = from {
            if let Err(e2) = add_json(cli, dir, name, from, scope) {
                return Err(format!(
                    "{e}; restoring the original also failed ({e2}). Restore it with: claude mcp add-json {name} '{from}' -s {}",
                    scope.as_str()
                ));
            }
            return Err(format!("{e}; the original entry was put back"));
        }
        return Err(e);
    }
    Ok(())
}

/// What `install` was asked to do.
#[derive(Debug, Default)]
pub struct InstallArgs {
    /// `--hooks`: the Bash output hook, independent of the MCP options.
    pub hooks: bool,
    pub wrap: Vec<String>,
    pub wrap_all: bool,
    pub add_files: Option<PathBuf>,
    pub files_name: Option<String>,
    pub scope: Option<Scope>,
    pub model: Option<String>,
    pub apply: bool,
}

pub struct Env {
    pub exe: PathBuf,
    /// Claude Code's user `settings.json`, where hooks go.
    pub settings: Option<PathBuf>,
    pub project: PathBuf,
    pub config: Option<PathBuf>,
    pub cli: Option<PathBuf>,
    pub manifest: Option<PathBuf>,
}

impl Env {
    pub fn detect() -> Result<Env, String> {
        let exe = std::env::current_exe()
            .and_then(|p| p.canonicalize())
            .map(|p| plain(&p))
            .map_err(|e| format!("cannot locate this binary: {e}"))?;
        let project = std::env::current_dir().map_err(|e| e.to_string())?;
        Ok(Env {
            exe,
            settings: claude_settings(),
            project,
            config: claude_json(),
            cli: claude_cli(),
            manifest: manifest_path(),
        })
    }

    fn read_config(&self) -> Result<Json, String> {
        let Some(p) = &self.config else {
            return Err("no home directory".into());
        };
        match fs::read(p) {
            Ok(b) => serde_json::from_slice(&b).map_err(|e| format!("{}: {e}", p.display())),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(json!({})),
            Err(e) => Err(format!("{}: {e}", p.display())),
        }
    }
}

fn describe(entry: &Json) -> String {
    let cmd = entry.get("command").and_then(Json::as_str).unwrap_or("?");
    let args = args_of(entry);
    if args.is_empty() {
        cmd.to_string()
    } else {
        format!("{cmd} {}", args.join(" "))
    }
}

/// Runs `install`; returns the text to print.
pub fn install(env: &Env, a: &InstallArgs) -> Result<String, String> {
    let mut o = String::new();
    let config = env.read_config()?;
    let found = servers(&config, &env.project);
    o.push_str(&format!("Claude Code, project {}\n", env.project.display()));
    if found.is_empty() {
        o.push_str("  no MCP servers in local or user scope\n");
    }
    for s in &found {
        let v = classify(&s.entry);
        let what = match &v {
            Verdict::Wrappable => format!("can be wrapped: {}", describe(&s.entry)),
            Verdict::Wrapped => "already behind ldf-mcp proxy".into(),
            Verdict::Ldf => "an ldf-mcp server".into(),
            Verdict::Skip(why) => format!("left alone: {why}"),
        };
        o.push_str(&format!("  {} ({}): {what}\n", s.name, s.scope.as_str()));
    }
    if let Some(p) = std::path::Path::new(".mcp.json")
        .canonicalize()
        .ok()
        .filter(|p| p.is_file())
    {
        o.push_str(&format!(
            "  {} (project scope) is shared through version control and is not changed\n",
            p.display()
        ));
    }

    let mut plan: Vec<Change> = Vec::new();
    for s in &found {
        let asked = a.wrap_all || a.wrap.iter().any(|w| w == &s.name);
        if !asked {
            continue;
        }
        match classify(&s.entry) {
            Verdict::Wrappable => plan.push(Change {
                scope: s.scope,
                project: (s.scope == Scope::Local).then(|| env.project.clone()),
                name: s.name.clone(),
                original: Some(s.entry.clone()),
                installed: wrapped(&env.exe, &s.name, a.model.as_deref(), &s.entry),
            }),
            Verdict::Wrapped => {}
            other if !a.wrap_all => {
                return Err(format!("{} cannot be wrapped: {other:?}", s.name));
            }
            _ => {}
        }
    }
    for w in &a.wrap {
        if !found.iter().any(|s| &s.name == w) {
            return Err(format!("no MCP server named {w} in local or user scope"));
        }
    }
    if let Some(dir) = &a.add_files {
        let root = dir
            .canonicalize()
            .map(|p| plain(&p))
            .map_err(|e| format!("{}: {e}", dir.display()))?;
        let name = a.files_name.clone().unwrap_or_else(|| "ldf-files".into());
        let scope = a.scope.unwrap_or(Scope::Local);
        let entry = files_entry(&env.exe, &name, &root, a.model.as_deref());
        match found.iter().find(|s| s.name == name && s.scope == scope) {
            Some(s) if s.entry == entry => {}
            Some(_) => {
                return Err(format!(
                    "a server named {name} already exists in {} scope; pick another with --files-name",
                    scope.as_str()
                ));
            }
            None => plan.push(Change {
                scope,
                project: (scope == Scope::Local).then(|| env.project.clone()),
                name,
                original: None,
                installed: entry,
            }),
        }
    }

    if plan.is_empty() {
        o.push_str("\nNothing to change.");
        if a.wrap.is_empty() && !a.wrap_all && a.add_files.is_none() {
            o.push_str(" Choose with --hooks, --wrap NAME, --wrap-all or --add-files DIR.");
        }
        o.push('\n');
        return Ok(o);
    }
    o.push_str(&format!(
        "\n{} {} change(s):\n",
        if a.apply { "Applying" } else { "Would make" },
        plan.len()
    ));
    for c in &plan {
        o.push_str(&format!(
            "  {} {} ({}): {}\n",
            if c.original.is_some() { "wrap" } else { "add" },
            c.name,
            c.scope.as_str(),
            describe(&c.installed)
        ));
    }
    if !a.apply {
        o.push_str(
            "\nNothing changed. Run again with --apply to make these changes through `claude mcp`;\n\
             `ldf-mcp uninstall claude-code` undoes them. Restart Claude Code sessions afterwards.\n",
        );
        return Ok(o);
    }
    let cli = env.cli.as_deref().ok_or("claude not found on PATH")?;
    let manifest = env
        .manifest
        .as_deref()
        .ok_or("no state directory for the change record")?;
    let mut records = read_manifest(manifest)?;
    let before = records.len();
    for c in &plan {
        let dir = c.project.as_deref().unwrap_or(&env.project);
        swap(
            cli,
            dir,
            &c.name,
            c.scope,
            c.original.as_ref(),
            Some(&c.installed),
        )
        .map_err(|e| {
            let earlier = records.len() - before;
            let undo = if earlier > 0 {
                format!(
                    " ({earlier} earlier change(s) are recorded; `ldf-mcp uninstall claude-code --apply` undoes them)"
                )
            } else {
                String::new()
            };
            format!("{}: {e}{undo}", c.name)
        })?;
        records.push(c.to_json());
        write_manifest(manifest, &records)?;
        o.push_str(&format!("  done: {}\n", c.name));
    }
    o.push_str(&format!(
        "Recorded in {}. Restart Claude Code sessions to pick the change up.\n",
        manifest.display()
    ));
    Ok(o)
}

/// Claude Code's user settings: `$CLAUDE_CONFIG_DIR/settings.json` when
/// set, else `~/.claude/settings.json`.
pub fn claude_settings() -> Option<PathBuf> {
    std::env::var_os("CLAUDE_CONFIG_DIR")
        .filter(|v| !v.is_empty())
        .map(PathBuf::from)
        .or_else(|| home().map(|h| h.join(".claude")))
        .map(|d| d.join("settings.json"))
}

/// The events the hook serves and the matcher each group gets.
const HOOK_EVENTS: [(&str, Option<&str>); 3] = [
    ("PostToolUse", Some("Bash")),
    ("SessionStart", None),
    ("PostModelSwitch", None),
];

/// Seconds Claude Code waits for the hook; it keeps the original output
/// after that.
const HOOK_TIMEOUT: u64 = 10;

/// The handler group `install --hooks` adds for `event`, in exec form
/// (`command` plus `args`), so no shell parses the binary's path.
pub fn hook_group(exe: &Path, event: &str, model: Option<&str>) -> Json {
    let mut args = vec!["hook".to_string(), AGENT.to_string()];
    if let Some(m) = model {
        args.push("--model".into());
        args.push(m.to_string());
    }
    let handler = json!({
        "type": "command",
        "command": exe.to_string_lossy(),
        "args": args,
        "timeout": HOOK_TIMEOUT,
    });
    let mut group = json!({"hooks": [handler]});
    if let Some(m) = HOOK_EVENTS
        .iter()
        .find(|(e, _)| *e == event)
        .and_then(|(_, m)| *m)
    {
        group["matcher"] = json!(m);
    }
    group
}

type Ordered = ldf_data::Value;

fn ordered(v: &Json) -> Ordered {
    serde_json::from_value(v.clone()).unwrap_or(Ordered::Null)
}

fn read_settings(path: &Path) -> Result<Ordered, String> {
    match fs::read(path) {
        Ok(b) => match serde_json::from_slice::<Ordered>(&b) {
            Ok(v @ Ordered::Object(_)) => Ok(v),
            Ok(_) => Err(format!("{}: not a JSON object", path.display())),
            Err(e) => Err(format!("{}: {e}; fix it before installing", path.display())),
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Ordered::Object(Vec::new())),
        Err(e) => Err(format!("{}: {e}", path.display())),
    }
}

/// Writes through a temporary file and a rename, keeping the file's
/// permissions. A symlinked settings file (dotfiles) is written where the
/// link points, so the link stays a link.
fn write_settings(path: &Path, v: &Ordered) -> Result<(), String> {
    let resolved;
    let path = if fs::symlink_metadata(path).is_ok_and(|m| m.file_type().is_symlink()) {
        resolved = fs::canonicalize(path).map_err(|e| format!("{}: {e}", path.display()))?;
        resolved.as_path()
    } else {
        path
    };
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    }
    let text = serde_json::to_string_pretty(v).map_err(|e| e.to_string())? + "\n";
    let tmp = path.with_extension("json.ldf-tmp");
    fs::write(&tmp, text).map_err(|e| format!("{}: {e}", tmp.display()))?;
    if let Ok(m) = fs::metadata(path) {
        let _ = fs::set_permissions(&tmp, m.permissions());
    }
    fs::rename(&tmp, path).map_err(|e| format!("{}: {e}", path.display()))
}

/// The array of groups for `event` under `hooks`, created when absent.
/// `None` when `hooks` or that entry is there but not the expected shape.
fn event_groups<'a>(settings: &'a mut Ordered, event: &str) -> Option<&'a mut Vec<Ordered>> {
    let Ordered::Object(top) = settings else {
        return None;
    };
    if !top.iter().any(|(k, _)| k == "hooks") {
        top.push(("hooks".into(), Ordered::Object(Vec::new())));
    }
    let (_, Ordered::Object(hooks)) = top.iter_mut().find(|(k, _)| k == "hooks")? else {
        return None;
    };
    if !hooks.iter().any(|(k, _)| k == event) {
        hooks.push((event.to_string(), Ordered::Array(Vec::new())));
    }
    match hooks.iter_mut().find(|(k, _)| k == event)? {
        (_, Ordered::Array(groups)) => Some(groups),
        _ => None,
    }
}

/// Every hook handler command in `settings` for `event`, for the plan.
fn handler_commands(settings: &Ordered, event: &str) -> Vec<String> {
    settings
        .get("hooks")
        .and_then(|h| h.get(event))
        .and_then(Ordered::as_array)
        .into_iter()
        .flatten()
        .flat_map(|g| {
            g.get("hooks")
                .and_then(Ordered::as_array)
                .into_iter()
                .flatten()
        })
        .filter_map(|h| h.get("command").and_then(Ordered::as_str))
        .map(|c| {
            let first = c.split_whitespace().next().unwrap_or(c);
            Path::new(first.trim_matches('"'))
                .file_name()
                .map_or_else(|| first.to_string(), |f| f.to_string_lossy().into_owned())
        })
        .collect()
}

/// Handlers of other `PostToolUse` groups whose matcher can match `Bash`.
/// Claude Code runs them in parallel with ours on the original output and
/// keeps the last rewrite, so one that redacts Bash output can be undone.
fn bash_siblings(settings: &Ordered) -> Vec<String> {
    let groups = settings
        .get("hooks")
        .and_then(|h| h.get("PostToolUse"))
        .and_then(Ordered::as_array);
    groups
        .into_iter()
        .flatten()
        .filter(|g| !is_our_group(g))
        .filter(|g| match g.get("matcher").and_then(Ordered::as_str) {
            None => true,
            Some(m) => m.is_empty() || m == "*" || m.contains("Bash") || m.contains(".*"),
        })
        .flat_map(|g| {
            g.get("hooks")
                .and_then(Ordered::as_array)
                .into_iter()
                .flatten()
        })
        .map(|h| {
            h.get("command")
                .and_then(Ordered::as_str)
                .map(|c| c.split_whitespace().next().unwrap_or(c).to_string())
                .unwrap_or_else(|| {
                    h.get("type")
                        .and_then(Ordered::as_str)
                        .unwrap_or("?")
                        .to_string()
                })
        })
        .collect()
}

const SIBLING_NOTE: &str = "Claude Code runs PostToolUse hooks in parallel on the original output and keeps the last rewrite: if one of these rewrites Bash output (redaction), the LDF rewrite can undo it";

fn is_our_group(g: &Ordered) -> bool {
    g.get("hooks")
        .and_then(Ordered::as_array)
        .into_iter()
        .flatten()
        .any(|h| {
            h.get("command")
                .and_then(Ordered::as_str)
                .is_some_and(is_ldf_mcp)
                && h.get("args")
                    .and_then(Ordered::as_array)
                    .and_then(|a| a.first())
                    .and_then(Ordered::as_str)
                    == Some("hook")
        })
}

/// Runs `install --hooks`; returns the text to print.
pub fn install_hooks(env: &Env, a: &InstallArgs) -> Result<String, String> {
    let path = env.settings.as_deref().ok_or("no home directory")?;
    let mut settings = read_settings(path)?;
    let mut o = format!("Claude Code hooks, {}\n", path.display());
    let mut plan: Vec<(&str, Json)> = Vec::new();
    for (event, _) in HOOK_EVENTS {
        let want = hook_group(&env.exe, event, a.model.as_deref());
        let others = handler_commands(&settings, event);
        let groups = event_groups(&mut settings.clone(), event)
            .ok_or_else(|| format!("{}: hooks.{event} is not a list", path.display()))?
            .clone();
        let ours: Vec<&Ordered> = groups.iter().filter(|g| is_our_group(g)).collect();
        if ours
            .iter()
            .any(|g| g.semantic_eq_unordered(&ordered(&want)))
        {
            o.push_str(&format!("  {event}: already installed\n"));
            continue;
        }
        if !ours.is_empty() {
            return Err(format!(
                "{}: hooks.{event} already runs a different ldf-mcp hook; run `ldf-mcp uninstall claude-code --apply` or edit it by hand",
                path.display()
            ));
        }
        if !others.is_empty() {
            o.push_str(&format!(
                "  {event}: keeps {} existing handler(s): {}\n",
                others.len(),
                others.join(", ")
            ));
        }
        let siblings = bash_siblings(&settings);
        if event == "PostToolUse" && !siblings.is_empty() {
            o.push_str(&format!(
                "  warning: other PostToolUse hooks match Bash ({}). {SIBLING_NOTE}.\n",
                siblings.join(", ")
            ));
        }
        plan.push((event, want));
    }
    if plan.is_empty() {
        o.push_str("\nNothing to change.\n");
        return Ok(o);
    }
    o.push_str(&format!(
        "\n{} {} hook group(s):\n",
        if a.apply { "Adding" } else { "Would add" },
        plan.len()
    ));
    for (event, g) in &plan {
        o.push_str(&format!("  {event}: {g}\n"));
    }
    if env.exe.components().any(|c| c.as_os_str() == "target") {
        o.push_str("  note: this binary is in a build directory; a rebuild replaces it under Claude Code\n");
    }
    if !a.apply {
        o.push_str(
            "\nNothing changed. Run again with --apply to add these hooks;\n\
             `ldf-mcp uninstall claude-code` removes them. New Claude Code sessions pick them up.\n",
        );
        return Ok(o);
    }
    let manifest = env
        .manifest
        .as_deref()
        .ok_or("no state directory for the change record")?;
    let mut records = read_manifest(manifest)?;
    if path.exists() {
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let backup = manifest.with_file_name(format!("settings.json.before-{ts}"));
        if let Some(dir) = backup.parent() {
            fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        }
        fs::copy(path, &backup).map_err(|e| format!("{}: {e}", backup.display()))?;
        o.push_str(&format!(
            "  copy of the file as it was: {}\n",
            backup.display()
        ));
    }
    for (event, g) in &plan {
        event_groups(&mut settings, event)
            .ok_or_else(|| format!("hooks.{event} is not a list"))?
            .push(ordered(g));
    }
    write_settings(path, &settings)?;
    for (event, g) in &plan {
        records.push(json!({
            "agent": AGENT,
            "kind": "hook",
            "settings": path.to_string_lossy(),
            "event": event,
            "installed": g,
        }));
    }
    write_manifest(manifest, &records)?;
    o.push_str(&format!(
        "  done. Recorded in {}. New Claude Code sessions pick the hooks up.\n",
        manifest.display()
    ));
    Ok(o)
}

/// Removes the hook groups `install --hooks` recorded and returns the
/// records left (other kinds, and groups that changed since install).
fn uninstall_hooks(records: Vec<Json>, apply: bool, o: &mut String) -> Result<Vec<Json>, String> {
    let (hooks, mut rest): (Vec<Json>, Vec<Json>) = records.into_iter().partition(|r| {
        r.get("agent").and_then(Json::as_str) == Some(AGENT)
            && r.get("kind").and_then(Json::as_str) == Some("hook")
    });
    let mut by_file: Vec<(PathBuf, Vec<Json>)> = Vec::new();
    for r in hooks {
        let Some(p) = r.get("settings").and_then(Json::as_str).map(PathBuf::from) else {
            rest.push(r);
            continue;
        };
        match by_file.iter_mut().find(|(f, _)| *f == p) {
            Some((_, v)) => v.push(r),
            None => by_file.push((p, vec![r])),
        }
    }
    for (path, recs) in by_file {
        let mut settings = match read_settings(&path) {
            Ok(s) => s,
            Err(e) => {
                o.push_str(&format!("  hooks in {e}; left alone\n"));
                rest.extend(recs);
                continue;
            }
        };
        let mut changed = false;
        for r in recs {
            let event = r.get("event").and_then(Json::as_str).unwrap_or("");
            let want = ordered(r.get("installed").unwrap_or(&Json::Null));
            let groups = settings
                .get("hooks")
                .and_then(|h| h.get(event))
                .and_then(Ordered::as_array);
            match groups.and_then(|g| g.iter().position(|g| g.semantic_eq_unordered(&want))) {
                Some(i) => {
                    o.push_str(&format!("  hook {event}: remove\n"));
                    if let Some(g) = event_groups(&mut settings, event) {
                        g.remove(i);
                        changed = true;
                    }
                }
                None if groups.is_some_and(|g| g.iter().any(is_our_group)) => {
                    o.push_str(&format!(
                        "  hook {event}: changed since install; left alone\n"
                    ));
                    rest.push(r);
                }
                None => o.push_str(&format!(
                    "  hook {event}: no longer configured; record dropped\n"
                )),
            }
        }
        if changed {
            tidy_hooks(&mut settings);
            if apply {
                write_settings(&path, &settings)?;
                o.push_str(&format!("  updated {}\n", path.display()));
            }
        }
    }
    Ok(rest)
}

/// Drops event lists left empty, and `hooks` itself when nothing is left.
fn tidy_hooks(settings: &mut Ordered) {
    let Ordered::Object(top) = settings else {
        return;
    };
    if let Some((_, Ordered::Object(hooks))) = top.iter_mut().find(|(k, _)| k == "hooks") {
        hooks.retain(|(_, v)| !matches!(v, Ordered::Array(a) if a.is_empty()));
    }
    top.retain(|(k, v)| !(k == "hooks" && matches!(v, Ordered::Object(h) if h.is_empty())));
}

/// Whether the Bash hook is in `settings`, for `doctor`.
fn hooks_installed(path: &Path) -> Option<bool> {
    let s = read_settings(path).ok()?;
    Some(
        s.get("hooks")
            .and_then(|h| h.get("PostToolUse"))
            .and_then(Ordered::as_array)
            .is_some_and(|g| g.iter().any(is_our_group)),
    )
}

/// Runs `uninstall`; returns the text to print.
pub fn uninstall(env: &Env, apply: bool) -> Result<String, String> {
    let manifest = env.manifest.as_deref().ok_or("no state directory")?;
    let all = read_manifest(manifest)?;
    let had_hooks = all
        .iter()
        .any(|r| r.get("kind").and_then(Json::as_str) == Some("hook"));
    let mut o = String::new();
    let records = uninstall_hooks(all.clone(), apply, &mut o)?;
    let hooks_done = had_hooks && records.len() < all.len();
    if apply && hooks_done {
        write_manifest(manifest, &records)?;
    }
    let config = env.read_config()?;
    let mut keep: Vec<Json> = Vec::new();
    let mut todo: Vec<(Change, Json)> = Vec::new();
    for r in records {
        let Some(c) = Change::from_json(&r) else {
            keep.push(r);
            continue;
        };
        let project = c.project.clone().unwrap_or_else(|| env.project.clone());
        let current = servers(&config, &project)
            .into_iter()
            .find(|s| s.name == c.name && s.scope == c.scope);
        match current {
            None => o.push_str(&format!(
                "  {} ({}): no longer configured; record dropped\n",
                c.name,
                c.scope.as_str()
            )),
            Some(s) if s.entry != c.installed => {
                o.push_str(&format!(
                    "  {} ({}): changed since install; left alone\n",
                    c.name,
                    c.scope.as_str()
                ));
                keep.push(r);
            }
            Some(_) => {
                o.push_str(&format!(
                    "  {} ({}): {}\n",
                    c.name,
                    c.scope.as_str(),
                    match &c.original {
                        Some(orig) => format!("restore {}", describe(orig)),
                        None => "remove".into(),
                    }
                ));
                todo.push((c, r));
            }
        }
    }
    if todo.is_empty() {
        if o.is_empty() {
            o.push_str("Nothing installed by ldf-mcp for Claude Code.\n");
        } else if !apply && hooks_done {
            o.push_str("\nNothing changed. Run again with --apply to undo these.\n");
        }
        if apply {
            write_manifest(manifest, &keep)?;
        }
        return Ok(o);
    }
    if !apply {
        o.push_str("\nNothing changed. Run again with --apply to undo these.\n");
        return Ok(o);
    }
    let cli = env.cli.as_deref().ok_or("claude not found on PATH")?;
    // Newest first, so a server changed twice unwinds in order.
    let mut failed = Vec::new();
    for (c, r) in todo.into_iter().rev() {
        let dir = c.project.clone().unwrap_or_else(|| env.project.clone());
        match swap(
            cli,
            &dir,
            &c.name,
            c.scope,
            Some(&c.installed),
            c.original.as_ref(),
        ) {
            Ok(()) => o.push_str(&format!("  undone: {}\n", c.name)),
            Err(e) => {
                o.push_str(&format!("  failed: {}: {e}\n", c.name));
                failed.push(r);
            }
        }
    }
    keep.extend(failed);
    write_manifest(manifest, &keep)?;
    o.push_str("Restart Claude Code sessions to pick the change up.\n");
    Ok(o)
}

/// Runs `doctor`; returns the text to print.
pub fn doctor(env: &Env) -> String {
    let mut o = String::new();
    let ok = |b: bool| if b { "ok  " } else { "--  " };
    o.push_str(&format!(
        "{}ldf-mcp {} at {}\n",
        ok(true),
        env!("CARGO_PKG_VERSION"),
        env.exe.display()
    ));
    if env.exe.components().any(|c| c.as_os_str() == "target") {
        o.push_str(
            "      a build directory: a rebuild replaces this binary under any agent using it\n",
        );
    }
    match stats::default_path() {
        Some(p) => {
            let events = fs::read_to_string(&p)
                .map(|t| t.lines().count())
                .unwrap_or(0);
            o.push_str(&format!(
                "{}statistics: {} ({events} responses)\n",
                ok(true),
                p.display()
            ));
        }
        None => o.push_str(&format!(
            "{}statistics: off (LDF_STATS=0 or no home directory)\n",
            ok(false)
        )),
    }
    for agent in ["claude", "codex", "opencode"] {
        let path = if agent == "claude" {
            env.cli.clone()
        } else {
            which(agent)
        };
        match path {
            Some(p) => o.push_str(&format!("{}{agent}: {}\n", ok(true), p.display())),
            None => o.push_str(&format!("{}{agent}: not found on PATH\n", ok(false))),
        }
    }
    o.push_str("      codex and opencode: not supported by install yet; see site/src/content/docs/docs/agents.md\n");
    match env.read_config() {
        Ok(cfg) => {
            let found = servers(&cfg, &env.project);
            o.push_str(&format!(
                "{}Claude Code MCP servers for {}: {}\n",
                ok(true),
                env.project.display(),
                found.len()
            ));
            for s in &found {
                let v = classify(&s.entry);
                let mut line = format!(
                    "      {} ({}): {}",
                    s.name,
                    s.scope.as_str(),
                    match &v {
                        Verdict::Wrappable => "not wrapped, can be".to_string(),
                        Verdict::Wrapped => "behind ldf-mcp proxy".into(),
                        Verdict::Ldf => "ldf-mcp server".into(),
                        Verdict::Skip(why) => format!("not wrapped: {why}"),
                    }
                );
                if matches!(v, Verdict::Wrapped | Verdict::Ldf) {
                    let cmd = s.entry.get("command").and_then(Json::as_str).unwrap_or("");
                    if !Path::new(cmd).is_file() {
                        line.push_str(&format!(" -- BROKEN: {cmd} does not exist"));
                    }
                    let args = args_of(&s.entry);
                    if let Some(i) = args.iter().position(|a| a == "--model") {
                        line.push_str(&format!(
                            ", model {}",
                            args.get(i + 1).map_or("?", String::as_str)
                        ));
                    }
                }
                o.push_str(&line);
                o.push('\n');
            }
        }
        Err(e) => o.push_str(&format!("{}Claude Code config: {e}\n", ok(false))),
    }
    if let Some(p) = &env.settings {
        match hooks_installed(p) {
            Some(true) => o.push_str(&format!(
                "{}Bash output hook: installed in {}\n",
                ok(true),
                p.display()
            )),
            Some(false) => o.push_str(&format!(
                "{}Bash output hook: not installed (ldf-mcp install claude-code --hooks)\n",
                ok(false)
            )),
            None => o.push_str(&format!(
                "{}Claude Code settings: {} unreadable\n",
                ok(false),
                p.display()
            )),
        }
        let claude = env.project.join(".claude");
        for f in [
            p.clone(),
            claude.join("settings.json"),
            claude.join("settings.local.json"),
        ] {
            let siblings = read_settings(&f)
                .map(|s| bash_siblings(&s))
                .unwrap_or_default();
            if !siblings.is_empty() {
                o.push_str(&format!(
                    "{}other PostToolUse hooks match Bash in {}: {}\n      {SIBLING_NOTE}\n",
                    ok(false),
                    f.display(),
                    siblings.join(", ")
                ));
            }
        }
    }
    if let Some(m) = &env.manifest {
        let n = read_manifest(m).map(|r| r.len()).unwrap_or(0);
        o.push_str(&format!(
            "{}install record: {} ({n} changes)\n",
            ok(true),
            m.display()
        ));
    }
    let models: std::collections::BTreeSet<String> = ldf_select::builtin_evidence()
        .into_iter()
        .filter(|e| e.n >= ldf_select::Policy::default().min_n)
        .map(|e| e.model)
        .collect();
    o.push_str(&format!(
        "      --model with evidence: {}\n",
        models.into_iter().collect::<Vec<_>>().join(", ")
    ));
    o
}

/// A stand-in for `claude mcp remove|add-json NAME [JSON] -s SCOPE` that
/// edits `$HOME/.claude.json` the way Claude Code does, for the tests.
/// `LDF_FAKE_CLAUDE_FAIL` makes `add-json` fail when the JSON contains it.
pub fn fake_claude(args: &[String]) -> Result<(), String> {
    let path = claude_json().ok_or("no HOME")?;
    let mut cfg: Json = match fs::read(&path) {
        Ok(b) => serde_json::from_slice(&b).map_err(|e| e.to_string())?,
        Err(_) => json!({}),
    };
    let project = std::env::current_dir().map_err(|e| e.to_string())?;
    let key = find_project(&cfg, &project)
        .map(str::to_string)
        .unwrap_or_else(|| project.to_string_lossy().into_owned());
    let a: Vec<&str> = args.iter().map(String::as_str).collect();
    let (op, name, entry, scope) = match a.as_slice() {
        ["mcp", "remove", name, "-s", scope] => ("remove", *name, None, *scope),
        ["mcp", "add-json", name, entry, "-s", scope] => ("add", *name, Some(*entry), *scope),
        _ => return Err(format!("fake-claude: unsupported {a:?}")),
    };
    let servers = match scope {
        "user" => cfg
            .as_object_mut()
            .ok_or("config not an object")?
            .entry("mcpServers")
            .or_insert_with(|| json!({})),
        "local" => cfg
            .as_object_mut()
            .ok_or("config not an object")?
            .entry("projects")
            .or_insert_with(|| json!({}))
            .as_object_mut()
            .ok_or("projects not an object")?
            .entry(key)
            .or_insert_with(|| json!({}))
            .as_object_mut()
            .ok_or("project not an object")?
            .entry("mcpServers")
            .or_insert_with(|| json!({})),
        other => return Err(format!("fake-claude: scope {other}")),
    }
    .as_object_mut()
    .ok_or("mcpServers not an object")?;
    match (op, entry) {
        ("remove", _) => {
            servers
                .remove(name)
                .ok_or_else(|| format!("No MCP server found with name: {name}"))?;
        }
        (_, Some(entry)) => {
            if let Ok(bad) = std::env::var("LDF_FAKE_CLAUDE_FAIL")
                && entry.contains(&bad)
            {
                return Err("injected failure".into());
            }
            if servers.contains_key(name) {
                return Err(format!(
                    "MCP server {name} already exists in {scope} config"
                ));
            }
            let v: Json = serde_json::from_str(entry).map_err(|e| e.to_string())?;
            servers.insert(name.to_string(), v);
        }
        _ => unreachable!(),
    }
    fs::write(
        &path,
        serde_json::to_string_pretty(&cfg).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_entries() {
        assert_eq!(
            classify(&json!({"command": "npx", "args": ["-y", "srv"]})),
            Verdict::Wrappable
        );
        assert_eq!(
            classify(&json!({"type": "stdio", "command": "srv", "env": {}})),
            Verdict::Wrappable
        );
        assert!(matches!(
            classify(&json!({"type": "http", "url": "https://x"})),
            Verdict::Skip(_)
        ));
        assert!(matches!(
            classify(&json!({"command": "srv", "env": {"A": "1"}})),
            Verdict::Skip(_)
        ));
        assert!(matches!(
            classify(&json!({"command": "srv", "args": ["--api-key", "x"]})),
            Verdict::Skip(_)
        ));
        assert!(matches!(
            classify(&json!({"command": "srv", "args": ["--TOKEN=x"]})),
            Verdict::Skip(_)
        ));
        assert_eq!(
            classify(&json!({"command": "/x/ldf-mcp", "args": ["proxy", "--", "a"]})),
            Verdict::Wrapped
        );
        assert_eq!(
            classify(&json!({"command": "/x/ldf-mcp.exe", "args": ["files"]})),
            Verdict::Ldf
        );
    }

    #[test]
    fn wrapping_keeps_the_original_command_and_arguments() {
        let orig = json!({"type": "stdio", "command": "npx", "args": ["-y", "srv", "--port", "1"]});
        let w = wrapped(
            Path::new("/bin/ldf-mcp"),
            "db",
            Some("claude-opus-5-5"),
            &orig,
        );
        assert_eq!(
            w,
            json!({"type": "stdio", "command": "/bin/ldf-mcp", "args": ["proxy", "--name", "db", "--model", "claude-opus-5-5", "--", "npx", "-y", "srv", "--port", "1"]})
        );
        assert_eq!(classify(&w), Verdict::Wrapped);
    }

    #[test]
    fn reads_local_and_user_scope() {
        let cfg = json!({
            "mcpServers": {"u": {"command": "a"}},
            "projects": {"/p": {"mcpServers": {"l": {"command": "b"}}}, "/q": {"mcpServers": {"x": {"command": "c"}}}}
        });
        let s = servers(&cfg, Path::new("/p"));
        let names: Vec<(&str, Scope)> = s.iter().map(|s| (s.name.as_str(), s.scope)).collect();
        assert_eq!(names, vec![("l", Scope::Local), ("u", Scope::User)]);
        // Trailing separators and (on Windows) case and slashes do not matter.
        assert_eq!(servers(&cfg, Path::new("/p/")).len(), 2);
    }

    #[test]
    fn verbatim_prefix_is_stripped_before_a_drive_letter_only() {
        assert_eq!(plain(Path::new(r"\\?\C:\x\y")), PathBuf::from(r"C:\x\y"));
        assert_eq!(
            plain(Path::new(r"\\?\UNC\srv\s")),
            PathBuf::from(r"\\?\UNC\srv\s")
        );
        assert_eq!(plain(Path::new("/usr/bin")), PathBuf::from("/usr/bin"));
    }
}
