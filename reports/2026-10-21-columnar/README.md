# Columnar database results, NULLs, and the selector's acceptance rule (2026-10-21)

Review of 0.5.3 for database MCP workflows, following
`reports/2026-10-21-clickhouse`, `reports/2026-10-21-duckdb` and
`reports/2026-10-20-large`. No model was called. One paid evaluation is
built and waits for approval (last section).

Machine: Apple M2 Pro, 10 cores, 16 GB, macOS 27.0.1. Repository at 05388dc
plus this branch. DuckDB 1.5.6, `mcp-server-motherduck==1.1.0` through
`uvx --exclude-newer 2026-10-14 --no-build`. Tokens are o200k_base from
`ldf-bench count`: exact for GPT, a proxy for Claude.

Reproduce:

```bash
cargo build --release
uv run --exclude-newer 2026-10-14 --no-build --no-project --with duckdb==1.5.6 python -I -c 'import duckdb, os; c = duckdb.connect("results/columnar/fixtures.duckdb"); [c.execute(f"CREATE OR REPLACE TABLE {f[:-6]} AS SELECT * FROM read_json_auto(?)", ["fixtures/medium/" + f]) for f in sorted(os.listdir("fixtures/medium")) if f.endswith(".jsonl")]'
python3 -I eval/bench_columnar.py --db results/columnar/fixtures.duckdb --out results/columnar
python3 -I eval/bench_mcp.py --mode duckdb --db results/mcp-duckdb/ldf.duckdb --repeats 10 --ldf-arg=--columnar --out results/mcp-duckdb-columnar
python3 -I eval/analysis/selector_audit.py reports/evidence/evidence.json > results/selector-audit.md
```

`columnar.md` / `columnar.json` are `bench_columnar.py`'s output,
`proxy/` is `bench_mcp.py`'s, `selector-audit.md` is the audit's, and
`texts/` keeps four sent results with their frames.

## 1. The columnar adapter

