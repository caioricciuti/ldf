"""End-to-end MCP tool-call benchmark: the same queries through
`ldf-mcp clickhouse` (or `ldf-mcp files`) twice, once returning LDF and
once with --passthrough returning JSON, timed from the client side of the
stdio pipe (request written to response line read), so the time covers
JSON-RPC, the query or file read, and the encoding.

For each query it saves both texts, then runs `ldf-discover select` on the
JSON result for each model, which also reports o200k tokens for JSON,
TOON and LDF and their round-trip level. No model is called: the model
side of the latency (prefill of the tool result) is not measured here.
Standard library only. The ClickHouse password is read by ldf-mcp from
CLICKHOUSE_PASSWORD in the environment; this script never reads or prints it.

    python3 -I eval/bench_mcp.py --mode files --root fixtures/medium --out results/mcp-files
    python3 -I eval/bench_mcp.py --mode duckdb --db results/mcp-duckdb/ldf.duckdb --out results/mcp-duckdb
    python3 -I eval/bench_mcp.py --mode clickhouse --url http://127.0.0.1:8124 --user ldf --out results/mcp-clickhouse
"""
import argparse, json, os, statistics, subprocess, sys, time

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
MCP = os.path.join(REPO, 'target/release/ldf-mcp')
DISCOVER = os.path.join(REPO, 'target/release/ldf-discover')
MODELS = ['claude-haiku-5-5', 'deepseek-chat', 'claude-opus-5-5', 'gpt-6.1-sol']

# (name, tool, arguments): an agent's session, discovery first, then reads.
CLICKHOUSE = [
    ('list_tables', 'list_tables', {}),
    ('describe_orders', 'describe_table', {'table': 'orders'}),
    ('users_50', 'run_select', {'sql': 'SELECT * FROM users ORDER BY id LIMIT 50'}),
    ('users_by_country', 'run_select', {'sql': 'SELECT country, count() AS n, round(avg(score), 2) AS avg_score FROM users GROUP BY country ORDER BY n DESC'}),
    ('events_200', 'run_select', {'sql': 'SELECT * FROM events ORDER BY event_id LIMIT 200'}),
    ('orders_100', 'run_select', {'sql': 'SELECT * FROM orders ORDER BY order_id LIMIT 100'}),
    ('logs_errors_500', 'run_select', {'sql': "SELECT * FROM logs WHERE level IN ('ERROR', 'WARN') ORDER BY ts DESC LIMIT 500"}),
    ('wide_1000', 'run_select', {'sql': 'SELECT * FROM wide ORDER BY id LIMIT 1000'}),
    ('logs_10000', 'run_select', {'sql': 'SELECT * FROM logs'}),
]
# The same session against DuckDB through a third-party MCP server
# (MotherDuck's, pinned, wheels only, nothing newer than the date) behind
# `ldf-mcp proxy`.
DUCKDB_SERVER = ['uvx', '--exclude-newer', '2026-10-14', '--no-build', '--from',
                 'mcp-server-motherduck==1.1.0', 'mcp-server-motherduck']
DUCKDB = [
    ('list_tables', 'list_tables', {}),
    ('describe_orders', 'list_columns', {'table': 'orders'}),
    ('users_50', 'execute_query', {'sql': 'SELECT * FROM users ORDER BY id LIMIT 50'}),
    ('users_by_country', 'execute_query', {'sql': 'SELECT country, count(*) AS n, round(avg(score), 2) AS avg_score FROM users GROUP BY country ORDER BY n DESC, country'}),
    ('events_200', 'execute_query', {'sql': 'SELECT * FROM events ORDER BY event_id LIMIT 200'}),
    ('orders_100', 'execute_query', {'sql': 'SELECT * FROM orders ORDER BY order_id LIMIT 100'}),
    ('logs_errors_500', 'execute_query', {'sql': "SELECT * FROM logs WHERE level IN ('ERROR', 'WARN') ORDER BY ts DESC LIMIT 500"}),
    ('wide_1000', 'execute_query', {'sql': 'SELECT * FROM wide ORDER BY id LIMIT 1000'}),
    ('logs_10000', 'execute_query', {'sql': 'SELECT * FROM logs'}),
]
FILES = [
    ('list_files', 'list_files', {}),
    ('describe_orders', 'describe_file', {'path': 'nested_orders.json'}),
    ('users_50', 'read_rows', {'path': 'flat_users.parquet', 'limit': 50}),
    ('events_200', 'read_rows', {'path': 'sparse_events.jsonl', 'limit': 200}),
    ('orders_100', 'read_rows', {'path': 'nested_orders.json', 'limit': 100}),
    ('logs_500', 'read_rows', {'path': 'repetitive_logs.parquet', 'limit': 500}),
    ('wide_1000', 'read_rows', {'path': 'wide_table.parquet', 'limit': 1000}),
    ('logs_10000', 'read_rows', {'path': 'repetitive_logs.parquet', 'limit': 10000}),
]


