---
title: "ldf-mcp"
description: "LDF between a data source and the model: the proxy and the ClickHouse mode."
---

`ldf-mcp` is the "middle man": the model keeps using ordinary MCP tools and
receives LDF instead of JSON. Three modes:

* `ldf-mcp proxy -- <server command>` wraps any stdio MCP server and
  re-encodes its JSON tool results (README, "MCP proxy"). A result
  that stays JSON goes out minified if it was pretty-printed, and LDF is
  sent only when it is shorter than that minified JSON. `--columnar`
  (opt-in) also converts results shaped `{"columns": [...], "rows":
  [[...], ...]}`, as MotherDuck's DuckDB server answers: see "Columnar
  results" below.
* `ldf-mcp clickhouse` is an MCP server of its own over ClickHouse's HTTP
  interface. Most of this page is about that mode and a local setup to try
  it.
* `ldf-mcp files --root DIR` is an MCP server over a directory of data
  files (json, jsonl, csv, tsv, parquet), read with the streaming readers
  and paged, so a file larger than memory still works. See the end of this
  page.

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

## Columnar results

With `ldf-mcp proxy --columnar`, a text result that is a JSON object with
one `columns` (non-empty, unique names) and one `rows` (every row an array
as long as `columns`) goes out as a frame: the result as one line of compact
JSON with `rows` replaced by the string `"@records"`, then the rows as the
LDF table `@records`, one column per entry of `columns` and `null` for SQL
NULL. Every other member (`success`, `columnTypes`, `rowCount`,
`truncated`, `warning`) stays on the first line, as it was:

```text
{"success":true,"columns":["id","name","score"],"columnTypes":["BIGINT","VARCHAR","DOUBLE"],"rows":"@records","rowCount":2}
@records id:i,name:s,score:f
1,Fay Goodall,19.27
2,Cleo Jackson,0.1
```

A frame is sent only when decoding it gives back exactly the original
result (members, their order, and integer or float kind) and it is shorter
than the original minified; otherwise the minified JSON goes out. Duplicate
column names, ragged rows, an empty result, and any number outside the
value model (an integer beyond u64, as DuckDB's `HUGEINT` can send, or a
decimal f64 cannot hold) stay JSON. `isError` results are never touched.
MotherDuck's server, like any FastMCP tool that returns a string, also
sends the text as `structuredContent` `{"result": "..."}`, and Claude Code
shows the model that copy rather than the text block. When
`structuredContent` is exactly `{"result": S}` with `S` the original text,
the proxy puts the rewritten text (frame or minified JSON) there as well,
so both copies agree; any other `structuredContent` is left as it is. The
primer gains
one paragraph describing the frame. With `--model`, columnar results stay
JSON: no model has been graded on a frame yet.

On the twelve medium fixtures through MotherDuck's server
(`reports/2026-10-21-columnar`), minifying alone removes 31 to 59 percent of
the tokens on most tables; the frame removes a further 1 to 7 percent on
flat tables and 13 to 28 percent where the rows have structs, lists or
mixed types. NULL-heavy results stay JSON, because the sparse-key rule
writes their NULLs as `"key":null` pairs.

## Choosing the format per model

`--model ID`, in all three modes, hands the JSON-or-LDF decision to the
selector in `ldf-select` (the same rule as `ldf-discover select`). For each
result it profiles the rows, encodes LDF at sparse thresholds 0.5 and 0.75,
each with and without hoisted arrays (spec section 7, rule 2a), round-trips
all four, and uses LDF only when there is measured evidence for this
exact model on data with the same traits (child tables, JSON cells,
sparse keys): at least 50 questions, no more than 3 points below JSON, no
significant loss, judged at the smallest measured size at or above the
input's row count (200 or 1 000 records; nothing larger than measured), and at least 5 percent
smaller than the JSON in bytes. Anything else stays JSON, and a stderr line
says why:

    ldf-mcp: query json for claude-haiku-5-5 (1 rows): no evidence for ldf (sparse threshold 0.5) on a measured dataset with traits {"child_tables"}

The evidence is a copy of `reports/evidence/evidence.json` built into
the binary; `--evidence FILE` uses another file of the same shape. A model
with no row of 50 questions or more gets JSON throughout, and the proxy then
leaves the primer and tool notes out. As of the Stage C run (2026-10-19) that
is GPT (15 to 20 questions per cell). Opus 5.5 gets LDF on flat, nested,
sparse and mixed-type data, DeepSeek too, Haiku 5.5 on flat, sparse and
mixed-type data with JSON on nested orders (v0.1 is 4.4 points and hoisted
7.4 points below JSON there). On nested orders Opus and DeepSeek get the
hoisted form, 40 percent of the JSON bytes against 46 for child tables.
The primer explains hoisted columns, so they are only sent with it. TOON is not
offered here because the primer explains LDF only. Each result is decided
on its own, so pages of one file can differ. Without `--model`, LDF is used
whenever it is shorter, as before.

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

