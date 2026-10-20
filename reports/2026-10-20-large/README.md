# Large: Opus 5.5 and Sonnet 5.5 at 1 000 records

Run 2026-10-20, `eval/plan.sh large-build large-opus large-sonnet`. Until
this run the selector had evidence only at 200 records and gave JSON to
anything larger. Per-bundle scorer output in `<dataset>.<model>.md`; the
evidence rows are in `reports/evidence/` (runs `large/*`).

**Setup.** First 1 000 records of the held-out medium fixtures
(`ldf-gen --seed 20261118`), question seed 2. Only kinds a model can answer
at that size without long arithmetic (`ldf-eval build --kinds`): lookup,
filter, compare, multi_record, rank, missing_value, nested_extract,
child_count. Sums and means failed in every format at 500 records, so
aggregate, aggregate_where and child_aggregate are left out. 52 to 68
questions per dataset (flat_users at 20 per kind, the rest at 10). Each
model got only the layouts its selector could pick: hoisted on nested
orders for Opus, threshold 0.75 for Sonnet. Effort low, max_tokens 4096,
data block prompt-cached. 1 488 prompts, no API errors, no truncation;
$20.72 from the reported usage at list prices (estimate $21.50 to $24.40,
caps $16 and $13). Data blocks run from 25 000 (mixed_types, JSON) to
100 000 (nested_orders, JSON) o200k tokens.

**Results.** Strict grading: a bare answer (the right value without its
JSON quotes) and a wrapped one (the right value inside an object such as
`{"final_answer": 791}`) are wrong. Tokens are Claude input tokens per
prompt relative to JSON. Sign tests from the evidence table.

| model | dataset | format | tokens / JSON | accuracy | bare | wrapped | vs JSON |
|---|---|---|---:|---:|---:|---:|---|
| Opus | flat_users | json | 1.00 | 64/64 | 0 | 0 | |
| Opus | flat_users | ldf | 0.56 | 59/64 | 0 | 3 | +0 / -5, p=0.06 |
| Opus | nested_orders | json | 1.00 | 66/68 | 0 | 2 | |
| Opus | nested_orders | ldf | 0.57 | 66/68 | 0 | 2 | +2 / -2, p=1.00 |
| Opus | nested_orders | ldf-hoisted | 0.47 | 67/68 | 0 | 0 | +2 / -1, p=1.00 |
| Opus | sparse_events | json | 1.00 | 56/60 | 0 | 2 | |
| Opus | sparse_events | ldf | 0.77 | 57/60 | 0 | 0 | +3 / -2, p=1.00 |
| Opus | mixed_types | json | 1.00 | 52/52 | 0 | 0 | |
| Opus | mixed_types | ldf | 0.76 | 52/52 | 0 | 0 | +0 / -0 |
| Sonnet | flat_users | json | 1.00 | 64/64 | 0 | 0 | |
| Sonnet | flat_users | ldf | 0.56 | 64/64 | 0 | 0 | +0 / -0 |
| Sonnet | nested_orders | json | 1.00 | 65/68 | 2 | 0 | |
| Sonnet | nested_orders | ldf | 0.57 | 57/68 | 8 | 0 | +0 / -8, p=0.01 |
| Sonnet | nested_orders | ldf at 0.75 | 0.61 | 58/68 | 8 | 0 | +0 / -7, p=0.02 |
| Sonnet | sparse_events | json | 1.00 | 59/60 | 0 | 0 | |
| Sonnet | sparse_events | ldf | 0.77 | 54/60 | 0 | 0 | +1 / -6, p=0.12 |
| Sonnet | mixed_types | json | 1.00 | 52/52 | 0 | 0 | |
| Sonnet | mixed_types | ldf | 0.76 | 52/52 | 0 | 0 | +0 / -0 |

## Findings

1. **At 1 000 records LDF holds for Opus on nested, sparse and mixed-type
   data and for Sonnet on flat and mixed-type data,** at 47 to 77 percent
   of JSON's input tokens. Hoisted nested orders on Opus: 67 of 68 at 47
   percent of JSON's tokens, the best cell in the run.
2. **Opus on flat data fails the policy, mostly on answer shape.** Three
   of its five misses are the right id wrapped in an object
   (`{"final_answer": 791}`), all in the LDF arm; the other two are a
   filter list one id short and a compare answered `91` for `691`. With
   wrapped answers counted right it is 62 against 64 (3.1 points, just
   over the tolerance). At 200 records Opus tied JSON on the same dataset
   (65 of 65).
3. **Sonnet on nested orders: the bare-id effect again, now significant.**
   Eight of LDF's eleven misses (both layouts) are `compare` answers with
   the right order id unquoted; JSON has two of those. Counted right, LDF
   is 65 against JSON's 67.
4. **Sonnet on sparse events is a real loss.** Five of LDF's six misses are
   `filter` lists one id short over 1 000 mostly empty rows (JSON: one).
   This is the failure the `~` column was introduced to prevent at 200
   records on Haiku; at 1 000 it returns on Sonnet.

## What the selector does with it

The rule rejects a layout when any matching evidence row fails, whatever
its size, and accepts inputs up to the largest passing size. With this
evidence (`ldf-discover select`, held-out fixtures, 150 to 1 000 rows):

| model | flat | nested | sparse | mixed |
|---|---|---|---|---|
| Opus | JSON (was LDF up to 200) | hoisted LDF up to 1 000 | LDF up to 1 000 | LDF up to 1 000 |
| Sonnet | LDF up to 1 000 | JSON (was LDF 0.75 up to 200) | JSON (was LDF up to 200) | LDF up to 1 000 |

Above 1 000 rows every model still gets JSON. Whether a failure at 1 000
records should also withdraw LDF from inputs of 200 or fewer, where the
same model passed, is a policy question this report does not settle.

**Limits.** Two models, one size, 52 to 68 questions per cell (one answer
moves a cell by 1.5 to 2 points). Arithmetic kinds are not measured at this
size in any format.
