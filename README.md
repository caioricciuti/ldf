# LDF: Language Data Format (research workspace)

LDF is an attempt to design a lossless serialization format that is cheap for
language models to read: fewer tokens than JSON, no silent flattening, GB-scale
streaming, and accuracy that is measured rather than assumed. The competitors
are JSON, JSONL, CSV/TSV, Markdown tables and TOON.

The repository holds the measurement apparatus that decided the syntax
(deterministic datasets, faithful baseline implementations, a performance
harness, exact token counting, an LLM comprehension evaluation) and the first
version of the format itself: **LDF v0.1**, specified in `docs/ldf-spec-v0.md`
and implemented in `crates/ldf`. See `docs/design-candidates.md` for how the
measurements led to it, `docs/roadmap.md` for what comes next and
`docs/research.md` for the state of the art.

## What is here

| path | contents |
|---|---|
| `crates/ldf-data` | value model, number semantics, readers for JSON, JSONL, CSV, Parquet |
| `crates/ldf-fixtures` | `ldf-gen`: twelve deterministic datasets at three scales |
| `crates/ldf-formats` | baselines: json, json-pretty, jsonl, csv, tsv, markdown, toon (spec v4.4, conformance tested), yaml, and the `ldf` adapter |
| `crates/ldf-bench` | `ldf-bench`: round trips, process-isolated perf, token counts, Criterion benches |
| `crates/ldf-eval` | `ldf-eval`: tasks with ground truth, prompt bundles, scoring |
| `crates/ldf` | LDF v0.1 encoder, decoder and conformance fixtures |
| `crates/ldf-mcp` | `ldf-mcp`: MCP stdio proxy that re-encodes JSON tool results as LDF on their way to the model; `clickhouse` mode, an MCP server over ClickHouse HTTP returning LDF; `convert` for files; a `fake-server` test double |
| `eval/` | stdlib-only Python: `run_eval.py` (API calls, spend-gated), `count_tokens_claude.py` |
| `fixtures/small` | committed correctness fixtures (regenerable) |
| `docs/` | research notes, benchmark methodology, roadmap, measured reports |

## Build

Requirements: Rust stable 1.98 or newer (edition 2024), Python 3.9 or newer
for the evaluation scripts. No other tooling.

```bash
cargo build --release
cargo test
```

`cargo test` runs the unit tests, format round-trip tests and the 564 official
TOON conformance fixtures.

## Run

```bash
# datasets
./target/release/ldf-gen list
./target/release/ldf-gen --scale small      # regenerate fixtures/small (committed)
./target/release/ldf-gen --scale medium     # 10k rows per dataset, ~60 MB, gitignored
./target/release/ldf-gen --scale large --dataset flat_users,wide_table   # 1M rows

# correctness: every format on every small fixture, with the round-trip level
./target/release/ldf-bench roundtrip --scale small

# performance: one child process per (dataset, op, format), cold + 6 warm iterations
./target/release/ldf-bench perf --scale medium --iters 7 --out results/medium

# tokens: exact o200k_base and cl100k_base counts, encoded documents saved
./target/release/ldf-bench tokens --scale medium --sample 1000 --out results/medium

# Criterion microbenchmarks
cargo bench -p ldf-bench

# LLM comprehension: build a bundle, inspect, then run only with explicit approval
./target/release/ldf-eval build --input fixtures/medium/flat_users.json --records 50 --out eval/out/flat_users
python3 eval/run_eval.py --bundle eval/out/flat_users --provider anthropic --model claude-opus-5-5 --dry-run
python3 eval/run_eval.py --bundle eval/out/flat_users --provider openai --model <gpt model id> --confirm-spend
./target/release/ldf-eval score --bundle eval/out/flat_users --responses eval/out/flat_users/responses.openai.<model>.jsonl
eval/plan.sh                 # the agreed paid stages, each with a price and a budget cap; `eval/plan.sh score` regrades all

# MCP proxy: LDF between any stdio MCP server and the model
./target/release/ldf-mcp proxy -- <server command> [args...]
./target/release/ldf-mcp convert fixtures/small/nested_orders.json   # any supported input file to LDF on stdout
./target/release/ldf-mcp clickhouse --url http://127.0.0.1:8124 --user ldf   # MCP server over ClickHouse HTTP (docs/ldf-mcp.md)
```

