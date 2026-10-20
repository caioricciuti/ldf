# Stage C: Sonnet 5.5 on held-out data

Run 2026-10-20, `eval/plan.sh stage-c10-sonnet`: the same 864 prompts as
the Opus run in `reports/2026-10-19-stage-c` (held-out medium fixtures,
first 200 records, question seed 2, 10 questions per kind). Per-bundle
scorer output in `<dataset>.sonnet.md`; the evidence rows are in
`reports/evidence/` (runs `stage-c10/*`, model `claude-sonnet-5-5`).

**Setup.** Formats JSON, LDF v0.1 (sparse threshold 0.5), LDF at threshold
0.75 where it differs (nested_orders, sparse_events) and `ldf-hoisted` on
nested_orders, all shown to the model as `ldf` with no explanation. Claude
Sonnet 5.5, effort low, max_tokens 4096, data block prompt-cached. No API
errors, no truncation, 169 output tokens per answer on average. $4.41 from
the reported usage at list prices ($2 input, $0.20 cache read, $10 output;
estimate $4.30 to $6.50, cap $7).

**Results.** Accuracy under strict grading (a bare answer, the right value
without its JSON quotes, is wrong), and questions one format got right and
JSON wrong (+) or the reverse (-), with an exact sign test. Tokens are
Claude input tokens per prompt from the usage, relative to JSON (o200k in
brackets, from the evidence table).

| dataset | format | tokens / JSON | accuracy | bare | vs JSON |
|---|---|---:|---:|---:|---|
| flat_users | json | 1.00 | 62/65 | 0 | |
| flat_users | ldf | 0.57 (0.61) | 64/65 | 0 | +2 / -0, p=0.50 |
| nested_orders | json | 1.00 | 87/95 | 0 | |
| nested_orders | ldf | 0.57 (0.65) | 80/95 | 8 | +3 / -10, p=0.09 |
| nested_orders | ldf at 0.75 | 0.62 (0.68) | 85/95 | 3 | +5 / -7, p=0.77 |
| nested_orders | ldf-hoisted | 0.47 (0.55) | 81/95 | 1 | +1 / -7, p=0.07 |
| sparse_events | json | 1.00 | 78/80 | 0 | |
| sparse_events | ldf | 0.76 (0.80) | 77/80 | 0 | +1 / -2, p=1.00 |
| sparse_events | ldf at 0.75 | 0.86 (0.89) | 75/80 | 0 | +0 / -3, p=0.25 |
| mixed_types | json | 1.00 | 57/57 | 0 | |
| mixed_types | ldf | 0.77 (0.73) | 57/57 | 0 | +0 / -0 |

## Findings

1. **Sonnet reads LDF v0.1 as well as JSON on flat, sparse and mixed-type
   data,** at 57 to 77 percent of JSON's input tokens. The selector now
   gives Sonnet LDF at threshold 0.5 there (`ldf-discover select --model
   claude-sonnet-5-5` on the small fixtures); before this run it had no
   evidence and always gave JSON.
2. **Nested orders: threshold 0.75 or JSON, under strict grading.** LDF
   v0.1 is 7.4 points below JSON (80 against 87, p=0.09), over the
   selector's 3-point tolerance. Threshold 0.75 is 2.1 points below (within
   tolerance, not significant), so on nested data shaped like this test set
   (`child_tables` + `sparse`, `fixtures/holdout/medium/nested_orders.json`)
   the selector gives Sonnet LDF at 0.75: 13 677 o200k against JSON's
   20 069 on 200 records. On nested data without sparse keys (the small
   fixture, lowest fill 0.80) there is no evidence and Sonnet gets JSON.
3. **Most of that gap is answer formatting, not reading.** Eight of LDF's
   misses are bare answers: `compare` asks for an order id, LDF shows ids
   unquoted, and Sonnet answered `ORD-000158` where JSON's quoted value made
   it answer `"ORD-000158"`. `compare` is 2 of 10 on LDF against 10 of 10 on
   JSON. Counting bare answers as right, LDF is 88 of 95 against JSON's 87.
   The selector keeps strict grading, as for Haiku: an agent that gets an
   id back without quotes may pass it on wrong. Whether a prompt line
   ("strings in the answer are JSON strings") closes the gap is a
   measurement for a later run, not an assumption.
4. **Hoisted arrays lose on Sonnet.** 17 percent fewer Claude tokens than
   v0.1, but `child_aggregate` (sums over each order's items) fell to 0 of
   10 against JSON's 3 of 10, and the total is 6.3 points below JSON
   (p=0.07). With Haiku significantly worse (+2 / -12) and Opus and DeepSeek
   level, hoisting is a per-model choice; the selector already makes it.
5. **Threshold 0.75 brings Sonnet nothing on sparse_events,** 3.8 points
   below JSON (75 against 78, rejected) at 12 percent more tokens than 0.5,
   which ties JSON. Same as Opus; only DeepSeek gained from it there.

**Limits.** One model, 200 records, 57 to 95 questions per cell (a single
answer moves a cell by 1 to 2 points). JSON's own nested score (87 of 95)
is lower than Opus's (94): `child_aggregate` is 3 of 10 on JSON too, so
sums over child rows are hard for Sonnet in any format. Nothing here is
about more than 200 rows; the selector still refuses to extrapolate.
