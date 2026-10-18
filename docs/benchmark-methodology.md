# Benchmark methodology

This document states exactly what `ldf-bench`, `ldf-gen` and `ldf-eval`
measure, how, and what the known blind spots are. Numbers that appear in
`docs/reports/` were produced by these procedures; nothing else is reported.

## 1. Data model and equality

All formats are driven from one in-memory model (`ldf-data::Value`): the JSON
data model with ordered object keys and integer-vs-float kind preserved.
Documented numeric domain: `i64`, `u64`, `f64`. JSON integer literals outside
`u64` are parsed to the nearest `f64`; that loss is detectable in the
round-trip check and is never hidden.

serde_json is compiled with `float_roundtrip`. Without it, its parser is not
correctly rounded for integer literals above the `u64` range (observed: the
fixture `123456789012345680000` parsed one ULP low). The feature makes float
parsing somewhat slower; every JSON parse number in the reports includes that
cost.

Round-trip levels, checked before any timing:

| level | meaning |
|---|---|
| byte | decode(encode(x)) is structurally identical to x (number kinds included) and re-encodes to the same bytes |
| semantic | equal by mathematical value with key order preserved (`1.0` vs `1` allowed) |
| semantic-unordered | equal ignoring object key order |
| lossy | decodes, but a value differs; the first differing path is recorded |
| unsupported | the format refused the dataset and said why |
| decode-error | the format emitted something it cannot read back |

Formats are never allowed to flatten or retype silently. CSV, TSV and Markdown
accept only flat records; their decoders apply one documented inference rule
(empty cell to null, `true`/`false`, the JSON number grammar), so strings such
as `"42"` or `"null"` come back retyped and the dataset is marked lossy.

## 2. Datasets

`ldf-gen` produces every dataset from a frozen PRNG (xoshiro256** seeded by
splitmix64, implemented in `ldf-fixtures::rng`, no external crate) so that
record `i` of dataset `d` under seed `s` is a pure function. The default seed
is 20261017. The committed `fixtures/small` tree is the correctness set;
`medium` and `large` are regenerated locally (`ldf-gen --scale medium`).

| dataset | shape | rows small / medium / large | what it stresses |
|---|---|---|---|
| flat_users | flat, uniform | 20 / 10k / 1M | the best case for every tabular format |
| nested_orders | nested (2 levels, array of objects) | 20 / 10k / 500k | nested structure, tabular fallback |
| sparse_events | flat, non-uniform | 20 / 10k / 1M | missing vs null, union schemas |
| high_card_strings | flat, uniform | 20 / 10k / 500k | incompressible strings, tokenizer cost |
| repetitive_logs | flat, uniform | 20 / 10k / 1M | low cardinality, repetition |
| unicode_strings | flat, uniform | 20 / 10k / 200k | quoting and escaping rules |
| numeric_mix | flat, uniform | 20 / 10k / 1M | number formatting and precision |
| mixed_types | nested, non-uniform | 20 / 10k / 500k | heterogeneous value types |
| wide_table | flat, uniform, 41 columns | 20 / 10k / 1M | wide tables, header amortisation |
| deep_nesting | nested, 4-16 levels | 10 / 2k / 50k | depth, arrays of arrays |
| empty_values | nested | 20 / 10k / 200k | `[]`, `{}`, `""`, null, awkward keys |
| long_text | flat, uniform | 10 / 2k / 100k | long free text |

Schemas and generator parameters are in `crates/ldf-fixtures/src/datasets.rs`
(the `catalog()` function is the source of truth; `ldf-gen list` prints it).

Containers written per dataset: JSON array, JSONL, CSV (flat and uniform
only), Parquet (derived from the JSONL through arrow-json schema inference;
skipped and reported when inference cannot type the data, which happens for
`mixed_types` and `empty_values`).

## 3. Performance measurement

`ldf-bench perf` runs each (dataset, operation, format) in a **separate child
process**:

* **io**: `std::fs::read` of the input file. Page cache is warm after the
  first iteration; the first iteration is reported as "cold" but is only
  cold with respect to this process, not the OS cache.
* **parse**: input bytes already in memory (JSON) or a fresh reader (JSONL,
  CSV, Parquet) to `Vec<Value>`. Run once per JSON engine (serde_json,
  simd-json) for JSON and JSONL inputs.
* **encode**: in-memory records to a freshly allocated output buffer. The
  buffer allocation is inside the timed region for every format.
* **decode**: the format's own output (produced once, outside timing) back
  to records.

Iteration 0 is the cold run (first execution in a fresh process: code pages,
allocator, branch predictors all cold). Iterations 1..N are warm; tables show
their median and p95, and the JSON output keeps every sample.

