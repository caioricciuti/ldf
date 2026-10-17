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

## LLM comprehension (`eval/`)

Two models, three bundles of 50 records each, identical prompts. Tables, raw
responses and tasks are in `eval/`.

* **Claude Opus 5.5**, effort low, `max_tokens` 256, prompt caching on the
  data block (92 percent of input tokens were cache reads). Cost under $3.
* **GPT-6.1 Sol**, `reasoning_effort` low, `max_completion_tokens` 2048,
  temperature rejected by the model and dropped. No cache hits were reported
  by the API (`cached_tokens` 0 on every call), so input was billed in full.
  Cost $2.65 for 364 prompts.

Correct answers out of prompts sent. YAML and LDF were added on 2026-10-18
and have GPT results only (the Claude key was not available for that run):

| format | flat_users (15) Claude / GPT | nested_orders (20) Claude / GPT | sparse_events (18) Claude / GPT | GPT input tokens per prompt, flat / nested / sparse |
|---|---:|---:|---:|---|
| json | 15 / 15 | 19 / 20 | 17 / 18 | 2 447 / 5 176 / 2 826 |
| json-pretty | 15 / 15 | 19 / 20 | 17 / 18 | 3 798 / 9 091 / 4 165 |
| jsonl | 15 / 15 | 19 / 20 | 17 / 18 | 2 495 / 5 224 / 2 874 |
| csv | 14 / 15 | unsupported | 17 / 18 | 1 534 / - / 1 797 |
| tsv | 15 / 15 | unsupported | 16 / 18 | 1 528 / - / 1 789 |
| markdown | 15 / 15 | unsupported | 17 / 18 | 1 858 / - / 2 831 |
| toon | 14 / 15 | 19 / 20 | 17 / 18 | 1 638 / 5 549 / 3 436 |
| toon-tab | 15 / 15 | 19 / 20 | 17 / 18 | 1 633 / 5 551 / 3 438 |
| yaml | - / 15 | - / 20 | - / 18 | 3 026 / 6 476 / 3 371 |
| **ldf** | **- / 15** | **- / 20** | **- / 18** | **1 545 / 3 430 / 1 884** |

GPT answered all 470 prompts correctly, in every format including LDF
zero-shot (the prompt names the format and shows the data, nothing more),
and never added prose. LDF was the cheapest input in all three bundles: on
nested orders 34 percent below compact JSON and 38 percent below TOON, on
sparse events 33 percent below JSON and 45 percent below TOON, on flat users
within 1 percent of CSV.

**First size step.** flat_users at 200 records, GPT, stopped after 25 of 150
prompts when the account ran out of credit (`eval/flat_users_200.*`): json
15 of 15, json-pretty 9 of 10. The miss is the sum over 200 scores:
json-pretty answered 10 077.78 for a true 9 677.78, off by exactly 400, a
row double-counted or a transposition, after 1 326 reasoning tokens. The
same question in compact JSON was right after 1 891 reasoning tokens. One
data point, but the first format-correlated error, and it appeared where the
literature says it should: at larger inputs on aggregation. The remaining 125
prompts of that bundle resume with the same command. Its input token counts equal the o200k_base counts in
`tokens-*.md` exactly, which confirms those tables as real GPT cost. It spent
400 to 540 reasoning tokens on the "mean of total" question and got it right
in all five formats; Claude at low effort got that question wrong in the four
formats where it finished.

Claude's misses are one of three things, none format specific:

* **Truncation on sums.** The "sum of latency_ms" question made Claude
  write a running-total table and hit the 256-token output cap in all eight
  formats on sparse_events (and json on nested_orders). Scored as
  truncated, not wrong; the runner's default is now 1 024 tokens.
* **Arithmetic.** The "mean of total" question on nested_orders completed
  in four formats and all four answers were wrong by 2 to 10 percent.
* **Two-record comparison.** Three answers gave both values instead of
  picking one (csv flat, tsv sparse) or named the wrong record (toon flat).

Lookup, filter, missing-value, nested extraction and top-k were answered
correctly by both models in every format, including the "absent versus
null" questions, which CSV and TSV got right from empty cells.

Reading: at 50 records both models are at ceiling, so accuracy cannot rank
the formats and tokens are the only separator. On the nested bundle TOON is
7 percent more expensive than compact JSON for GPT and the only one of CSV,
TSV, Markdown and TOON that can hold the data; on the sparse bundle TOON
costs 22 percent more than compact JSON while CSV costs 36 percent less.
Separation, if it exists, has to come from larger samples or weaker models;
200 and 500 record bundles are built under `eval/out/`.

## What was not measured

* No comprehension numbers yet beyond 50 records or below the frontier tier
  (Haiku, mini and nano class models).
* No Claude token counts: needs the count_tokens endpoint and a key.
* No large-scale (1M row) runs.
* No cross-machine or Linux numbers.

