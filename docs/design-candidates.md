# LDF design candidates

Status: proposal for discussion, 2026-10-17. Nothing here is implemented in
`crates/ldf`. Every number is measured with `o200k_base` on the same three
50-record datasets the comprehension runs used, via throwaway encoders in
`design/prototypes/candidates.py` and `ldf-bench count`. To regenerate:

```bash
for b in flat_users nested_orders sparse_events; do python3 -I design/prototypes/candidates.py eval/out/$b/data/$b.json results/design; done
./target/release/ldf-bench count --tokenizers o200k_base results/design/*.txt
```

## 1. Targets, from the measurements

From `docs/reports/2026-10-17-baseline/` and `docs/research.md`:

| target | evidence |
|---|---|
| Match CSV on flat uniform data | CSV is 0.60x compact JSON tokens on flat_users; TOON is 0.65x; nothing beats CSV there. |
| Beat compact JSON on nested, sparse and mixed data | Every compact format today costs more than compact JSON there: TOON 1.08x nested, 1.22x sparse, 1.33x mixed. This is the empty quadrant. |
| Lossless | Every non-JSON baseline except YAML loses something: CSV merges missing and null and retypes `"42"`; TOON drops float kind and misreads integers above 2^53; YAML pays 1.07x to 1.34x JSON tokens for being lossless. |
| One-pass streaming | TOON needs the record count and a whole-array schema decision before the first byte. JSONL and CSV stream. |
| Little or no prompt tax | Both frontier models read CSV, Markdown and TOON zero-shot without errors. Familiar primitives (header row, `key: value`, dotted paths) cost nothing to explain. |
| Tokenizer aware | Bytes per token differ by 40 percent between formats on the same data. Punctuation runs merge into cheap tokens; indentation and newlines do not. |

Accuracy at 50 records cannot rank formats (both models at ceiling), so the
design is driven by tokens and losslessness, with accuracy to be re-checked on
larger samples and weaker models once a candidate exists.

## 2. Candidates measured

Three families, several variants each. Same records, same tokenizer.

**T, typed table.** One header line per schema: `#name col:type,col:type,...`.
Rows are delimiter-separated cells. Types in the header (`i` int, `f` float,
`s` string, `b` bool, `j` json value, `v` variant) mean a string cell never
needs quotes to be a string, and `42` in an `s` column is the string `"42"`.
An empty cell is *missing*, `null` is null. Nested objects become dotted
columns (`customer.address.city`). Arrays of objects become **child tables**
keyed by the parent's first field (`#orders.items order_id,sku,qty,price`),
or, in the *inline* variant, a compact JSON cell.

**H, hoisted-key JSON.** First line is a JSON array of keys, each row a JSON
array of values; everything is JSON tokens. Sparse rows either fall back to
JSON objects (*strict*) or use `-` for a missing cell (*union*).

**L, logfmt rows.** `key=value` pairs, only present keys per row, types
declared once. The natural shape for sparse data.

### Tokens (o200k_base, 50 records)

| encoding | flat_users | vs json | nested_orders | vs json | sparse_events | vs json |
|---|---:|---:|---:|---:|---:|---:|
| json (compact) | 2 346 | 1.00 | 5 068 | 1.00 | 2 722 | 1.00 |
| csv | 1 433 | 0.61 | unsupported | | 1 693 (lossy) | 0.62 |
| toon | 1 536 | 0.65 | 5 440 | 1.07 | 3 331 | 1.22 |
| yaml | 2 925 | 1.25 | 6 368 | 1.26 | 3 267 | 1.20 |
| **T comma, child tables** | **1 444** | **0.62** | **3 212** | **0.63** | **1 778** | **0.65** |
| T tab, inline | 1 437 | 0.61 | 4 264 | 0.84 | 1 775 | 0.65 |
| T comma, inline | 1 444 | 0.62 | 4 263 | 0.84 | 1 778 | 0.65 |
| T pipe, child tables | 1 521 | 0.65 | 3 423 | 0.68 | 1 788 | 0.66 |
| T comma, child, untyped header | 1 461 | 0.62 | 3 334 | 0.66 | 1 896 | 0.70 |
| H union | 1 511 | 0.64 | 4 369 | 0.86 | 2 144 | 0.79 |
| H strict | 1 511 | 0.64 | 4 369 | 0.86 | 2 807 | 1.03 |
| L typed | 2 010 | 0.86 | 4 666 | 0.92 | 2 267 | 0.83 |

Bytes per token: T comma 2.17 (flat), 2.18 (nested), 2.13 (sparse). JSON
2.95, 2.90, 2.69. The table shape tokenizes as well as CSV because it *is*
CSV with a smarter header.

### What the numbers say

