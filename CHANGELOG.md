# Changelog

All notable changes to this repository. The format follows Keep a Changelog.
Versions are tags on this repository and, from 0.1.0, the versions of the
`ldf`, `ldf-data` and `ldf-mcp` crates (and `ldf-select` from 0.2.0); the specification version is
stated in `site/src/content/docs/docs/spec.md`.

## Unreleased

- `ldf-mcp proxy`: a JSON result that stays JSON (not a dataset, LDF not
  shorter, or the selector's choice) now goes out minified when it was
  pretty-printed. Tokens are kept byte for byte, so the JSON is
  semantically equal. Small, error and non-JSON results are untouched,
  `--passthrough` sends the original, and the stats event carries
  `"minified": true`. MotherDuck's DuckDB server answers in pretty
  columnar JSON, which LDF does not convert; this takes it to 0.40 to 0.58
  of its bytes (`reports/2026-10-21-duckdb/minified`, bench_mcp.py --mode duckdb after this change).

## 0.5.2 (2026-10-20)

Selector evidence at 1 000 records, size bands, and Claude Code hook
fixes. The specification stays v0.2.

### Changed

- The evidence built into `ldf-select` now has Opus 5.5 and Sonnet 5.5 at
  1 000 held-out records (`reports/2026-10-20-large`). From 201 to 1 000
  rows: Opus gets LDF on nested (hoisted), sparse and mixed-type data and
  JSON on flat data; Sonnet gets LDF on flat and mixed-type data and JSON
  on nested and sparse data. Haiku and DeepSeek have no evidence above 200
  rows and get JSON there. Above 1 000 rows every model gets JSON.
- `ldf-select` judges an input by size band: only evidence at the smallest
  measured size at or above the input's row count counts (200 or 1 000
  records today). Before, any failing row withdrew a layout at every size,
  so the 1 000-record losses also gave JSON to Opus on flat data and to
  Sonnet on nested and sparse data at 200 rows or fewer; those get LDF
  again up to 200 rows. Inputs larger than the largest measured size still
  get JSON.

### Added

- `ldf-eval build --kinds` limits a bundle to the listed task kinds.
- Claude Code hook: a command containing `LDF_RAW=1` keeps its JSON
  (reason `raw`); the conversion note tells the model so.
- `install claude-code --hooks` and `doctor` warn when another
  `PostToolUse` hook matches Bash: Claude Code keeps the last rewrite, so
  a redacting hook can lose to the LDF rewrite.

### Fixed

- Claude Code hook in subagents: a subagent's calls carry the parent's
  session id, so it shared the main thread's primer flag (LDF without the
  primer in the subagent's context) and model (the selector asked about
  the wrong model). Each subagent now has its own primer flag, and its
  model comes from `--model` or its own transcript; otherwise its output
  passes through (reason `subagent`).
- Claude Code hook: the primer was marked as sent before the reply was
  written; it is now marked only after a whole write.
- `install` and `uninstall` replaced a symlinked `settings.json` with a
  plain file; they now write through the link.
- `ldf-mcp hook claude-code --evidence FILE` without `--model` was ignored
  silently and the built-in evidence used; the hook now reads the file and
  keeps taking the model from the session.

## 0.5.1 (2026-10-20)

Selector evidence for Claude Sonnet 5.5. No code or format change; the
specification stays v0.2.

### Changed

- The evidence built into `ldf-select` (and so `ldf-mcp --model` and the
  Claude Code hook) now has Stage C on Sonnet 5.5
  (`reports/2026-10-20-stage-c-sonnet`): Sonnet gets LDF at sparse
  threshold 0.5 on flat, sparse and mixed-type data, threshold 0.75 on
  nested data with sparse keys, and JSON on other nested data. Before this
  release Sonnet always got JSON.

## 0.5.0 (2026-10-20)

Automatic Bash output for Claude Code: `ldf-mcp install claude-code --hooks
--apply`, and JSON datasets that shell commands print reach the model as
LDF. The specification stays v0.2.

### Added

