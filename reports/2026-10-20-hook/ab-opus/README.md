# Bash hook A/B on provider usage, claude-opus-5-5

Written by `eval/ab_hook.py`. Same headless Claude Code task per case, with the hook (`hook`) and without (`json`). Tokens are Anthropic-reported usage from the session transcripts, in Claude tokens. "added" is the second request's prompt minus the first request's prompt and output: what the tool call put in the context (tool result, hook note and primer, Claude Code's wrapping). Medians over reps.

| case | data bytes | hook outcome | added json | added hook | hook / json | total prompt json | total prompt hook | cost json | cost hook | correct json | correct hook |
|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| flat_users_100_pretty | 19206 | ldf | 9474 | 4580 | 0.48 | 30431 | 25534 | $0.1460 | $0.0671 | 6/6 | 6/6 |
| repetitive_logs_100_pretty | 19353 | ldf | 9114 | 4621 | 0.51 | 30062 | 25571 | $0.1243 | $0.0513 | 6/6 | 6/6 |
| wide_table_20_pretty | 15810 | ldf | 8055 | 3327 | 0.41 | 29023 | 24347 | $0.1108 | $0.0577 | 6/6 | 6/6 |
| sparse_events_100_pretty | 20927 | ldf | 10367 | 6452 | 0.62 | 31431 | 27517 | $0.1312 | $0.0658 | 6/6 | 6/6 |
| unicode_strings_100_compact | 12950 | ldf | 6806 | 4398 | 0.65 | 27873 | 25464 | $0.1105 | $0.0521 | 6/6 | 6/6 |
| nested_orders_20_compact | 5998 | json (selector) | 3083 | 3085 | 1.00 | 24139 | 24144 | $0.0743 | $0.0388 | 6/6 | 6/6 |
| **all** | | | 46899 | 26463 | 0.56 | 172959 | 152577 | $0.6971 | $0.3328 | 36/36 | 36/36 |

Runs: 36, total cost $3.0897 (Claude Code total_cost_usd). Costs are mean per run; cache reads and writes make cost depend on run order, the token columns do not.

