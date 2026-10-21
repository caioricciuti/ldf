"""Database questions answered by headless Claude Code through an MCP server.

Each run is `claude -p` with one MCP server, MotherDuck's DuckDB server
(pinned as in reports/2026-10-21-duckdb) over the medium fixtures, no
built-in tools, and one question. Three arms differ only in what sits
between the server and Claude Code:

- `json`: nothing; pretty-printed JSON as the server ships it;
- `minified`: eval/minify_proxy.py; the same JSON without whitespace, so a
  saving LDF shows against this arm is not whitespace;
- `ldf`: `ldf-mcp proxy --columnar`; primer, columnar frames, LDF where
  shorter than minified JSON, minified JSON otherwise.

Questions cover schema discovery, filtering, joins, nested records, NULL
semantics and exact-value retrieval. Ground truth is computed here from the
fixture JSONL files, not from the database, except schema questions, which
come from the server's own catalog (`list_columns`) at build time. NULL in
the database is a key absent from or null in the fixture record.

Usage is read from the session transcripts (prompt tokens per request,
Claude tokens) and tool results are measured there too. Cost is Claude
Code's total_cost_usd. Paid: `run` needs --confirm-spend and --max-cost and
stops before a run would start past the cap; finished runs are kept in
runs.jsonl and skipped on a rerun.

    python3 -I eval/db_eval.py build --db results/columnar/fixtures.duckdb --out eval/out/db-eval
    python3 -I eval/db_eval.py plan --out eval/out/db-eval --reps 2
    python3 -I eval/db_eval.py run --out eval/out/db-eval --model claude-sonnet-5-5 --reps 1 --per-category 1 --max-cost 7 --confirm-spend
    python3 -I eval/db_eval.py run --out eval/out/db-eval --model claude-sonnet-5-5 --reps 2 --max-cost 10 --confirm-spend
    python3 -I eval/db_eval.py report --out eval/out/db-eval
"""
import argparse, glob, json, math, os, random, re, shutil, subprocess, sys, tempfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from bench_mcp import DUCKDB_SERVER, Server  # noqa: E402

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
FIX = os.path.join(REPO, 'fixtures/medium')
MCP = os.path.join(REPO, 'target/release/ldf-mcp')
ARMS = ['json', 'minified', 'ldf']
PER_RUN_CAP = 0.50


def rows(name):
    with open(os.path.join(FIX, name + '.jsonl'), encoding='utf-8') as f:
        return [json.loads(l) for l in f]


