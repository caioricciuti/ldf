# Non-inferiority study: pre-registration (2026-10-21)

Written and committed before any response was collected. The commit that
adds this file is the registration; results go in a separate commit.

## Question

For each model, data shape and the LDF layout the selector would send at
200 records: is LDF's accuracy at most 3 percentage points below JSON's on
the same questions? Earlier evidence could only fail to detect a loss
(`reports/2026-10-21-columnar`, section 4). This study is sized to show the
absence of a loss larger than 3 points, or to fail to.

## Design

- **Data:** the first 200 records of the held-out fixtures
  (`fixtures/holdout`, `ldf-gen --seed 20261118`): flat_users,
  nested_orders, sparse_events, mixed_types; plus `sql_events`, the same
  sparse_events records as a SQL table returns them (every key on every row,
  `null` where the record has no value, first-seen key order).
- **Questions:** `ldf-eval build --seed 3 --per-kind 30`. Seed 3 has not been
  used in any earlier run. 151 to 249 questions per dataset, the same
  questions for every format and model.
- **Condition:** the system prompt is the bundle's usual one plus the LDF
  primer `ldf-mcp primer` prints, in **both** arms, because the proxy sends
  the primer whatever format a result ends up in. Earlier runs had no
  primer.
- **Cells (format against JSON):**

  | model | flat_users | nested_orders | sparse_events | mixed_types | sql_events |
  |---|---|---|---|---|---|
  | Sonnet 5.5 | ldf | ldf-s75 | ldf | ldf | ldf-s0 |
  | DeepSeek (deepseek-chat) | ldf | ldf-hoisted | ldf-s75 | ldf | |
  | Opus 5.5 | ldf | ldf-hoisted | ldf | ldf | |

  `ldf` is the default (sparse threshold 0.5), `ldf-s75` threshold 0.75,
  `ldf-hoisted` hoisted arrays at 0.5, `ldf-s0` threshold 0 (no `~`
  column; NULL written as `null`).
- **Settings:** `eval/plan.sh ni-sonnet`, `ni-deepseek`, `ni-opus`, as
  committed with this file: temperature and effort as in Stage C,
  max_tokens 4096, data block prompt-cached.
- **Budget caps:** Sonnet $13, DeepSeek $5, Opus at most $11 (set through
  `NI_OPUS_BUDGET` after the Sonnet run, keeping all Anthropic spend under
  $26). A cell cut short by a cap is reported with the questions it has and
  the bound they give.

## Analysis

- **Grading:** `ldf-eval score`, strict, unchanged: a bare answer (the
  right value without its JSON quotes) and a wrapped one are wrong. No
  change to grading after responses are seen.
- **Pairing:** by question. w = questions the format got right and JSON
  wrong, l = the reverse, n = questions answered in both arms.
- **Primary rule, per cell:** pass when the one-sided 95 percent paired
  score bound on the loss (`ldf_select::loss_upper_bound`, Tango) is at
  most **3.0** points. Otherwise the cell is **inconclusive**, or **fails**
  when the existing harm rule also rejects it (drop over 3 points, or a
  sign test loss at p < 0.05).
- **No retries** except requests that failed at the API (no response).
  Every cell is reported, passing or not. No cell, model or question is
  dropped after the fact.

## What follows from it

- The selector's policy gains `max_loss_bound_pp = 3.0` as its default
  (implemented, off until this study is graded). With it a cell needs
  evidence that passes the bound, so every size band without such evidence
  gives JSON, which includes everything above 200 records today.
- The new rows enter `reports/evidence/evidence.json` through
  `eval/analysis/evidence.py` as usual, labelled with their run (`ni/...`)
  so the primer condition stays visible.
- Whatever the outcome, the site and README claims are rewritten to the
  cells that pass, with their bounds.
