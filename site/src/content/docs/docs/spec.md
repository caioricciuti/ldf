---
title: "Specification (v0.2)"
description: "Every rule of the format, checked by the reference implementation's tests."
---

Language Data Format: a lossless, line-oriented, token-efficient text
encoding of JSON records for language-model consumption. Version 0.2 is
the prototype that `crates/ldf` implements; every rule here is checked by
that implementation's tests. Rationale and measurements:
[design notes](/docs/design/).

Changes from v0.1: the hoisted column type `[k1,k2,...]` (sections 3, 4
and 7, rule 2a), measured in Stage B and Stage C
(`reports/2026-10-18-stage-b`, `reports/2026-10-19-stage-c`).
The change is additive. Every v0.1 document is a valid v0.2 document with
the same meaning, and an encoder writes a hoisted column only when asked
to (`Options::hoist`, off by default), so default output is v0.1. A v0.1
decoder cannot read a hoisted column: it rejects the header as an unknown
type.

Changes from v0 to v0.1: the `~` column and the sparse-key rule (sections
3, 4 and 7.4), adopted after the comprehension measurements in the
baseline report.

## 1. Data model

Same as JSON: null, boolean, number, string, array, object; object key order
is significant; `1` and `1.0` are different values. Numeric domain of the
reference implementation: i64, u64, f64.

A document encodes a **dataset**: an ordered sequence of records. Records are
usually objects; any JSON value is allowed.

## 2. Lines

UTF-8, LF line ends, no BOM. A document is a sequence of **tables**. Each
table is one header line followed by zero or more row lines. Blank lines are
ignored outside tables and forbidden inside a table. There are no comments.

## 3. Header

```text
@name col,col,col
```

`name` is the table path: `records` for the root table, `records.items` for
the child table of field `items`, recursively. Each `col` is
`key:type` or `key:type?`:

| type | cell content |
|---|---|
| `i` | integer in decimal, optional leading `-` |
| `f` | float: shortest round-trip decimal, `.0` kept on integral values, exponent allowed (`1e21`, `1e-7`) |
| `s` | string, bare or JSON-quoted (section 5) |
| `b` | `true` or `false` |
| `j` | any JSON value as a compact JSON literal |
| `t` | child table reference: the number of child rows for this record |
| `[k1,k2,...]` | hoisted array of objects (v0.2): a JSON array with one array of values per element, in key order |

A hoisted type is a key list in brackets instead of a letter, for example
`items:[sku,qty,price]` or `items:[sku,qty]?`. Each key is one segment,
bare or a JSON string literal under the rules for path segments below
(`["a,b","x y"]`); the list is never empty. Commas inside the brackets
separate keys, not columns.

`?` marks a column that may hold `null` or be absent in some rows. It is
informative; decoders accept `null` and empty cells in any column.

Column keys are **paths**: segments joined by `.`. A segment is bare when it
matches `[A-Za-z_][A-Za-z0-9_]*`; otherwise it is a JSON string literal
(`"dotted.key"`, `"with space"`, `""`). Decoders split paths on dots outside
quotes.

A column whose key is the bare character `~` (never a valid bare segment,
so it cannot collide with a real key, which would be quoted as `"~"`) is
the **rest column**: type `j`, at most one per table, by convention last.
Its cell is a JSON object of keys that have no column of their own
(section 7.4).

A header for a table name that already appeared replaces the previous
header for that name (schema change mid-stream). Rows always belong to the
most recent header.

## 4. Rows

A row has exactly as many cells as its header has columns, separated by
commas. Cell meaning:

* empty cell: the key is **absent** from the record;
* `null`: the key is present with value null;
* otherwise the value according to the column type.

Cells of type `j` and hoisted cells are parsed as one JSON value starting
at the cell's first character; the comma that follows the value ends the
cell. Cells of other types end at the first comma outside a JSON-quoted
string.

A hoisted cell is either empty (absent), `null`, or a JSON array whose
items are arrays with exactly as many values as the column has keys. The
decoder turns each item into an object with the column's keys in order:
under `items:[sku,qty]` the cell `[["E5",3],["C3",4]]` is
`[{"sku":"E5","qty":3},{"sku":"C3","qty":4}]`, and `[]` is an empty
array. An item that is not an array, or has the wrong number of values,
is an error.

A `~` cell is either empty (no extra keys) or a JSON object; the decoder
appends its members to the record, in the object's order, after the keys
that came from columns. A value that is not an object is an error.

A row whose table is a child table is attached to a parent row (section 7).

## 5. Strings

A string cell is bare unless it

* is empty, or equals `null`, `true` or `false`;
* starts or ends with a space or tab;
* contains `,`, `"`, `\`, CR, LF or a control character below U+0020;
* is the first cell of a row and starts with `@` (a line starting with `@`
  is a header).

A non-bare string is a JSON string literal (`"..."` with JSON escapes).
Because the column type is `s`, a bare `42`, `1e5`, `-` or `007` is a string.

## 6. Numbers

`i`: the integer's decimal text. `f`: shortest digits that round-trip, as
Rust's `{:?}` and JavaScript's `Number#toString` produce, except that
integral values keep `.0` (`1.0`, `100.0`) so float kind survives. NaN and
infinities are not representable and encode as `null`.

## 7. Nesting

Flattening is decided per field over the whole dataset (batch encoder):

