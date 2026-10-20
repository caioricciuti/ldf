---
title: "Research notes: existing LLM-facing data formats"
description: "What exists already, with facts established by implementing and measuring it."
---

State as of 2026-10-17. Two parts:

* Part A: facts established in this repository by implementing and measuring
  (reproducible with the commands in `README.md`).
* Part B: a literature and ecosystem survey compiled with web access on the
  same day. Every claim carries a provenance tag. Before quoting a `[S]`
  item externally, fetch the primary source.

Legend: **[V]** verified against a primary source that was fetched, or
against a measurement in this repo (URL or command given). **[S]** seen only
in a secondary source or search snippet, primary not reached. **[H]**
hypothesis, not measured anywhere found.

---

## Part A: findings from building and measuring the baselines

All numbers below are from `reports/2026-10-17-baseline/` (Apple M2 Pro,
10 000-row fixtures, release build, `ldf-bench perf --iters 7` and
`ldf-bench tokens --sample 1000`).

1. **TOON v4.4 is implementable from the spec and the fixture suite is
   usable.** An independent Rust encoder and decoder written from SPEC.md
   passes all 160 encode and 404 decode fixtures at tag v4.4.0. Two harness
   mistakes along the way are worth recording for anyone else running the
   suite: fixture JSON must be loaded with an order-preserving object type,
   and a blank line between a header and its first row is legal only when no
   enclosing header span is open. **[V]** `cargo test -p ldf-formats`.

2. **TOON is not streamable in one pass.** The root header carries `[N]`
   and tabular eligibility is a property of the whole array (every element,
   every column), so the encoder must see all records before emitting the
   first byte. JSON, JSONL, CSV and Markdown can all be emitted record by
   record (`StreamEncoder` in `ldf-formats`). **[V]** spec sections 6 and
   9.3; `Capabilities::requires_count`.

3. **TOON's size advantage is confined to uniform flat data.** Bytes
   relative to compact JSON: flat_users 0.47, repetitive_logs 0.50,
   wide_table 0.51, high_card_strings 0.79, long_text 0.97, nested_orders
   1.04, sparse_events 1.11, empty_values 1.18, mixed_types 1.21,
   deep_nesting 2.50. o200k_base tokens relative to compact JSON follow the
   same pattern: 0.65, 0.69, 0.44, 0.90, 0.97, 1.08, 1.22, 1.27, 1.33, 1.82.
   On every flat dataset CSV is smaller than TOON in both bytes and tokens.
   **[V]** `perf.md`, `tokens.md`.

4. **Bytes per token differ by format on the same data.** On flat_users:
   JSON 2.98, JSONL 2.92, Markdown 2.35, CSV 2.22, TOON 2.14 bytes per
   o200k token. A format can be shorter in bytes and yet tokenize worse, so
   byte ratios are not a proxy for token ratios. **[V]** `tokens.md`.

5. **Sparse data defeats tabular forms.** sparse_events has 3 required and
   12 optional fields with distinct "absent" and "null" states. TOON cannot
   use tabular form (key sets differ) and costs 1.22x JSON tokens. CSV costs
   0.61x but is lossy: absent and null both become an empty cell (15 fields
   in, 9 out on record 0). Markdown matches JSON's token count (0.97x) while
   still being lossy. **[V]** round-trip levels in `perf.md`.

6. **Canonical number text is ambiguous in typed hosts.** TOON, CSV and
   Markdown print a double such as 2.167128000339913e18 as
   `2167128000339913000` (shortest round-trip digits, zero padded, no
   exponent below 1e21). A JavaScript decoder reads that back as the same
   double. A decoder with an integer type reads it as the integer
   2167128000339913000, which is not the double's value
   (2167128000339913216). The round-trip is therefore "lossy" for every
   text format except JSON on numeric_mix. **[V]** `perf.md` row
   `944.huge`; TOON spec section 2.

7. **TOON canonicalisation drops float-vs-integer kind.** `1.0` encodes
   as `1` and decodes as an integer. Semantic equality holds; byte-level
   equality does not. Nine of twelve datasets round-trip byte-exact through
   TOON, three only semantically. **[V]** `ldf-bench roundtrip`.

8. **serde_json's default number parser is not correctly rounded for
   integer literals above u64.** `123456789012345680000` parsed one ULP low.
   The `float_roundtrip` feature fixes it. **[V]**
   `crates/ldf-data/tests/bigint_literal.rs`.

