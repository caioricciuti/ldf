# Baseline report, 2026-10-17

Every number here is copied from the files next to this README, which
`ldf-bench` wrote. Machine: Apple M2 Pro, 10 logical CPUs, 16 GiB, macOS
(Darwin 27.0.0), rustc 1.98.0, release profile. Crate versions are in each
file's header. Procedure: `docs/benchmark-methodology.md`.

Files:

| file | command |
|---|---|
| `perf-small.md` / `.json` | `ldf-bench perf --scale small --iters 7` (20-row fixtures; sub-millisecond, noise-dominated, kept for completeness) |
| `perf-medium.md` / `.json` | `ldf-bench perf --scale medium --iters 7` (10 000 rows, or 2 000 for deep_nesting and long_text) |
| `tokens-small.md` / `.json` | `ldf-bench tokens --scale small` |
| `tokens-medium.md` / `.json` | `ldf-bench tokens --scale medium --sample 1000` (first 1 000 records of each medium fixture) |

Criterion microbenchmarks were run (`cargo bench -p ldf-bench`); their HTML
and JSON output lives under `target/criterion/` and is not committed. They
agree with the process-isolated numbers within the noise discussed below.

## What was measured

* Round-trip level per format and dataset (byte, semantic, lossy,
  unsupported), checked before any timing.
* I/O, parse (serde_json and simd-json for JSON input), encode and decode
  wall time, cold (first iteration in a fresh process) and six warm
  iterations, one child process per measurement.
* Output bytes and ratio to compact JSON.
* Peak RSS of each child process after the operation.
* Exact `o200k_base` and `cl100k_base` token counts and bytes per token.

## What was not measured

* No LLM accuracy numbers: no paid API call was made. The evaluation
  bundles exist under `eval/out/` and were dry-run only.
* No Claude token counts: needs the count_tokens endpoint and a key.
* No large-scale (1M row) runs.
* No cross-machine or Linux numbers.

## Size and tokens (medium fixtures, first 1 000 records, from `tokens-medium.md`)

Ratios to compact JSON. Bytes and o200k_base tokens.

| dataset | csv bytes | csv tok | markdown bytes | markdown tok | toon bytes | toon tok | json-pretty tok |
|---|---:|---:|---:|---:|---:|---:|---:|
| flat_users | 0.45 | 0.60 | 0.58 | 0.74 | 0.46 | 0.65 | 1.58 |
| repetitive_logs | 0.48 | 0.65 | 0.59 | 0.74 | 0.50 | 0.69 | 1.54 |
| wide_table | 0.51 | 0.43 | 0.66 | 0.57 | 0.51 | 0.44 | 1.50 |
| high_card_strings | 0.78 | 0.88 | 0.84 | 0.93 | 0.79 | 0.90 | 1.19 |
| numeric_mix | 0.57 | 0.65 | 0.69 | 0.79 | 0.58 | 0.67 | 1.40 |
| unicode_strings | 0.47 | 0.52 | 0.59 | 0.65 | 0.53 | 0.61 | 1.53 |
| long_text | 0.97 | 0.94 | 0.98 | 0.97 | 0.97 | 0.97 | 1.07 |
| sparse_events | 0.45 (lossy) | 0.61 | 0.67 (lossy) | 0.97 | 1.11 | 1.22 | 1.49 |
| nested_orders | unsupported | | unsupported | | 1.04 | 1.08 | 1.77 |
| mixed_types | unsupported | | unsupported | | 1.21 | 1.33 | 2.00 |
| empty_values | unsupported | | unsupported | | 1.18 | 1.27 | 1.62 |
| deep_nesting | unsupported | | unsupported | | 2.50 | 1.82 | 2.45 |

Observations:

* On uniform flat data the two header-once formats (CSV, TOON) cut bytes by
  half and o200k tokens by a third. CSV is always at or below TOON.
* Token ratios are worse than byte ratios for every compact format on
  flat data: compact JSON's punctuation (`","`, `":"`, `"},{"`) merges into
  cheap tokens, so JSON has 2.9 to 3.2 bytes per token where CSV and TOON
  have 2.1 to 2.4. Exception: wide_table, where the 41 short keys repeated
  per JSON record tokenize poorly and the header-once formats win more in
  tokens (0.43) than in bytes (0.51).
