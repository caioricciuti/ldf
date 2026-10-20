# Engine report, 2026-10-19: quality, tokens, throughput, latency

One page over everything measured so far for the serialization engine:
which representation a model reads as well as JSON, what it costs in
tokens, how fast it is produced, and what it adds to a tool call. Every
number below is copied from a file in this directory or from
`reports/evidence/`, each written by a tool in this repository.
Machine for the timings: Apple M2 Pro, 10 logical CPUs, macOS (Darwin
27.0.0), release profile, commit 2291f06 plus the uncommitted changes
described here.

## Status

| item | state |
|---|---|
| Direct Arrow RecordBatch to LDF path (`ldf::arrow`, feature `arrow`) | done, tested byte-identical to the Value path, benchmarked at 10k and 1M rows |
| `ldf-mcp convert` uses it for Parquet, with the Value path as fallback | done |
| Selector evidence table from every graded run (`eval/analysis/evidence.py`) | done, 179 rows |
| Adaptive selector (`ldf-discover select`, `ldf_discover::select`) | done, first version, rules below |
| MCP end-to-end bench (`eval/bench_mcp.py`), files mode | done |
| MCP end-to-end bench, ClickHouse mode | script ready, not run: needs `CLICKHOUSE_PASSWORD`, which only Caio types |
| Stage C (GPT and Claude on held-out data) | bundles built, stages in `eval/plan.sh`, priced, **not run** (needs approval) |
| Direct Arrow path for lists of structs (child tables) | planned; such schemas use the Value path today |
| Selector inside `ldf-mcp` (per-call choice) | planned; the selector is a library and CLI only |
| LDF v0.1 specification | unchanged |

## 1. Quality: where a model reads LDF or TOON as well as JSON

Source: `reports/evidence/evidence.md`. Each row is one model, one
dataset, one format, paired against JSON on the same questions; p is an
exact two-sided sign test on the questions where exactly one of the two
was right. Rows with at least 50 questions (Stage B and the sparse sweep,
200 records each):

| model | dataset (traits) | format | tokens / JSON | accuracy | JSON | +/- | p |
|---|---|---|---:|---:|---:|---|---:|
| claude-haiku-5-5 | flat_users (flat) | ldf | 0.61 | 97.8% | 97.8% | +2 / -2 | 1.00 |
| claude-haiku-5-5 | flat_users (flat) | toon | 0.65 | 96.6% | 97.8% | +2 / -3 | 1.00 |
| claude-haiku-5-5 | nested_orders (child tables, sparse) | ldf | 0.65 | 91.9% | 96.3% | +1 / -7 | 0.07 |
| claude-haiku-5-5 | nested_orders | toon | 1.08 | 92.6% | 96.3% | +3 / -8 | 0.23 |
| claude-haiku-5-5 | sparse_events (sparse) | ldf | 0.78 | 93.9% | 92.2% | +6 / -4 | 0.75 |
| claude-haiku-5-5 | sparse_events | toon | 1.22 | 93.9% | 92.2% | +6 / -4 | 0.75 |
| claude-haiku-5-5 | mixed_types (JSON cells, sparse) | ldf | 0.73 | 96.3% | 98.8% | +0 / -2 | 0.50 |
| claude-haiku-5-5 | mixed_types | toon | 1.33 | 100.0% | 98.8% | +1 / -0 | 1.00 |
| deepseek-chat | flat_users | ldf | 0.61 | 68.5% | 66.3% | +7 / -5 | 0.77 |
| deepseek-chat | nested_orders | ldf | 0.65 | 62.2% | 62.2% | +6 / -6 | 1.00 |
| deepseek-chat | sparse_events | ldf (threshold 0.5) | 0.78 | 52.2% | 60.9% | +5 / -15 | 0.04 |
| deepseek-chat | sparse_events | ldf (threshold 0.75) | 0.89 | 66.1% | 60.9% | +8 / -2 | 0.11 |
| deepseek-chat | sparse_events | toon | 1.22 | 62.6% | 60.9% | +6 / -4 | 0.75 |
| deepseek-chat | mixed_types | ldf | 0.73 | 100.0% | 100.0% | +0 / -0 | 1.00 |

Reading:

- No format is significantly better than JSON anywhere. The one
  significant loss is LDF at threshold 0.5 on sparse rows for DeepSeek;
  threshold 0.75 (a valid v0.1 encoder setting, no spec change) removes it.
- Haiku on nested data is 4.4 points below JSON with LDF (p=0.07) and 3.7
  with TOON (p=0.23). Not significant, but larger than the 3-point
  tolerance, so the selector keeps JSON there.