Throughput is `bytes / median seconds` with MB = 1e6 bytes, where bytes are
the input size for io/parse/decode and the output size for encode. Records
per second is also recorded.

Peak memory is `ru_maxrss` from `getrusage(RUSAGE_SELF)` read in the child
after the operation. It is a process high-water mark, so for encode and
decode it includes the loaded input records. The "before" reading (after
loading, before the operation) is in the JSON so the increment is derivable.

Fixture generation is never inside a timed region: benchmarks read files
that `ldf-gen` wrote earlier.

Criterion microbenchmarks (`cargo bench -p ldf-bench`) time encode and decode
in-process on generated records with Criterion's statistical machinery. They
are better for comparing two encoders on the same data; the process-isolated
runs are better for memory and cold-start numbers.

Always use `--release`. The reports record CPU model, core count, RAM, OS,
rustc version, crate versions (from the committed Cargo.lock) and git commit.

### Known limitations

* No CPU pinning, no frequency control, no disabled background processes.
  Medians over 6 warm iterations on a quiet laptop are repeatable to within
  a few percent for operations above a millisecond; sub-millisecond numbers
  on the small fixtures are noise-dominated and should be read as orders of
  magnitude only.
* `ru_maxrss` cannot measure the memory of an operation that peaks lower
  than an earlier phase of the same process.
* Parquet "parse" time includes decompression and Arrow to `Value`
  conversion; it is not comparable to a columnar Arrow pipeline that never
  materialises rows.

## 4. Token measurement

`ldf-bench tokens` encodes each dataset with each format and counts tokens
with `tiktoken-rs` (`o200k_base` for GPT-4o/4.1/5 family, `cl100k_base` for
GPT-4/3.5). These are exact: the same BPE ranks OpenAI publishes, no special
tokens, `encode_ordinary`. Encoded documents are written to `encoded/` under
the results directory.

Anthropic does not publish the Claude 3+ tokenizer. Exact Claude counts come
from two places only: the `count_tokens` endpoint (free, needs an API key;
`eval/count_tokens_claude.py` adds them to a results directory) and the
`usage.input_tokens` field of real responses recorded by `eval/run_eval.py`.
No report shows an estimated Claude count labelled as a measurement.

Bytes per token is reported alongside counts because it separates "the format
is shorter" from "the format tokenizes better".

## 5. LLM comprehension evaluation

`ldf-eval build` takes a bounded sample of records (default 50) and generates
tasks with programmatic ground truth:

| kind | question shape |
|---|---|
| lookup | value of field F for the record whose key is K |
| filter | keys of all records with numeric F above/below a pivot (1..12 results) |
| aggregate | count, sum, min, max or mean of F |
| compare | which of two records has the larger F |
| missing_value | value of an optional field that is absent or null (answer: null) |
| nested_extract | value at a dotted path inside a nested record |
| multi_record | top-k keys by F, highest first, with a strict gap after k |

One prompt per (task, format). The system prompt and the user template are
byte-identical across formats except for the format name and the data block
(`crates/ldf-eval/src/bundle.rs`). Sampling is fixed by the runner
(temperature 0 where the provider allows it, fixed `max_tokens`, model id
pinned by the caller). Prompt token counts (o200k) are in the manifest so
context budgets can be checked before spending.

Outcomes are reported separately: correct, wrong, malformed (unparseable
answer), truncated (`finish_reason` length / `max_tokens`), context exceeded,
api error, and format unsupported (no prompt existed because the format could
not encode the dataset). Accuracy is `correct / n` over prompts actually sent.

`eval/run_eval.py` refuses to send anything without `--confirm-spend` and
prints the prompt count and total o200k prompt tokens first. API keys come
from `OPENAI_API_KEY`, `ANTHROPIC_API_KEY` and `DEEPSEEK_API_KEY`, read from
the environment or from a gitignored `.env` at the repo root.

### Caveats

* 50-record samples fit comfortably in every current context window; the
  harness is for comparing formats at equal information content, not for
  stress-testing context limits. Record counts can be raised per bundle and
  the manifest reports the largest prompt.
* A model may be more familiar with JSON than with any new format; the
  evaluation measures comprehension of the serialization as presented, with
  no few-shot examples, which disadvantages unfamiliar syntaxes. Report both
  zero-shot and, later, one-example conditions before drawing conclusions.
* Answer comparison is lenient on representation (quoting, code fences,
  `"42"` vs `42`) and strict on content. Aggregates allow 0.5 percent
  relative error; everything else is exact.
