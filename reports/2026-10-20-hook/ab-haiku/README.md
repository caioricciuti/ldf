# Bash hook A/B on provider usage, claude-haiku-5-5

Written by `eval/ab_hook.py`. Same headless Claude Code task per case, with the hook (`hook`) and without (`json`). Tokens are Anthropic-reported usage from the session transcripts, in Claude tokens. "added" is the second request's prompt minus the first request's prompt and output: what the tool call put in the context (tool result, hook note and primer, Claude Code's wrapping). Medians over reps.

| case | data bytes | hook outcome | added json | added hook | hook / json | total prompt json | total prompt hook | cost json | cost hook | correct json | correct hook |
|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| flat_users_100_pretty | 19206 | ldf | 9478 | 4580 | 0.48 | 32113 | 27217 | $0.0030 | $0.0015 | 6/6 | 6/6 |
| repetitive_logs_100_pretty | 19353 | ldf | 9114 | 4622 | 0.51 | 31745 | 27255 | $0.0028 | $0.0016 | 6/6 | 6/6 |
| wide_table_20_pretty | 15810 | ldf | 8055 | 3324 | 0.41 | 30704 | 25975 | $0.0025 | $0.0014 | 6/6 | 6/6 |
| sparse_events_100_pretty | 20927 | ldf | 10367 | 6452 | 0.62 | 33004 | 29091 | $0.0032 | $0.0021 | 6/6 | 6/6 |
| unicode_strings_100_compact | 12950 | ldf | 6810 | 4395 | 0.65 | 29451 | 27038 | $0.0027 | $0.0019 | 6/6 | 6/6 |
| nested_orders_20_compact | 5998 | json (selector) | 3083 | 3083 | 1.00 | 25714 | 25716 | $0.0015 | $0.0011 | 6/6 | 6/6 |
| **all** | | | 46907 | 26456 | 0.56 | 182731 | 162292 | $0.0156 | $0.0097 | 36/36 | 36/36 |

Runs: 36, total cost $0.0759 (Claude Code total_cost_usd). Costs are mean per run; cache reads and writes make cost depend on run order, the token columns do not.

