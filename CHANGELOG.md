# Changelog

All notable changes to this repository. The format follows Keep a Changelog.
Versions are tags on this repository and, from 0.1.0, the versions of the
`ldf`, `ldf-data` and `ldf-mcp` crates (and `ldf-select` from 0.2.0); the specification version is
stated in `site/src/content/docs/docs/spec.md`.

## Unreleased

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

### Fixed

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