* TOON costs more tokens than compact JSON on every dataset that is not
  uniform flat: nested orders (array-valued column), sparse events (varying
  key sets), mixed types, empty values, deep nesting.
* Markdown is the most expensive of the tabular formats and on sparse data
  it costs as many tokens as JSON while still being lossy.
* long_text is dominated by the body field; format choice moves tokens by
  at most 6 percent.
* JSON Lines costs 1 to 2 percent more tokens than a compact JSON array.
* o200k_base and cl100k_base agree to within 1 percent except on unicode
  strings, where o200k is 5 to 12 percent cheaper.

## Round trips (`perf-medium.md`)

| format | byte-exact | semantic only | lossy | unsupported |
|---|---|---|---|---|
| json, json-pretty, jsonl | all 12 | | | |
| toon, toon-tab | high_card_strings, repetitive_logs, unicode_strings, deep_nesting, empty_values, long_text | flat_users, nested_orders, sparse_events, mixed_types, wide_table (float kind lost: `1.0` becomes `1`) | numeric_mix (`huge` above 2^53, see research.md Part A item 6) | |
| csv, tsv | high_card_strings, repetitive_logs, long_text | flat_users | sparse_events (absent and null merge), unicode_strings (`""` becomes null), wide_table (`"null"` becomes null), numeric_mix | nested_orders, mixed_types, deep_nesting, empty_values |
| markdown | same as csv | | | |

## Throughput (medium fixtures, warm medians, `perf-medium.md`)

Ranges across datasets. MB = 1e6 bytes of the format's own output (encode)
or input (decode, parse).

| operation | serde_json / compact json | simd-json | csv (this repo, csv crate) | toon (this repo) | markdown (this repo) |
|---|---|---|---|---|---|
| parse JSON input | 200 to 2 100 MB/s | 270 to 1 900 MB/s; 20 to 40 percent faster than serde_json on 9 of 12 datasets, slower on deep_nesting and long_text | | | |
| encode | 580 to 2 000 MB/s | | 90 to 2 900 MB/s | 110 to 400 MB/s | 130 to 480 MB/s |
| decode | 220 to 2 200 MB/s | | 140 to 1 200 MB/s | 56 to 250 MB/s | 60 to 340 MB/s |

The slowest cell for every tabular format is wide_table (41 columns): csv
encodes it at 106 MB/s, toon at 114 MB/s, compact JSON at 634 MB/s. Column
lookup per cell dominates there; an earlier version of the TOON encoder
without a positional fast path was at 44 MB/s on the same data.

Peak RSS: parsing a 1.4 MiB JSON file into records needs about 15 MiB with
serde_json and 23 MiB with simd-json; the 5.2 MiB wide_table needs 64 MiB
and 112 MiB. Records as `Value` trees cost roughly 10x their JSON size in
memory, which is the argument for streaming paths at scale.

Caveats:

* The TOON, CSV and Markdown encoders and decoders here are this
  repository's implementations. Their speed is a property of this code, not
  of the formats. serde_json is a mature, heavily optimised library. The
  TOON decoder in particular is a line-oriented recursive-descent parser
  that allocates per line.
* Cold iterations are 5 to 30 percent slower than warm medians for
  operations above a millisecond; on the small fixtures cold-start noise
  swamps the signal.
* `ru_maxrss` includes the loaded input, so encode and decode RSS numbers
  are upper bounds on the operation's own footprint.

## Takeaways for the design phase

1. The byte-to-token gap is real and format dependent; design decisions
   must be checked against tokenizers, not byte counts.
2. Header-once tabular layout is worth about a third of the tokens on flat
   data and nothing on irregular data. The open design problem is irregular
   data: sparse keys, array-valued fields, heterogeneous types, where every
   compact format today costs more than compact JSON.
3. Missing-versus-null and float-versus-integer kind are where every
   non-JSON baseline is lossy. Any LDF proposal needs an explicit, cheap
   answer to both.
4. Streaming: of the baselines only JSON, JSONL and CSV can be emitted
   record by record. TOON needs the count and the whole-array shape first.
