# Non-inferiority study: cells

Pass: one-sided 95 percent paired score bound on the loss at most 3 points. Fails: drop over 3 points or a sign-test loss at p < 0.05. Otherwise inconclusive. Grading strict (`ldf-eval score`); per-bundle scorer output in `<dataset>.<model>.md`.

| model | dataset | format | n | correct / JSON | +w / -l | drop pp | p | loss bound pp | verdict | losses by kind | non-wrong misses |
|---|---|---|---:|---|---|---:|---:|---:|---|---|---|
| claude-opus-5-5 | flat_users | ldf | 151 | 150 / 148 | +3 / -1 | -1.3 | 0.62 | 1.2 | **pass** | filter 1 | - |
| claude-opus-5-5 | mixed_types | ldf | 151 | 151 / 150 | +1 / -0 | -0.7 | 1.00 | 1.1 | **pass** | - | - |
| claude-opus-5-5 | nested_orders | ldf-hoisted | 249 | 238 / 247 | +1 / -10 | 3.6 | 0.01 | 6.3 | **fails** | aggregate_where 2, child_aggregate 5, nested_extract 3 | wrapped 1 |
| claude-opus-5-5 | sparse_events | ldf | 220 | 214 / 214 | +5 / -5 | 0.0 | 1.00 | 2.6 | **pass** | compare 2, filter 2, rank 1 | wrapped 2 |
| claude-sonnet-5-5 | flat_users | ldf | 151 | 148 / 150 | +0 / -2 | 1.3 | 0.50 | 3.9 | **inconclusive** | aggregate_where 2 | - |
| claude-sonnet-5-5 | mixed_types | ldf | 151 | 151 / 151 | +0 / -0 | 0.0 | 1.00 | 1.8 | **pass** | - | - |
| claude-sonnet-5-5 | nested_orders | ldf-s75 | 249 | 227 / 229 | +6 / -8 | 0.8 | 0.79 | 3.5 | **inconclusive** | aggregate_where 1, child_aggregate 5, compare 1, filter 1 | bare 1 |
| claude-sonnet-5-5 | sparse_events | ldf | 220 | 214 / 216 | +4 / -6 | 0.9 | 0.75 | 3.6 | **inconclusive** | aggregate 2, filter 4 | - |
| claude-sonnet-5-5 | sql_events | ldf-s0 | 220 | 216 / 217 | +2 / -3 | 0.5 | 1.00 | 2.6 | **pass** | aggregate_where 1, filter 1, rank 1 | - |
| deepseek-chat | flat_users | ldf | 151 | 83 / 92 | +3 / -12 | 6.0 | 0.04 | 10.7 | **fails** | aggregate_where 2, compare 3, filter 5, multi_record 1, rank 1 | - |
| deepseek-chat | mixed_types | ldf | 151 | 150 / 150 | +1 / -1 | 0.0 | 1.00 | 2.3 | **pass** | missing_value 1 | - |
| deepseek-chat | nested_orders | ldf-hoisted | 249 | 159 / 162 | +4 / -7 | 1.2 | 0.55 | 3.7 | **inconclusive** | aggregate_where 1, child_aggregate 1, child_count 5 | truncated 3 |
| deepseek-chat | sparse_events | ldf-s75 | 220 | 132 / 136 | +9 / -13 | 1.8 | 0.52 | 5.5 | **inconclusive** | aggregate 3, aggregate_where 1, compare 1, filter 1, lookup 1, missing_value 1, multi_record 2, rank 3 | - |