## Row caps and paging

Both servers cap what one call returns (`--max-rows`, default 10 000 for
ClickHouse and 1 000 for files). A capped result carries a second text
block saying so and how to continue: narrow with `WHERE`, aggregate with
`GROUP BY`, or page with `LIMIT n OFFSET m` (ClickHouse) and `offset`
(files). The tool descriptions say the same, so the model asks for the next
slice instead of a bigger one.

## Files mode

```bash
claude mcp add exports -- "$PWD/target/release/ldf-mcp" files --root /path/to/exports
```

| tool | arguments | returns |
|---|---|---|
| `list_files` | none | every supported file under the root, recursively: path, kind, bytes |
| `describe_file` | `path` | the LDF header inferred from the first 200 records, child tables included |
| `read_rows` | `path`, `limit`, `offset` | records in file order, at most `--max-rows` per call, with a note when more follow |

Paths are relative to the root and resolved inside it; anything else is
refused. Hidden directories, `node_modules` and `target` are skipped.

## Register with Claude Code

```bash
claude mcp add ldf-clickhouse -- "$PWD/target/release/ldf-mcp" clickhouse --url http://127.0.0.1:8124 --user ldf
```

For an A/B of the same questions on JSON, register a second server with
`--passthrough` added after `clickhouse`. Remove with `claude mcp remove
ldf-clickhouse`.

## Local statistics

Every mode (`proxy`, `files`, `clickhouse`, and the Claude Code hook,
`ldf-mcp hook claude-code`, as server `claude-code`, tool `Bash`) appends
one line per tool response to a file on this machine, and `ldf-mcp stats`
summarises it:

```text
tool responses     3
  sent as LDF      2 (67%)
  left as JSON     1
         1  under --min-bytes

bytes              141.6 KB in, 78.2 KB out, 63.4 KB saved (45%)
  converted only   141.1 KB -> 77.7 KB (55% of the JSON)
tokens saved       about 16224 (estimate: bytes / 4, not a tokenizer)
conversion time    p50 2.1 ms, p95 6.4 ms, max 6.4 ms per response
```

A line holds the time, mode, server label (`--name NAME`, default the
downstream command's file name), tool name, outcome (`ldf` or `json`), the
reason code when it stayed JSON (`small`, `not_json`, `not_dataset`,
`not_shorter`, `selector`, `is_error`, `out_of_domain`, `not_lossless`,
`passthrough`, `no_text`, and for the
hook `persisted`, `exit_code`, `interrupted`, `background`, `image`,
`bad_input`, `too_large`), bytes in (for the hook, out includes the note it
adds)
and out, conversion time and, in proxy mode, request-to-reply time and the
`--model` given. It never holds response content, tool arguments or
environment values, and nothing is sent anywhere.

The file is `LDF_STATS_FILE` when set, otherwise `ldf/events.jsonl` under
`XDG_STATE_HOME`, `~/.local/state` or (Windows) `LOCALAPPDATA`. At 16 MiB it
moves to `events.1.jsonl`, replacing the previous one. `--no-stats` or
`LDF_STATS=0` records nothing. `ldf-mcp stats --days 7` limits the window,
`--json` prints the totals for scripts, `--file PATH` reads another file.
With `--passthrough` the responses are recorded as `passthrough` with their
real sizes, so an A/B run shows both sides.

## Measuring

The statistics count serialization savings: bytes that did not go into the
model's context. Three things sit between that and what a model is billed.

1. Tokens are not bytes. `bytes / 4` is a rule of thumb; for the measured
   o200k token ratios of LDF against JSON per dataset see
   `reports/`, and for Claude the ratio is measured with Anthropic's
   count_tokens (`eval/count_tokens_claude.py`), not estimated.
2. Agents truncate or spill large tool results before the model sees them
   (Claude Code saves results over 25 000 tokens or 50 000 characters to a
   file; Codex truncates to its per-model limit; OpenCode at 2 000 lines or
   50 KB). A result that LDF brings under such a limit can mean the model
   sees all of it instead of a preview, which no byte count captures.
3. Prompt caching changes what repeated context costs.

For real usage, compare the same task with and without LDF using the agent's
own counters: Claude Code's OpenTelemetry metric `claude_code.token.usage`
(`CLAUDE_CODE_ENABLE_TELEMETRY=1`, console or Prometheus exporter) or the
status line's `context_window` fields; Codex's `TokenCount` events in
`~/.codex/sessions/`; OpenCode's `opencode stats`.