def tasks(db):
    rnd = random.Random(20261021)
    users, orders, events = rows('flat_users'), rows('nested_orders'), rows('sparse_events')
    logs, hc, uni = rows('repetitive_logs'), rows('high_card_strings'), rows('unicode_strings')
    by_id = {u['id']: u for u in users}
    out = []

    def add(cat, q, answer, kind='exact'):
        out.append({'id': f'{cat}_{sum(t["category"] == cat for t in out) + 1}', 'category': cat,
                    'question': q, 'answer': answer, 'kind': kind})

    # Schema discovery: from the server's catalog.
    srv = Server(DUCKDB_SERVER + ['--db-path', db, '--read-only'])
    try:
        cols = {}
        for t in ('flat_users', 'sparse_events', 'nested_orders', 'repetitive_logs', 'wide_table'):
            reply, _ = srv.rpc('tools/call', {'name': 'list_columns', 'arguments': {'table': t}})
            doc = json.loads(reply['result']['content'][0]['text'])
            cols[t] = [(c['name'], c['type']) for c in doc['columns']]
    finally:
        srv.close()
    add('schema', 'Which table in the database has a column named error_code? Answer with the table name.', 'sparse_events')
    add('schema', 'How many columns does the table nested_orders have?', len(cols['nested_orders']))
    add('schema', 'What is the SQL type of the column signup_date in table flat_users?', dict(cols['flat_users'])['signup_date'])
    add('schema', 'List the column names of table repetitive_logs in table order.', [c for c, _ in cols['repetitive_logs']], 'ordered')
    add('schema', 'How many columns of table wide_table have type DOUBLE?', sum(t == 'DOUBLE' for _, t in cols['wide_table']))

    # Filtering: questions whose natural query returns rows to read.
    for _ in range(3):
        c = rnd.choice(sorted({u['country'] for u in users}))
        ids = sorted(u['id'] for u in users if u['country'] == c and u['score'] > 97 and u['active'])
        add('filter', f'List the ids of active users from country {c} with score above 97, ascending.', ids, 'ordered')
    svc = rnd.choice(sorted({l['service'] for l in logs}))
    add('filter', f'How many ERROR log lines does service {svc} have?',
        sum(l['service'] == svc and l['level'] == 'ERROR' for l in logs))
    # A threshold that leaves about ten orders.
    cut = int(sorted((o['total'] for o in orders if o['status'] == 'refunded'), reverse=True)[10])
    add('filter', f'Which order ids have status refunded and total above {cut}? List them ascending.',
        sorted(o['order_id'] for o in orders if o['status'] == 'refunded' and o['total'] > cut), 'ordered')

    # Joins: orders.customer.id is users.id.
    for o in rnd.sample(orders, 3):
        add('join', f'What is the email of the user who placed order {o["order_id"]}? (orders.customer.id matches users.id)',
            by_id[o['customer']['id']]['email'])
    c = rnd.choice(sorted({u['country'] for u in users}))
    add('join', f'How many orders were placed by users whose country in flat_users is {c}? (orders.customer.id matches users.id)',
        sum(by_id[o['customer']['id']]['country'] == c for o in orders))
    u = by_id[rnd.choice(sorted({o['customer']['id'] for o in orders}))]
    add('join', f'List the order ids of user {u["id"]} ({u["name"]}), ascending.',
        sorted(o['order_id'] for o in orders if o['customer']['id'] == u['id']), 'ordered')

    # Nested records: struct fields and the items list.
    for o in rnd.sample(orders, 2):
        it = rnd.choice(o['items'])
        add('nested', f'In order {o["order_id"]}, what is the quantity of SKU {it["sku"]}?',
            sum(i['qty'] for i in o['items'] if i['sku'] == it['sku']))
    o = rnd.choice(orders)
    add('nested', f'How many line items does order {o["order_id"]} have?', len(o['items']))
    o = rnd.choice(orders)
    add('nested', f'Which city is the customer of order {o["order_id"]} in?', o['customer']['address']['city'])
    o = rnd.choice([o for o in orders if len(o['items']) >= 3])
    add('nested', f'List the SKUs of order {o["order_id"]} in item order.', [i['sku'] for i in o['items']], 'ordered')

    # NULL semantics: absent and null are both NULL in the table.
    null = lambda e, k: e.get(k) is None
    add('null', 'How many events in sparse_events have a NULL referrer?', sum(null(e, 'referrer') for e in events))
    add('null', 'How many purchase events have a non-NULL amount?',
        sum(e['type'] == 'purchase' and not null(e, 'amount') for e in events))
    e = rnd.choice([e for e in events if null(e, 'error_code')])
    add('null', f'What is the error_code of event {e["event_id"]}? Answer null if it has none.', None)
    e = rnd.choice([e for e in events if not null(e, 'error_code')])
    add('null', f'What is the error_code of event {e["event_id"]}? Answer null if it has none.', e['error_code'])
    first = [e for e in events if e['event_id'] <= 100]
    add('null', 'Among events with event_id from 1 to 100, list the event_ids whose device is NULL, ascending.',
        sorted(e['event_id'] for e in first if null(e, 'device')), 'ordered')

    # Exact values: long, unusual or precise values copied back.
    u = rnd.choice(users)
    add('exact', f'What is the exact score of user {u["id"]}?', u['score'], 'number')
    h = rnd.choice(hc)
    add('exact', f'What is the hash of the row with id {h["id"]} in high_card_strings?', h['hash'])
    h = rnd.choice(hc)
    add('exact', f'What is the uuid of the row with id {h["id"]} in high_card_strings?', h['uuid'])
    x = rnd.choice([x for x in uni if x['id'] > 1])
    add('exact', f'What is the name of the row with id {x["id"]} in unicode_strings?', x['name'])
    o = rnd.choice(orders)
    add('exact', f'What is the total of order {o["order_id"]}?', o['total'], 'number')
    return out


def prompt(q):
    return (f'{q}\n\nUse the database tools to find the answer. End your reply with one line of JSON: '
            '{"answer": ...} where the value is a number, string, null or array as the question asks.')


def server_cmd(arm, db):
    up = DUCKDB_SERVER + ['--db-path', db, '--read-only', '--max-rows', '1000']
    if arm == 'json':
        return up
    if arm == 'minified':
        return [sys.executable, '-I', os.path.join(REPO, 'eval/minify_proxy.py'), '--'] + up
    return [MCP, 'proxy', '--columnar', '--quiet', '--no-stats', '--'] + up


def answer(text):
    for m in reversed(list(re.finditer(r'\{"answer"\s*:.*\}', text or ''))):
        try:
            return True, json.loads(m.group(0))['answer']
        except (ValueError, KeyError):
            continue
    return False, None


def correct(t, got):
    want = t['answer']
    if t['kind'] == 'number' or (isinstance(want, (int, float)) and not isinstance(want, bool)):
        try:
            return got is not None and not isinstance(got, bool) and math.isclose(float(got), float(want), rel_tol=0, abs_tol=1e-9)
        except (TypeError, ValueError):
            return False
    if t['kind'] == 'ordered':
        return isinstance(got, list) and len(got) == len(want) and all(
            correct({'answer': w, 'kind': 'exact'}, g) for g, w in zip(got, want))
    return got == want


