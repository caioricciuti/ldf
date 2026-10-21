# Selector acceptance audit

Evidence rows the policy counts (ldf and toon families, n >= 50): 47.
A perfect tie (no question differs) shows a loss of at most 3 points only from n = 122 questions.
A perfect tie (no question differs) shows a loss of at most 5 points only from n = 72 questions.

| model | run | dataset | records | format | n | k / JSON | +w / -l | drop pp | policy | loss upper bound pp | at 3 pp | at 5 pp |
|---|---|---|---:|---|---:|---|---|---:|---|---:|---|---|
| claude-haiku-5-5 | stage-b/flat_users | flat_users | 200 | ldf | 89 | 87 / 87 | +2 / -2 | 0.0 | pass | 7.6 | inconclusive | inconclusive |
| claude-haiku-5-5 | stage-b/flat_users | flat_users | 200 | toon | 89 | 86 / 87 | +2 / -3 | 1.1 | pass | 9.3 | inconclusive | inconclusive |
| claude-haiku-5-5 | stage-b/mixed_types | mixed_types | 200 | ldf | 82 | 79 / 81 | +0 / -2 | 2.4 | pass | 8.5 | inconclusive | inconclusive |
| claude-haiku-5-5 | stage-b/mixed_types | mixed_types | 200 | ldf-v0 | 82 | 79 / 81 | +0 / -2 | 2.4 | pass | 8.5 | inconclusive | inconclusive |
| claude-haiku-5-5 | stage-b/mixed_types | mixed_types | 200 | toon | 82 | 82 / 81 | +1 / -0 | -1.2 | pass | 4.4 | inconclusive | non-inferior |
| claude-haiku-5-5 | stage-b/nested_orders | nested_orders | 200 | ldf | 135 | 124 / 130 | +1 / -7 | 4.4 | fail | 10.4 | fails | fails |
| claude-haiku-5-5 | stage-b/nested_orders | nested_orders | 200 | ldf-hoisted | 135 | 120 / 130 | +2 / -12 | 7.4 | fail | 14.8 | fails | fails |
| claude-haiku-5-5 | stage-b/nested_orders | nested_orders | 200 | toon | 135 | 125 / 130 | +3 / -8 | 3.7 | fail | 10.9 | fails | fails |
| claude-haiku-5-5 | stage-b/sparse_events | sparse_events | 200 | ldf | 115 | 108 / 106 | +6 / -4 | -1.7 | pass | 6.7 | inconclusive | inconclusive |
| claude-haiku-5-5 | stage-b/sparse_events | sparse_events | 200 | ldf-v0 | 115 | 103 / 106 | +5 / -8 | 2.6 | pass | 11.8 | inconclusive | inconclusive |
| claude-haiku-5-5 | stage-b/sparse_events | sparse_events | 200 | toon | 115 | 108 / 106 | +6 / -4 | -1.7 | pass | 6.7 | inconclusive | inconclusive |
| claude-opus-5-5 | stage-c10/flat_users | flat_users | 200 | ldf | 65 | 65 / 65 | +0 / -0 | 0.0 | pass | 5.5 | inconclusive | inconclusive |
| claude-opus-5-5 | large/flat_users | flat_users | 1000 | ldf | 64 | 59 / 64 | +0 / -5 | 7.8 | fail | 17.3 | fails | fails |
| claude-opus-5-5 | stage-c10/mixed_types | mixed_types | 200 | ldf | 57 | 57 / 57 | +0 / -0 | 0.0 | pass | 6.3 | inconclusive | inconclusive |
| claude-opus-5-5 | large/mixed_types | mixed_types | 1000 | ldf | 52 | 52 / 52 | +0 / -0 | 0.0 | pass | 6.8 | inconclusive | inconclusive |
| claude-opus-5-5 | stage-c10/nested_orders | nested_orders | 200 | ldf | 95 | 94 / 94 | +1 / -1 | 0.0 | pass | 5.7 | inconclusive | inconclusive |
| claude-opus-5-5 | stage-c10/nested_orders | nested_orders | 200 | ldf-hoisted | 95 | 93 / 94 | +1 / -2 | 1.1 | pass | 7.4 | inconclusive | inconclusive |
| claude-opus-5-5 | stage-c10/nested_orders | nested_orders | 200 | ldf-s75 | 95 | 95 / 94 | +1 / -0 | -1.1 | pass | 3.8 | inconclusive | non-inferior |
| claude-opus-5-5 | large/nested_orders | nested_orders | 1000 | ldf | 68 | 66 / 66 | +2 / -2 | 0.0 | pass | 9.9 | inconclusive | inconclusive |
| claude-opus-5-5 | large/nested_orders | nested_orders | 1000 | ldf-hoisted | 68 | 67 / 66 | +2 / -1 | -1.5 | pass | 7.6 | inconclusive | inconclusive |
| claude-opus-5-5 | stage-c10/sparse_events | sparse_events | 200 | ldf | 80 | 78 / 78 | +2 / -2 | 0.0 | pass | 8.4 | inconclusive | inconclusive |
| claude-opus-5-5 | stage-c10/sparse_events | sparse_events | 200 | ldf-s75 | 80 | 76 / 78 | +2 / -4 | 2.5 | pass | 12.0 | inconclusive | inconclusive |
| claude-opus-5-5 | large/sparse_events | sparse_events | 1000 | ldf | 60 | 57 / 56 | +3 / -2 | -1.7 | pass | 10.5 | inconclusive | inconclusive |
| claude-sonnet-5-5 | stage-c10/flat_users | flat_users | 200 | ldf | 65 | 64 / 62 | +2 / -0 | -3.1 | pass | 5.1 | inconclusive | inconclusive |
| claude-sonnet-5-5 | large/flat_users | flat_users | 1000 | ldf | 64 | 64 / 64 | +0 / -0 | 0.0 | pass | 5.6 | inconclusive | inconclusive |
| claude-sonnet-5-5 | stage-c10/mixed_types | mixed_types | 200 | ldf | 57 | 57 / 57 | +0 / -0 | 0.0 | pass | 6.3 | inconclusive | inconclusive |
| claude-sonnet-5-5 | large/mixed_types | mixed_types | 1000 | ldf | 52 | 52 / 52 | +0 / -0 | 0.0 | pass | 6.8 | inconclusive | inconclusive |
| claude-sonnet-5-5 | stage-c10/nested_orders | nested_orders | 200 | ldf | 95 | 80 / 87 | +3 / -10 | 7.4 | fail | 17.9 | fails | fails |
| claude-sonnet-5-5 | stage-c10/nested_orders | nested_orders | 200 | ldf-hoisted | 95 | 81 / 87 | +1 / -7 | 6.3 | fail | 14.6 | fails | fails |
| claude-sonnet-5-5 | stage-c10/nested_orders | nested_orders | 200 | ldf-s75 | 95 | 85 / 87 | +5 / -7 | 2.1 | pass | 12.9 | inconclusive | inconclusive |
| claude-sonnet-5-5 | large/nested_orders | nested_orders | 1000 | ldf | 68 | 57 / 65 | +0 / -8 | 11.8 | fail | 21.9 | fails | fails |
| claude-sonnet-5-5 | large/nested_orders | nested_orders | 1000 | ldf-s75 | 68 | 58 / 65 | +0 / -7 | 10.3 | fail | 20.1 | fails | fails |
| claude-sonnet-5-5 | stage-c10/sparse_events | sparse_events | 200 | ldf | 80 | 77 / 78 | +1 / -2 | 1.2 | pass | 8.7 | inconclusive | inconclusive |
| claude-sonnet-5-5 | stage-c10/sparse_events | sparse_events | 200 | ldf-s75 | 80 | 75 / 78 | +0 / -3 | 3.8 | fail | 10.6 | fails | fails |
| claude-sonnet-5-5 | large/sparse_events | sparse_events | 1000 | ldf | 60 | 54 / 59 | +1 / -6 | 8.3 | fail | 20.5 | fails | fails |
| deepseek-chat | stage-b/flat_users | flat_users | 200 | ldf | 89 | 61 / 59 | +7 / -5 | -2.2 | pass | 9.4 | inconclusive | inconclusive |
| deepseek-chat | stage-b/flat_users | flat_users | 200 | toon | 89 | 61 / 59 | +7 / -5 | -2.2 | pass | 9.4 | inconclusive | inconclusive |
| deepseek-chat | stage-b/mixed_types | mixed_types | 200 | ldf | 82 | 82 / 82 | +0 / -0 | 0.0 | pass | 4.4 | inconclusive | non-inferior |
| deepseek-chat | stage-b/mixed_types | mixed_types | 200 | ldf-v0 | 82 | 82 / 82 | +0 / -0 | 0.0 | pass | 4.4 | inconclusive | non-inferior |
| deepseek-chat | stage-b/mixed_types | mixed_types | 200 | toon | 82 | 82 / 82 | +0 / -0 | 0.0 | pass | 4.4 | inconclusive | non-inferior |
| deepseek-chat | stage-b/nested_orders | nested_orders | 200 | ldf | 135 | 84 / 84 | +6 / -6 | 0.0 | pass | 7.8 | inconclusive | inconclusive |
| deepseek-chat | stage-b/nested_orders | nested_orders | 200 | ldf-hoisted | 135 | 81 / 84 | +1 / -4 | 2.2 | pass | 7.4 | inconclusive | inconclusive |
| deepseek-chat | stage-b/nested_orders | nested_orders | 200 | toon | 135 | 87 / 84 | +6 / -3 | -2.2 | pass | 4.7 | inconclusive | non-inferior |
| deepseek-chat | stage-b/sparse_events | sparse_events | 200 | ldf | 115 | 60 / 70 | +5 / -15 | 8.7 | fail | 19.2 | fails | fails |
| deepseek-chat | sparse-sweep/sparse_events, json from stage-b/sparse_events | sparse_events | 200 | ldf-s75 | 115 | 76 / 70 | +8 / -2 | -5.2 | pass | 3.1 | inconclusive | non-inferior |
| deepseek-chat | stage-b/sparse_events | sparse_events | 200 | ldf-v0 | 115 | 56 / 70 | +3 / -17 | 12.2 | fail | 22.1 | fails | fails |
| deepseek-chat | stage-b/sparse_events | sparse_events | 200 | toon | 115 | 72 / 70 | +6 / -4 | -1.7 | pass | 6.7 | inconclusive | inconclusive |

- margin 3 pp: non-inferior 0, inconclusive 35, fails 12
- margin 5 pp: non-inferior 7, inconclusive 28, fails 12
