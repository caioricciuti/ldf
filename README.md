# LDF: Language Data Format

[![ci](https://github.com/caioricciuti/ldf/actions/workflows/ci.yml/badge.svg)](https://github.com/caioricciuti/ldf/actions/workflows/ci.yml)
[![license](https://img.shields.io/badge/license-MIT%20or%20Apache--2.0-blue)](#license)
[![docs](https://img.shields.io/badge/docs-ldflabs.org-f2b56b)](https://ldflabs.org)

A lossless, line-oriented text encoding of JSON records for language
models. One typed header, one line per record, child tables for nested
arrays. Nothing is flattened away, and the claims are measured: on held-out
data Claude Opus 5.5 answers from LDF as well as from JSON (294 of 297 each)
at 57 to 77 percent of JSON's input tokens, with the misses in the reports.

```text
@records id:s,customer.name:s,items:t,total:f,status:s
ORD-1,Val Thompson,2,715.94,shipped
ORD-2,Sol Ritchie,1,2757.03,cancelled
ORD-3,Kim Abara,2,53.78,shipped
@records.items id:s,sku:s,qty:i,price:f
ORD-1,E5,3,28.78
ORD-1,C3,4,157.4
ORD-2,A1,1,2757.03
ORD-3,B2,2,12.5
ORD-3,E5,1,28.78
```

That is three orders, each with a customer object and an items array, as a
root table and one child table: 143 o200k tokens against 169 for the same
records as compact JSON. Headers are a fixed cost, so the gap grows with the
row count (table below).

Docs, spec and results: **[ldflabs.org](https://ldflabs.org)**.

## Why

| | JSON | CSV | TOON | LDF |
|---|---|---|---|---|
| nested records | yes | no | yes | yes, as child tables |
| missing versus null | yes | lost | yes | yes |
| number kinds (`1` vs `1.0`, big integers) | yes | lost | yes | yes |
| tokens on flat data, share of JSON | 1.00 | 0.60 | 0.65 | 0.61 |
| tokens on nested orders | 1.00 | n/a | 1.08 | 0.64 |
| tokens on sparse events | 1.00 | 0.61, lossy | 1.22 | 0.80 |

Token figures are o200k_base on the first 1 000 records of the benchmark
datasets, from `reports/2026-10-17-baseline/tokens-medium-2026-10-18.md`.

Comprehension: Claude Opus 5.5 on held-out data ties JSON, 294 of 297
(`reports/2026-10-19-stage-c`); in headless Claude Code sessions the Bash
hook brought JSON tool output to 0.56 of its Claude tokens on Haiku 5.5 and
Opus 5.5 with no answer lost (`reports/2026-10-20-hook`). Below the frontier
tier no format is reliably better than another, and the reports say so; a
model without enough evidence gets JSON.

## Use it

LDF is plain text in a prompt, so it works with any model and any client:
convert files, put `ldf-mcp` in front of any MCP server, or call the Rust
crate. One `cargo install ldf-mcp --locked` (or a binary from the GitHub
releases) covers all of it.

### Claude Code shell output

With Claude Code, two commands make JSON that shell commands print reach the model as LDF:

```bash
cargo install ldf-mcp --locked                # or a binary from the GitHub releases
ldf-mcp install claude-code --hooks --apply   # without --apply: the plan only
ldf-mcp stats                                 # what was converted, and why the rest was not
```

MCP servers can be wrapped too (`--wrap-all`). See
[site/src/content/docs/docs/agents.md](site/src/content/docs/docs/agents.md): how it works, what is intercepted and what is not, Codex and OpenCode by hand.

### As a middle man: `ldf-mcp`

The model never has to adopt a format. `ldf-mcp` is an MCP server over
stdio. In proxy mode it wraps any stdio MCP server you already use, forwards
every message, and re-encodes JSON tool results as LDF when that is shorter.
In ClickHouse mode it is the server: read-only SQL over the HTTP interface,
rows returned as LDF. In files mode it serves a directory of JSON, JSONL,
CSV, TSV or Parquet files, streamed and paged.

```bash
cargo build --release
claude mcp add my-data    -- "$PWD/target/release/ldf-mcp" proxy -- <any stdio MCP server> [its args]
claude mcp add clickhouse -- "$PWD/target/release/ldf-mcp" clickhouse --url http://127.0.0.1:8123
claude mcp add exports    -- "$PWD/target/release/ldf-mcp" files --root /path/to/exports
```

The initialize result carries a short primer, so the model has read the
format once before it sees any data. Results under 1 KB, error results and
non-text blocks pass through untouched; `--passthrough` forwards JSON
unchanged for an A/B in the same session. Setup with a local ClickHouse and
test data: `site/src/content/docs/docs/mcp.md`.

### As a file

```bash
./target/release/ldf-mcp convert data.json > data.ldf     # json, jsonl, csv, tsv, parquet
```

### As a library

```rust
let text = ldf::encode(&records);           // records: &[ldf_data::Value]
let back = ldf::decode(&text)?;             // Vec<ldf_data::Value>, byte-exact on dense data
```

The crate is `crates/ldf`; the value model and readers are `crates/ldf-data`.
Both are on crates.io.

## The format

Eight rules, specified in `site/src/content/docs/docs/spec.md` (v0.2):

1. A header `@name col:type,...` opens a table. Types: `i` `f` `s` `b` `j` `t`; `?` marks nullable.
2. A row is one record, cells separated by commas; strings are bare unless they need JSON quoting.
3. An empty cell means absent, `null` means null.
4. Objects with a stable shape flatten into dotted columns; irregular ones are a `j` cell.
5. Arrays of objects become a child table joined on the parent's first column; the parent keeps a count.
6. Keys present in fewer than half the rows travel in a per-row `~` object instead of a mostly empty column.
7. Numbers keep their kind and their shortest round-trip text.
8. Opt-in since v0.2: an array of flat objects with one key order can stay in its row, `items:[sku,qty]` with `[["E5",3],...]` cells.

## Repository

| path | contents |
|---|---|
| `crates/ldf` | the encoder, the decoder and 46 conformance fixtures |
| `crates/ldf-mcp` | `ldf-mcp`: proxy mode, ClickHouse mode, files mode, `convert`, a fake server for tests |
| `crates/ldf-select` | the per-model rule that picks JSON or an LDF layout from the measured evidence |
| `crates/ldf-discover` | the format discovery engine: 51 480 lossless layouts, each round-trip checked and token counted |
| `crates/ldf-data` | value model, number semantics, readers for JSON, JSONL, CSV, Parquet |
| `crates/ldf-formats` | the ten baseline formats, implemented faithfully (TOON passes its official suite) |
| `crates/ldf-fixtures` | `ldf-gen`: twelve deterministic datasets at three scales |
| `crates/ldf-bench` | `ldf-bench`: round trips, process-isolated perf, exact token counts |
| `crates/ldf-eval` | `ldf-eval`: questions with ground truth, prompt bundles, scoring |
| `eval/` | stdlib-only Python: the spend-gated API runner and the agreed run plan |
| `site/` | ldflabs.org and the documentation: spec, ldf-mcp, coding agents, methodology, design, research, roadmap (`site/src/content/docs/docs/`) |
| `reports/` | measurement reports with the raw files the tools wrote |

Build and test with Rust 1.98 or newer:

```bash
cargo build --release
cargo test --release
./target/release/ldf-bench roundtrip --scale small
```

`AGENTS.md` has every command, the conventions and the measured facts worth
knowing. `CONTRIBUTING.md` has the bar for changes; `SECURITY.md` says what
processes untrusted input and how to report a problem.

## Status

Specification v0.2; the crates are at 0.5.0 on crates.io. Releases since
the spec's last change touched the tools, not the format. Every release is
in `CHANGELOG.md`; what is next is in `site/src/content/docs/docs/roadmap.md`.

## License

MIT or Apache-2.0, at your option. The TOON conformance fixtures under
`crates/ldf-formats/tests` are vendored verbatim with their own licence.
