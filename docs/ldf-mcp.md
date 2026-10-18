# ldf-mcp: LDF between a data source and the model

`ldf-mcp` is the "middle man": the model keeps using ordinary MCP tools and
receives LDF instead of JSON. Two modes:

* `ldf-mcp proxy -- <server command>` wraps any stdio MCP server and
  re-encodes its JSON tool results (README, "MCP proxy").
* `ldf-mcp clickhouse` is an MCP server of its own over ClickHouse's HTTP
  interface. This page is about that mode and a local setup to try it.

## Tools

| tool | arguments | runs |
|---|---|---|
| `list_tables` | none | `system.tables` for the configured database: name, engine, rows, bytes |
| `describe_table` | `table` | `DESCRIBE TABLE <table>` (the name is validated as an identifier) |
| `run_select` | `sql` | the statement as given, without a `FORMAT` clause |

Every query goes to `POST /` with `readonly=1`, so ClickHouse refuses
anything that writes or changes settings, `max_result_rows` (default
10 000, `--max-rows`) with `result_overflow_mode=break`, and
`default_format=JSONEachRow` with 64-bit integers unquoted and named tuples
as objects. The rows come back as one text block: LDF when the JSON is at
least `--min-bytes` long (default 1 024) and the LDF is shorter, JSON
otherwise, JSON always with `--passthrough`. The `initialize` result carries
the LDF primer in `instructions`. One stderr line per result reports rows
and bytes (`--quiet` turns it off).

Networking is plain HTTP/1.1 over `std::net`: `--url` must be `http://`,
there is no TLS. The user comes from `--user` (default `default`) and the
password from the `CLICKHOUSE_PASSWORD` environment variable only; it is
never accepted as an argument, so it cannot end up in a process list or a
shell history.

## Local ClickHouse with the benchmark datasets

Any ClickHouse 24.x or 25.x works. With Docker (OrbStack on macOS), using a
pinned image and a port that does not collide with another instance:

```bash
docker run -d --name ldf-clickhouse -p 127.0.0.1:8124:8123 -e CLICKHOUSE_USER=ldf --ulimit nofile=262144:262144 clickhouse/clickhouse-server:25.8
```

The official image locks the `default` user to connections from inside the
container unless a user or password is configured, and reports that as
"Authentication failed". `CLICKHOUSE_USER=ldf` creates a passwordless
user that can connect from the host; the commands below pass it. For a
server with a password, export `CLICKHOUSE_PASSWORD` instead.

Generate the 10 000 row fixtures, copy five of them into the container, and
let ClickHouse infer the schemas:

```bash
cargo build --release && ./target/release/ldf-gen --scale medium && for f in flat_users sparse_events nested_orders wide_table repetitive_logs; do docker cp fixtures/medium/$f.jsonl ldf-clickhouse:/var/lib/clickhouse/user_files/$f.jsonl; done
mk() { curl -s -X POST -H 'X-ClickHouse-User: ldf' 'http://127.0.0.1:8124/' --data-binary "CREATE TABLE $1 ENGINE = MergeTree ORDER BY tuple() AS SELECT * FROM file('$2.jsonl', JSONEachRow)"; }
mk users flat_users && mk events sparse_events && mk orders nested_orders && mk wide wide_table && mk logs repetitive_logs
```

`orders` keeps its nesting: `customer` becomes a named tuple and `items` an
array of tuples, which ClickHouse returns as JSON objects and arrays, so the
LDF has a `records.items` child table. `events` has nullable columns for
the keys that are absent in most rows.

## Try it over stdio

```bash
printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18","capabilities":{},"clientInfo":{"name":"c","version":"0"}}}' '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"run_select","arguments":{"sql":"SELECT * FROM orders ORDER BY order_id LIMIT 200"}}}' | ./target/release/ldf-mcp clickhouse --url http://127.0.0.1:8124 --user ldf
```

The integration test `crates/ldf-mcp/tests/clickhouse.rs` does the same
and runs only when `LDF_CLICKHOUSE_URL` is set:

```bash
LDF_CLICKHOUSE_URL=http://127.0.0.1:8124 LDF_CLICKHOUSE_USER=ldf cargo test --release -p ldf-mcp --test clickhouse
```

## Register with Claude Code

```bash
claude mcp add ldf-clickhouse -- "$PWD/target/release/ldf-mcp" clickhouse --url http://127.0.0.1:8124 --user ldf
```

For an A/B of the same questions on JSON, register a second server with
`--passthrough` added after `clickhouse`. Remove with `claude mcp remove
ldf-clickhouse`.
