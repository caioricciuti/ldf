# Changelog

All notable changes to this repository. The format follows Keep a Changelog;
versions follow the specification, not the crates, until the first release.

## Unreleased

### Added

- Block encoder: the streaming path plans one block of records at a time
  and writes child tables with ordinals numbered across the document, so
  memory is bounded (10 MB peak against 32 MB for the batch path on the
  2.8 MB nested_orders fixture) and the output matches the batch encoder
  on flat data; on nested data it is smaller, since ordinal joins are
  shorter than key joins. `ldf-mcp convert --block N` streams a file
  through it. The decoder attaches a child table name once from all its
  occurrences, which block-interleaved documents need.
- `ldf-mcp`, an MCP server over stdio: `proxy` wraps any stdio MCP server
  and re-encodes JSON tool results as LDF; `clickhouse` exposes read-only
  SQL over ClickHouse's HTTP interface and returns rows as LDF; `convert`
  prints any supported input file as LDF.
- The website, ldflabs.org, with the docs mirrored from this repository.
- A CI workflow: formatting, clippy, tests and the small round trip.

## 0.1 (2026-10-18)

The specification is `docs/ldf-spec-v0.md`, section "v0.1".

### Added

- The sparse-key rule: a top-level key filled in fewer than half the rows
  moves from a positional column into a per-row `~` object. Adopted after
  Claude Haiku 5.5 went from 13 to 17 of 18 on 200 sparse records with its
  reasoning cost gone; dense data encodes exactly as before.

### Changed

- Planner and decoder no longer scan the column list per cell; wide tables
  encode and decode two to three times faster.

## 0 (2026-10-18)

### Added

- The format: typed header, comma-separated rows, dotted paths for nested
  objects, child tables for arrays of objects, `j` cells for the rest,
  empty cell versus `null` for absent versus null.
- The reference encoder and decoder in Rust, forty conformance fixtures.
- The measurement apparatus the syntax was designed from: twelve
  deterministic datasets, ten baseline formats (TOON passes its official
  suite), a process-isolated benchmark harness with exact token counts, an
  evaluation harness with ground truth and a spend-gated API runner.
- The baseline report: perf and tokens on an M2 Pro, comprehension runs on
  Claude Opus 5.5, Claude Haiku 5.5, GPT-6.1 Sol and DeepSeek at 50, 200
  and 500 records.
