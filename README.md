# LDF: Language Data Format

[![ci](https://github.com/caioricciuti/ldf/actions/workflows/ci.yml/badge.svg)](https://github.com/caioricciuti/ldf/actions/workflows/ci.yml)
[![license](https://img.shields.io/badge/license-MIT%20or%20Apache--2.0-blue)](#license)
[![docs](https://img.shields.io/badge/docs-ldflabs.org-f2b56b)](https://ldflabs.org)

A lossless, line-oriented text encoding of JSON records that language models
read for 35 to 45 percent fewer tokens than compact JSON. One typed header,
one line per record, child tables for nested arrays. Nothing is flattened
away, and the claims are measured: four models, three sizes, ten formats,
with the misses in the report.

```text
@records order_id:s,customer.id:i,customer.name:s,items:t,total:f,status:s,note:s?
ORD-000001,4654,Val Thompson,2,715.94,shipped,context compare
ORD-000002,50,Sol Ritchie,1,2757.03,cancelled,null
@records.items order_id:s,sku:s,qty:i,price:f
ORD-000001,E5,3,28.78
ORD-000001,C3,4,157.4
ORD-000002,A1,1,2757.03
```

That is two orders, each with a customer object and an items array, as a
root table and one child table. The same records as compact JSON cost about
1.6 times the tokens.

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
datasets, from `docs/reports/2026-10-17-baseline/tokens-medium-2026-10-18.md`.

Comprehension, from the same report: Claude Opus 5.5 at 200 records answers
131 of 150 in LDF with zero wrong answers, the same as every other format
(the misses are an output cap on sums); GPT-6.1 Sol answers 53 of 53 on LDF
zero-shot; Claude Haiku 5.5 on 200 sparse records scores what JSON scores at
25 percent fewer tokens. Below the frontier tier no format is reliably
better than another, and the report says so.

## Use it

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
test data: `docs/ldf-mcp.md`.

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
Not on crates.io yet.

## The format

Seven rules, specified in `docs/ldf-spec-v0.md` (v0.1):

1. A header `@name col:type,...` opens a table. Types: `i` `f` `s` `b` `j` `t`; `?` marks nullable.
2. A row is one record, cells separated by commas; strings are bare unless they need JSON quoting.
3. An empty cell means absent, `null` means null.
4. Objects with a stable shape flatten into dotted columns; irregular ones are a `j` cell.
5. Arrays of objects become a child table joined on the parent's first column; the parent keeps a count.
6. Keys present in fewer than half the rows travel in a per-row `~` object instead of a mostly empty column.
7. Numbers keep their kind and their shortest round-trip text.

## Repository

| path | contents |
|---|---|
| `crates/ldf` | the encoder, the decoder and 46 conformance fixtures |
| `crates/ldf-mcp` | `ldf-mcp`: proxy mode, ClickHouse mode, files mode, `convert`, a fake server for tests |
| `crates/ldf-data` | value model, number semantics, readers for JSON, JSONL, CSV, Parquet |
| `crates/ldf-formats` | the ten baseline formats, implemented faithfully (TOON passes its official suite) |
| `crates/ldf-fixtures` | `ldf-gen`: twelve deterministic datasets at three scales |
| `crates/ldf-bench` | `ldf-bench`: round trips, process-isolated perf, exact token counts |
| `crates/ldf-eval` | `ldf-eval`: questions with ground truth, prompt bundles, scoring |
| `eval/` | stdlib-only Python: the spend-gated API runner and the agreed run plan |
| `docs/` | spec, methodology, design candidates, research notes, the measured report |
| `site/` | ldflabs.org, built from `docs/` |

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

v0.1, a prototype with its evidence attached. What is next is in
`docs/roadmap.md`: more sources for `ldf-mcp`, a first tagged release, the
open questions at the end of the spec.

## License

MIT or Apache-2.0, at your option. The TOON conformance fixtures under
`crates/ldf-formats/tests` are vendored verbatim with their own licence.