- Frontier models (Opus 5.5, GPT-6.1 Sol) have only 15 to 20 questions per
  cell (50 and 200 records). They tie JSON everywhere they were measured,
  but that is too few questions to separate a 5-point drop from noise. Stage
  C exists to fix this.
- Layouts outside v0.1 (hoisted arrays, tab delimiter, the `-` null marker)
  were measured in Stage B; none is better than v0.1 and `ldf-global` is
  significantly worse. No spec change is proposed from this data.

## 2. Token cost

o200k_base tokens of the data block, relative to compact JSON, 200 records
(Stage B manifests; exact for GPT, a proxy for Claude, which reads about
1.34 times as many tokens on these prompts by the Opus usage counts):

| dataset | CSV | TOON | LDF v0.1 | LDF threshold 0.75 |
|---|---:|---:|---:|---:|
| flat_users | 0.61 | 0.65 | 0.61 | 0.61 |
| nested_orders | n/a | 1.08 | 0.65 | 0.69 |
| sparse_events | 0.61 (lossy) | 1.22 | 0.78 | 0.89 |
| mixed_types | n/a | 1.33 | 0.73 | 0.73 |

On realistic tool results (section 4) LDF is 0.43 to 0.78 of JSON's
tokens; TOON is 0.44 to 0.69 on flat results and 1.08 to 1.22 on nested
and sparse ones.

## 3. Serialization throughput: Parquet to LDF

Source: `arrow-large.md` (1M rows; 500k for high_card_strings) and
`arrow-medium.md` (10k rows). File on disk to LDF text, median of 3 (large)
or 5 (medium) runs, each path in a fresh process. The direct path writes
the same bytes as the Value path in every case (`same` column, FNV-1a of
the whole output), at both thresholds and in batch and block form; the
tests in `crates/ldf/tests/arrow_path.rs` check this on every small
Parquet fixture and on edge cases (nulls in structs, `@` first cells,
Float32, u64 above i64, NaN, timestamps, lists, all-null columns,
uneven batch splits).

Block form (streaming, block of 1 024 rows, bounded memory):

| dataset | rows | LDF | Value path | direct path | speedup | peak RSS Value / direct |
|---|---:|---:|---:|---:|---:|---|
| flat_users | 1M | 65.6 MiB | 665.6 ms | 232.0 ms | 2.87x | 20.8 / 16.3 MiB |
| high_card_strings | 500k | 88.8 MiB | 401.7 ms | 215.8 ms | 1.86x | 31.9 / 36.8 MiB |
| numeric_mix | 1M | 132.9 MiB | 1081.2 ms | 559.4 ms | 1.93x | 32.9 / 22.8 MiB |
| repetitive_logs | 1M | 66.1 MiB | 630.0 ms | 199.8 ms | 3.15x | 14.0 / 12.4 MiB |
| sparse_events | 1M | 199.7 MiB | 1548.7 ms | 694.2 ms | 2.23x | 21.8 / 19.8 MiB |
| wide_table | 1M | 264.5 MiB | 3335.6 ms | 1384.4 ms | 2.41x | 83.8 / 49.9 MiB |

Whole-document form (one plan over all rows, what `ldf::encode` does):
2.76x to 4.77x faster and 1.4x to 4.3x less peak memory (wide_table: 6.18 s
and 3.8 GiB to 1.42 s and 874 MiB). The direct path runs at 195 to 431 MB/s
of LDF output. nested_orders (a list of structs, which LDF writes as a child
table) is refused by the direct path and goes through the Value path; that
is the next piece of work on this path.

## 4. End to end: MCP tool calls

Source: `mcp-files.md`. `ldf-mcp files` over `fixtures/medium`, one
long-lived server returning LDF and one with `--passthrough` returning
JSON, 20 timed calls per query after 3 warm-up calls, client-side wall time
per `tools/call` over stdio:

| query | rows | traits | JSON tokens | TOON / JSON | LDF / JSON | p50 ms JSON | p50 ms LDF | LDF overhead |
|---|---:|---|---:|---:|---:|---:|---:|---:|
| users_50 | 50 | flat | 2346 | 0.65 | 0.64 | 0.98 | 1.05 | +0.07 |
| events_200 | 200 | sparse | 11050 | 1.22 | 0.78 | 0.30 | 0.61 | +0.31 |
| orders_100 | 100 | child tables, sparse | 9914 | 1.08 | 0.65 | 12.30 | 12.50 | +0.20 |
| logs_500 | 500 | flat | 23052 | 0.69 | 0.65 | 0.90 | 1.17 | +0.28 |
| wide_1000 | 1000 | flat | 234202 | 0.44 | 0.43 | 5.45 | 8.54 | +3.09 |
| logs_10000 | 10000 | flat | 460419 | 0.69 | 0.65 | 8.09 | 13.69 | +5.60 |