9. **Encode and decode cost.** With serde_json, compact JSON encodes at
   0.58 to 2.0 GB/s and decodes at 0.22 to 2.2 GB/s depending on dataset;
   simd-json parses 20 to 40 percent faster on most datasets at 1.3 to 2x
   the peak RSS, and is slower on deep_nesting and long_text. This repo's
   TOON implementation encodes at 110 to 400 MB/s and decodes at 56 to
   250 MB/s; CSV (csv crate) encodes at 90 to 2 900 MB/s and decodes at 140
   to 1 200 MB/s. Those are properties of these implementations, not of the
   formats. **[V]** `perf-medium.md`.

10. **Block YAML costs more tokens than compact JSON everywhere.** Bytes are
    within 4 percent of compact JSON on regular data, but o200k tokens are
    1.07x to 1.34x (deep nesting 1.88x): indentation and newlines are
    tokenized less efficiently than JSON's punctuation. YAML is the only
    non-JSON baseline that round-trips byte-exact on all twelve datasets.
    The literature's nested-data accuracy wins for YAML (Part B, section 5)
    are therefore paid for in tokens. **[V]** `tokens-medium.md`,
    `perf-medium.md`.

11. **A typed table with child tables closes the irregular-data gap.** LDF
    v0 ([the specification](/docs/spec/)), built from items 3 to 7, measures 0.61 /
    0.64 / 0.63 / 0.68 / 0.80 / 0.36 of compact JSON's o200k tokens on
    flat_users / nested_orders / sparse_events / mixed_types / deep_nesting
    / empty_values, against TOON's 0.65 / 1.08 / 1.22 / 1.33 / 1.82 / 1.27,
    and round-trips byte-exact on 11 of 12 datasets. The design choices that
    matter, each measured separately in [design notes](/docs/design/): a typed
    header costs nothing and removes quoting; relational child tables save
    37 percent on arrays of objects; comma tokenizes 5 to 7 percent better
    than pipe. **[V]** `tokens-medium.md`, `perf-medium.md`.

12. **Arrow JSON schema inference cannot type heterogeneous or empty
    columns.** mixed_types (a field that is sometimes a scalar, sometimes an
    object) and empty_values (empty arrays with no element type) have no
    Parquet fixture. Any Parquet to text pipeline needs a schema source
    other than inference for such data. **[V]** `ldf-gen` output.

---

## Part B: survey

Compiled 2026-10-17 by a research assistant with web access; primary
sources are listed per claim and in the Sources section.

### 1. TOON (Token-Oriented Object Notation)

**Spec and governance.** Spec is "Version 4.4, Working Draft", dated
2026-10-08 in the CHANGELOG. MIT, copyright 2025-present Johann Schopplich.
1.0 was 2025-10-28; 4.0 (2026-07-22) was a breaking release that removed key
folding and path expansion ("dotted keys are always literal"), made `#` a
line comment, and introduced keyed tabular form. 4.1 made tabular form
mandatory where eligible and put the delimiter in the header. 4.4 forbids
blank lines from encoders and tightens non-strict recovery to five listed
cases. **[V]** https://github.com/toon-format/spec/blob/main/CHANGELOG.md ,
https://raw.githubusercontent.com/toon-format/spec/main/SPEC.md

**Core syntax (SPEC.md sections 2, 6, 7.2, 9, 14).** **[V]**
- Indentation for nested objects; `key: value` lines.
- Array headers carry a length marker: `items[N]:`. Inline primitive arrays
  `tags[2]: a,b`. List form `- ` for non-uniform arrays. Tabular form
  `users[3]{id,name,role}:` followed by delimited rows. Keyed tabular
  `users[2:]{age,city}:` for objects whose values are uniform objects. Nested
  field groups `customer{name,country}` fold a uniform nested object column
  into the header.
- Tabular eligibility: every element is a non-empty object, all share the
  same key set, and every column is "uniform-primitive or nested-uniform". A
  column mixing null with objects, or containing arrays, disqualifies.
- Delimiters: comma (default, no symbol in header), tab, pipe; declared per
  header, "no inheritance".
- Quoting is minimal but rule-heavy: quote if empty, leading/trailing
  whitespace, equals `true|false|null`, numeric-like, contains `: " \ [ ] { }`,
  control chars, the active delimiter, or starts with `-` or `#`.
- Numbers: canonical shortest round-trip decimal for 1e-6 <= |n| < 1e21,
  `-0` normalised to `0`.
- Strict mode (default) errors when row count differs from N or a row's
  cell count differs from the field count.