MotherDuck's server answers `execute_query` with pretty-printed
`{"success", "columns", "columnTypes", "rows": [[...]], "rowCount"}`, plus
`truncated` and `warning` when it caps rows. Errors come back with
`isError: true` and a prose prefix. Every result also carries the same text
in `structuredContent.result` (FastMCP's wrapping of a `str` return).

`ldf-mcp proxy --columnar` (opt-in, off by default) sends such a result as a
frame: line one is the result as compact JSON with `rows` set to
`"@records"`, every other member kept in place; then the rows as the LDF
table `@records`, one column per `columns` entry, `null` for SQL NULL. The
spec is unchanged: the table is ordinary LDF, the first line ordinary JSON.
Safety rules, all tested:

- The frame is decoded and compared with the parsed original (members,
  member order, integer or float kind) before it is sent; any difference
  sends JSON (`not_lossless`). `bench_columnar.py` checks it again
  independently in Python: 36 of 36 frames exact.
- Duplicate column names (legal SQL: `SELECT 1 AS h, 2 AS h`), ragged rows,
  empty results and non-string names are not treated as columnar.
- A number outside the value model stays JSON (`out_of_domain`). This also
  fixes the existing record path: before, an integer beyond u64 (DuckDB
  `HUGEINT`, seen in a probe: `100000000000000000000`) was converted to a
  float.
- `isError` results are never touched. `structuredContent` is left alone
  except in FastMCP's `{"result": S}` form with `S` the original text,
  where `S` gets the same rewritten text (section 5 found why).
- With `--model`, columnar results stay JSON: no model was graded on a
  frame, and the selector's sparse evidence has absent keys where a frame
  has NULLs (section 4).

Not covered yet: column metadata as objects (`[{"name", "type"}]`, as Trino
or ClickHouse `JSONCompact` `meta`/`data` send). Those are a second shape
with the same frame and wait for a server that needs them.

## 2. Original, minified and LDF

Every result `ldf-mcp` keeps as JSON already goes out minified (0.5.3). This
branch also makes LDF compete against the minified size, never the text as
sent, in the proxy, the hook and the selector's size check. That is a
behaviour change for pretty-printed record results that LDF beat only by
their whitespace: they now go out as minified JSON.

Twelve tables at 50, 200 and 1 000 rows plus six realistic aggregates and
joins (`columnar.md` for every row):

| shape | whitespace saved (sent to minified) | LDF frame vs minified |
|---|---:|---:|
| flat tables (users, logs, wide, numeric, strings, long text) | 5 to 59 percent | 0.1 to 6.9 percent smaller; `wide_table_50` 4 tokens larger |
| structs and lists (nested_orders, deep_nesting) | 44 to 55 percent | 15 to 25 percent smaller |
| mixed types, empty values | 43 to 53 percent | 22 to 28 percent smaller |
| `unnest` join, 200 rows | 42 percent | 13 percent smaller |
| GROUP BY results, 5 to 40 rows | 43 to 51 percent | 3 to 6 percent smaller, or JSON (too small) |
| NULL-heavy (sparse_events, its sparse columns) | 43 to 59 percent | JSON: LDF would be 1.6 to 2.4 times larger |

Most of what a database MCP server wastes is whitespace, and minifying
recovers it without any format. On flat SQL results LDF adds only a few
percent over that. Where the data has structure (structs, lists, JSON
columns), LDF saves 13 to 28 percent more, on any server.

The proxy decides by bytes, not tokens. On `wide_table_50` the frame is
smaller in bytes and 4 tokens (0.1 percent) larger. A token check would
need a tokenizer in the proxy, which is not worth it at that margin.

Latency through the real server (`proxy/mcp.md`, client-side p50): +0.3 to
+1.0 ms up to 500 rows, +16 ms at 1 000 wide rows and +25 ms at 10 000
rows, against +2 to +3 ms for minifying alone. Most of the difference is
the decode-and-compare check, which is the price of sending only exact
frames.

## 3. Why SQL NULLs grow under the sparse-key rule

Rule 4 of spec section 7 counts, per key, the rows where the key is present
**with a non-null value**. A SQL result has every column on every row, so a
mostly-NULL column has low fill, gets no column, and goes into the per-row
`~` object. The `~` object holds the row's sparse keys "that row has", and a
NULL column is present, so every row carries `"referrer":null,"amount":null,
...` as JSON pairs. A columnar JSON row writes each NULL as `null` with no
key. On `sparse_events` at 200 rows that is 13 725 tokens against 8 440
minified.

Layouts measured on the same rows (head line plus table, percent of
minified JSON; `columnar.md`, "NULL layouts"):

| result | kept, sparse 0.5 (default) | kept, sparse 0 | dropped, sparse 0.5 | dropped, sparse 0 |
|---|---:|---:|---:|---:|
| sparse_events, 50 / 200 / 1 000 rows | 179 / 163 / 171 | 97 / 94 / 94 | 98 / 91 / 93 | 82 / 79 / 78 |
| events_sparse_cols (5 columns, 200 rows) | 244 | 96 | 91 | 71 |
| mixed_types, 50 / 200 / 1 000 rows | 77 / 77 / 76 | 65 / 64 / 64 | 67 / 66 / 67 | 64 / 63 / 63 |

- *kept, sparse 0*: every column positional, NULL as `null`. Valid v0.1, no
  rule change, a few percent below minified.
- *dropped*: inside a frame every column exists on every row, so a NULL can
  be left out of the record and restored on decode (absent means NULL).
  Lossless and spec-compliant, 18 to 29 percent below minified on sparse
  data, but the model then reads an empty cell (or a missing `~` key) as
  NULL, which the primer calls "absent".
- A rule change, counting presence instead of non-null values, would give
  the *kept, sparse 0* numbers for frames and change nothing for record
  data where keys are absent.

None of these has comprehension evidence. The `~` column exists because
Haiku lost answers counting across positional mostly-empty rows (baseline
report), and at 1 000 rows Sonnet lost filter answers on sparse rows even
with `~` (`reports/2026-10-20-large`). The adapter keeps the default and
falls back to minified JSON. The format question is which layout to grade
first. *kept, sparse 0* is the smallest change, since it is what the data
looks like in SQL.

## 4. The selector's acceptance rule

A row passes the policy when the accuracy drop is at most 3 points (point
estimate) and the sign test does not show a significant loss at 0.05. Both
look for harm. Neither shows that harm is small, so a row with too few
questions to see anything passes. `selector-audit.md` adds two one-sided
95 percent upper bounds on the loss for each of the 47 evidence rows the
policy counts. The paired score bound (Tango) is the usual test for
non-inferiority on matched pairs. The exact bound (Clopper-Pearson on each
side, Bonferroni) is a conservative check on it.

- 12 rows fail the current policy.
- Of the other 35, **5 show the loss is under 3 points** (the policy's own
  tolerance) by the score bound, and 23 show it is under 5 points. By the
  exact bound the counts are 0 and 7.
- Opus and Sonnet rows that pass the policy have score bounds of 1.0 to 8.7
  points. Only two are under 3: Opus on nested orders at sparse 0.75
  (+1 / -0 of 95) and Sonnet on flat users (+2 / -0 of 65), both at 200
  records.
- A perfect tie needs 88 questions for the score bound to reach 3 points
  (122 for the exact one). The runs have 52 to 95 per cell, and every
  question that differs pushes the bound up.

So "LDF ties JSON" in the reports mostly means "no loss detected at this
sample size". Treating inconclusive as unsafe would make the selector give
JSON almost everywhere today. That is a product decision, so this branch
does not change the policy. Section 6 is the study that would settle it.

Other findings from the audit:

- Every graded run showed models LDF with no primer (the bundle prompt only
  names the format). The proxy and the hook send a primer, so the deployed
  condition was never the measured one. The primer also does not describe
  the `~` column, which default output uses.
- `knows_model` and trait matching use exact set equality on traits; a
  frame's NULLs make it `sparse` and match evidence measured on absent
  keys, a different layout (now excluded for frames, section 1).
- No multiplicity correction is needed for the harm test as used: all
  rows in a band must pass, which is conservative. A non-inferiority rule
  would need the same "all rows" form.

## 5. Database questions through Claude Code (pilot)

`eval/db_eval.py` runs headless Claude Code (`claude -p`, no built-in
tools, one MCP server, project settings only, started outside this
repository) on 30 database questions, five each of schema discovery,
filtering, joins, nested records, NULL semantics and exact values. The arms
are `json` (the server as shipped), `minified` (`eval/minify_proxy.py`) and
`ldf` (`ldf-mcp proxy --columnar`). Ground truth comes from the fixture
files, except schema questions, which use the server's catalog. Arm order
is shuffled per task. Approved pilot: Sonnet 5.5, one question per
category, one rep, 18 runs per pilot. Files are in `pilot/`.

**Pilot 1 found a bug ($0.53, `pilot/runs-before-fix.jsonl`).** Tool
results in the transcripts had the same length in all three arms. Claude
Code 2.1.306 shows the model a result's `structuredContent` when it is
present, not its text block. MotherDuck's server sends the text a second
time as `structuredContent` `{"result": "<pretty JSON>"}`, and the model saw
that copy, with its quotes and newlines escaped. Neither the 0.5.3 minify
nor the frame reached the model. `docs/agents.md` said Claude Code gets the
proxy's output, which was wrong for any server that sends
`structuredContent`. Fixed in #33: the proxy (and the `minified` arm)
rewrites `structuredContent` in FastMCP's `{"result": S}` form, and any
other form is still left alone.

**Pilot 2 after the fix ($0.45, `pilot/report.md`):**

| arm | correct | prompt tokens per run (mean) | tool result chars per run (mean) | turns (mean) |
|---|---:|---:|---:|---:|
| json | 6/6 | 19 922 | 2 420 | 3.3 |
| minified | 6/6 | 19 840 | 2 026 | 3.3 |
| ldf | 6/6 | 20 508 | 1 688 | 3.7 |

Six questions answer nothing about accuracy. What the pilot does show is
the scale. A run's prompt is about 20 000 tokens summed over its requests,
nearly all of it Claude Code's own system prompt and tool definitions. The
model writes SQL that returns a few rows, so tool results are a few hundred
tokens. LDF cut the tool-result text by 17 percent against minified JSON,
but the primer (sent on every request) and one run that took two extra
turns made the `ldf` arm's total prompt 3 percent larger. On questions an
agent answers with targeted SQL, the format of the result hardly matters
to the total. It matters when results are large: wide or long row reads,
exports, and agents that pull rows to reason over them.

Full run, not started: 30 tasks x 3 arms x 2 reps = 180 runs at about
$0.025 per run measured, so about $4.50. It would measure accuracy over 60
paired runs per comparison, which can find a clear loss but cannot show
non-inferiority at 3 points (section 4).

## 6. A study that can show non-inferiority (proposed)

Single-prompt questions over one data block, as in Stage C, because
there the data is most of the prompt and the format effect is not diluted.

- **Pre-registered rule:** LDF is accepted for a model, shape and size
  when the one-sided 95 percent paired score bound on its loss against JSON
  is at most 3 points. Anything else is inconclusive and the selector gives
  JSON. Written into the policy before the run.
- **Cells:** Opus 5.5 and Sonnet 5.5, each on flat_users, nested_orders,
  sparse_events and mixed_types at 200 held-out records (question seed 3,
  unseen), the layouts the selector would send (also hoisted for Opus and
  threshold 0.75 for Sonnet on nested data), plus one Sonnet cell for
  SQL-shaped sparse data with NULLs as `null` cells (section 3).
- **Size:** at most 30 questions per kind, so 150 to 250 paired questions
  per cell (one 200-record block yields 386 to 573 distinct questions).
  With about 3 percent of questions differing between formats, which is
  what most Opus cells show, a true tie gives a bound near 2 points, so a
  pass is likely if LDF really is as good. Sonnet on nested data differs
  on 13 percent of questions, so that cell would need about 750 questions,
  and 250 is expected to come out inconclusive.
- **Deployed condition:** the system prompt carries the LDF primer in both
  arms, as the proxy sends it whatever format a result ends up in. That
  needs a `--primer` flag in `ldf-eval build`, a small change.
- **Cost:** Stage C cost $0.0051 per prompt on Sonnet and $0.0069 on Opus.
  About 1 800 prompts per model, so about $9 to $12 for Sonnet, $13 to $17
  for Opus and $3 for the NULL cell: about $25 to $32, capped per stage in
  `eval/plan.sh`.

It can also come out against LDF in some cells. Either way the selector's
claims would then rest on a test that can show safety, not only on the
absence of detected harm.
