# AGENTS.md

Working notes for anyone (human or agent) touching this repository. The
project is LDF, a research effort towards a lossless, token-efficient data
format for LLM consumption. Phases 1 and 2 built the measurement foundation;
Phase 3 produced LDF v0, now v0.1 (`docs/ldf-spec-v0.md`, `crates/ldf`), from those
measurements. Read `docs/roadmap.md` and `docs/design-candidates.md` before
proposing syntax changes.

## Architecture

Cargo workspace, Rust stable, edition 2024.

| crate | role | binary |
|---|---|---|
| `ldf-data` | `Value` model (ordered keys, int/float kind kept), `Number` canonical text and JSON number grammar, `FlatSchema` inference, input readers for JSON (serde_json or simd-json), JSONL, CSV/TSV, Parquet (arrow-rs, feature `parquet`) | |
| `ldf-fixtures` | frozen PRNG (xoshiro256** + splitmix64), dataset catalog, streaming writers for JSON/JSONL/CSV/Parquet | `ldf-gen` |
| `ldf-formats` | `Format` trait, baseline encoders/decoders: json, json-pretty, jsonl, csv, tsv, markdown, toon, toon-tab, yaml, plus the `ldf` adapter (`ldf` current, `ldf-v0` old layout, same display name in prompts). TOON v4.4 is implemented from the spec and tested against the vendored official fixtures; YAML has its own emitter and decodes with yaml-rust2 | |
| `ldf-bench` | round-trip validation, process-isolated perf (io/parse/encode/decode, cold/warm, peak RSS), exact OpenAI token counts, Markdown and JSON reports, Criterion benches | `ldf-bench` |
| `ldf-eval` | task generation with ground truth (eleven kinds; new kinds are appended to `TaskKind::all` and a golden test pins the original stream, so old bundles regenerate as an exact prefix; the manifest reports per-kind counts, short where a dataset runs out of distinct questions), prompt bundles (`--candidate name=ID` adds an `ldf-discover` configuration, shown to the model as `ldf`), response scoring | `ldf-eval` |
| `ldf` | LDF v0.1 encoder and decoder (`docs/ldf-spec-v0.md`), plan analysis for flattening, child tables and the sparse-key `~` column (`Options::sparse_threshold`, default 0.5; `Options::V0` for the old layout), conformance fixtures under `tests/fixtures`; feature `arrow` adds `ldf::arrow`, a direct RecordBatch encoder (batch and block form) byte-identical to the Value path, which refuses lists of structs (child tables) so the caller falls back | |
| `ldf-mcp` | MCP stdio proxy: forwards every JSON-RPC message, appends an LDF primer to `initialize` instructions and a note to tool descriptions, re-encodes JSON `tools/call` text results as LDF when shorter (`rewrite.rs` holds the pure rules, `proxy.rs` the pumps). `convert` CLI for files, `fake-server` test double; `clickhouse` mode is its own MCP server with `list_tables`, `describe_table`, `run_select` (readonly=1) over ClickHouse HTTP; `files` mode serves a directory of data files through the `ldf-data` streaming readers (`list_files`, `describe_file`, `read_rows` paged); both return rows as LDF with a note block when a cap is hit; `rpc.rs` is the shared stdio loop; `--model ID` in every server mode picks JSON or LDF per result through `ldf-select` (JSON and no primer for a model without evidence, LDF-when-shorter without the flag) | `ldf-mcp` |
| `ldf-select` | the selector's decision rule, shared by `ldf-discover select` and `ldf-mcp --model`: dataset profile and traits, evidence rows, policy (n >= 50, drop <= 3 points, sign test, rows <= measured, saving >= 5 percent), `select_ldf` for JSON-or-LDF in bytes; `evidence.json` is a built-in copy of `docs/reports/evidence/evidence.json` (a test fails when they differ; `evidence.py` writes both) | |
| `ldf-discover` | format discovery engine, step 1: `knobs.rs` is the search space (13 knobs, 154 440 valid configurations; `Config::LDF_V01` and `LDF_V0` are points in it, byte-identical to `crates/ldf`, tested), one encoder and one decoder for every point (`encode.rs`, `decode.rs`, planner in `plan.rs`), `Candidate` puts a config behind `Format` so `ldf-bench` round-trip checks it, `stage_a.rs` hashes every output, round-trips and o200k-counts the distinct ones, ranks against JSON and v0.1; `select.rs` runs the `ldf-select` rule over JSON, TOON and LDF v0.1 at sparse threshold 0.5 or 0.75 in o200k tokens, from `docs/reports/evidence/evidence.json`, JSON whenever evidence is missing | `ldf-discover` |

`site/` is the website (ldflabs.org): Astro with Starlight, the docs pages
mirrored from `docs/` and this file by `site/scripts/sync-docs.mjs` at build
time, deployed by Cloudflare from the repository (root directory `/site`,
`bun run build`). Its two npm dependencies are pinned exactly and locked;
`bun install --frozen-lockfile`, never a bare install.

