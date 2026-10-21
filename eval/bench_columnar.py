"""Columnar MCP results: original JSON, minified JSON and LDF, in one tokenizer.

Runs a set of queries against MotherDuck's DuckDB MCP server (pinned, as in
`reports/2026-10-21-duckdb`) and, for each result text the server sends:

- `sent`: the text as the server sent it (pretty-printed JSON);
- `minified`: the same JSON without whitespace. This is what `ldf-mcp proxy`
  already sends for JSON it keeps, so it is the baseline LDF must beat;
  whitespace savings belong to this column, never to LDF;
- `frame`: `ldf-mcp convert --columnar`, the text `ldf-mcp proxy --columnar`
  sends, or the reason it keeps JSON. Every frame is decoded back with
  `convert --columnar --decode` and compared to the sent JSON here, member
  order and number kinds included (an independent check of the Rust one);
- for results with NULLs, the rows table in four layouts (spec untouched,
  all valid LDF): nulls kept at sparse threshold 0.5 (the default, `~`
  holds `"key":null`) and at 0 (no `~`, `null` cells); nulls dropped at 0.5
  and at 0. Dropping is lossless only inside a frame, where every column is
  known to exist on every row, so absent means NULL. Models were never
  measured on the dropped layouts.

Counts are o200k_base (`ldf-bench count`): exact for GPT, a proxy for
Claude. No model is called. Standard library only; the server runs through
uvx with nothing published after the pinned date.

    python3 -I eval/bench_columnar.py --db results/columnar/fixtures.duckdb --out results/columnar
"""
import argparse, json, os, subprocess, sys, tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from bench_mcp import DUCKDB_SERVER, Server  # noqa: E402

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BENCH = os.path.join(REPO, 'target/release/ldf-bench')
MCP = os.path.join(REPO, 'target/release/ldf-mcp')

TABLES = [('flat_users', 'id'), ('sparse_events', 'event_id'), ('nested_orders', 'order_id'),
          ('repetitive_logs', 'ts, service'), ('wide_table', 'id'), ('mixed_types', 'id'),
          ('numeric_mix', 'id'), ('unicode_strings', 'id'), ('high_card_strings', 'id'),
          ('long_text', 'id'), ('deep_nesting', 'id'), ('empty_values', 'id')]
SIZES = [50, 200, 1000]
AGGREGATES = [
    ('users_by_country', 'SELECT country, count(*) AS n, round(avg(score), 2) AS avg_score FROM flat_users GROUP BY country ORDER BY n DESC, country'),
    ('logs_by_level', 'SELECT level, service, count(*) AS n, max(latency_ms) AS max_latency FROM repetitive_logs GROUP BY ALL ORDER BY n DESC, level, service'),
    ('orders_by_status', 'SELECT status, count(*) AS n, round(sum(total), 2) AS revenue, count(note) AS with_note FROM nested_orders GROUP BY status ORDER BY status'),
    ('events_nulls', 'SELECT type, count(*) AS n, count(referrer) AS referrer, count(amount) AS amount, count(error_code) AS error_code FROM sparse_events GROUP BY type ORDER BY type'),
    ('orders_items_join', 'SELECT o.order_id, i.sku, i.qty, i.price FROM nested_orders o, unnest(o.items) AS u(i) ORDER BY o.order_id, i.sku LIMIT 200'),
    ('events_sparse_cols', 'SELECT event_id, referrer, amount, currency, error_code FROM sparse_events ORDER BY event_id LIMIT 200'),
]


def o200k(text, tmp):
    p = os.path.join(tmp, 'count.txt')
    with open(p, 'w', encoding='utf-8') as f:
        f.write(text)
    out = subprocess.run([BENCH, 'count', p], capture_output=True, text=True, check=True).stdout
    return int(out.strip().splitlines()[-1].split('|')[3])


def run(argv, text):
    p = subprocess.run(argv, input=text, capture_output=True, text=True)
    return p.returncode, p.stdout, p.stderr.strip()


class Obj(list):
    """A JSON object as its ordered members."""


def load(text):
    return json.loads(text, object_pairs_hook=Obj)


def strict(a, b):
    """Equal values, member order and number kinds (1 is not 1.0, true is not 1)."""
    if type(a) is not type(b):
        return False
    if isinstance(a, Obj):
        return len(a) == len(b) and all(ka == kb and strict(va, vb) for (ka, va), (kb, vb) in zip(a, b))
    if isinstance(a, list):
        return len(a) == len(b) and all(strict(x, y) for x, y in zip(a, b))
    return a == b


def minify(text):
    return json.dumps(json.loads(text), separators=(',', ':'), ensure_ascii=False)