**Implementations.** Official, under the toon-format org: TypeScript/JS
(reference, spec 4.4), .NET, Dart, Go, Java, Julia (4.4), Python (4.2), Rust
and Swift (4.1). About 25 community ports (C, C#, Clojure, Crystal, Elixir,
Gleam, Kotlin, OCaml, Perl, PHP, R, Ruby, Scala, Zig, and others), many
pinned to older spec versions (1.0 to 3.3). The ecosystem page recommends
running the fixture suite in CI but "does not state that any specific
implementation has passed the suite". **[V]**
https://toonformat.dev/ecosystem/implementations.html . npm reports 22 809
GitHub stars and `@toon-format/toon` 4.4.0 published roughly a week ago.
**[S]** https://www.npmjs.com/package/@toon-format/toon

**Official benchmark methodology (README).** **[V]**
https://github.com/toon-format/toon
- Token counting: `gpt-tokenizer` with `o200k_base`, baseline is 2-space
  pretty JSON; also reports compact JSON, YAML, XML. README states "actual
  savings vary by model and tokenizer".
- Accuracy: 244 retrieval questions x 6 formats x 4 models = 5 856 calls;
  models `claude-haiku-4-5-20251001`, `gemini-3.6-flash`, `gpt-5.4-nano`,
  `grok-4.5`; reasoning disabled; Wilson 95% CIs. CSV only on the 109
  flat-dataset questions.
- Datasets: 13, including 5 "structural validation" sets (truncated arrays,
  extra rows, inconsistent field count, missing fields, control).

**Official numbers.** **[V]**

| Format | Accuracy | Tokens | acc%/1K tok |
|---|---|---|---|
| TOON | 72.2% +-2.8 | 2 474 | 29.2 |
| JSON compact | 69.0% +-2.9 | 2 892 | 23.8 |
| YAML | 70.1% +-2.9 | 3 487 | 20.1 |
| JSON pretty | 71.4% +-2.8 | 4 308 | 16.6 |
| XML | 70.7% +-2.9 | 4 909 | 14.4 |

Headline: "72.2% accuracy (vs JSON's 71.4%) while using 42.6% fewer tokens".
The README itself notes the CIs overlap, so "the difference between them is
not statistically meaningful". Token totals: mixed-structure track TOON vs
pretty JSON -32.7%, vs compact JSON **+1.6%** (TOON loses to compact JSON on
orders +5.6%, event logs +19.9%, deep config +6.7%). Flat track: TOON is
**+5.9% vs CSV**, -58.7% vs pretty JSON. Per question type, TOON's only clear
win is "Structural Validation" (100% vs JSON 50%) and "Structure Awareness"
(90.3% vs 84.0%); on Field Retrieval it is slightly below JSON (97.8% vs
99.2%); Aggregation ties at 48.4%; Filtering 38.0% vs 41.1%. Per model,
gpt-5.4-nano prefers XML; grok-4.5 scores 95 to 97% on everything, making
format nearly irrelevant.

These README numbers agree in direction with Part A items 3 and 5: the
repo's own measurement puts TOON at +4% bytes and +8% tokens over compact
JSON on nested orders and +11% bytes / +22% tokens on sparse events.

**Stated limitations (README).** Deep or non-uniform data: "compact JSON
often wins outright"; semi-uniform arrays: savings shrink; flat data: CSV is
smaller and TOON's "~5–10% overhead buys declared lengths, field lists, and
delimiter scoping"; latency: some local or quantised models process compact
JSON faster. **[V]**

**Independent re-evaluations.**
- improvingagents.com, 2025-10-28, GPT-4.1-nano, 1 000 flat employee
  records, 1 000 lookup questions: TOON 47.5% [44.4, 50.6] at 21 518 tokens,
  ranked 9th of 12; JSON 52.3%; Markdown-KV 60.7%; CSV 44.3% at 19 524
  tokens. Nested test on GPT-5-nano: TOON **last** at 43.1% (45 436 tokens)
  vs YAML 62.1%, Markdown 54.3%, JSON 50.3%, XML 44.4%. **[V]**
  https://www.improvingagents.com/blog/toon-benchmarks
- McMillan, arXiv 2602.05447 (Feb 2026): 9 649 runs, 11 models,
  YAML/Markdown/JSON/TOON, SQL generation over schemas of 10 to 10 000
  tables. "Format does not significantly affect aggregate accuracy
  (chi-squared=2.45, p=0.484)"; model capability gap of 21 pp dwarfs format;
  compact formats "can consume significantly more tokens at scale due to
  format-unfamiliar search patterns". **[V]** https://arxiv.org/abs/2602.05447
  . The "38% more tokens" and "7.4x at largest schema" figures are reported
  by a vendor blog, not in the abstract. **[S]**
  https://usewire.io/blog/toon-vs-json-why-smaller-doesnt-mean-cheaper/