Python under `eval/` is standard library only: `run_eval.py` (API calls,
spend-gated), `count_tokens_claude.py` (Anthropic count_tokens),
`bench_mcp.py` (end-to-end MCP tool-call timing, LDF vs `--passthrough`) and
`analysis/evidence.py` (regrades every run into the selector's evidence
table, identifying each bundle's real LDF layout by re-encoding).

Data flow: `ldf-gen` writes fixtures to `fixtures/<scale>/`; `ldf-bench`
reads them through `ldf-data::input`, encodes with `ldf-formats`, and writes
`results/<name>/{perf,tokens}.{json,md}`; `ldf-eval build` writes a bundle
under `eval/out/`, `eval/run_eval.py` appends responses, `ldf-eval score`
grades them. Curated reports are copied to `docs/reports/`.

## Commands

```bash
cargo build --release                 # everything
cargo test                            # unit, round-trip and TOON conformance tests
cargo clippy --all-targets            # keep clean
cargo fmt --all -- --check           # CI fails on formatting drift; clippy runs there with -D warnings
cargo bench -p ldf-bench              # Criterion microbenchmarks
./target/release/ldf-gen list         # dataset catalog
./target/release/ldf-gen --scale small          # regenerate committed fixtures
./target/release/ldf-gen --scale medium         # 10k-row fixtures (gitignored)
./target/release/ldf-gen --scale medium --seed 20261118 --out fixtures/holdout --containers json   # held-out data for Stage C (gitignored)
./target/release/ldf-bench roundtrip --scale small
./target/release/ldf-bench perf --scale medium --iters 7 --out results/medium
./target/release/ldf-bench tokens --scale medium --sample 1000 --out results/medium
./target/release/ldf-eval build --input fixtures/small/flat_users.json --out eval/out/flat_users
python3 eval/run_eval.py --bundle eval/out/flat_users --provider anthropic --model claude-opus-5-5 --dry-run
eval/plan.sh                          # the agreed paid stages with prices and budget caps; `score` regrades everything
eval/plan.sh stage-b-build            # Stage B bundles, free and deterministic; the paid stages need Caio's go
./target/release/ldf-mcp proxy -- <mcp server command>   # LDF between a stdio MCP server and the model
./target/release/ldf-mcp convert FILE  # json, jsonl, csv, tsv or parquet to LDF on stdout
./target/release/ldf-mcp clickhouse --url http://127.0.0.1:8124   # MCP server over ClickHouse HTTP, see docs/ldf-mcp.md
./target/release/ldf-mcp files --root fixtures/small --model claude-haiku-5-5   # JSON or LDF per result from measured evidence
./target/release/ldf-eval score --bundle eval/out/flat_users --responses eval/out/flat_users/responses.anthropic.claude-opus-5-5.jsonl
./target/release/ldf-discover space   # knob list and size
./target/release/ldf-discover stage-a --scale medium --sample 200 --recheck 1000 --out results/discovery/<name>   # about a minute, no API calls
./target/release/ldf-discover show --id v0.1 --input fixtures/small/nested_orders.json   # one candidate's output; --id takes any id from a report
./target/release/ldf-bench arrow fixtures/medium/*.parquet --iters 5 --out results/arrow   # Parquet to LDF, Value path vs direct Arrow path, output hashes compared
python3 -I eval/analysis/evidence.py eval/out docs/reports/evidence   # rebuild the selector evidence after any graded run
./target/release/ldf-discover select --model claude-haiku-5-5 --input F --sample 200   # selector decision with its reasons
python3 -I eval/bench_mcp.py --mode files --root fixtures/medium --out results/mcp-files   # MCP end to end; --mode clickhouse needs CLICKHOUSE_PASSWORD
```

Benchmarks only count in release builds. Perf runs should happen on a quiet
machine with nothing else compiling.

## Engineering conventions

- Correctness before speed: every format is round-trip checked before it is
  timed, and an unsupported dataset is reported as such, never coerced.
- No fabricated numbers. A table in `docs/reports/` is a copy of a file that
  `ldf-bench` wrote, with the machine description it recorded. If something
  was not measured, the report says so.
- Equality has levels (byte, semantic, semantic-unordered, lossy). Say which
  one you mean.
- Streaming by default: readers yield records; encoders that can stream
  implement `StreamEncoder`. A format that needs the whole dataset first
  (TOON) declares `requires_count`.
- Numbers: domain is i64/u64/f64; canonical text rules live in
  `ldf_data::Number` and are shared by every text format. Do not add a second
  float formatter.
- Keep the data model free of format concerns. Format-specific quoting,
  escaping and layout stay in `ldf-formats`.
- Tests next to code for units; `tests/` for conformance suites and cross-crate
  round trips. The TOON fixtures are vendored verbatim with their licence; do
  not edit them, re-fetch on a spec bump.
- Named exports, no `unwrap` on user input paths, `clippy --all-targets`
  clean.
- Secrets never enter the repo. API keys are read by the Python scripts only,
  from the environment or from a gitignored `.env` at the repo root. The
  Rust code makes no calls to paid APIs; the only network code is
  `ldf-mcp clickhouse`, plain HTTP over `std::net` to the ClickHouse you point
  it at, password from `CLICKHOUSE_PASSWORD` only.
- No paid API runs without explicit approval from the project owner.
  `run_eval.py` enforces `--confirm-spend`; do not remove the gate.

## Dependencies

Every external crate is pinned to an exact version in `Cargo.toml` and the
lockfile is committed. Transitive versions were checked against crates.io
publish dates on 2026-10-17 and any version younger than seven days was pinned
back with `cargo update <crate> --precise <older>`; keep doing that after any
`cargo update`. The direct set is deliberately small: serde, serde_json
(`float_roundtrip`), simd-json, csv, arrow and parquet (feature gated),
tiktoken-rs, yaml-rust2 (decode only, default features off), criterion
(dev). Adding a crate is a discussion, not a commit.

## Measured facts worth knowing

- serde_json without `float_roundtrip` mis-rounds integer literals above u64
  by one ULP (`123456789012345680000`). The feature is on.
- TOON's tabular form needs every element of an array to be an object with
  the same keys and no array-valued fields, so an array of orders with an
  `items` array falls back to list form and ends up larger than compact JSON
  (medium `nested_orders`: 1.04x bytes).
- Arrow JSON schema inference refuses heterogeneous columns and typeless empty
  arrays; `mixed_types` and `empty_values` have no Parquet fixture.
- yaml-rust2 has no u64: integers above i64::MAX decode as f64.
- Frontier models explain before answering even when told not to; the
  scorer takes the trailing JSON value (`ldf_eval::answer::parse_response`).
- At 200 and 500 records a frontier model (Opus 5.5) is still format
  indifferent and only misses by running out of output cap on sums; weak
  models (Haiku 5.5, DeepSeek) show no stable format ordering, and Haiku
  reasons longer over LDF's sparse rows (870 vs 342 output tokens on
  sparse_events_200 with a 4 096 cap) and still loses two questions that
  need counting to a column across 200 mostly-empty rows; CSV and TSV lose
  the same question. Positional columns on sparse rows were the price of
  the token saving there; the v0.1 sparse-key rule (keys filled in under
  half the rows move into a per-row `~` object) brought Haiku to JSON's
  score at 25 percent fewer tokens than JSON, dense data unchanged.
  Tabular formats (CSV, TOON, LDF) lost a key lookup on
  Haiku to a one-row slide that JSON did not.