orders_100 reads a 2.8 MB JSON file whole on every call, hence 12 ms in
both variants. Encoding adds 0.07 to 5.6 ms per
call (about 0.5 microseconds per row on the large reads). On the model side, the measured response latencies in the evidence
table show no consistent format effect: latency follows output length, not
input format (Haiku on the v0 sparse layout took 4.8 s against JSON's 2.3 s
because it wrote 1 059 output tokens instead of 395; on v0.1 it is 2.4 s).
Prefill savings from fewer input tokens are real but were not isolated:
the runs used prompt caching and were not interleaved in time.

ClickHouse mode: the same script with `--mode clickhouse` runs an agent-like
session (list_tables, describe_table, then 50 to 10 000 row SELECTs on the
users, events, orders, logs and wide tables). It needs the ClickHouse
password in the environment and was not run in this session.

## 5. The selector

`ldf-discover select --model M --input FILE` (library:
`ldf_discover::select::select`). Candidates: JSON, TOON, LDF v0.1 at sparse
threshold 0.5 and 0.75. A candidate other than JSON is accepted only when:

1. it round-trips this data (byte, semantic, or semantic-unordered: values
   all kept, object member order changed when keys move to the `~` column,
   as spec section 7 documents; `--require-key-order` turns that off);
2. for this exact model id there is evidence on a measured dataset with
   the same structural traits (child tables, JSON cells, sparse keys), at
   least 50 questions per row, and every such row is within 3 points of
   JSON and not significantly worse (sign test, alpha 0.05);
3. the data has no more rows than the largest measured dataset (200 today),
   unless `--extrapolate-rows` is given;
4. it saves at least 5 percent of JSON's tokens on this data.

The smallest accepted candidate wins; otherwise JSON. Decisions on the
held-out data (`selector-holdout.md`, with the full reasoning per case):

| dataset | traits | Haiku 5.5 | DeepSeek | Opus 5.5 | GPT-6.1 Sol | unknown model |
|---|---|---|---|---|---|---|
| flat_users | flat | ldf 0.5 | ldf 0.5 | json | json | json |
| nested_orders | child tables, sparse | json | ldf 0.5 | json | json | json |
| sparse_events | sparse | ldf 0.5 | ldf 0.75 | json | json | json |
| mixed_types | JSON cells, sparse | ldf 0.5 | ldf 0.5 | json | json | json |

On the MCP queries every result over 200 rows gets JSON by default, since
nothing larger has been measured with enough questions; with
`--extrapolate-rows` flat results go to LDF for Haiku and DeepSeek.

## 6. Stage C: priced, not run

Bundles: `eval/out/stage-c`, held-out records (`ldf-gen --seed 20261118`)
and questions (seed 2), 200 records, 82 to 135 questions per dataset. Only
renderings that differ are sent: 1 789 prompts, 20.6M o200k input tokens,
about 27.6M Claude input tokens, nearly all of it prompt-cache reads.

| stage | model | estimate | cap |
|---|---|---:|---:|
| `eval/plan.sh stage-c-opus` | claude-opus-5-5, effort low | $9 to $17 | $17 |
| `eval/plan.sh stage-c-sonnet` | claude-sonnet-5-5, effort low | $7.50 to $11.50 | $12 |
| `eval/plan.sh stage-c-gpt` | set `GPT_MODEL` and prices | input 20.6M o200k, output about 12 tokens per answer | `GPT_BUDGET` |

The Claude range is cache reads at $0.20 per million (about $5.50 for
either model) plus cache writes plus output, which is 76 tokens per answer
on average in earlier Opus runs but longer on the sum and rank questions
Stage C has more of. The GPT price is not in the repository and the GPT
credit ran out on 2026-10-18. Each run feeds `eval/analysis/evidence.py`
directly; the selector picks up the new rows with no code change.

## Limitations

- Evidence covers four synthetic datasets at 200 records and two
  non-frontier models with enough questions. Real data and other models
  are unknown workloads and get JSON.
- One answer per question, temperature or effort fixed, one run per
  configuration: run-to-run variance is not measured. The sparse sweep at
  0.75 was a separate run from its JSON baseline.
- Traits are coarse (three booleans). Two datasets with the same traits
  can still behave differently.
- o200k token counts are exact for GPT only. Claude counts come from usage
  fields and are about 1.34x o200k here.
- Timings are from one laptop, warm page cache, nothing else running.
  The ClickHouse end-to-end run is pending.
- Model-side latency was recorded but not controlled (caching, time of
  day, output length), so no latency claim about formats is made.
