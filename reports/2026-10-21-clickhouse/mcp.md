# MCP end to end: ldf-mcp clickhouse

20 timed calls per query and variant after 3 warm-up calls, one long-lived server per variant, client-side wall time per tools/call over stdio. `ldf` is the server default (LDF when shorter than the JSON), `json` is --passthrough. Tokens are o200k_base (exact for GPT, a proxy for Claude) from `ldf-discover select`, on the JSON result; `-` means the format does not round-trip that result. No model was called.

| query | rows | traits | JSON tokens | TOON / JSON | LDF / JSON | p50 ms JSON | p50 ms LDF | p95 ms LDF | LDF overhead p50 |
|---|---:|---|---:|---:|---:|---:|---:|---:|---:|
| list_tables | 5 | flat | 108 | 0.67 | 0.65 | 2.29 | 2.93 | 3.86 | +0.65 |
| describe_orders | 7 | flat | 250 | 0.60 | 0.60 | 1.62 | 2.44 | 3.05 | +0.82 |
| users_50 | 50 | flat | 2346 | 0.65 | 0.62 | 2.92 | 3.68 | 4.60 | +0.76 |
| users_by_country | 20 | flat | 302 | 0.63 | 0.57 | 3.55 | 4.75 | 6.13 | +1.20 |
| events_200 | 200 | json_cells,sparse | 17154 | 0.50 | 0.81 | 4.48 | 6.11 | 7.84 | +1.63 |
| orders_100 | 100 | child_tables,json_cells,sparse | 10004 | 1.07 | 0.56 | 4.70 | 5.52 | 6.04 | +0.82 |
| logs_errors_500 | 500 | flat | 23008 | 0.69 | 0.65 | 3.92 | 5.67 | 7.98 | +1.74 |
| wide_1000 | 1000 | json_cells | 234216 | 0.44 | 0.43 | 9.65 | 13.46 | 15.17 | +3.81 |
| logs_10000 | 10000 | flat | 460418 | 0.69 | 0.65 | 12.61 | 19.48 | 20.59 | +6.87 |

## Selector decision per model (default policy; with --extrapolate-rows in brackets)

| query | rows | claude-haiku-5-5 | deepseek-chat | claude-opus-5-5 | gpt-6.1-sol |
|---|---:|---|---|---|---|
| list_tables | 5 | ldf@0.5 (ldf@0.5) | ldf@0.5 (ldf@0.5) | ldf@0.5 (ldf@0.5) | json (json) |
| describe_orders | 7 | ldf@0.5 (ldf@0.5) | ldf@0.5 (ldf@0.5) | ldf@0.5 (ldf@0.5) | json (json) |
| users_50 | 50 | ldf@0.5 (ldf@0.5) | ldf@0.5 (ldf@0.5) | ldf@0.5 (ldf@0.5) | json (json) |
| users_by_country | 20 | ldf@0.5 (ldf@0.5) | ldf@0.5 (ldf@0.5) | ldf@0.5 (ldf@0.5) | json (json) |
| events_200 | 200 | toon (toon) | toon (toon) | ldf@0.5 (ldf@0.5) | json (json) |
| orders_100 | 100 | json (json) | json (json) | json (json) | json (json) |
| logs_errors_500 | 500 | json (ldf@0.5) | json (ldf@0.5) | json (json) | json (json) |
| wide_1000 | 1000 | json (json) | json (json) | json (json) | json (json) |
| logs_10000 | 10000 | json (ldf@0.5) | json (ldf@0.5) | json (json) | json (json) |
