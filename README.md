# LDF: smaller tool results for LLM agents

[![ci](https://github.com/caioricciuti/ldf/actions/workflows/ci.yml/badge.svg)](https://github.com/caioricciuti/ldf/actions/workflows/ci.yml)
[![license](https://img.shields.io/badge/license-MIT%20or%20Apache--2.0-blue)](#license)
[![docs](https://img.shields.io/badge/docs-ldflabs.org-f2b56b)](https://ldflabs.org)

> **Research project, no longer developed.** LDF saves tokens, but in
> practice minifying, capping big results and checking which copy your
> client reads get you nearly all of it, and the format was proven safe
> only for a few models and data shapes. The write-up:
> [I spent $70 proving my LLM data format isn't worth using](https://ldflabs.org/docs/writing/not-worth-it/).
> The crates and binaries stay published as they are.

`ldf-mcp` sits between an agent and its data (MCP servers, shell output,
files) and cuts what tool results cost in the model's context without
changing what they mean. Most of the saving needs no new format:

- **Minify.** Database MCP servers often send pretty-printed JSON. Removing
  the whitespace alone takes 31 to 59 percent off most results (13 percent on
  high-cardinality strings, 5 on long text), for every model.
- **Keep one copy in sync.** FastMCP servers send each result twice, as
  text and as `structuredContent`, and Claude Code shows the model the
  second copy. The proxy rewrites both, so its savings reach the model.
- **Cap big results** (opt-in). `--max-result-bytes` keeps the rows that
  fit and tells the model how many exist and how to narrow the query.
- **Compact encoding where it pays and is proven.** On rows that repeat
  their keys, or hold structs, lists and nested arrays, LDF (below) is 13
  to 57 percent smaller than minified JSON. With `--model`, it is used only
  where a pre-registered test ruled out a loss of more than 3 points
  against JSON; everywhere else the model gets JSON.

Where it does not help much: flat SQL results that are already columnar
(`columns` plus row arrays, or CSV). There LDF saves 0.1 to 7 percent over
minified JSON (one table came out 4 tokens larger). If you control the server, ask your database for a compact
output format and you get most of that for free.

## The format

LDF is a lossless, line-oriented text encoding of JSON records: one typed
header, one line per record, child tables for nested arrays.

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
records as compact JSON.

Docs, spec and results: **[ldflabs.org](https://ldflabs.org)**.

## What is measured

Tokens are o200k against minified JSON. Comprehension is the
pre-registered study in `reports/2026-10-21-ni-study`: 200 held-out
records, 151 to 249 questions per cell, the same questions in both
formats, the LDF primer in both arms. "Bound" is the largest loss against
JSON the data cannot rule out.

| data | tokens | Opus 5.5 | Sonnet 5.5 | DeepSeek |
|---|---|---|---|---|
| flat rows that repeat keys | 0.43 to 0.65 | pass (bound 1.2) | no loss seen, not proven alone (3.9); passes pooled with an earlier run | **loses** 6 points |
| nested orders | 0.56 to 0.75 | hoisted arrays **lose** 3.6 points (p = 0.01); threshold 0.75 passes on an earlier run | not proven (3.5) | not proven (3.7) |
| sparse events | about 0.80 | pass (2.6) | not proven (3.6) | not proven (5.5) |
| mixed types | about 0.75 | pass (1.1) | pass (1.8) | pass (2.3) |
| SQL rows with many NULLs, as `null` cells | 0.47 of key-repeating JSON | | pass (2.6) | |

"Not proven" means accuracy within about a point of JSON, but not enough
questions to rule out a 3-point loss; the selector sends JSON there. Above
200 rows no cell has enough questions yet, so every model gets JSON. Haiku
5.5 and GPT models get JSON throughout.

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
