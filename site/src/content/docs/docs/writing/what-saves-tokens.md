---
title: "What actually saves tokens when LLMs query databases"
description: "Whitespace, the copy your client reads, result size, and a pre-registered test of whether models still answer correctly."
---

*Caio Ricciuti, 2026-10-22. Every number links to the report it comes
from.*

I built a token-efficient data format for LLMs, LDF. The idea was simple:
JSON repeats every key on every row, so a table format with one typed
header should let a model read the same data for fewer tokens. Then I
pointed it at real database MCP servers and ran a pre-registered study on
whether models still answer correctly. Most of what I expected turned out
to be wrong or small. Here is what holds up.

## 1. Most of the waste is whitespace

MotherDuck's DuckDB MCP server returns query results as pretty-printed
JSON. Over twelve test tables at 50 to 1 000 rows, removing the whitespace
alone cut 31 to 59 percent of the tokens on most results (13 percent on
high-cardinality strings, 5 percent on long text, which is mostly string
content). No format, no model risk, nothing to learn.
([columnar report](https://github.com/caioricciuti/ldf/tree/main/reports/2026-10-21-columnar))

My format, on top of the minified JSON, saved 0.1 to 7 percent on flat
tables, and one table came out 4 tokens larger. The server already sends
columns once and rows as arrays, so there is little left to remove. If
you control the server, the lesson is boring: ask the database for a
compact output format.

LDF did win where the data has structure: structs, lists and nested
arrays came out 13 to 28 percent smaller than minified JSON, and rows that
repeat their keys (what most MCP servers send) 19 to 57 percent smaller
([ClickHouse report](https://github.com/caioricciuti/ldf/tree/main/reports/2026-10-21-clickhouse)).

## 2. Claude Code may not be reading the copy you optimized

FastMCP servers return each result twice: as a text block and as
`structuredContent`, `{"result": "<the same text>"}`. In a headless Claude
Code pilot, the tool results in the transcripts had the same length in
all three of my test arms. Claude Code (2.1.306) shows the model
`structuredContent` when it is present, not the text block. So my proxy's
minified JSON and its compact format never reached the model; it saw the
original pretty JSON, with quotes and newlines escaped on top.

If you build anything that rewrites MCP tool output, check which copy your
client reads. The fix in my proxy is narrow: when `structuredContent` is
exactly FastMCP's wrapping of the text, rewrite it the same way.

## 3. In an agent loop, the result format barely matters, until results are big

Each Claude Code run in the pilot used about 20 000 prompt tokens, and
nearly all of it was Claude Code's own system prompt and tool definitions.
The model wrote targeted SQL, so results were a few rows. A compact format
saved 17 percent of the tool-result text, and the total prompt in that
arm still came out 3 percent larger: the primer that explains the format,
plus one run that took two extra turns.

What does matter is the occasional big result. `SELECT * FROM logs` on a
10 000-row table came back as 1.4 MB of JSON, and still 683 KB in my
format. No syntax fixes that. A cap does: keep the rows that fit in, say,
20 000 bytes, and tell the model how many rows exist and how to narrow the
query. That one feature saves more than any format
([how the cap works](/docs/mcp/#capping-result-size)).

## 4. "No loss detected" is not "no loss"

My earlier evaluations said frontier models read LDF as well as JSON. They
did: no significant difference. But with 52 to 95 questions per cell, that
mostly meant the test was too small to see a loss. When I audited the
evidence afterwards, only 5 of the 35 results that passed actually ruled
out a loss of 3 points. A perfect tie needs 88 questions just to reach
that bound.

So I pre-registered a non-inferiority study: 200 held-out records, 151 to
249 questions per cell, the same questions in both formats, and a rule
fixed in advance. A format passes only if the one-sided 95 percent bound on
its loss against JSON is at most 3 points. Results, 13 cells
([study report](https://github.com/caioricciuti/ldf/tree/main/reports/2026-10-21-ni-study)):

- 6 pass. Opus 5.5 on flat, sparse and mixed-type data; Sonnet 5.5 and
  DeepSeek on mixed types; Sonnet on SQL rows with NULLs written as
  `null` cells (at 0.47 of the tokens of the same rows as JSON objects).
- 5 are inconclusive: within about a point of JSON, but not provably
  within 3.
- 2 fail. DeepSeek on flat tables (6 points worse). And Opus 5.5 with my
  most aggressive nested layout, the one my own selector was sending it,
  got 10 questions wrong that it answered from JSON (p = 0.01).

The cheap evaluations had cleared that layout. The study cost about $21,
and the selector now sends LDF only where a cell passed
([evidence at a glance](/docs/evidence/)).

## What I would do with this

1. Minify tool output. Free and safe.
2. Know which copy of a tool result your client shows the model.
3. Cap big results and tell the model it was capped.
4. Use a compact format only for structured data, and only for models where
   you measured, with enough questions to rule a loss out rather than fail
   to detect one.

Everything above, including the misses and the raw responses, is in the
[repository](https://github.com/caioricciuti/ldf). The proxy that does
points 1 to 3 is [ldf-mcp](/docs/mcp/).
