# AGENTS.md

Working notes for anyone (human or agent) touching this repository. The
project is LDF, a research effort towards a lossless, token-efficient data
format for LLM consumption. Phase 1 (this state) is the measurement
foundation; the format itself is not designed yet. Read `docs/roadmap.md`
before proposing syntax.

## Architecture

Cargo workspace, Rust stable, edition 2024.

| crate | role | binary |
|---|---|---|
| `ldf-data` | `Value` model (ordered keys, int/float kind kept), `Number` canonical text and JSON number grammar, `FlatSchema` inference, input readers for JSON (serde_json or simd-json), JSONL, CSV/TSV, Parquet (arrow-rs, feature `parquet`) | |
| `ldf-fixtures` | frozen PRNG (xoshiro256** + splitmix64), dataset catalog, streaming writers for JSON/JSONL/CSV/Parquet | `ldf-gen` |
| `ldf-formats` | `Format` trait, baseline encoders/decoders: json, json-pretty, jsonl, csv, tsv, markdown, toon, toon-tab. TOON v4.4 is implemented from the spec and tested against the vendored official fixtures | |
| `ldf-bench` | round-trip validation, process-isolated perf (io/parse/encode/decode, cold/warm, peak RSS), exact OpenAI token counts, Markdown and JSON reports, Criterion benches | `ldf-bench` |
| `ldf-eval` | task generation with ground truth, prompt bundles, response scoring | `ldf-eval` |
| `ldf` | reserved for the format; empty | |

Python under `eval/` is standard library only: `run_eval.py` (API calls,
spend-gated) and `count_tokens_claude.py` (Anthropic count_tokens).

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
cargo bench -p ldf-bench              # Criterion microbenchmarks
./target/release/ldf-gen list         # dataset catalog
./target/release/ldf-gen --scale small          # regenerate committed fixtures
./target/release/ldf-gen --scale medium         # 10k-row fixtures (gitignored)
./target/release/ldf-bench roundtrip --scale small
./target/release/ldf-bench perf --scale medium --iters 7 --out results/medium
./target/release/ldf-bench tokens --scale medium --sample 1000 --out results/medium
./target/release/ldf-eval build --input fixtures/small/flat_users.json --out eval/out/flat_users
python3 eval/run_eval.py --bundle eval/out/flat_users --provider anthropic --model claude-opus-5-5 --dry-run
./target/release/ldf-eval score --bundle eval/out/flat_users --responses eval/out/flat_users/responses.anthropic.claude-opus-5-5.jsonl
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
- Secrets never enter the repo. API keys are read from environment variables
  by the Python scripts only. The Rust code makes no network calls.
- No paid API runs without explicit approval from the project owner.
  `run_eval.py` enforces `--confirm-spend`; do not remove the gate.

## Dependencies

Every external crate is pinned to an exact version in `Cargo.toml` and the
lockfile is committed. Transitive versions were checked against crates.io
publish dates on 2026-10-17 and any version younger than seven days was pinned
back with `cargo update <crate> --precise <older>`; keep doing that after any
`cargo update`. The direct set is deliberately small: serde, serde_json
(`float_roundtrip`), simd-json, csv, arrow and parquet (feature gated),
tiktoken-rs, criterion (dev). Adding a crate is a discussion, not a commit.

## Measured facts worth knowing

- serde_json without `float_roundtrip` mis-rounds integer literals above u64
  by one ULP (`123456789012345680000`). The feature is on.
- TOON's tabular form needs every element of an array to be an object with
  the same keys and no array-valued fields, so an array of orders with an
  `items` array falls back to list form and ends up larger than compact JSON
  (medium `nested_orders`: 1.04x bytes).
- Arrow JSON schema inference refuses heterogeneous columns and typeless empty
  arrays; `mixed_types` and `empty_values` have no Parquet fixture.
