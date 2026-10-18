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

* **Claude Opus 5.5**, effort low, `max_tokens` 256 (1 024 for the LDF rows,
  which ran on 2026-10-18), prompt caching on the data block (92 percent of
  input tokens were cache reads). Cost under $3, plus $0.14 for the LDF rows.
* **GPT-6.1 Sol**, `reasoning_effort` low, `max_completion_tokens` 2048,
  temperature rejected by the model and dropped. No cache hits were reported
  by the API (`cached_tokens` 0 on every call), so input was billed in full.
  Cost $2.65 for 364 prompts.

Correct answers out of prompts sent. YAML was added on 2026-10-18 and has
GPT results only; LDF's Claude rows ran on 2026-10-18 with the same prompts:

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
| **ldf** | **15 / 15** | **20 / 20** | **17 / 18** | **1 545 / 3 430 / 1 884** |

GPT answered all 470 prompts correctly, in every format including LDF
zero-shot (the prompt names the format and shows the data, nothing more),
and never added prose. LDF was the cheapest input in all three bundles: on
nested orders 34 percent below compact JSON and 38 percent below TOON, on
sparse events 33 percent below JSON and 45 percent below TOON, on flat users
within 1 percent of CSV.

Claude answered 52 of 53 LDF prompts. On nested_orders LDF is the only
format where Claude scored 20 of 20 (the others lost the "mean of total"
question). The one miss is compare-02 on sparse_events: the right record,
returned as `{"answer":35}` instead of `35`, which the scorer counts as
wrong. Claude's input for LDF was 2 084 / 4 560 / 2 562 tokens per prompt
(flat / nested / sparse), against 3 548 / 7 733 / 3 878 for compact JSON.

**First size step.** flat_users at 200 records, GPT, stopped after 25 of 150
prompts when the account ran out of credit (`eval/flat_users_200.*`): json
15 of 15, json-pretty 9 of 10. The miss is the sum over 200 scores:
json-pretty answered 10 077.78 for a true 9 677.78, off by exactly 400, a
row double-counted or a transposition, after 1 326 reasoning tokens. The
same question in compact JSON was right after 1 891 reasoning tokens. One
data point, but the first format-correlated error, and it appeared where the
literature says it should: at larger inputs on aggregation. The remaining 125
prompts were not sent; the OpenAI credit ran out, and the 200-record
question moved to Claude and DeepSeek (next section). Its input token counts equal the o200k_base counts in
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

## Size step: 200 and 500 records (2026-10-18)

Three more models on the 200 and 500 record bundles of flat_users and
sparse_events, same tasks, same prompt template, all ten formats. Raw
responses, tasks and per-task tables are in `eval/` (`*_200.*`, `*_500.*`).

* **Claude Opus 5.5**, effort low, `max_tokens` 1 024, caching on the data
  block (93 percent cache reads), flat_users_200 only. Runner estimate $1.27.
* **Claude Haiku 5.5**, effort low, `max_tokens` 1 024, 93 to 94 percent cache
  reads, all four bundles. $0.33.
* **DeepSeek `deepseek-chat`**, temperature 0, `max_tokens` 1 024, 91 to 94
  percent cache hits, all four bundles. $0.56.

Correct answers per cell, with the misses split by cause: `T` is a response
that hit the 1 024 token output cap (for the Claude models that cap includes
thinking tokens, so `T` means "ran out of room to think", not a wrong
answer), `W` is a finished answer that was wrong. No response was malformed.

| format | flat 200 (15) Opus / Haiku / DeepSeek | flat 500 (15) Haiku / DeepSeek | sparse 200 (18) Haiku / DeepSeek | sparse 500 (18) Haiku / DeepSeek |
|---|---:|---:|---:|---:|
| json | 13 (2T) / 12 (3T) / 11 (4W) | 10 (4T) (1W) / 11 (4W) | 16 (2T) / 12 (6W) | 15 (2T) (1W) / 12 (6W) |
| json-pretty | 13 (2T) / 10 (4T) (1W) / 11 (4W) | 11 (3T) (1W) / 11 (4W) | 16 (2T) / 12 (6W) | 14 (4T) / 13 (5W) |
| jsonl | 13 (2T) / 13 (2T) / 10 (5W) | 11 (2T) (2W) / 11 (4W) | 16 (2T) / 12 (6W) | 15 (3T) / 13 (5W) |
| csv | 13 (2T) / 10 (3T) (2W) / 10 (5W) | 11 (4T) / 9 (6W) | 15 (3T) / 10 (8W) | 13 (3T) (2W) / 13 (5W) |
| tsv | 13 (2T) / 11 (3T) (1W) / 9 (6W) | 13 (2T) / 9 (6W) | 14 (2T) (2W) / 9 (9W) | 13 (4T) (1W) / 12 (6W) |
| markdown | 14 (1T) / 12 (3T) / 11 (4W) | 11 (3T) (1W) / 11 (4W) | 16 (2T) / 11 (7W) | 15 (2T) (1W) / 12 (6W) |
| toon | 13 (2T) / 11 (3T) (1W) / 12 (3W) | 9 (3T) (3W) / 9 (6W) | 16 (2T) / 12 (6W) | 15 (3T) / 13 (5W) |
| toon-tab | 13 (2T) / 10 (3T) (2W) / 12 (3W) | 11 (3T) (1W) / 10 (5W) | 15 (3T) / 12 (6W) | 14 (4T) / 13 (5W) |
| yaml | 13 (2T) / 13 (2T) / 10 (5W) | 12 (2T) (1W) / 11 (4W) | 16 (2T) / 11 (7W) | 14 (4T) / 13 (5W) |
| **ldf** | **13 (2T) / 10 (2T) (3W) / 12 (3W)** | **10 (3T) (2W) / 8 (7W)** | **12 (5T) (1W) / 10 (8W)** | **12 (3T) (3W) / 13 (5W)** |

