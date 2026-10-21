# MCP end to end: ldf-mcp duckdb

10 timed calls per query and variant after 3 warm-up calls, one long-lived server per variant, client-side wall time per tools/call over stdio. `ldf` is the server default (LDF when shorter than the JSON), `json` is --passthrough. Tokens are o200k_base (exact for GPT, a proxy for Claude) from `ldf-discover select`, on the JSON result; `-` means the format does not round-trip that result. No model was called.

| query | rows | traits | JSON tokens | TOON / JSON | LDF / JSON | p50 ms JSON | p50 ms LDF | p95 ms LDF | LDF overhead p50 |
|---|---:|---|---:|---:|---:|---:|---:|---:|---:|
| list_tables | 1 | child_tables | 109 | 0.78 | 0.76 | 16.67 | 15.87 | 15.93 | -0.81 |
| describe_orders | 1 | child_tables | 179 | 0.75 | 0.75 | 24.08 | 24.41 | 25.18 | +0.33 |
| users_50 | 1 | json_cells | 1549 | 1.15 | 1.00 | 6.22 | 6.33 | 6.62 | +0.10 |
| users_by_country | 1 | json_cells | 215 | 1.41 | 1.01 | 5.99 | 6.14 | 6.29 | +0.15 |
| events_200 | 1 | json_cells | 8441 | 1.09 | 1.00 | 7.16 | 7.62 | 8.01 | +0.46 |
| orders_100 | 1 | json_cells | 8565 | 1.45 | 1.00 | 7.91 | 7.71 | 7.93 | -0.20 |
| logs_errors_500 | 1 | json_cells | 15060 | 1.16 | 1.00 | 7.80 | 8.41 | 8.78 | +0.61 |
| wide_1000 | 1 | json_cells | 103437 | 1.03 | 1.00 | 19.01 | 21.13 | 22.84 | +2.12 |
| logs_10000 | 1 | json_cells | 300471 | 1.16 | 1.00 | 39.98 | 43.09 | 43.84 | +3.11 |

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