def usage(sid):
    """Per-request prompt tokens and the tool results' text, from the transcript."""
    tr = glob.glob(os.path.expanduser(f'~/.claude/projects/*/{sid}.jsonl'))
    reqs, seen, tool_chars = [], set(), 0
    if not tr:
        return reqs, tool_chars
    for line in open(tr[0], encoding='utf-8'):
        try:
            ev = json.loads(line)
        except ValueError:
            continue
        m = ev.get('message') or {}
        if ev.get('type') == 'user' and isinstance(m.get('content'), list):
            for b in m['content']:
                if isinstance(b, dict) and b.get('type') == 'tool_result':
                    c = b.get('content')
                    tool_chars += len(c) if isinstance(c, str) else sum(len(x.get('text', '')) for x in c or [] if isinstance(x, dict))
        u = m.get('usage')
        if ev.get('type') == 'assistant' and u and m.get('id') not in seen:
            seen.add(m.get('id'))
            reqs.append({'prompt': u.get('input_tokens', 0) + u.get('cache_creation_input_tokens', 0)
                         + u.get('cache_read_input_tokens', 0), 'output': u.get('output_tokens', 0)})
    return reqs, tool_chars


def cmd_build(a):
    os.makedirs(a.out, exist_ok=True)
    ts = tasks(os.path.abspath(a.db))
    with open(os.path.join(a.out, 'tasks.jsonl'), 'w', encoding='utf-8') as f:
        for t in ts:
            f.write(json.dumps(t, ensure_ascii=False) + '\n')
    json.dump({'db': os.path.abspath(a.db), 'tasks': len(ts)}, open(os.path.join(a.out, 'manifest.json'), 'w'))
    print(f'{len(ts)} tasks in {a.out}/tasks.jsonl')


def load_tasks(out):
    return [json.loads(l) for l in open(os.path.join(out, 'tasks.jsonl'), encoding='utf-8')]


def cmd_plan(a):
    ts = load_tasks(a.out)
    n = len(ts) * len(ARMS) * a.reps
    print(f'{len(ts)} tasks x {len(ARMS)} arms x {a.reps} reps = {n} claude -p runs, each capped at ${PER_RUN_CAP:.2f} '
          f'(--max-budget-usd); hard upper bound ${n * PER_RUN_CAP:.2f}. Expected spend is far lower; run a pilot '
          'with --per-category 1 and read the runs file before the full run.')


def cmd_run(a):
    if not a.confirm_spend:
        sys.exit('this runs paid model calls; pass --confirm-spend')
    out = os.path.abspath(a.out)
    db = json.load(open(os.path.join(out, 'manifest.json')))['db']
    ts = load_tasks(out)
    if a.per_category:
        ts = [t for t in ts if int(t['id'].rsplit('_', 1)[1]) <= a.per_category]
    ts = ts[: a.limit or None]
    # Runs start outside the repository, so Claude Code does not load this
    # project's CLAUDE.md (which describes LDF) into the model's context.
    work = tempfile.mkdtemp(prefix='ldf-db-eval-')
    log = os.path.join(out, f'runs.{a.model}.jsonl')
    done = {}
    if os.path.exists(log):
        for line in open(log):
            r = json.loads(line)
            done[(r['task'], r['arm'], r['rep'])] = r
    spent = sum(r['cost_usd'] for r in done.values())
    for rep in range(a.reps):
        for t in ts:
            arms = ARMS[:]
            random.Random(f'{t["id"]}/{rep}').shuffle(arms)
            for arm in arms:
                if (t['id'], arm, rep) in done:
                    continue
                if spent >= a.max_cost:
                    sys.exit(f'stopped: ${spent:.4f} spent, cap ${a.max_cost:.2f}')
                d = os.path.join(work, f'{t["id"]}.{arm}.{rep}')
                shutil.rmtree(d, ignore_errors=True)
                os.makedirs(d)
                cmd = server_cmd(arm, db)
                json.dump({'mcpServers': {'db': {'command': cmd[0], 'args': cmd[1:]}}}, open(os.path.join(d, 'mcp.json'), 'w'))
                argv = ['claude', '-p', '--model', a.model, '--tools', '', '--strict-mcp-config',
                        '--mcp-config', os.path.join(d, 'mcp.json'), '--allowedTools', 'mcp__db',
                        '--setting-sources', 'project', '--disable-slash-commands', '--max-turns', '12',
                        '--max-budget-usd', str(PER_RUN_CAP), '--output-format', 'json', prompt(t['question'])]
                p = subprocess.run(argv, cwd=d, capture_output=True, text=True, timeout=900)
                try:
                    res = json.loads(p.stdout)
                except ValueError:
                    sys.exit(f'{t["id"]} {arm}: no JSON result: {p.stdout[-500:]} {p.stderr[-500:]}')
                reqs, tool_chars = usage(res.get('session_id', ''))
                found, got = answer(res.get('result'))
                row = {'task': t['id'], 'category': t['category'], 'arm': arm, 'rep': rep, 'model': a.model,
                       'answered': found, 'answer': got, 'correct': found and correct(t, got),
                       'cost_usd': res.get('total_cost_usd') or 0.0, 'turns': res.get('num_turns'),
                       'duration_ms': res.get('duration_ms'), 'requests': reqs, 'tool_result_chars': tool_chars,
                       'is_error': res.get('is_error'), 'tail': (res.get('result') or '')[-300:]}
                with open(log, 'a') as f:
                    f.write(json.dumps(row, ensure_ascii=False) + '\n')
                done[(t['id'], arm, rep)] = row
                spent += row['cost_usd']
                print(f'{t["id"]:10} {arm:8} rep {rep}: {"ok " if row["correct"] else "BAD"} ${row["cost_usd"]:.4f} '
                      f'turns {row["turns"]} tool chars {tool_chars}, total ${spent:.4f}', flush=True)
    cmd_report(a)