class Server:
    def __init__(self, argv):
        self.p = subprocess.Popen(argv, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True, bufsize=1)
        self.n = 0
        self.rpc('initialize', {'protocolVersion': '2025-06-18', 'capabilities': {}, 'clientInfo': {'name': 'bench', 'version': '0'}})
        self.p.stdin.write(json.dumps({'jsonrpc': '2.0', 'method': 'notifications/initialized'}) + '\n')

    def rpc(self, method, params):
        self.n += 1
        t = time.perf_counter()
        self.p.stdin.write(json.dumps({'jsonrpc': '2.0', 'id': self.n, 'method': method, 'params': params}) + '\n')
        self.p.stdin.flush()
        line = self.p.stdout.readline()
        ms = (time.perf_counter() - t) * 1000
        if not line:
            raise SystemExit(f'server exited during {method}')
        return json.loads(line), ms

    def close(self):
        self.p.stdin.close()
        self.p.wait(timeout=10)


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('--mode', choices=['clickhouse', 'files', 'duckdb'], required=True)
    ap.add_argument('--url', default='http://127.0.0.1:8124')
    ap.add_argument('--user', default='default')
    ap.add_argument('--root', default='fixtures/medium')
    ap.add_argument('--db', help='DuckDB file for --mode duckdb')
    ap.add_argument('--max-rows', type=int, default=10000)
    ap.add_argument('--repeats', type=int, default=20)
    ap.add_argument('--warmup', type=int, default=3)
    ap.add_argument('--out', required=True)
    a = ap.parse_args()
    os.makedirs(os.path.join(a.out, 'texts'), exist_ok=True)
    if a.mode == 'duckdb':
        # --max-chars well above the largest result, so the server never truncates.
        upstream = DUCKDB_SERVER + ['--db-path', a.db, '--max-rows', str(a.max_rows), '--max-chars', '100000000']
        argv = lambda extra: [MCP, 'proxy', '--quiet'] + extra + ['--'] + upstream
        queries = DUCKDB
    else:
        base = [MCP, a.mode, '--max-rows', str(a.max_rows), '--quiet']
        base += ['--url', a.url, '--user', a.user] if a.mode == 'clickhouse' else ['--root', a.root]
        argv = lambda extra: base + extra
        queries = CLICKHOUSE if a.mode == 'clickhouse' else FILES
    results = {q[0]: {'tool': q[1], 'arguments': q[2]} for q in queries}
    for variant, extra in [('ldf', []), ('json', ['--passthrough'])]:
        srv = Server(argv(extra))
        for name, tool, args in queries:
            times = []
            for i in range(a.warmup + a.repeats):
                reply, ms = srv.rpc('tools/call', {'name': tool, 'arguments': args})
                if i >= a.warmup:
                    times.append(ms)
            res = reply.get('result') or {}
            if 'error' in reply or res.get('isError'):
                raise SystemExit(f'{variant} {name}: {json.dumps(reply)[:300]}')
            blocks = [b['text'] for b in res.get('content', []) if b.get('type') == 'text']
            text = blocks[0]
            path = os.path.join(a.out, 'texts', f'{name}.{variant}.' + ('json' if variant == 'json' else 'txt'))
            open(path, 'w', encoding='utf-8').write(text)
            times.sort()
            results[name][variant] = {
                'bytes': len(text.encode('utf-8')),
                'is_ldf': text.startswith('@'),
                'capped': len(blocks) > 1,
                'p50_ms': round(statistics.median(times), 3),
                'p95_ms': round(times[int(0.95 * (len(times) - 1))], 3),
                'min_ms': round(times[0], 3),
            }
        srv.close()
    for name in results:
        jpath = os.path.join(a.out, 'texts', f'{name}.json')
        src = os.path.join(a.out, 'texts', f'{name}.json.json')
        os.replace(src, jpath)
        decisions = {}
        for m in MODELS:
            for loose in (False, True):
                cmd = [DISCOVER, 'select', '--model', m, '--input', jpath, '--json'] + (['--extrapolate-rows'] if loose else [])
                r = subprocess.run(cmd, capture_output=True, text=True)
                if r.returncode != 0:
                    decisions[m] = {'error': r.stderr.strip()[:200]}
                    break
                d = json.loads(r.stdout)
                decisions[m + (' (extrapolate)' if loose else '')] = d['choice']
                results[name]['rows'] = d['profile']['rows']
                results[name]['traits'] = [k for k, v in d['profile']['traits'].items() if v]
                results[name]['o200k'] = {v['choice']['format'] + (f"@{v['choice']['sparse_threshold']}" if v['choice']['format'] == 'ldf' else ''):
                                          {'tokens': v['tokens'], 'round_trip': v['round_trip']} for v in d['verdicts']}
        results[name]['decisions'] = decisions
    json.dump({'mode': a.mode, 'repeats': a.repeats, 'warmup': a.warmup, 'queries': results}, open(os.path.join(a.out, 'mcp.json'), 'w'), indent=1)
    L = [f'# MCP end to end: ldf-mcp {a.mode}', '',
         f'{a.repeats} timed calls per query and variant after {a.warmup} warm-up calls, one long-lived server per variant, '
         'client-side wall time per tools/call over stdio. `ldf` is the server default (LDF when shorter than the JSON), '
         '`json` is --passthrough. Tokens are o200k_base (exact for GPT, a proxy for Claude) from `ldf-discover select`, '
         'on the JSON result; `-` means the format does not round-trip that result. No model was called.', '',
         '| query | rows | traits | JSON tokens | TOON / JSON | LDF / JSON | p50 ms JSON | p50 ms LDF | p95 ms LDF | LDF overhead p50 |',
         '|---|---:|---|---:|---:|---:|---:|---:|---:|---:|']
    for name, r in results.items():
        t = r.get('o200k', {})
        j = (t.get('json') or {}).get('tokens')
        def ratio(k):
            v = (t.get(k) or {}).get('tokens')
            return f'{v / j:.2f}' if v and j else '-'
        L.append(f"| {name} | {r.get('rows', '')} | {(','.join(r['traits']) or 'flat') if 'traits' in r else 'not records'} | {j or '-'} | {ratio('toon')} | {ratio('ldf@0.5')} | "
                 f"{r['json']['p50_ms']:.2f} | {r['ldf']['p50_ms']:.2f} | {r['ldf']['p95_ms']:.2f} | {r['ldf']['p50_ms'] - r['json']['p50_ms']:+.2f} |")
    L += ['', '## Selector decision per model (default policy; with --extrapolate-rows in brackets)', '',
          '| query | rows | ' + ' | '.join(MODELS) + ' |', '|---|---:|' + '---|' * len(MODELS)]
    def short(c):
        if not c or 'format' not in c:
            return 'n/a'
        return c['format'] + (f"@{c['sparse_threshold']}" if c['format'] == 'ldf' else '')
    for name, r in results.items():
        d = r['decisions']
        L.append(f"| {name} | {r.get('rows', '')} | " + ' | '.join(f"{short(d.get(m))} ({short(d.get(m + ' (extrapolate)'))})" for m in MODELS) + ' |')
    open(os.path.join(a.out, 'mcp.md'), 'w').write('\n'.join(L) + '\n')
    print('\n'.join(L))


if __name__ == '__main__':
    main()