`ldf-bench` writes `perf.md`/`perf.json` or `tokens.md`/`tokens.json` into the
output directory, including machine, toolchain and crate versions. Any input
file works too: `ldf-bench perf --input path/to/data.jsonl`.

API keys are read from `OPENAI_API_KEY`, `ANTHROPIC_API_KEY` and
`DEEPSEEK_API_KEY`, from the environment or from a gitignored `.env` at the
repo root. Nothing is sent without `--confirm-spend`, and nothing in the Rust code touches the
network.

## MCP proxy

`ldf-mcp proxy` launches a downstream MCP server (anything that speaks the
stdio transport) and sits between it and the client. Every JSON-RPC message
is forwarded as is, except three results on the way back:

* `initialize`: a short description of LDF is appended to `instructions`,
  so the model has read the format once before it sees any data;
* `tools/list`: one sentence is appended to each tool description;
* `tools/call`: a text content block that is a JSON array of objects, or an
  object holding such an array, is re-encoded as LDF when that is shorter.
  Blocks under 1 024 bytes (`--min-bytes`), error results, non-text blocks
  and `structuredContent` are left alone.

The model keeps calling the same tools with the same arguments; only the
bytes it reads change. One stderr line per converted result reports the
JSON and LDF sizes (`--quiet` turns it off). `--passthrough` forwards tool
results unchanged and `--plain` skips the primer and the descriptions, so
the same session can be compared with and without LDF.

With Claude Code, register the proxy in place of the server it wraps:

```bash
claude mcp add clickhouse -- /abs/path/to/target/release/ldf-mcp proxy -- <clickhouse mcp command> [its args]
```

`ldf-mcp fake-server` is a stand-in downstream server with one tool,
`query`, that returns generated rows; the integration tests drive the proxy
against it, and `ldf-mcp proxy -- ldf-mcp fake-server` is a way to watch
the rewrite without a database.

`ldf-mcp clickhouse` needs no other MCP server: it is one, with three
read-only tools (`list_tables`, `describe_table`, `run_select`) over
ClickHouse's HTTP interface, returning rows as LDF. Local setup with Docker,
test data and the Claude Code registration are in `docs/ldf-mcp.md`.

## Results so far

`docs/reports/` holds the measured baseline on this machine (Apple M2 Pro,
10k-row fixtures). Highlights, all from those files:

- Compact JSON encodes at 0.6 to 2.0 GB/s with serde_json and parses at 0.2
  to 0.7 GB/s on record-heavy data (2 GB/s on long free text); simd-json
  parses 20 to 40 percent faster on most datasets and uses more memory.
- On flat data CSV and TOON are both about 0.45 to 0.5 of JSON's bytes and
  0.60 to 0.65 of its o200k tokens. TOON's bytes-per-token is the best of the
  set on flat data (2.14 against JSON's 2.98 on `flat_users`).
- TOON loses its advantage as soon as records stop being uniform: 1.04x JSON
  bytes on nested orders, 1.11x on sparse events, 1.21x on mixed types, 2.5x
  on deep nesting, and more o200k tokens than JSON in each of those cases.
- CSV, TSV and Markdown cannot represent nested data at all and silently
  merge missing fields with nulls and the string `"null"` with null; the
  harness reports these as unsupported or lossy rather than hiding them.
- Block YAML matches compact JSON in bytes but costs 7 to 34 percent more
  tokens on regular data.
- Comprehension runs on three 50-record datasets with Claude Opus 5.5 and
  GPT-6.1 Sol: GPT answered 470 of 470 correctly across ten formats, LDF
  included zero-shot; Claude's few misses were output truncation,
  arithmetic, or a two-record comparison, none tied to a format. LDF was the
  cheapest input in every bundle. At 200 records the first format-correlated
  error appeared (a pretty-JSON sum off by one row). Larger samples and
  cheaper models are next.

Read `docs/benchmark-methodology.md` before quoting any number.

## Conventions

See `AGENTS.md`.
