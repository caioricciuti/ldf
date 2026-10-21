# ldf-mcp proxy in front of a DuckDB MCP server (2026-10-21)

The check behind the landing page's "proxy in front of a database MCP
server": the same nine calls as `reports/2026-10-21-clickhouse`, against
DuckDB through MotherDuck's MCP server, with `ldf-mcp proxy` in front. No
model was called.

Server: `mcp-server-motherduck==1.1.0` from PyPI, run with `uvx
--exclude-newer 2026-10-14 --no-build` (wheels only, nothing published
after that date; it resolved fastmcp 4.1.0 and duckdb 1.5.6), read-only,
`--max-chars 100000000` so no result was truncated. Database: the five
medium fixtures loaded with DuckDB 1.5.6 `read_json_auto` into tables
`users`, `events`, `orders`, `wide`, `logs`. Machine: Apple M2 Pro, 10
cores, 16 GB, macOS 27.0.1. Repository at 76b30e4 plus the `duckdb` mode
this run added to `bench_mcp.py`.

Reproduce:

```bash
cargo build --release
uv run --exclude-newer 2026-10-14 --no-build --no-project --with duckdb==1.5.6 python -I -c 'import duckdb; c = duckdb.connect("results/mcp-duckdb/ldf.duckdb"); [c.execute(f"CREATE OR REPLACE TABLE {t} AS SELECT * FROM read_json_auto(?)", [f"fixtures/medium/{f}.jsonl"]) for t, f in [("users","flat_users"),("events","sparse_events"),("orders","nested_orders"),("wide","wide_table"),("logs","repetitive_logs")]]'
python3 -I eval/bench_mcp.py --mode duckdb --db results/mcp-duckdb/ldf.duckdb --repeats 10 --out results/mcp-duckdb
python3 -I reports/2026-10-21-duckdb/compare.py results/mcp-duckdb/texts > reports/2026-10-21-duckdb/compare.md
```

`mcp.md` and `mcp.json` are the bench's own output, `compare.md` is
`compare.py`'s, `texts/` keeps the four smaller results.

## Findings

1. **The proxy passes everything through.** This server answers a query
   with one pretty-printed object, `{"success", "columns", "columnTypes",
   "rows": [[...], ...]}`, rows as arrays of values. The proxy converts
   arrays of records; to it this is a single record, LDF is not shorter,
   and the JSON goes out unchanged (same bytes in both variants in
   `mcp.json`). Only `describe_orders`, whose `columns` is an array of
   objects, was converted. The token ratios in `mcp.md` are for that
   one-record view and say nothing about the rows.
2. **Most of what could be saved is whitespace.** From `compare.md`
   (o200k): minifying the JSON alone takes it to 0.55 to 0.64 of what is
   sent. A columnar layout already writes each key once, so LDF of the
   rows is only 3 to 7 percent below minified JSON on flat tables
   (users, logs, wide). It is 25 percent below on `orders_100`, where the
   child table replaces repeated struct keys.
3. **SQL nulls meet the v0.1 sparse rule.** A table has every column on
   every row, so a sparse field comes back as `null`, never absent. The
   sparse-key rule (spec section 7, rule 4) counts non-null values only,
   so on `events` ten mostly-null columns move into `~` and every row
   carries them as `"key":null` pairs: LDF is 1.61 times the minified
   columnar JSON there. The v0 layout (rule off) is about 0.6 of v0.1 on
   these rows. The rule exists because Haiku lost answers on positional
   sparse rows, so this is a format question, not a bug fix: whether a
   key present on every row, even as null, should keep its column.

Consequences: the landing page now says the proxy converts servers that
return arrays of records and passes other shapes through. Teaching the
proxy the `columns`/`rows` shape, and the null question above, are open
and wait for a user who needs them.
