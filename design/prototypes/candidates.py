#!/usr/bin/env python3
"""Throwaway encoders for LDF design candidates.

Reads a JSON array of records and writes one text file per candidate variant
so the real tokenizer can price each layout on the same data. Nothing here is
a decoder or a spec; it exists to compare shapes before anything is built in
Rust. Standard library only.

    python3 design/prototypes/candidates.py eval/out/flat_users/data/flat_users.json out/

Families:
  T  typed table: one header per schema, delimiter-separated rows, no
     quotes unless needed, types in the header so "42" and 42 differ.
     nested=inline  arrays/objects in a cell as compact JSON
     nested=child   nested objects flattened with dotted paths, arrays of
                    objects as child tables keyed by the parent's first field
  H  hoisted-key JSON: header is a JSON array of keys, rows are JSON arrays.
     Pure JSON tokens, nested values inline. Sparse rows fall back to objects
     (strict) or use `-` for a missing cell (union).
  L  logfmt rows: only present keys per row as key=value, types in a header.
"""

import json
import os
import sys

# ---- helpers -----------------------------------------------------------------


def is_scalar(v):
    return v is None or isinstance(v, (bool, int, float, str))


def type_code(values):
    kinds = set()
    for v in values:
        if v is None:
            continue
        if isinstance(v, bool):
            kinds.add("b")
        elif isinstance(v, int):
            kinds.add("i")
        elif isinstance(v, float):
            kinds.add("f")
        elif isinstance(v, str):
            kinds.add("s")
        else:
            kinds.add("j")
    if not kinds:
        return "s"
    if kinds == {"i", "f"}:
        return "f"
    if len(kinds) == 1:
        return kinds.pop()
    return "v"  # variant: cells carry JSON literals


def fmt_num(v):
    if isinstance(v, bool):
        return "true" if v else "false"
    if isinstance(v, float):
        if v != v or v in (float("inf"), float("-inf")):
            return "null"
        r = repr(v)
        return r[:-2] if r.endswith(".0") else r
    return str(v)


def cell_typed(v, code, delim):
    """Cell text when the column type is known from the header."""
    if v is None:
        return "null"
    if code == "s":
        return quote_if_needed(v, delim)
    if code in ("i", "f", "b"):
        return fmt_num(v)
    # variant / json column: JSON literal, strings quoted
    return json.dumps(v, ensure_ascii=False, separators=(",", ":"))


def quote_if_needed(s, delim):
    needs = (
        s == ""
        or s == "null"
        or s[0] in " \t\""
        or s[-1] in " \t"
        or delim in s
        or "\n" in s
        or "\r" in s
        or "\\" in s
    )
    return json.dumps(s, ensure_ascii=False) if needs else s


def flatten(obj, prefix=""):
    """Dotted-path leaves; arrays stay as values (handled by the caller)."""
    out = []
    for k, v in obj.items():
        p = f"{prefix}.{k}" if prefix else k
        if isinstance(v, dict) and v:
            out.extend(flatten(v, p))
        else:
            out.append((p, v))
    return out


# ---- family T: typed table -----------------------------------------------------