- Matveev, arXiv 2603.03306 (Feb 2026), generation: "plain JSON generation
  shows the best one-shot and final accuracy"; TOON suffers a "prompt tax"
  from in-context instructions; for simple structures constrained-decoding
  JSON used fewer tokens than TOON. **[V]** https://arxiv.org/abs/2603.03306
- Kutschka and Geiger, "Notation Matters", arXiv 2605.29676 (May/June
  2026), agentic tool calling (BFCL, MCPToolBenchPP, MCP-Universe,
  StableToolBench), 5 open-weight models: TOON "up to 18% reduction at a
  similar 9pp accuracy cost", "cascades on multi-turn parsing failures and
  collapses parallel tool-call output for most models"; TRON up to 27%
  reduction, accuracy within 14 pp. **[V]** https://arxiv.org/abs/2605.29676
  . Worst case Mistral-Small-24B on BFCL: 89% JSON to 53% TOON. **[S]**
- IEEE Access paper (Ahaidous, Aarif, Schopplich, Tabaa et al., June 2026,
  DOI 10.1109/ACCESS.2026.3707409), coauthored by TOON's creator: 5 016
  evaluations, TOON 73.9% vs JSON 69.7%, and models "achieving 0% accuracy
  on truncation cases despite perfect metadata availability". ResearchGate
  and IEEE Xplore returned 403/empty, so these numbers are **[S]** only.
- HN thread (178 points, 58 comments): recurring critiques were tokenizer
  dependence, "yaml will still require each key on each object" (defence),
  unquoted `123` vs `"123"` ambiguity, models "lack TOON in training data",
  and that the key question is "do LLMs understand TOON the same as they
  would JSON?". **[V]** https://news.ycombinator.com/item?id=45715632
- Thoughtworks Radar vol. 34 (April 2026) placed TOON in Assess: "last-mile
  optimization", "not a replacement for JSON in APIs, databases or model
  outputs". **[S]**
  https://www.thoughtworks.com/en-br/radar/languages-and-frameworks/toon-token-oriented-object-notation

### 2. JSON and JSONL

- Whitespace alone is a third of pretty JSON's cost: TOON's own table shows
  pretty 4 308 vs compact 2 892 tokens (-33%). **[V]** TOON README. This
  repo measures 1.33x to 1.58x tokens for pretty over compact on flat data
  and up to 2.45x on deep nesting. **[V]** `tokens.md`.
- Repeated keys are the dominant cost in arrays of records. ONTO's
  ablation: "key repetition accounts for the majority of JSON overhead"; its
  motivating case had 1 000 telemetry records costing about 80 000 tokens.
  **[V]** https://arxiv.org/abs/2604.17512
- JSONL does not fix keys. In the improvingagents flat test JSONL scored
  45.0% at 54 407 tokens vs JSON 52.3% at 66 396. **[V]**
  https://www.improvingagents.com/blog/best-input-data-format-for-llms . In
  padolsey's 30-model weval rerun JSONL was 79.85% vs JSON 77.73%. **[S]** HN
  comment.
- JSON wins on generation and familiarity: Matveev (above), and WaveformQA
  (arXiv 2607.20638) found a JSON format beat a more compact domain format
  "despite this token disadvantage", attributing it to pre-training
  prevalence. **[S]**
- Structured Outputs: OpenAI claims "gpt-4o-2024-08-06 with Structured
  Outputs scores a perfect 100%. In comparison, gpt-4-0613 scores less than
  40%" on internal schema-following evals; training alone reached 93%,
  constrained decoding closed the rest. **[S]** (openai.com blocked; wording
  confirmed in multiple secondary sources). Current docs: "the first request
  you make with any schema will have additional latency". **[V]**
  https://developers.openai.com/api/docs/guides/structured-outputs

### 3. CSV and TSV

- Cheapest format in every measurement found: 19 524 tokens vs 66 396 for
  JSON on 1 000 x 8 records (-70.6%). **[V]** improvingagents. TOON's flat
  track: CSV 64 247 vs TOON 68 030 vs pretty JSON about 155k. **[V]** This
  repo: CSV 0.60x JSON tokens on flat_users, TOON 0.65x. **[V]**
- Accuracy is strongly model dependent. GPT-4.1-nano: CSV 44.3%, second
  worst. **[V]** The author stated input size was tuned to land near 50%
  accuracy, and at 100 rows nano scores "95%+" on both CSV and Markdown-KV.
  **[V]** https://news.ycombinator.com/item?id=45458455 . Across 30 models
  CSV was best at 84.25%. **[S]**
