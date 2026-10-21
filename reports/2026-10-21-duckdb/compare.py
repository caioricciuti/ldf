"""What the DuckDB server sends against what it could send, in o200k tokens.

For each saved JSON result of bench_mcp.py --mode duckdb: the text as the
server sent it (pretty-printed), the same JSON minified, and LDF of the rows
turned into records (`ldf-mcp convert`, wrapper keys dropped, so that column
is the rows only). Counts are `ldf-bench count`. Standard library only.

    python3 -I reports/2026-10-21-duckdb/compare.py results/mcp-duckdb/texts > reports/2026-10-21-duckdb/compare.md
"""
import json, os, subprocess, sys, tempfile

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
BENCH = os.path.join(REPO, 'target/release/ldf-bench')
MCP = os.path.join(REPO, 'target/release/ldf-mcp')
QUERIES = ['users_50', 'events_200', 'orders_100', 'logs_errors_500', 'wide_1000', 'logs_10000']


def o200k(path):
    out = subprocess.run([BENCH, 'count', path], capture_output=True, text=True, check=True).stdout
    return int(out.strip().splitlines()[-1].split('|')[3])


def main():
    texts = sys.argv[1]
    tmp = tempfile.mkdtemp()
    print('| query | rows | sent (pretty JSON) | minified JSON | LDF of the rows | LDF / sent | LDF / minified |')
    print('|---|---:|---:|---:|---:|---:|---:|')
    for q in QUERIES:
        sent = os.path.join(texts, f'{q}.json')
        doc = json.load(open(sent, encoding='utf-8'))
        mini = os.path.join(tmp, f'{q}.min.json')
        json.dump(doc, open(mini, 'w', encoding='utf-8'), separators=(',', ':'), ensure_ascii=False)
        recs = os.path.join(tmp, f'{q}.records.json')
        json.dump([dict(zip(doc['columns'], r)) for r in doc['rows']], open(recs, 'w', encoding='utf-8'),
                  separators=(',', ':'), ensure_ascii=False)
        ldf = os.path.join(tmp, f'{q}.ldf')
        with open(ldf, 'w', encoding='utf-8') as f:
            subprocess.run([MCP, 'convert', recs], stdout=f, stderr=subprocess.DEVNULL, check=True)
        s, m, l = o200k(sent), o200k(mini), o200k(ldf)
        print(f'| {q} | {len(doc["rows"])} | {s} | {m} | {l} | {l / s:.2f} | {l / m:.2f} |')


if __name__ == '__main__':
    main()
