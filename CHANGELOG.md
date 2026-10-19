# Changelog

All notable changes to this repository. The format follows Keep a Changelog.
Versions are tags on this repository and, from 0.1.0, the versions of the
`ldf`, `ldf-data` and `ldf-mcp` crates (and `ldf-select` from 0.2.0); the specification version is
stated in `docs/ldf-spec-v0.md`.

## Unreleased

Nothing yet.

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
  (reports under `docs/reports/`), `eval/analysis/evidence.py` for the
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