- In the TOON benchmark CSV was worst on Aggregation (32.8% vs 48.4%) and
  Filtering (33.3% vs 38.0%). **[V]**
- Alignment loss on long and wide tables is well documented, though not
  CSV-specific: Chen 2023: "After the table size exceeds 1000 tokens (e.g.
  1500 word pieces), GPT-3's performance almost degrades to random guesses",
  "unable to generalize to 'huge' tables with 30+ rows", errors are
  "grounding the inputs to the wrong rows or columns". **[V]**
  https://arxiv.org/html/2210.06710 . Width: GPT-4o 52.75% on narrow (1 to 5
  cols) to 27.91% on wide (>20 cols). **[S]** arXiv 2509.22983 (abstract
  fetched, numbers from search snippet). NeedleInATable (NeurIPS 2025)
  reports a "substantial performance gap" between downstream tasks and
  simple cell lookup. **[V]** https://arxiv.org/abs/2504.06560
- Table Meets LLM (Sui et al., WSDM 2024, SUC benchmark, GPT-3.5): size
  detection 40.67% (Markdown) to 67.00% (HTML); row retrieval 29 to 42%;
  HTML beat NL+Sep by 6.76%; removing the 1-shot example caused the largest
  drops. **[V]** https://arxiv.org/html/2305.13062v4

### 4. Markdown tables and Markdown key-value

- improvingagents (GPT-4.1-nano): Markdown-KV 60.7% at 52 104 tokens (best
  accuracy, 2.7x CSV's tokens); Markdown table 51.9% at 25 140 (second
  cheapest). INI 55.7%, YAML 54.7%, XML 56.0% at 76 114. **[V]**
- Nested data (2025-10-14, three models): Markdown (headings + `key: value`)
  was cheapest for every model (38 357 vs JSON 57 933 on GPT-5 Nano) and
  second in accuracy behind YAML on two of three; Llama 3.2 3B showed no
  significant format effect. **[V]**
  https://www.improvingagents.com/blog/best-nested-data-format/
- SUC: Markdown was the worst format for size detection and column
  retrieval on GPT-3.5. **[V]**
- TableLlama and "Rethinking tabular data understanding" were not fetched;
  no format-specific numbers from them are claimed here. **[H]** that
  fine-tuned table models change the format ranking.

### 5. YAML

- Blog measurements cluster at 15 to 33% fewer tokens than pretty JSON
  (cl100k: 99 vs 66 tokens on a nested config), but near parity with compact
  JSON. **[S]**
  https://dev.to/amareswer/yaml-vs-json-for-llms-real-token-counts-real-gotchas-41o1
- ONTO measured YAML at only 1 to 6% below JSON on record arrays because it
  keeps per-record keys. **[V]** abstract.
- TOON benchmark: YAML 3 487 tokens vs compact JSON 2 892, accuracy 70.1% vs
  69.0%. **[V]**
- YAML won two of three models on nested data (62.1% on GPT-5 Nano vs JSON
  50.3%). **[V]** improvingagents nested.
- Known hazards: boolean coercion of `yes/no/on/off`, invisible indentation
  errors. **[S]**

YAML is a baseline in this repo (own block-style emitter, yaml-rust2
decoder); see Part A item 10 for its measured cost.

### 6. Apache Arrow and Parquet

- Arrow columnar format goals: "Data adjacency for sequential access
  (scans)", "O(1) (constant-time) random access", "SIMD and
  vectorization-friendly", "Relocatable without 'pointer swizzling',
  allowing for true zero-copy access in shared memory"; validity bitmap +
  offsets + data buffers; 64-byte alignment to match AVX-512. **[V]**
  https://arrow.apache.org/docs/format/Columnar.html
