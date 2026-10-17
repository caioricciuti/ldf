# Roadmap

## Phase 1: research and benchmarking foundation (this repository state)

Done:

- Cargo workspace with six crates (`ldf-data`, `ldf-fixtures`, `ldf-formats`,
  `ldf-bench`, `ldf-eval`, `ldf` placeholder).
- Deterministic fixture generator, twelve datasets, three scales, four
  containers (JSON, JSONL, CSV, Parquet).
- Baselines: compact JSON, pretty JSON, JSONL, CSV, TSV, Markdown table, TOON
  v4.4 (comma and tab delimiters), each with decode and explicit
  unsupported-case reporting. TOON passes the full official conformance suite
  (160 encode, 404 decode fixtures).
- Process-isolated performance harness (I/O, parse, encode, decode, cold and
  warm, peak RSS), Criterion microbenchmarks, exact OpenAI token counts,
  evaluation bundle builder, scorer, and a stdlib-only Python API runner with a
  spend gate.
- Initial measured report on small and medium fixtures (`docs/reports/`).

Not done, by decision:

- No paid API calls were made. The comprehension evaluation is built and
  dry-run tested but has no results until a run is approved.
- No Claude token counts (needs an API key and a `count_tokens` call).
- No large-scale (1M row) numbers in the committed report; the tooling runs
  at that scale but the run was not part of the initial deliverable.

## Phase 2: measurements that inform the design (next)

1. Run `ldf-bench perf --scale large` for the flat and nested datasets and
   add the numbers to the report, including Parquet and JSONL parse paths.
2. Approve and run a first comprehension evaluation: GPT and Claude, 50
   records, all task kinds, formats json, jsonl, csv, toon, markdown. Record
   usage tokens (exact Claude counts fall out of this).
3. Add `eval/count_tokens_claude.py` results to the token tables.
4. Study the three questions the research notes leave open (see
   `docs/research.md`, "Open questions"): where tabular forms lose accuracy,
   how much header/schema repetition costs per record for each tokenizer, and
   whether quoting rules or delimiter choice move token counts materially.

## Phase 3: LDF design

Only after Phase 2 numbers exist. Design inputs, in priority order:

1. Measured accuracy per task kind and format, split by failure cause.
2. Measured tokens per record for each tokenizer, as a function of width,
   nesting and string content.
3. Streaming and bounded memory: the format must be encodable in one pass
   without a record count or a global schema commitment (TOON's `[N]` header
   and tabular detection both require the whole array first; this is a
   measured cost in the perf tables).
4. Lossless by construction: typed scalars, missing vs null, key order,
   numbers in the JSON domain, arbitrary strings.

Candidates to evaluate against each other, not to adopt by taste: schema
block plus rows; typed column headers; per-field delimiters chosen by data;
explicit null and missing markers; hybrid "table where uniform, document
where not" with a cheap switch.

## Phase 4: implementation and GB-scale paths

- `ldf` crate: streaming encoder and decoder over `RecordStream`, Arrow
  RecordBatch fast path for Parquet input without materialising `Value`.
- Compare direct streaming paths against Arrow for JSONL and CSV inputs;
  keep whichever is faster per input kind, measured.
- Round-trip and conformance fixtures for LDF itself, in the same style as
  the TOON suite.