def layouts(doc, head, tmp):
    """The rows table in the four null layouts, each with the frame's head line."""
    cols = doc['columns']
    keep = [dict(zip(cols, r)) for r in doc['rows']]
    drop = [{k: v for k, v in zip(cols, r) if v is not None} for r in doc['rows']]
    out = {}
    for name, recs in [('kept', keep), ('dropped', drop)]:
        path = os.path.join(tmp, 'records.json')
        with open(path, 'w', encoding='utf-8') as f:
            json.dump(recs, f, separators=(',', ':'), ensure_ascii=False)
        for t in ('0.5', '0'):
            code, table, err = run([MCP, 'convert', path, '--sparse', t], '')
            out[f'{name}@{t}'] = o200k(head + '\n' + table, tmp) if code == 0 else None
    return out


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('--db', required=True)
    ap.add_argument('--out', required=True)
    a = ap.parse_args()
    texts = os.path.join(a.out, 'texts')
    os.makedirs(texts, exist_ok=True)
    tmp = tempfile.mkdtemp()
    queries = [(f'{t}_{n}', f'SELECT * FROM {t} ORDER BY {k} LIMIT {n}') for t, k in TABLES for n in SIZES]
    queries += AGGREGATES
    srv = Server(DUCKDB_SERVER + ['--db-path', a.db, '--read-only', '--max-rows', '10000', '--max-chars', '100000000'])
    rows = []
    try:
        for name, sql in queries:
            reply, _ = srv.rpc('tools/call', {'name': 'execute_query', 'arguments': {'sql': sql}})
            res = reply['result']
            sent = res['content'][0]['text']
            r = {'query': name, 'sql': sql, 'is_error': res.get('isError', False)}
            if r['is_error']:
                r['error'] = sent[:300]
                rows.append(r)
                print(f'{name}: error', file=sys.stderr)
                continue
            with open(os.path.join(texts, name + '.json'), 'w', encoding='utf-8') as f:
                f.write(sent)
            doc = json.loads(sent)
            mini = minify(sent)
            r.update(rows=len(doc.get('rows', [])), sent_bytes=len(sent.encode()), mini_bytes=len(mini.encode()),
                     sent_tokens=o200k(sent, tmp), mini_tokens=o200k(mini, tmp),
                     nulls=sum(v is None for row in doc.get('rows', []) for v in row))
            code, frame, err = run([MCP, 'convert', '--columnar'], sent)
            if code == 0:
                with open(os.path.join(texts, name + '.frame.txt'), 'w', encoding='utf-8') as f:
                    f.write(frame)
                dcode, back, derr = run([MCP, 'convert', '--columnar', '--decode'], frame)
                r['round_trip'] = 'exact' if dcode == 0 and strict(load(back), load(sent)) else f'FAILED {derr}'
                r.update(frame_bytes=len(frame.encode()), frame_tokens=o200k(frame, tmp))
                head = frame.split('\n', 1)[0]
            else:
                r['kept'] = err.rsplit(': ', 1)[-1]
                head = None
            if r['nulls'] and doc.get('rows'):
                if head is None:
                    h = dict(doc)
                    h['rows'] = '@records'
                    head = json.dumps(h, separators=(',', ':'), ensure_ascii=False)
                r['null_layouts'] = layouts(doc, head, tmp)
            rows.append(r)
            print(f"{name}: {r.get('frame_tokens', r.get('kept'))} vs minified {r['mini_tokens']}", file=sys.stderr)
    finally:
        srv.close()
    with open(os.path.join(a.out, 'columnar.json'), 'w') as f:
        json.dump(rows, f, indent=1)
    with open(os.path.join(a.out, 'columnar.md'), 'w') as f:
        f.write(report(rows))


def pct(a, b):
    return f'{100 * (b - a) / b:.1f}'


def report(rows):
    out = ['# Columnar results: sent, minified, LDF frame (o200k_base)', '',
           'Savings are absolute o200k tokens and percent of the baseline named. '
           '`whitespace` is sent minus minified, which minifying alone gives; `LDF` is minified minus frame. '
           'A `-` frame means the proxy keeps the JSON (minified) and says why.', '',
           '| query | rows | nulls | sent | minified | frame | whitespace saved | LDF saved vs minified | LDF % of minified | round trip |',
           '|---|---:|---:|---:|---:|---:|---:|---:|---:|---|']
    for r in rows:
        if r['is_error']:
            out.append(f"| {r['query']} | error | | | | | | | | |")
            continue
        s, m = r['sent_tokens'], r['mini_tokens']
        if 'frame_tokens' in r:
            fr = r['frame_tokens']
            out.append(f"| {r['query']} | {r['rows']} | {r['nulls']} | {s} | {m} | {fr} | {s - m} ({pct(m, s)}%) | {m - fr} ({pct(fr, m)}%) | {100 * fr / m:.1f} | {r['round_trip']} |")
        else:
            out.append(f"| {r['query']} | {r['rows']} | {r['nulls']} | {s} | {m} | - ({r['kept']}) | {s - m} ({pct(m, s)}%) | 0 | 100.0 | |")
    out += ['', '## NULL layouts', '',
            'Head line plus the rows table, o200k tokens, as percent of minified JSON. `kept` writes SQL NULL as `null`; '
            '`dropped` leaves it out (absent means NULL inside a frame). `@0.5` is the default sparse threshold, `@0` turns the `~` column off.', '',
            '| query | rows | nulls | minified | kept@0.5 (default) | kept@0 | dropped@0.5 | dropped@0 |',
            '|---|---:|---:|---:|---:|---:|---:|---:|']
    for r in rows:
        nl = r.get('null_layouts')
        if not nl:
            continue
        m = r['mini_tokens']
        cell = lambda k: '-' if nl[k] is None else f'{nl[k]} ({100 * nl[k] / m:.0f}%)'
        out.append(f"| {r['query']} | {r['rows']} | {r['nulls']} | {m} | {cell('kept@0.5')} | {cell('kept@0')} | {cell('dropped@0.5')} | {cell('dropped@0')} |")
    return '\n'.join(out) + '\n'


if __name__ == '__main__':
    main()