Input tokens per prompt as each provider counted them (Claude tokenizer
from the Haiku runs, DeepSeek tokenizer from the DeepSeek runs):

| format | flat 200 Claude / DeepSeek | flat 500 Claude / DeepSeek | sparse 200 Claude / DeepSeek | sparse 500 Claude / DeepSeek |
|---|---:|---:|---:|---:|
| json | 13 742 / 10 086 | 34 136 / 25 030 | 15 317 / 11 389 | 37 811 / 28 094 |
| json-pretty | 18 947 / 15 289 | 47 141 / 38 033 | 20 668 / 16 737 | 51 113 / 41 393 |
| jsonl | 13 942 / 10 086 | 34 636 / 25 030 | 15 517 / 11 518 | 38 311 / 28 427 |
| csv | 7 755 / 6 159 | 19 147 / 15 210 | 9 383 / 7 198 | 22 984 / 17 569 |
| tsv | 7 756 / 6 223 | 19 148 / 15 398 | 7 971 / 7 133 | 19 395 / 17 377 |
| markdown | 9 784 / 7 255 | 24 176 / 17 931 | 12 683 / 10 951 | 31 220 / 26 988 |
| toon | 7 961 / 6 364 | 19 653 / 15 715 | 16 245 / 13 746 | 40 113 / 33 995 |
| toon-tab | 7 965 / 6 430 | 19 657 / 15 905 | 16 249 / 13 749 | 40 117 / 33 998 |
| yaml | 13 957 / 12 231 | 34 683 / 30 425 | 15 717 / 13 482 | 38 835 / 33 357 |
| **ldf** | **7 778 / 6 174** | **19 172 / 15 227** | **9 661 / 7 481** | **23 640 / 18 308** |

What the tables say:

* **The frontier model is still format-indifferent at 200 records.** Opus
  answered 131 of 150 with zero wrong answers. All 19 misses are the "sum of
  score" and "mean of age" questions, where it spent the whole cap thinking,
  in every format; markdown got the mean through. LDF is 13 of 15 like
  everything else, at 7 778 input tokens per prompt against 7 755 for CSV,
  13 742 for compact JSON and 18 947 for pretty JSON.
* **Weak models separate from the frontier, not from each other by format.**
  Haiku scored 10 to 16 per cell and DeepSeek 8 to 13. No ordering of
  formats survives across bundles: jsonl and yaml lead flat 200, tsv leads
  flat 500, six formats tie on sparse 200. DeepSeek fails the same questions
  in every format (filter lists, sums, top-3 by a float field), so its misses
  belong to the model.
* **LDF is at or near the bottom for Haiku** (10, 10, 12, 12) and swings from
  tied best to worst for DeepSeek (12, 8, 10, 13). Most of the gap is one to
  three answers, which is the swing between any two formats here. The one
  cell that is not noise is Haiku on sparse_events_200: five truncations
  against two for JSON, on two filters, a top-3 and the two sums. Haiku's
  mean output (thinking included) on that bundle was 403 tokens for LDF
  against 251 for JSON, so the model reasons longer over LDF's sparse rows
  (nullable `?` columns, empty cells). Whether a higher cap turns those into
  correct answers is untested; the rerun costs about ten cents.
* **First evidence that tabular forms lose accuracy on key lookup.** Haiku,
  asked for the email of id 140 in flat_users_200, answered with the row
  below in CSV, TOON and LDF, and said why: "pia.139@uni.edu is id 139, so
  id 140 is Nia Matsumoto". The fixture's emails carry id minus one and the
  rows are sorted by id; the model slid one row in every format where the id
  is a cell in a row rather than a key next to its value. JSON, JSONL,
  Markdown and YAML were right. This is the research note's open question
  ("where do tabular forms lose accuracy"), answered once, on one model.
