# MCP end to end: ldf-mcp files

20 timed calls per query and variant after 3 warm-up calls, one long-lived server per variant, client-side wall time per tools/call over stdio. `ldf` is the server default (LDF when shorter than the JSON), `json` is --passthrough. Tokens are o200k_base (exact for GPT, a proxy for Claude) from `ldf-discover select`, on the JSON result; `-` means the format does not round-trip that result. No model was called.

| query | rows | traits | JSON tokens | TOON / JSON | LDF / JSON | p50 ms JSON | p50 ms LDF | p95 ms LDF | LDF overhead p50 |
|---|---:|---|---:|---:|---:|---:|---:|---:|---:|
| list_files | 41 | flat | 705 | 0.67 | 0.63 | 0.12 | 0.13 | 0.14 | +0.01 |
| describe_orders |  | not records | - | - | - | 12.28 | 12.25 | 12.40 | -0.03 |
| users_50 | 50 | flat | 2346 | 0.65 | 0.64 | 0.98 | 1.05 | 1.24 | +0.07 |
| events_200 | 200 | sparse | 11050 | 1.22 | 0.78 | 0.30 | 0.61 | 0.64 | +0.31 |
| orders_100 | 100 | child_tables,sparse | 9914 | 1.08 | 0.65 | 12.30 | 12.50 | 13.14 | +0.20 |
| logs_500 | 500 | flat | 23052 | 0.69 | 0.65 | 0.90 | 1.17 | 1.21 | +0.28 |
| wide_1000 | 1000 | flat | 234202 | 0.44 | 0.43 | 5.45 | 8.54 | 8.72 | +3.09 |
| logs_10000 | 10000 | flat | 460419 | 0.69 | 0.65 | 8.09 | 13.69 | 14.71 | +5.60 |

## Selector decision per model (default policy; with --extrapolate-rows in brackets)

| query | rows | claude-haiku-5-5 | deepseek-chat | claude-opus-5-5 | gpt-6.1-sol |
|---|---:|---|---|---|---|
| list_files | 41 | ldf@0.5 (ldf@0.5) | ldf@0.5 (ldf@0.5) | json (json) | json (json) |
| describe_orders |  | n/a (n/a) | n/a (n/a) | n/a (n/a) | n/a (n/a) |
| users_50 | 50 | ldf@0.5 (ldf@0.5) | ldf@0.5 (ldf@0.5) | json (json) | json (json) |
| events_200 | 200 | ldf@0.5 (ldf@0.5) | ldf@0.75 (ldf@0.75) | json (json) | json (json) |
| orders_100 | 100 | json (json) | ldf@0.5 (ldf@0.5) | json (json) | json (json) |
| logs_500 | 500 | json (ldf@0.5) | json (ldf@0.5) | json (json) | json (json) |
| wide_1000 | 1000 | json (ldf@0.5) | json (ldf@0.5) | json (json) | json (json) |
| logs_10000 | 10000 | json (ldf@0.5) | json (ldf@0.5) | json (json) | json (json) |