def sign_test(w, l):
    n, k = w + l, min(w, l)
    if n == 0:
        return 1.0
    return min(1.0, 2 * sum(math.comb(n, i) for i in range(k + 1)) / 2 ** n)


def cmd_report(a):
    out = os.path.abspath(a.out)
    for log in sorted(glob.glob(os.path.join(out, 'runs.*.jsonl'))):
        runs = [json.loads(l) for l in open(log)]
        model = runs[0]['model'] if runs else '?'
        L = [f'# Database questions through Claude Code, {model}', '',
             'Accuracy is strict (the trailing {"answer": ...} must equal the ground truth; numbers exactly, lists in order). '
             'Prompt tokens are the sum over the run\'s API requests of input plus cache tokens, as Anthropic reported them. '
             'Tool result chars are the text of every tool result in the transcript.', '',
             '| arm | runs | correct | prompt tokens (mean) | tool result chars (mean) | turns (mean) | cost (sum) |',
             '|---|---:|---:|---:|---:|---:|---:|']
        for arm in ARMS:
            rs = [r for r in runs if r['arm'] == arm]
            if not rs:
                continue
            mean = lambda f: sum(f(r) for r in rs) / len(rs)
            L.append(f"| {arm} | {len(rs)} | {sum(r['correct'] for r in rs)} | {mean(lambda r: sum(q['prompt'] for q in r['requests'])):.0f} | "
                     f"{mean(lambda r: r['tool_result_chars']):.0f} | {mean(lambda r: r['turns'] or 0):.1f} | ${sum(r['cost_usd'] for r in rs):.2f} |")
        L += ['', '| category | ' + ' | '.join(ARMS) + ' |', '|---|' + '---:|' * len(ARMS)]
        for cat in sorted({r['category'] for r in runs}):
            cells = []
            for arm in ARMS:
                rs = [r for r in runs if r['arm'] == arm and r['category'] == cat]
                cells.append(f"{sum(r['correct'] for r in rs)}/{len(rs)}")
            L.append(f'| {cat} | ' + ' | '.join(cells) + ' |')
        key = {(r['task'], r['rep'], r['arm']): r['correct'] for r in runs}
        L.append('')
        for base in ('json', 'minified'):
            pairs = [(key[(t, rep, 'ldf')], key[(t, rep, base)]) for (t, rep, arm) in key if arm == base and (t, rep, 'ldf') in key]
            w = sum(x and not y for x, y in pairs)
            l = sum(y and not x for x, y in pairs)
            L.append(f'- ldf vs {base}: {len(pairs)} paired runs, +{w} / -{l}, sign test p={sign_test(w, l):.2f}')
        path = log.replace('runs.', 'report.').replace('.jsonl', '.md')
        open(path, 'w').write('\n'.join(L) + '\n')
        print('\n'.join(L))


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('command', choices=['build', 'plan', 'run', 'report'])
    ap.add_argument('--db')
    ap.add_argument('--out', required=True)
    ap.add_argument('--model', default='claude-sonnet-5-5')
    ap.add_argument('--reps', type=int, default=2)
    ap.add_argument('--limit', type=int, default=0, help='first N tasks only')
    ap.add_argument('--per-category', type=int, default=0, help='first N tasks of each category (pilot)')
    ap.add_argument('--max-cost', type=float, default=0.0)
    ap.add_argument('--confirm-spend', action='store_true')
    a = ap.parse_args()
    {'build': cmd_build, 'plan': cmd_plan, 'run': cmd_run, 'report': cmd_report}[a.command](a)


if __name__ == '__main__':
    main()