## Size and tokens (medium fixtures, first 1 000 records, from `tokens-medium.md`)

Ratios to compact JSON. Bytes and o200k_base tokens.

| dataset | csv bytes | csv tok | markdown bytes | markdown tok | toon bytes | toon tok | yaml bytes | yaml tok | json-pretty tok | **ldf bytes** | **ldf tok** |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| flat_users | 0.45 | 0.60 | 0.58 | 0.74 | 0.46 | 0.65 | 0.99 | 1.25 | 1.58 | **0.46** | **0.61** |
| repetitive_logs | 0.48 | 0.65 | 0.59 | 0.74 | 0.50 | 0.69 | 0.97 | 1.19 | 1.54 | **0.48** | **0.65** |
| wide_table | 0.51 | 0.43 | 0.66 | 0.57 | 0.51 | 0.44 | 1.04 | 1.27 | 1.50 | **0.51** | **0.43** |
| high_card_strings | 0.78 | 0.88 | 0.84 | 0.93 | 0.79 | 0.90 | 0.97 | 1.07 | 1.19 | **0.78** | **0.88** |
| numeric_mix | 0.57 | 0.65 | 0.69 | 0.79 | 0.58 | 0.67 | 1.04 | 1.21 | 1.40 | **0.57** | **0.66** |
| unicode_strings | 0.47 | 0.52 | 0.59 | 0.65 | 0.53 | 0.61 | 0.99 | 1.13 | 1.53 | **0.50** | **0.55** |
| long_text | 0.97 | 0.94 | 0.98 | 0.97 | 0.97 | 0.97 | 1.00 | 1.02 | 1.07 | **0.97** | **0.96** |
| sparse_events | 0.45 (lossy) | 0.61 | 0.67 (lossy) | 0.97 | 1.11 | 1.22 | 0.99 | 1.20 | 1.49 | **0.49** | **0.63** |
| nested_orders | unsupported | | unsupported | | 1.04 | 1.08 | 1.13 | 1.26 | 1.77 | **0.46** | **0.64** |
| mixed_types | unsupported | | unsupported | | 1.21 | 1.33 | 1.10 | 1.34 | 2.00 | **0.64** | **0.68** |
| empty_values | unsupported | | unsupported | | 1.18 | 1.27 | 1.04 | 1.22 | 1.62 | **0.32** | **0.36** |
| deep_nesting | unsupported | | unsupported | | 2.50 | 1.82 | 2.64 | 1.88 | 2.45 | **0.68** | **0.80** |

LDF v0 (`docs/ldf-spec-v0.md`, added 2026-10-18) is the project's own
format, designed from the rows above it. It is within 1 to 3 percent of CSV
in tokens on every dataset CSV can represent, lossless where CSV is not, and
20 to 64 percent below compact JSON on every dataset where TOON is above
JSON. Round trip: byte-exact on 11 of 12 datasets; sparse_events decodes
semantically equal with keys in header order (inherent to a column layout).

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
* Block YAML is about the same size as compact JSON in bytes but costs 7 to
  34 percent more tokens on regular data (indentation and newlines tokenize
  worse than JSON's punctuation), and 1.9x on deep nesting. It is the only
  non-JSON baseline that round-trips byte-exact on all twelve datasets
  (integer-valued floats keep their `.0`).
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
| yaml | all 12 | | | |
| ldf | 11 | sparse_events (key order follows the header) | | |

## Throughput (medium fixtures, warm medians, `perf-medium.md`)

Ranges across datasets. MB = 1e6 bytes of the format's own output (encode)
or input (decode, parse).

| operation | serde_json / compact json | simd-json | csv (this repo, csv crate) | toon (this repo) | markdown (this repo) | yaml (own emitter, yaml-rust2 decoder) | ldf v0 (this repo) |
|---|---|---|---|---|---|---|---|
| parse JSON input | 200 to 2 100 MB/s | 270 to 1 900 MB/s; 20 to 40 percent faster than serde_json on 9 of 12 datasets, slower on deep_nesting and long_text | | | | | |
| encode | 580 to 2 000 MB/s | | 100 to 2 900 MB/s | 110 to 520 MB/s | 130 to 480 MB/s | 210 to 500 MB/s | 49 to 1 100 MB/s |
| decode | 220 to 2 200 MB/s | | 140 to 1 000 MB/s | 56 to 250 MB/s | 60 to 340 MB/s | 30 to 140 MB/s | 53 to 760 MB/s |

LDF's slow end is wide_table on both sides (49 and 53 MB/s): schema analysis
and object rebuilding cost O(columns) per cell in this first implementation.
On nested_orders LDF decodes at 81 MB/s against TOON's 123 MB/s and encodes
at 127 MB/s against 186 MB/s.

The YAML and final TOON numbers come from a rerun while an unrelated
network-bound process was active; compact JSON medians moved by under 7
percent between the two runs, which bounds the perturbation.

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