- LDF ties CSV on flat data and beats compact JSON by 20 to 64 percent on
  irregular data (tokens). Its codec is young: the decoder attaches child
  rows through a hash map keyed by the join value's canonical text, and the
  encoder uses positional key hints; before those two fixes nested decode
  ran at 1 MB/s. The quadratic column scans in planning and decode insert
  were removed on 2026-10-18 (wide_table: encode 55 to 28 ms, decode 50 to
  18 ms, now level with CSV encode and JSON decode).
- Stage A (2026-10-18, `docs/reports/2026-10-18-stage-a/`): over 154 440
  configurations on 200 records per dataset, the best lossless layout per
  dataset is 0 to 20 percent below LDF v0.1 in o200k tokens; one config for
  all datasets is 4 percent below v0.1 on average. The large gaps are nested_orders
  (hoisted `items` keys, 15 percent) and sparse_events (sparse threshold 0,
  20 percent, which is the v0 layout that cost Haiku accuracy). Keys on every
  row and repeated headers always cost tokens; typed and untyped headers tie;
  `true`/`false` beat `t`/`f` and `1`/`0`. long_text has no layout 10 percent
  below JSON. The engine's tests found two v0.1 encoder bugs, fixed the
  same day (spec sections 5 and 7.1, conformance fixtures added): a nested
  object null in some row decoded as absent, and a string starting with
  `@` as the first cell of a row read as a header. Output on all twelve
  medium fixtures is unchanged by the fix.
- Stage C (2026-10-19, `docs/reports/2026-10-19-stage-c/`): Opus 5.5 on
  held-out data, 57 to 95 questions per cell: LDF v0.1 ties JSON (294 of
  297) at 57 to 77 percent of JSON's Claude input tokens, so the selector
  now gives Opus LDF. Threshold 0.75 adds tokens and nothing on Opus (it
  helps DeepSeek); hoisted arrays save 17 percent on nested_orders for
  +1 / -2 (n.s.), no significant loss on any model so far. $5.96.
- 2026-10-19 (`docs/reports/2026-10-19-engine/`): the direct Arrow path is
  1.9x to 3.2x faster than the Value path in block form at 1M rows (2.8x to
  4.8x for a whole document, 1.4x to 4.3x less memory), same bytes. The
  selector gives JSON to GPT: its runs have 15 to 20 questions per cell,
  under the 50 the policy needs (Opus had the same gap until Stage C). LDF at sparse
  threshold 0.5 changes object member order on rows where some keys move to
  `~` (`semantic-unordered`, documented in spec section 7).