1. **Objects.** A field whose present values are all objects with the same
   key set at every record, and which is null in no record, flattens into
   dotted columns, recursively. If the key sets differ across records, some
   values are non-objects, or some are null (dotted cells cannot tell a null
   object from an absent one), the field is a `j` column. An empty object `{}` is a `j` cell.
2. **Arrays of objects.** A field whose present values are all arrays whose
   elements are all objects becomes a **child table** `parent.field`. The
   parent gets a `t` column holding the element count. Child rows are the
   elements, flattened by the same rules, prefixed with one join column:
   * the parent table's first column (same key and type) when its values
     are unique across the parent table, or
   * `_:i`, the 1-based ordinal of the parent row within its table, when
     they are not.
   Child rows appear in parent order, then element order. A parent whose
   `t` cell is `0` has an empty array; empty cell means absent; `null`
   means null.

   2a. **Hoisted arrays (v0.2, opt-in).** With hoisting on, a field that
   rule 2 would make a child table is instead one hoisted column
   `field:[k1,k2,...]` when every element of every present array has the
   same keys in the same order, `k1,k2,...`, and every value in every
   element is null, a boolean, a number or a string. The cell holds the
   whole array (section 4). Any other shape (key lists that differ in set
   or order, a nested object or array in some element) stays a child
   table. That is the only shape models were measured on (nested_orders).
   Inside a child table the rule applies again, so a child table can have
   hoisted columns of its own. The block encoder applies the rule per
   block; the record encoder never hoists, since it never creates child
   tables either (section 8).
3. **Everything else** (arrays of scalars, mixed arrays, scalars that vary
   in type across records) is a `j` column.
4. **Sparse keys.** Before rules 1 to 3, each top-level key of a table is
   checked for fill: the share of the table's rows in which the key is
   present with a non-null value. A key filled in fewer than the
   threshold share of rows (default 0.5; `0` disables the rule) gets no
   column; the table ends with `~:j?` and every row's rest cell holds the
   sparse keys that row has, with their values as JSON, in record order.
   This applies before flattening and child tables, so a sparse object or
   a sparse array of objects travels inline as JSON inside the rest
   object rather than as dotted columns or a child table. Dense keys are
   unaffected, so a dataset without sparse keys encodes exactly as in v0.
   Rationale: positional cells in mostly-empty rows cost weak models
   accuracy; a key next to its value does not (baseline report, "The
   sparse-key rule, measured").

A record that is not an object is a row of the single column `_:j`.

Decoders rebuild objects from dotted paths in header order, append the
members of the `~` object, and attach child rows by the join column,
inserting the array at the position of the `t` column within the parent's
key order. Key order is therefore preserved exactly only when no key moved
to the rest column; otherwise equality is semantic (key order may differ).

## 8. Streaming encoders

Two encoders bound memory without changing the decoder.

**Block encoder** (the reference implementation's streaming path, 1 000
records per block). Records are buffered up to the block size, the block is
planned with the rules of section 7, and written as: the root header
(omitted only when the previous block ended on the same root header, since
rows belong to the most recent header), the block's root rows, then each
child table with its header and the block's child rows, recursively. Child
tables always join by ordinal (`_:i`), numbered across the whole document,
because a first column that is unique within one block need not be unique
in the next. Since a decoder concatenates the rows of every table with the
same name and attaches children by join value, a block-interleaved
document decodes exactly like a batch one. On uniform flat data the output is
the batch output; with child tables it is the batch output with ordinal
joins and one root and one child header per block; when a block's plan
differs (a type widens, a key appears, a sparse key crosses
the threshold) the new root header applies from that block on.

**Record encoder.** Emits a header when the next record's key set differs
from the current header, then one row per record. It never creates child
tables (arrays of objects are `j` cells) and never flattens objects (nested
objects are `j` cells), because both decisions need more than one record.
It exists for the case where even a block is too much state.

Both produce valid LDF that decodes with the same decoder.

## 9. Example

```text
@records order_id:s,customer.id:i,customer.name:s,customer.address.city:s,items:t,total:f,status:s,note:s?
ORD-000001,4654,Val Thompson,São Paulo,2,715.94,shipped,context compare
ORD-000002,50,Sol Ritchie,Buenos Aires,1,2757.03,cancelled,null
@records.items order_id:s,sku:s,qty:i,price:f
ORD-000001,E5,3,28.78
ORD-000001,C3,4,157.4
ORD-000002,A1,1,2757.03
```

The same records with hoisting on. On 200 held-out nested_orders records
this form is 15 percent smaller than the child-table form in o200k tokens
(11 076 against 12 989):

```text
@records order_id:s,customer.id:i,customer.name:s,customer.address.city:s,items:[sku,qty,price],total:f,status:s,note:s?
ORD-000001,4654,Val Thompson,São Paulo,[["E5",3,28.78],["C3",4,157.4]],715.94,shipped,context compare
ORD-000002,50,Sol Ritchie,Buenos Aires,[["A1",1,2757.03]],2757.03,cancelled,null
```

## 10. Open for v1

Whether child tables should always join by ordinal: on the nested_orders
fixture the block encoder's ordinal joins are 11 percent smaller than the
batch encoder's `order_id` joins, because an ordinal is shorter than most
keys. A tab delimiter variant; column statistics in headers; a decoder option to type `j` cells lazily;
whether a single-row parent should join its child table by ordinal rather
than by its first column.