def encode_T(records, delim, nested, name, types=True, missing=""):
    lines = []
    if nested == "child":
        rows = [dict(flatten(r)) for r in records]
    else:
        rows = records
    # Schema groups: consecutive records with the same key sequence share a
    # header (streaming friendly: the header is re-emitted on change).
    # For sparse data use the union so one header covers everything.
    union = []
    for r in rows:
        for k in r:
            if k not in union:
                union.append(k)
    key_field = union[0]
    child_tables = {}  # (table name) -> (columns, rows)

    def header(cols, table):
        parts = []
        for c in cols:
            if types:
                code = type_code([r.get(c) for r in rows if c in r]) if table == name else child_tables[table]["types"][c]
                parts.append(f"{c}:{code}")
            else:
                parts.append(c)
        return f"#{table} " + delim.join(parts)

    col_types = {c: type_code([r.get(c) for r in rows if c in r]) for c in union}
    if nested == "child":
        # Columns whose values are arrays of objects become child tables.
        for c in list(union):
            vals = [r[c] for r in rows if c in r and r[c] is not None]
            if vals and all(isinstance(v, list) and v and all(isinstance(x, dict) for x in v) for v in vals):
                ccols = []
                crows = []
                for r in rows:
                    for item in r.get(c) or []:
                        flat = dict(flatten(item))
                        for k in flat:
                            if k not in ccols:
                                ccols.append(k)
                        flat = {key_field: r[key_field], **flat}
                        crows.append(flat)
                ctypes = {k: type_code([x.get(k) for x in crows if k in x]) for k in [key_field] + ccols}
                child_tables[f"{name}.{c}"] = {"cols": [key_field] + ccols, "rows": crows, "types": ctypes}
                union.remove(c)
    lines.append(header(union, name) if types else f"#{name} " + delim.join(union))
    for r in rows:
        cells = []
        for c in union:
            if c not in r:
                cells.append(missing)
            else:
                cells.append(cell_typed(r[c], col_types[c] if types else "v", delim))
        lines.append(delim.join(cells))
    for tname, t in child_tables.items():
        lines.append(f"#{tname} " + delim.join(f"{c}:{t['types'][c]}" if types else c for c in t["cols"]))
        for r in t["rows"]:
            lines.append(delim.join(cell_typed(r[c], t["types"][c], delim) if c in r else missing for c in t["cols"]))
    return "\n".join(lines) + "\n"


# ---- family H: hoisted-key JSON --------------------------------------------------


def encode_H(records, mode):
    dumps = lambda v: json.dumps(v, ensure_ascii=False, separators=(",", ":"))
    union = []
    for r in records:
        for k in r:
            if k not in union:
                union.append(k)
    lines = [dumps(union)]
    for r in records:
        if mode == "strict" and (list(r.keys()) != union):
            lines.append(dumps(r))
        else:
            lines.append("[" + ",".join("-" if k not in r else dumps(r[k]) for k in union) + "]")
    return "\n".join(lines) + "\n"


# ---- family L: logfmt rows ---------------------------------------------------------


def encode_L(records, types=True):
    union = []
    for r in records:
        for k in r:
            if k not in union:
                union.append(k)
    col_types = {c: type_code([r.get(c) for r in records if c in r]) for c in union}
    lines = []
    if types:
        lines.append("#types " + " ".join(f"{c}:{col_types[c]}" for c in union))
    for r in records:
        parts = []
        for k, v in r.items():
            if is_scalar(v):
                txt = cell_typed(v, col_types[k] if types else "v", " ")
            else:
                txt = json.dumps(v, ensure_ascii=False, separators=(",", ":"))
            parts.append(f"{k}={txt}")
        lines.append(" ".join(parts))
    return "\n".join(lines) + "\n"


# ---- main --------------------------------------------------------------------------


def main():
    src, out = sys.argv[1], sys.argv[2]
    name = os.path.splitext(os.path.basename(src))[0]
    records = json.load(open(src, encoding="utf-8"))
    os.makedirs(out, exist_ok=True)
    variants = {
        "T-pipe-inline": lambda: encode_T(records, "|", "inline", name),
        "T-comma-inline": lambda: encode_T(records, ",", "inline", name),
        "T-tab-inline": lambda: encode_T(records, "\t", "inline", name),
        "T-pipe-child": lambda: encode_T(records, "|", "child", name),
        "T-comma-child": lambda: encode_T(records, ",", "child", name),
        "T-comma-child-untyped": lambda: encode_T(records, ",", "child", name, types=False),
        "H-union": lambda: encode_H(records, "union"),
        "H-strict": lambda: encode_H(records, "strict"),
        "L-typed": lambda: encode_L(records, True),
        "L-untyped": lambda: encode_L(records, False),
    }
    for vname, fn in variants.items():
        path = os.path.join(out, f"{name}.{vname}.txt")
        with open(path, "w", encoding="utf-8") as f:
            f.write(fn())
        print(path)


if __name__ == "__main__":
    main()
