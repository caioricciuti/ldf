# MCP end to end: ldf-mcp duckdb

10 timed calls per query and variant after 3 warm-up calls, one long-lived server per variant, client-side wall time per tools/call over stdio. `ldf` is the server default (LDF when shorter than the JSON), `json` is --passthrough. Tokens are o200k_base (exact for GPT, a proxy for Claude) from `ldf-discover select`, on the JSON result; `-` means the format does not round-trip that result. No model was called.

| query | rows | traits | JSON tokens | TOON / JSON | LDF / JSON | p50 ms JSON | p50 ms LDF | p95 ms LDF | LDF overhead p50 |
|---|---:|---|---:|---:|---:|---:|---:|---:|---:|
| list_tables | 1 | child_tables | 109 | 0.78 | 0.76 | 18.20 | 19.66 | 20.16 | +1.46 |
| describe_orders | 1 | child_tables | 179 | 0.75 | 0.75 | 28.61 | 25.68 | 26.27 | -2.93 |
| users_50 | 1 | json_cells | 1549 | 1.15 | 1.00 | 6.92 | 6.92 | 7.28 | +0.00 |
| users_by_country | 1 | json_cells | 215 | 1.41 | 1.01 | 6.89 | 6.89 | 7.00 | -0.00 |
| events_200 | 1 | json_cells | 8441 | 1.09 | 1.00 | 8.35 | 7.88 | 7.98 | -0.47 |
| orders_100 | 1 | json_cells | 8565 | 1.45 | 1.00 | 8.16 | 8.51 | 8.58 | +0.34 |
| logs_errors_500 | 1 | json_cells | 15060 | 1.16 | 1.00 | 8.64 | 8.42 | 8.50 | -0.23 |
| wide_1000 | 1 | json_cells | 103437 | 1.03 | 1.00 | 20.41 | 22.47 | 23.00 | +2.06 |
| logs_10000 | 1 | json_cells | 300471 | 1.16 | 1.00 | 40.85 | 48.30 | 48.81 | +7.44 |

## Selector decision per model (default policy; with --extrapolate-rows in brackets)

| query | rows | claude-haiku-5-5 | deepseek-chat | claude-opus-5-5 | gpt-6.1-sol |
|---|---:|---|---|---|---|
| list_tables | 1 | json (json) | json (json) | json (json) | json (json) |
| describe_orders | 1 | json (json) | json (json) | json (json) | json (json) |
| users_50 | 1 | json (json) | json (json) | json (json) | json (json) |
| users_by_country | 1 | json (json) | json (json) | json (json) | json (json) |
| events_200 | 1 | json (json) | json (json) | json (json) | json (json) |
| orders_100 | 1 | json (json) | json (json) | json (json) | json (json) |
| logs_errors_500 | 1 | json (json) | json (json) | json (json) | json (json) |
| wide_1000 | 1 | json (json) | json (json) | json (json) | json (json) |
| logs_10000 | 1 | json (json) | json (json) | json (json) | json (json) |