- `ldf-mcp hook claude-code`: a Claude Code command hook. On `PostToolUse`
  for Bash it replaces `stdout` with LDF when stdout is one complete JSON
  dataset and LDF plus its note is smaller (and the selector agrees for the
  session's model), keeping every other field of the tool result; in every
  other case it prints nothing, so the original stays. `SessionStart` and
  `PostModelSwitch` record the model (with the transcript as fallback), and
  the LDF primer goes once per context. Always exits 0; no network, no
  child processes; input over 16 MiB is not read.
- `ldf-mcp install claude-code --hooks [--apply]` adds it to the user
  `settings.json` with key order and other hooks kept, a copy of the file in
  the state directory, and a record in `installed.json`; `uninstall` removes
  only the groups that are still as installed. Independent of `--wrap`.
- `doctor` reports whether the hook is installed; `stats` labels the hook's
  reason codes (`persisted`, `exit_code`, `interrupted`, `background`,
  `image`, `bad_input`, `too_large`).
- `eval/bench_hook.py` and `reports/2026-10-20-hook/`: exact o200k counts and
  per-call timing of the hook on the medium fixtures.
- `eval/ab_hook.py` (spend-gated) and `reports/2026-10-20-hook/ab-haiku/`:
  the same headless Claude Code task with and without the hook, on
  provider-reported usage. Haiku 5.5 and Opus 5.5, 36 runs each: tool
  results cost 0.56 of the JSON tokens overall on both (0.41 to 0.65 where
  converted), 36 of 36 answers correct in both arms
  (`reports/2026-10-20-hook/ab-haiku/`, `ab-opus/`).

## 0.4.0 (2026-10-20)

Agent integration: `ldf-mcp install claude-code`, local statistics,
`doctor`, a hardened proxy and a Windows binary. The specification stays
v0.2. The documentation moves into the website's sources and the
measurement reports to `reports/`.

### Added

- Local statistics: every `ldf-mcp` mode appends one line per tool response
  (outcome, reason code, bytes, timings; never content) to
  `ldf/events.jsonl` under the user's state directory, and `ldf-mcp stats`
  summarises it (`--days`, `--json`, `--file`). `--name NAME` labels a
  server, `--no-stats` or `LDF_STATS=0` turns recording off.
- `ldf-mcp install claude-code` puts Claude Code's stdio MCP servers behind
  `ldf-mcp proxy` (`--wrap NAME`, `--wrap-all`) and adds a files server
  (`--add-files DIR`), through `claude mcp remove` / `add-json` only. It
  prints the plan unless given `--apply`, records every change with the
  original entry, changes nothing on a second run, and restores the
  original when an add fails. Servers with env, credential-like arguments,
  remote servers and `.mcp.json` are listed and left alone.
  `ldf-mcp uninstall claude-code [--apply]` restores each entry that is
  still the one install wrote.
- `ldf-mcp doctor`: binary, statistics file, agents on PATH, Claude Code
  servers and whether they are behind LDF or broken, models with evidence.
  `ldf-mcp --version`.
- `site/src/content/docs/docs/agents.md`: integration architecture, compatibility matrix for
  Claude Code, Codex CLI and OpenCode (MCP, hooks, plugins, model
  visibility, usage counters), setup in five minutes, limitations.
  `eval/demo_agents.py` and `reports/2026-10-19-agent-demos/`: the
  proxy and files data paths, JSON against LDF in bytes and o200k tokens.
- Windows: the release builds `x86_64-pc-windows-msvc` as a zip with its
  SHA-256, and CI builds and tests `ldf-mcp` on Windows.

### Changed

- The documentation pages live in `site/src/content/docs/docs/` (spec,
  agents, mcp, methodology, design, research, roadmap) and the measurement
  reports in `reports/`; `docs/` is gone. `ldf-discover select` reads its
  default evidence from `reports/evidence/evidence.json`.

### Fixed

- On Windows, `ldf-mcp proxy` finds a server's `.cmd` launcher (as Node
  servers ship): a bare program name is looked up with PATHEXT, since the
  standard library only tries `.exe`. `install` matches Claude Code's
  project keys whatever their spelling (`\\?\` prefix, separators, case)
  and writes paths without the `\\?\` prefix.

- `ldf-mcp proxy` no longer panics on a `tools/list` entry or `initialize`
  result that is not an object, no longer mistakes a server-to-client
  request reusing a client id for that client's reply, forgets cancelled
  requests, forwards invalid UTF-8 untouched instead of exiting, and stops
  a child that does not exit within 5 s of end of input.
- The `files` and `clickhouse` servers answer a line that is not JSON with a
  `-32700` parse error instead of ignoring it (or exiting on invalid UTF-8).

## 0.3.0 (2026-10-19)

Specification v0.2: hoisted arrays, an additive and opt-in column type.
The built-in selector evidence now includes the Stage C runs on Opus 5.5.

### Added

- Selector evidence from Stage C (`reports/2026-10-19-stage-c`):
  Opus 5.5 on held-out data, LDF ties JSON (294 of 297) at 57 to 77
  percent of its input tokens, so `ldf-mcp --model claude-opus-5-5` now
  sends LDF. GPT still has too few questions per cell and gets JSON.
- Spec v0.2 (`site/src/content/docs/docs/spec.md`, section 7, rule 2a): with hoisting on,
  an array of objects whose elements all have the same keys in the same
  order and only scalar values is one column `items:[sku,qty,price]` whose
  cell is `[["E5",3,28.78],["C3",4,157.4]]`, instead of a `t` column and a
  child table. Every v0.1 document is a valid v0.2 document; a v0.1
  decoder cannot read a hoisted column.
- `ldf::Options::hoist` (default false, so default output is unchanged)
  and `ldf-mcp convert --hoist`. The decoder reads hoisted columns always.
- `ldf-select`: LDF candidates with and without hoisting at both sparse
  thresholds; evidence rows carry `hoist_thresholds`. With the built-in
  evidence, `--model claude-opus-5-5` and `--model deepseek-chat` get the
  hoisted form on nested orders; Haiku stays on JSON there.
- The `ldf-mcp` primer explains hoisted columns in one sentence and an
  example.
- Conformance fixtures for hoisted columns (encode cases carry
  `"options": {"hoist": true}`); fixture files are version 0.2.

### Changed

- `ldf-select::Choice::Ldf` has a `hoist` field and `measure_ldf` a `hoist`
  argument (breaking for direct users of the crate).
- `ldf-discover`: the hoisted knob now follows spec v0.2 (scalar values
  only, a child table otherwise, where it used to fall back to a JSON
  cell), so `Config::LDF_HOISTED` is byte-identical to `crates/ldf` with
  hoisting on. No committed Stage A output changes: of the twelve
  fixtures only nested_orders has arrays of objects, and they hoist under
  both rules.

## 0.2.0 (2026-10-19)

The format discovery engine, a direct Arrow path for Parquet, and a
per-model format selector in `ldf-mcp`. The specification stays v0.1;
two encoder bugs are fixed against it.

### Added

- `ldf-mcp --model ID` (in `proxy`, `clickhouse` and `files`): each result
  goes out as LDF only when measured evidence for that exact model on data
  of the same shape shows no accuracy loss against JSON (at least 50
  questions, no more than 3 points below, no significant loss, no more
  rows than measured) and LDF is at least 5 percent smaller; otherwise
  JSON, with the reason on stderr. A model without evidence gets JSON
  throughout and no LDF primer. `--evidence FILE` replaces the built-in
  evidence. Without `--model` nothing changes.
- `ldf-select`, a new crate with the selector's rule and the evidence
  table built in, shared by `ldf-mcp` and `ldf-discover`.
- `ldf::arrow` (feature `arrow`): encodes Arrow RecordBatches directly,
  byte-identical to the record path, 1.9x to 3.2x faster in block form at
  1M rows and 1.4x to 4.3x less memory. `ldf-mcp convert` uses it for
  Parquet and falls back to the record path for lists of structs.
  `ldf_data::input::ColumnReader` is public for it.
- `ldf-discover` (not published): a parameterized encoder whose layout
  choices are knobs, a decoder for every point, Stage A token search over
  the space, and the `select` command.
- Evaluation: four new task kinds, more questions per kind, held-out
  fixtures, a *bare* answer outcome, Stage B and the DeepSeek sparse sweep
  (reports under `reports/`), `eval/analysis/evidence.py` for the
  selector's evidence, and `eval/bench_mcp.py` for end-to-end MCP timing.

### Fixed

- An object field that is null in some rows decoded as absent; such a
  field is now a `j` column (spec 7.1).
- A bare string starting with `@` as the first cell of a row was read back
  as a header; it is now quoted (spec 5). LDF output on the twelve medium
  fixtures is byte-identical before and after both fixes.

## 0.1.0 (2026-10-18)

The first tagged release: specification v0.1, the reference codec, the
measurement apparatus and `ldf-mcp`.

### Added

- The format: typed header, comma-separated rows, dotted paths for nested
  objects, child tables for arrays of objects, `j` cells for the rest,
  empty cell versus `null` for absent versus null, and the sparse-key rule
  of v0.1: a top-level key filled in fewer than half the rows moves from a
  positional column into a per-row `~` object. The rule was adopted after
  Claude Haiku 5.5 went from 13 to 17 of 18 on 200 sparse records with its
  reasoning cost gone; dense data encodes exactly as before.
- The reference encoder and decoder in Rust (`ldf`) with 47 conformance
  fixtures, and the block encoder: the streaming path plans one block of
  records at a time and writes child tables with ordinals numbered across
  the document, so memory is bounded (10 MB peak against 32 MB for the
  batch path on the 2.8 MB nested_orders fixture) and the output matches
  the batch encoder on flat data; on nested data it is smaller, since
  ordinal joins are shorter than key joins.
- `ldf-mcp`, an MCP server over stdio: `proxy` wraps any stdio MCP server
  and re-encodes JSON tool results as LDF; `clickhouse` exposes read-only
  SQL over ClickHouse's HTTP interface and returns rows as LDF; `files`
  serves a directory of json, jsonl, csv, tsv and parquet files, streamed
  and paged; `convert` prints any supported input file as LDF, streaming
  with `--block`. Capped results say so in a second text block with how to
  page on.
- The measurement apparatus the syntax was designed from (`ldf-data`,
  `ldf-fixtures`, `ldf-formats`, `ldf-bench`, `ldf-eval`): twelve
  deterministic datasets, ten baseline formats (TOON passes its official
  suite), a process-isolated benchmark harness with exact token counts, an
  evaluation harness with ground truth and a spend-gated API runner.
- The baseline report: perf and tokens on an M2 Pro, comprehension runs on
  Claude Opus 5.5, Claude Haiku 5.5, GPT-6.1 Sol and DeepSeek at 50, 200
  and 500 records.
- The website, ldflabs.org, with the docs mirrored from this repository;
  CI for formatting, lints, tests and the small round trip; this release
  workflow.

### Changed

- Planner and decoder no longer scan the column list per cell; wide tables
  encode and decode two to three times faster than the first cut.
- The evaluation scorer reports *wrapped* answers (the right value inside a
  one-key object) apart from *wrong*, so answer-shape slips are visible
  separately from misreads.