1. **A typed header is free.** Typed T costs 1 444 tokens on flat_users,
   untyped 1 461. The type codes cost about 15 tokens once; without them,
   strings that look like numbers or contain the delimiter must be quoted
   on every row, which costs more. Types also buy losslessness.
2. **Child tables are the win on nested data.** Relational splitting of
   `items` takes nested_orders from 4 263 (inline JSON cells) to 3 212, a
   37 percent saving against compact JSON where TOON loses 7 percent.
   Dotted paths handle the nested `customer` object at zero cost.
3. **Sparse data is fine as a union table.** Empty cells tokenize as runs
   of commas (`,,,,,,` is one or two tokens). T comma on sparse_events is
   1 778 against CSV's 1 693, and the 5 percent gap is exactly the
   losslessness: `null` cells and the typed header. Logfmt, the "natural"
   sparse shape, is 27 percent worse than the table.
4. **Comma beats pipe for GPT's tokenizer.** Pipe costs 5 percent more on
   flat and 7 percent more on nested data. Tab ties with comma. This is the
   opposite of the TOON community's default assumption; it needs checking
   on Claude's tokenizer (`count_tokens`), which is cheap to do.
5. **Hoisted-key JSON is the safe middle** (0.64 / 0.86 / 0.79) and
   degrades to compact JSON by construction, but it never beats T anywhere
   and its quoting costs 0.4 bytes per token.

## 3. Recommendation

Prototype **T, comma, typed header, dotted paths, child tables** as LDF
version 0, with these properties made explicit in the spec:

- **Streaming**: a header line is emitted whenever the schema of the next
  record differs from the current one (first record, or a schema change
  mid-stream). No counts anywhere. A reader keeps one current header per
  table name.
- **Missing vs null**: empty cell is absent, `null` is null. Reversible
  because the header lists every column.
- **Strings**: quoted (JSON string syntax) only when the cell is empty,
  equals `null`, starts or ends with whitespace, or contains the delimiter,
  a quote, a backslash or a newline. The column type decides what a bare
  cell is, so `007` in an `s` column is a string and `7` in an `f` column is
  a float.
- **Numbers**: `i` cells use integer text, `f` cells use shortest
  round-trip text and keep `.0` on integral values, so float kind survives.
  Integers above 2^53 in an `f` column are a documented approximation, as in
  JSON.
- **Nesting**: objects flatten to dotted columns; arrays of objects become
  child tables named `parent.field`, with the parent's first column
  prepended as the join key; arrays of scalars and anything heterogeneous
  are a `j` column holding compact JSON. The worst case is therefore
  "compact JSON inside a cell", never worse than JSON.
- **Keys** containing `.`, `,`, `:` or whitespace are quoted in the header
  with JSON string syntax; everything else is bare.
- **Root**: a document is one or more tables. A single record is a table
  with one row. A root scalar is out of scope.

Expected cost against the baselines: within 1 to 5 percent of CSV on flat
and sparse data, 35 to 37 percent below compact JSON on nested and sparse
data, lossless, streamable.

## 4. Decisions to make together

1. **Delimiter**: comma (best on o200k, familiar, needs quoting of commas
   in text) or tab (ties on o200k, almost never needs quoting, invisible in
   chat UIs). Measure both on Claude's tokenizer before deciding.
2. **Child-table linkage**: parent's first column (readable, repeats a
   value per child row) or a row ordinal (shorter, less readable). Measured
   cost of the repeated key on nested_orders is about 230 tokens of 3 212.
3. **Interleaving for streaming**: emit child rows after each parent block
   with headers re-emitted, or tag each row with its table (`items: ...`),
   or require readers to buffer parents until the stream ends. Each has a
   token and a memory cost; needs a measurement on a larger nested set.
4. **Type codes**: single letters (`i f s b j v`) or words (`int float str
   bool json any`). Letters are cheaper; words explain themselves. Prompt
   tax versus token cost, decidable with one comprehension run.
5. **Nullable marker in the header** (`country:s?`): costs a token per
   optional column, tells the reader which columns may be absent. Probably
   worth it on sparse data; measurable.
6. **Name**: file extension and the header sigil (`#` collides with comment
   conventions; `@` or `=` are alternatives).

## 5. What a prototype needs before it counts

- An encoder and a decoder in `crates/ldf` behind the existing `Format`
  trait, so it enters every table automatically.
- Round-trip at the `byte` level on all twelve datasets, or a documented
  reason per exception.
- Token tables regenerated; a comprehension run against the same bundles,
  same prompts, both models, plus the 200 and 500 record bundles where CSV
  should start to lose alignment.
- A conformance fixture set in the TOON style, written before the second
  implementation.
