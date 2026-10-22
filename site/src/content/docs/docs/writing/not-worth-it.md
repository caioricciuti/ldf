---
title: "I spent $70 proving my LLM data format isn't worth using"
description: "A compact format for LLM tool output does save tokens. Here is why that turned out not to matter, and why paying to find out was cheap."
---

*Caio Ricciuti, 2026-10-22. Every number links to the report it comes
from.*

## The bet

JSON repeats every key on every row. Send a model 200 user records and it
reads `"email":` 200 times. So I built LDF: a lossless table format with
one typed header and one line per record, nested arrays as child tables,
sparse keys handled separately. Same data, fewer tokens. TOON and other
formats make the same bet.

## It does save tokens

That part was true. On irregular data (nested orders, sparse events,
mixed types) LDF came out 20 to 64 percent smaller than compact JSON, and
within 1 to 3 percent of CSV on flat tables
([baseline report](https://github.com/caioricciuti/ldf/tree/main/reports/2026-10-17-baseline)).
The codec round-trips exactly and runs as fast as JSON. I wrapped it in an
MCP proxy so any agent could get it without changing its tools.

Then I pointed it at real database MCP servers and real agents, and the
saving mostly disappeared.

## Why it didn't matter

**Most of the waste was whitespace.** MotherDuck's DuckDB MCP server sends
query results as pretty-printed JSON. Removing the whitespace alone cut 31
to 59 percent of the tokens on most results. That is one line of code in
any server, no format, no risk. On top of minified JSON, my format saved
0.1 to 7 percent on flat tables, and one table came out 4 tokens larger.
The server already sends columns once and rows as arrays, so there was
little left to win
([columnar report](https://github.com/caioricciuti/ldf/tree/main/reports/2026-10-21-columnar)).

**The model wasn't even reading my output.** FastMCP servers return every
result twice: as text and as `structuredContent`. Claude Code (2.1.306)
shows the model the second copy. My proxy rewrote the first one, so in
my first agent pilot every arm sent the model the same original
pretty-printed JSON.

**In an agent session, results are a rounding error.** Each headless
Claude Code run used about 20 000 prompt tokens, nearly all of it Claude
Code's own system prompt and tool definitions. The model wrote targeted
SQL and got a few rows back. LDF cut the tool-result text by 17 percent,
and the total prompt in that arm still came out 3 percent larger, because
the model has to be told what the format is.

**What does blow up context is one huge result,** and syntax doesn't fix
that. `SELECT * FROM logs` on a 10 000-row table came back as 1.4 MB of
JSON, and still 683 KB in my format. A cap that keeps the rows that fit
and tells the model how to narrow the query fixes it.

## The part that cost the money

My first evaluations said frontier models read LDF as well as JSON: no
significant difference. When I audited them, that mostly meant the tests
were too small to see a loss. With 52 to 95 questions per cell, only 5 of
35 passing results actually ruled out a loss of 3 points. A perfect tie
needs 88 questions just to get there.

So I pre-registered a non-inferiority study: the rule written and
committed before the first response, 200 held-out records, 151 to 249
questions per cell, the same questions in both formats. LDF passes only if
the one-sided 95 percent bound on its loss against JSON is at most 3
points
([study report](https://github.com/caioricciuti/ldf/tree/main/reports/2026-10-21-ni-study)).

Of 13 cells, 6 passed, 5 were inconclusive and 2 failed. One failure was a
layout I was already shipping: Opus 5.5 with hoisted nested arrays got 10
questions wrong that it answered correctly from JSON (p = 0.01). Every
cheaper evaluation had cleared it.

Where LDF passes, it passes clearly. But that is Opus on three data
shapes, Sonnet and DeepSeek on one or two, and no Haiku or GPT at all.
For everyone else the honest answer is: send JSON.

## Why $70 is cheap

All of it, across every evaluation in the repository, was about $60 in API
calls at list prices, plus $10 for the domain. The study alone was $21.

The alternative was shipping on the small benchmark. It said "no loss",
I believed it, and I had a selector sending Opus a layout that cost it
answers. I would have found out from users, or never. Weeks of building
are the expensive part of a project like this. The $60 of measurement is
what told me to stop.

## What to do instead

1. Minify tool output. Free and safe, for every model.
2. Check which copy of a tool result your client shows the model.
3. Cap big results and tell the model it was capped.
4. Before you trust any "the model reads format X fine" claim, mine
   included, ask how many questions it rests on. Under about 90 per cell
   it cannot rule out a 3-point loss.

LDF stays published as research. The code, the pre-registration, the raw
responses and the misses are all in the
[repository](https://github.com/caioricciuti/ldf).
