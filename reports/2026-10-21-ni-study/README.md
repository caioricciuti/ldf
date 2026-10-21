# Non-inferiority study: results (2026-10-21)

Pre-registered in `PREREGISTRATION.md`, committed (08cfae5) before the
first response. The responses and graded cells were committed (2b7faf3)
before this text and before any change to the selector. Everything below
follows from those two commits.

Spend, from the usage the APIs reported, at list prices: Sonnet 5.5 $11.09,
Opus 5.5 $9.11, DeepSeek $0.64. No request failed.

## Cells

`cells.md` is `eval/analysis/ni_study.py`'s output; per-bundle scorer
output is in `<dataset>.<model>.md`, the raw responses in `responses/`.
"Bound" is the one-sided 95 percent paired score bound on the loss against
JSON: the largest loss the data cannot rule out.

| model | dataset | layout | n | LDF / JSON correct | +w / -l | bound (pp) | verdict |
|---|---|---|---:|---|---|---:|---|
| Opus 5.5 | flat_users | ldf | 151 | 150 / 148 | +3 / -1 | 1.2 | **pass** |
| Opus 5.5 | nested_orders | ldf-hoisted | 249 | 238 / 247 | +1 / -10 | 6.3 | **fails** (p=0.01) |
| Opus 5.5 | sparse_events | ldf | 220 | 214 / 214 | +5 / -5 | 2.6 | **pass** |
| Opus 5.5 | mixed_types | ldf | 151 | 151 / 150 | +1 / -0 | 1.1 | **pass** |
| Sonnet 5.5 | flat_users | ldf | 151 | 148 / 150 | +0 / -2 | 3.9 | inconclusive |
| Sonnet 5.5 | nested_orders | ldf-s75 | 249 | 227 / 229 | +6 / -8 | 3.5 | inconclusive |
| Sonnet 5.5 | sparse_events | ldf | 220 | 214 / 216 | +4 / -6 | 3.6 | inconclusive |
| Sonnet 5.5 | mixed_types | ldf | 151 | 151 / 151 | +0 / -0 | 1.8 | **pass** |
| Sonnet 5.5 | sql_events | ldf-s0 | 220 | 216 / 217 | +2 / -3 | 2.6 | **pass** |
| DeepSeek | flat_users | ldf | 151 | 83 / 92 | +3 / -12 | 10.7 | **fails** (p=0.04) |
| DeepSeek | nested_orders | ldf-hoisted | 249 | 159 / 162 | +4 / -7 | 3.7 | inconclusive |
| DeepSeek | sparse_events | ldf-s75 | 220 | 132 / 136 | +9 / -13 | 5.5 | inconclusive |
| DeepSeek | mixed_types | ldf | 151 | 150 / 150 | +1 / -1 | 2.3 | **pass** |

Six cells pass, five are inconclusive, two fail.

## What it shows

1. **Opus with hoisted arrays loses on nested orders.** It got 10 questions
   wrong that it got right from JSON: five sums over child items
   (`child_aggregate`), three nested lookups, two filtered sums. Stage C
   (95 questions, no primer) had this layout at 93 against 94 and the
   1 000-record run at 67 against 66, so the selector was sending it to
   Opus. With 249 questions and the primer it is a significant loss. The
   selector no longer sends hoisted arrays to anyone.
2. **Where LDF passes, it passes clearly.** Opus on flat, sparse and mixed
   data, Sonnet and DeepSeek on mixed types: the bound is 1.1 to 2.6
   points, and LDF is at 0.61 to 0.78 of JSON's tokens there.
3. **SQL NULLs written as `null` cells work.** Sonnet on `sql_events` (every
   key on every row, NULL as `null`, sparse rule off): 216 against 217, a
   bound of 2.6, at 0.47 of the tokens of the same rows as JSON objects.
   This is the layout `reports/2026-10-21-columnar` proposed for NULL-heavy
   SQL results. The selector does not offer threshold 0 yet, so this result
   is evidence for a follow-up, not a change today.
4. **Sonnet is close everywhere but proven only on mixed and NULL data.**
   Its other three cells are within 0.8 to 1.3 points of JSON, but with
   151 to 249 questions the bound stays at 3.5 to 3.9. More questions would
   probably settle them; nothing here says they lose.
5. **DeepSeek loses on flat tables.** It is the weakest model here (55 to
   60 percent right even with JSON), and on flat rows LDF costs it 6
   points: filter and compare questions over a plain table.

## What changed in the selector

As registered, the policy now has `max_loss_bound_pp = 3.0` by default.
One detail was not settled in the registration and was decided after
seeing the results: how rows of the same size band combine. Taken
literally, "every row passes the bound" would let the small Stage C rows
(52 to 95 questions, which cannot prove 3 points) veto cells this study
proved. The rule implemented instead:

- **pooled bound:** the band's rows, one per run, are added up (wins,
  losses, questions) and the pooled bound must be at most 3 points;
- **harm veto:** every row must still pass the old harm rule (drop at most
  3 points, no significant sign-test loss).

Choosing the best single row was tried first and dropped, because it lets
one lucky small run decide. Pooling counts every question once.

Effect on what `ldf-mcp --model` sends (`reports/evidence/verdicts.json`,
the site's grid, medium fixtures, 200 and 1 000 rows):

| model | flat | nested | sparse | mixed |
|---|---|---|---|---|
| Opus 5.5 | LDF up to 200 | LDF 0.75 up to 200 (was hoisted up to 1 000) | LDF up to 200 (was up to 1 000) | LDF up to 200 (was up to 1 000) |
| Sonnet 5.5 | LDF up to 200 (pooled: +2 / -2 of 216) (was up to 1 000) | JSON (was LDF 0.75) | JSON (was LDF) | LDF up to 200 (was up to 1 000) |
| DeepSeek | JSON (was LDF) | JSON (was hoisted) | LDF 0.75 up to 200 (pooled) | LDF up to 200 |
| Haiku 5.5 | JSON (was LDF) | JSON | JSON (was LDF) | JSON (was LDF) |

Nothing above 200 rows passes now: the 1 000-record runs have 52 to 68
questions per cell, too few to prove 3 points. GPT is unchanged (JSON
everywhere). Without `--model` the proxy still sends LDF whenever it is
shorter; this rule only governs `--model`.

## Limits

- One data block per dataset (200 held-out records), one question seed.
- The primer was in both arms. Earlier evidence had no primer, and pooling
  mixes the two conditions in the bands where both exist.
- Strict grading: a bare or wrapped answer is wrong. Sonnet had one bare
  answer in the LDF arm on nested orders.
- 244 Sonnet prompts on `sql_events` were sent twice by two overlapping
  runs (an operator error). The first response per prompt was kept, by a
  rule fixed before those responses were read. The second responses are
  in `responses/sql_events.duplicates.anthropic.claude-sonnet-5-5.jsonl`.