- Parquet layout: row groups to column chunks to pages; footer with min/max
  stats, page index, Bloom filters; dictionary, RLE, delta encodings.
  arrow-rs techniques: vectorised decode, dictionary preservation ("in
  excess of 60x" in some cases), projection and predicate pushdown, late
  materialisation, range-coalescing async I/O; "query GBs of Parquet in a
  matter of milliseconds". **[V]**
  https://arrow.apache.org/blog/2022/12/26/querying-parquet-with-millisecond-latency/
  . Parquet 57 decodes footers 3 to 9x faster via a custom Thrift parser.
  **[V]** https://arrow.apache.org/blog/2025/10/23/rust-parquet-metadata/
- Crates: `arrow`, `parquet` (this repo pins 60.0.0; MSRV 1.88,
  Apache-2.0, monthly releases, shared version numbers), `arrow-flight`,
  `parquet_derive`; docs state "Parquet ... focuses on storage efficiency
  whereas Arrow prioritizes compute efficiency". **[V]**
  https://docs.rs/parquet/latest/parquet/ , https://github.com/apache/arrow-rs
- No headline GB/s decode figure exists in official docs. ClickBench
  reports per-query seconds for DataFusion on a 14.7 GB Parquet dataset, not
  throughput. **[S]**
- Why not LLM-facing: binary, length-prefixed, dictionary-indexed bytes have
  no token-level meaning; a model cannot read a validity bitmap. They are
  the right *source* for an LLM encoder, not the wire format.

### 7. Other LLM-oriented formats

- **TRON** (Tim Huang, MIT, JSON superset): `class Point: x, y` then
  `Point(10, 20)`; nesting `User(1, Profile("Alice", ...))`; class
  inheritance `class Point3D(Point): z`; arrays keep brackets. **[V]**
  https://raw.githubusercontent.com/tron-format/tron-format.github.io/main/SPEC.md
  . Up to 27% savings but up to -14 pp accuracy in agentic tests. **[V]**
  arXiv 2605.29676.
- **JTON / Zen Grid** (arXiv 2604.05865): strict JSON superset, header row
  + semicolon rows; 15 to 60% (avg 28.5%) vs compact JSON; +0.3 pp
  comprehension over 10 LLMs; 100% syntactic validity in generation over 12
  LLMs; Rust/PyO3 parser 1.4x Python json. **[V]**
- **ONTO** (arXiv 2604.17512): fields declared once per entity, pipe rows,
  indentation for hierarchy; 46 to 51% vs JSON; 5 to 10% latency gain on
  Qwen2.5-7B; "no material degradation" *when format context was provided*.
  **[V]**
- **Anthropic guidance**: "XML tags help Claude parse complex prompts
  unambiguously"; nest `<documents><document index="n">` with `<source>` and
  `<document_content>`; put long documents at the top. **[V]**
  https://platform.claude.com/docs/en/build-with-claude/prompt-engineering/claude-prompting-best-practices
- CSON, HJSON, JSON5: no LLM measurements found. "Reading Between the Curly
  Braces" (arXiv 2607.26211, N=215 humans) found HJSON/YAML advantages that
  "vanished in more realistic settings"; nothing about LLMs. **[V]**
- No format found is designed around tokenizer pre-tokenisation boundaries;
  all optimise empirically on character or token counts. **[V]** by absence.

### 8. Tokenizer facts

- tiktoken `openai_public.py`: cl100k_base pattern contains `\p{N}{1,3}`;
  o200k_base is seven alternatives including `\p{N}{1,3}`,
  ` ?[^\s\p{L}\p{N}]+[\r\n/]*`, `\s*[\r\n]+`, `\s+(?!\S)`, `\s+`. Digits
  chunk left to right in groups of at most three, with no leading-space
  option, unlike letters. `<|endoftext|>` is 100257 (cl100k) and 199999
  (o200k); o200k_harmony also exists. **[V]**
  https://raw.githubusercontent.com/openai/tiktoken/main/tiktoken_ext/openai_public.py
- Consequence: punctuation runs like `","` or `"},{"` can merge into one
  token; a leading space attaches to the following word; `1000000` becomes
  `100|000|0`. **[V]** from the regex; specific merge tables not checked.
- Anthropic: no public tokenizer for Claude 3+; the
  `/v1/messages/count_tokens` endpoint is free (5 000 to 20 000 RPM by
  tier), supports all active models, and its count "is an **estimate**".
  Claude Opus 4.7 and later (including Fable/Mythos 5.x) use a newer
  tokenizer producing "approximately 30 percent more tokens" than earlier
  models. **[V]**
  https://platform.claude.com/docs/en/docs/build-with-claude/token-counting
  . Anthropic's own skills repo: tiktoken "undercounts Claude tokens by
  ~15–20% on typical text, and by much more on code or non-English input".
  **[S]**
  https://github.com/anthropics/skills/blob/main/skills/claude-api/shared/token-counting.md
  . An empirical rerun found undercounting of about 27% on JSON and
  lockfiles. **[S]**
  https://dev.to/ji_ai/tiktoken-undercounts-claude-tokens-37-of-418-requests-blew-the-limit-46bd

### 9. Do token savings buy or cost accuracy?

- Within one tokenizer and model, compaction is roughly accuracy-neutral on
  retrieval when CIs are honoured (TOON README, JTON +0.3 pp, ONTO "no
  material degradation", McMillan p=0.484). **[V]**
- Compaction hurts when: the model is small (GPT-4.1-nano: CSV and TOON near
  the bottom; Llama 3B indifferent), the task is generation or multi-turn
  tool calling (Notation Matters: -9 to -14 pp; Matveev: JSON best), or the
  data is nested (TOON last on nested, 43.1%). **[V]**
- Compaction can *raise total* tokens in agentic retrieval because
  unfamiliar formats trigger more search and retries (McMillan). **[V]**
- Model strength dominates format: a 21 pp tier gap (McMillan); GPT-5 100%
  on CSV, JSON and Markdown-KV where nano scored 44 to 61% (HN rerun).
  **[V]/[S]**

---

## Limitations worth attacking

1. **Repeated keys** remain in JSON, JSONL, YAML; the fix (header once) is
   proven at 40 to 60% savings. Any new format must at least match CSV on
   flat data; TOON is +5.9% over CSV in its own benchmark and +8% in ours.
2. **Compact formats lose to compact JSON on nested or non-uniform data**
   (TOON +1.6% overall and +19.9% on event logs in its README; +8% to +82%
   tokens on our nested, sparse, mixed and deep datasets). A format that
   degrades gracefully to compact-JSON cost on irregular data, without a
   mode switch, has no competitor.
3. **Row and column grounding errors** grow with row count (random-guess
   territory past about 1 000 tokens for GPT-3-class models; width effect
   52.75% to 27.91%). No format tested re-anchors the reader: there is no
   periodic header repetition, no row index, no column-count check within
   the body. Measurable target: accuracy on row 900 of 1 000 vs row 10.
4. **Length metadata is ignored by models**: 0% truncation detection
   despite `[N]` (IEEE Access, [S]). Validation metadata should be
   positioned and phrased so a model uses it, or dropped as dead weight.
5. **Prompt tax**: every new syntax needs in-context explanation that costs
   more than the savings on small payloads (Matveev). A format that is
   self-explanatory from training-distribution primitives (CSV header,
   Markdown, `key: value`) avoids this.
6. **Tokenizer-blind design**: delimiter and whitespace choices are never
   justified against the `o200k` regex or Claude counts, and Claude counts
   differ from tiktoken by 15 to 30% and shifted 30% between Claude
   generations. Part A item 4 shows bytes-per-token varies by 40% across
   formats on the same data. A format should publish counts from both
   `o200k_base` and `count_tokens` for every claim.
7. **Generation and round-trip**: TOON cascades on multi-turn parsing and
   parallel tool calls. The output side is mostly unaddressed; constrained
   decoding exists only for JSON.
8. **Type ambiguity**: unquoted `123` vs `"123"`, YAML booleans, and the
   integer-vs-double ambiguity of canonical number text above 2^53 (Part A
   item 6). TOON's quoting rules have about ten triggers. Simpler, lossless
   typing is a measurable correctness win.
9. **Missing vs null**: CSV-style tables cannot express it; TOON gives up
   tabular form entirely for sparse records (Part A item 5). A cheap
   explicit "absent" marker inside a tabular row is unexplored.
10. **Streaming**: TOON needs the record count and a whole-array schema
    decision before the first byte (Part A item 2). GB-scale pipelines need
    one-pass encoding with bounded memory.
11. **Columnar sources**: nobody streams Arrow/Parquet batches straight into
    an LLM-facing encoding with projection and predicate pushdown applied
    first; the cheapest token is the column you never serialise.
12. **Benchmark hygiene**: official TOON numbers come from the author; the
    independent tests disagree in direction. A neutral harness with
    per-model, per-row-count, per-width curves is itself a gap. This repo's
    `ldf-eval` is a start; it has not been run against a model yet.

## Open questions to settle with benchmarks

1. At what row count and column count does each format's lookup accuracy
   fall below 95% per model tier (nano/flash class vs frontier)? Fix the
   format, sweep size, as the HN critics asked.
2. Does repeating the header every K rows, or adding a row ordinal, restore
   accuracy on long tables, and what is the token cost of K?
3. Does a model ever act on `[N]` or field counts? Test truncation detection
   with the marker before vs after the data, and with vs without an
   instruction to check it.
4. Aggregation and filtering (38 to 48% in TOON's test) are where every
   format fails. Is that format-sensitive at all, or purely a reasoning
   limit?
5. What does the same payload cost under `o200k_base`, `cl100k_base`,
   Claude pre-4.7 and Claude 4.7+ tokenizers? Do pipe, tab, comma and
   semicolon differ in merge behaviour? (`ldf-bench tokens` already shows
   tab vs comma TOON differing by up to 9% on unicode-heavy data.)
6. Prompt tax break-even: at how many records does a new syntax plus its
   explanation become cheaper than compact JSON with zero explanation, per
   model?
7. Generation: can a model emit the format with 100% validity zero-shot, as
   JTON claims, and does it survive multi-turn tool loops where TOON
   cascades?
8. Does a format that mirrors a columnar source (one block per column
   instead of per row) help or hurt retrieval and aggregation? Nobody has
   tested column-major text.
9. Is nested-data accuracy driven by the format or by depth? improvingagents
   calibrated depth to 6 to 7 levels; a depth sweep per format is missing.
10. Does "format context" (a one-paragraph description) change accuracy for
    CSV and JSON as much as it does for new formats? If yes, prior
    comparisons are confounded.

## Sources

- https://github.com/toon-format/toon
- https://github.com/toon-format/spec
- https://raw.githubusercontent.com/toon-format/spec/main/SPEC.md
- https://github.com/toon-format/spec/blob/main/CHANGELOG.md
- https://toonformat.dev/ and https://toonformat.dev/ecosystem/implementations.html
- https://www.npmjs.com/package/@toon-format/toon
- https://en.wikipedia.org/wiki/Token-Oriented_Object_Notation
- https://news.ycombinator.com/item?id=45715632
- https://news.ycombinator.com/item?id=45458455
- https://www.improvingagents.com/blog/best-input-data-format-for-llms
- https://www.improvingagents.com/blog/best-nested-data-format/
- https://www.improvingagents.com/blog/toon-benchmarks
- https://dev.to/ikaganacar/toon-benchmarks-a-critical-analysis-of-different-results-5h66
- https://usewire.io/blog/toon-vs-json-why-smaller-doesnt-mean-cheaper/
- https://arxiv.org/abs/2602.05447 (McMillan)
- https://arxiv.org/abs/2603.03306 (Matveev)
- https://arxiv.org/abs/2605.29676 (Notation Matters)
- https://arxiv.org/abs/2604.05865 (JTON)
- https://arxiv.org/abs/2604.17512 (ONTO)
- https://arxiv.org/abs/2607.26211 (Reading Between the Curly Braces)
- https://arxiv.org/html/2305.13062v4 (Table Meets LLM)
- https://arxiv.org/html/2210.06710 (Chen 2023)
- https://arxiv.org/abs/2504.06560 (NeedleInATable)
- https://arxiv.org/abs/2509.22983 (Same Content, Different Representations)
- https://www.researchgate.net/publication/408101928 (IEEE Access TOON paper, 403, not read)
- https://www.thoughtworks.com/en-br/radar/languages-and-frameworks/toon-token-oriented-object-notation
- https://www.piotr-sikora.com/blog/2025-12-05-toon-tron-csv-yaml-json-format-comparison
- https://raw.githubusercontent.com/tron-format/tron-format.github.io/main/SPEC.md
- https://dev.to/amareswer/yaml-vs-json-for-llms-real-token-counts-real-gotchas-41o1
- https://arrow.apache.org/docs/format/Columnar.html
- https://arrow.apache.org/blog/2022/12/26/querying-parquet-with-millisecond-latency/
- https://arrow.apache.org/blog/2025/10/23/rust-parquet-metadata/
- https://docs.rs/parquet/latest/parquet/ and https://github.com/apache/arrow-rs
- https://raw.githubusercontent.com/openai/tiktoken/main/tiktoken_ext/openai_public.py
- https://platform.claude.com/docs/en/docs/build-with-claude/token-counting
- https://platform.claude.com/docs/en/build-with-claude/prompt-engineering/claude-prompting-best-practices
- https://github.com/anthropics/skills/blob/main/skills/claude-api/shared/token-counting.md
- https://dev.to/ji_ai/tiktoken-undercounts-claude-tokens-37-of-418-requests-blew-the-limit-46bd
- https://developers.openai.com/api/docs/guides/structured-outputs
- https://openai.com/index/introducing-structured-outputs-in-the-api/ (blocked, wording confirmed via secondary sources)

Not reached: ResearchGate and IEEE Xplore (IEEE Access TOON paper), the
openai.com announcement, the tron-format.github.io landing page (SPEC.md
fetched instead), TableLlama and "Rethinking tabular data understanding".
