# MCP end to end: ldf-mcp duckdb

10 timed calls per query and variant after 3 warm-up calls, one long-lived server per variant, client-side wall time per tools/call over stdio. `ldf` is the server default (LDF when shorter than the JSON), `json` is --passthrough. Tokens are o200k_base (exact for GPT, a proxy for Claude) from `ldf-discover select`, on the JSON result; `-` means the format does not round-trip that result. No model was called.

| query | rows | traits | JSON tokens | TOON / JSON | LDF / JSON | p50 ms JSON | p50 ms LDF | p95 ms LDF | LDF overhead p50 |
|---|---:|---|---:|---:|---:|---:|---:|---:|---:|
| list_tables | 1 | child_tables | 109 | 0.78 | 0.76 | 16.06 | 16.25 | 17.00 | +0.18 |
| describe_orders | 1 | child_tables | 179 | 0.75 | 0.75 | 25.75 | 24.20 | 24.62 | -1.55 |
| users_50 | 1 | json_cells | 1549 | 1.15 | 1.00 | 6.84 | 6.52 | 6.68 | -0.32 |
| users_by_country | 1 | json_cells | 215 | 1.41 | 1.01 | 6.14 | 6.20 | 6.28 | +0.06 |
| events_200 | 1 | json_cells | 8441 | 1.09 | 1.00 | 7.57 | 7.85 | 7.92 | +0.28 |
| orders_100 | 1 | json_cells | 8565 | 1.45 | 1.00 | 8.23 | 8.77 | 9.11 | +0.53 |
| logs_errors_500 | 1 | json_cells | 15060 | 1.16 | 1.00 | 7.94 | 8.93 | 9.21 | +0.99 |
| wide_1000 | 1 | json_cells | 103437 | 1.03 | 1.00 | 18.48 | 34.50 | 34.98 | +16.02 |
| logs_10000 | 1 | json_cells | 300471 | 1.16 | 1.00 | 39.18 | 64.03 | 65.10 | +24.85 |

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