* **Two wrapped answers.** Opus (sparse_events, 50 records) and Haiku
  (sparse_events_200) both picked the right record on a compare question and
  returned it as `{"answer":35}` and `{"event_id":94}`; the scorer takes the
  trailing JSON value as given and counts both as wrong. Both were LDF rows.

Reading: the token advantage of LDF holds on every tokenizer and bundle
(within 4 percent of CSV, 34 to 44 percent below compact JSON, 40 to 46
percent below TOON on sparse data), and a frontier model reads it as well as anything else at 200
records. Below the frontier, no format is reliably better, and LDF's sparse
encoding costs a weak model extra reasoning. That is the first format-level
finding against LDF, and the next design question for v1.

### Rerun with a 4 096 token cap (Haiku 5.5, sparse_events_200)

Same 180 prompts, `max_tokens` 4 096 instead of 1 024, $0.08
(`eval/sparse_events_200.*.cap4096.*`):

| format | cap 1 024 | cap 4 096 | mean output tokens, 1 024 / 4 096 |
|---|---:|---:|---:|
| json | 16 (2T) | 17 (1W) | 252 / 342 |
| json-pretty | 16 (2T) | 17 (1W) | 251 / 329 |
| jsonl | 16 (2T) | 18 | 245 / 441 |
| csv | 15 (3T) | 16 (1T) (1W) | 357 / 563 |
| tsv | 14 (2T) (2W) | 15 (3W) | 337 / 577 |
| markdown | 16 (2T) | 17 (1W) | 279 / 449 |
| toon | 16 (2T) | 18 | 235 / 387 |
| toon-tab | 15 (3T) | 17 (1W) | 253 / 382 |
| yaml | 16 (2T) | 16 (2W) | 258 / 382 |
| **ldf** | **12 (5T) (1W)** | **13 (2T) (3W)** | **403 / 870** |

The extra room converts almost every other format's truncations into
correct answers (15 to 18 of 18, one truncation left in CSV). LDF gains one:
two questions still fill 4 096 tokens of thinking (every record whose
"retries" is less than 2; the sum of "latency_ms"), and the answers it does
finish include two more wrapped values (`{"event_id": 138}`) and one wrong
top-3 by "amount". Its mean output is 870 tokens against 342 for JSON and 387
for TOON.

The mechanism is visible in the data. sparse_events has 15 columns and most
cells are empty, so an LDF or CSV row is a run of commas:
`4,signup,2023-11-14T22:13:44Z,,,,,,4326,MX,false,,,,`. Answering "retries
less than 2" means counting to the eighth cell in every one of 200 such
rows. CSV and TSV got that question wrong at both caps and LDF never
finished it; JSON, JSONL, YAML and TOON (which falls back to its keyed list
form on this dataset) carry the key next to each value and got it right.
LDF adds a second thing to track, `null` versus an empty cell, which the
other positional formats do not have. So the cost is positional columns on
sparse rows, which is the price of the token saving on this dataset (9 661
tokens per prompt against 16 245 for TOON), and a weak model pays it in
reasoning and sometimes in answers. The wrapped answers are a separate,
smaller effect: all three seen so far (Opus once, Haiku twice) were on LDF,
whose header names the columns the model then echoes as keys.

## Codec update, 2026-10-18: wide tables

`perf-medium.md` recorded LDF at 56 ms encode and 50 ms decode on
wide_table (41 columns, 10 000 rows), four to seven times slower than
serde_json and twice CSV. Two scans were quadratic in the column count: the
planner checked every key of every row against the key list with a linear
`contains`, and the decoder looked each cell's key up in the row object
before pushing it. The planner now compares a row's key list positionally
against the first row's (one pass, and a set only for rows that differ),
and the decoder pushes a key without a lookup when no earlier column shares
its first path segment. Same machine, `ldf-bench perf --input
fixtures/medium/wide_table.json --iters 7`, before in
`perf-wide-table-2026-10-18-before.md`, after in
`perf-wide-table-2026-10-18.md`:

| format | encode median before | after | decode median before | after |
|---|---:|---:|---:|---:|
| json | 8.03 ms | 8.34 ms | 17.72 ms | 17.87 ms |
| csv | 24.98 ms | 26.53 ms | 12.79 ms | 13.71 ms |
| **ldf** | **54.53 ms** | **27.85 ms** | **49.70 ms** | **17.71 ms** |

LDF decode now sits at JSON's decode time and LDF encode at CSV's, so what
remains is the cost of walking the value tree and writing cells, shared
with CSV. The other medium datasets were not re-run; their LDF numbers in
`perf-medium.md` predate this change and can only have improved.

## What was not measured

* No comprehension numbers beyond 500 records, none for GPT beyond the 25
  prompts at 200 records, and none for nested_orders beyond 50 records.
* The 4 096 token cap rerun covers sparse_events_200 only; the other three
  Haiku bundles still have truncation and inability mixed.
* No `count_tokens` runs; the Claude per-format input counts come from the
  `usage` field of real responses, which is the billed number.
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
