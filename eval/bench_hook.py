"""Claude Code Bash hook benchmark: what `ldf-mcp hook claude-code` does
to JSON a shell command prints, measured the way Claude Code runs it (one
process per tool call, the event JSON on stdin).

For each dataset (the first ROWS records of fixtures/medium/<name>.json,
printed compact like `curl` and pretty like `gh api` or `kubectl -o json`)
and each model setting, it builds a PostToolUse payload shaped like Claude
Code 2.1.304's, runs the hook in a fresh session (first conversion, primer
included) and again in the same session (note only), and saves the text the
model reads in each case: stdout, plus the hook's additionalContext when it
converted. Exact o200k_base and cl100k_base counts come from `ldf-bench
count`; Claude's tokenizer is not public, so Claude counts are not here
(eval/count_tokens_claude.py, or usage from a real session). Timing is wall
clock per invocation, process start included, over --iters warm calls.

Output Claude Code would not show inline (over 30 000 characters, its
default BASH_MAX_OUTPUT_LENGTH) reaches the hook incomplete; those rows are
run with persistedOutputPath set, as Claude Code sends them, and stay JSON.
No model is called. Standard library only.

    python3 -I eval/bench_hook.py --out results/hook
"""
import argparse, json, os, shutil, statistics, subprocess, sys, tempfile, time

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
MCP = os.path.join(REPO, 'target/release/ldf-mcp')
BENCH = os.path.join(REPO, 'target/release/ldf-bench')
INLINE_LIMIT = 30000
DATASETS = ['flat_users', 'nested_orders', 'sparse_events', 'repetitive_logs', 'wide_table',
            'deep_nesting', 'mixed_types', 'empty_values', 'numeric_mix', 'high_card_strings',
            'unicode_strings', 'long_text']
ROWS = [20, 100]
MODELS = [None, 'claude-opus-5-5', 'claude-haiku-5-5', 'claude-sonnet-5-5']


def payload(session, stdout, persisted):
    resp = {'stdout': stdout, 'stderr': '', 'interrupted': False, 'isImage': False,
            'noOutputExpected': False, 'dangerouslyDisableSandbox': False}
    if persisted:
        resp['persistedOutputPath'] = '/tmp/tool-results/x.txt'
        resp['persistedOutputSize'] = len(stdout.encode())
    return json.dumps({'session_id': session, 'hook_event_name': 'PostToolUse', 'tool_name': 'Bash',
                       'tool_input': {'command': 'cat rows.json'}, 'tool_response': resp,
                       'tool_use_id': 'toolu_bench'}).encode()


def run_hook(state, model, data):
    argv = [MCP, 'hook', 'claude-code'] + (['--model', model] if model else [])
    env = dict(os.environ, LDF_STATE_DIR=state, LDF_STATS_FILE=os.path.join(state, 'events.jsonl'))
    t = time.perf_counter()
    p = subprocess.run(argv, input=data, capture_output=True, env=env)
    ms = (time.perf_counter() - t) * 1000
    if p.returncode != 0:
        sys.exit(f'hook exited {p.returncode}: {p.stderr.decode()}')
    return p.stdout.decode(), ms


def seen(stdout, reply):
    """What the model reads: the tool result and any added context."""
    if not reply:
        return stdout
    h = json.loads(reply)['hookSpecificOutput']
    return h['updatedToolOutput']['stdout'] + '\n' + h.get('additionalContext', '')


def last_event(state):
    with open(os.path.join(state, 'events.jsonl')) as f:
        return json.loads(f.read().splitlines()[-1])


def count(paths):
    out = subprocess.run([BENCH, 'count'] + sum((['--input', p] for p in paths), []),
                         capture_output=True, text=True, check=True).stdout
    rows = {}
    for line in out.splitlines()[2:]:
        cells = [c.strip() for c in line.strip('|').split('|')]
        rows[cells[0]] = {'o200k_base': int(cells[2]), 'cl100k_base': int(cells[4])}
    return rows


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--fixtures', default=os.path.join(REPO, 'fixtures/medium'))
    ap.add_argument('--iters', type=int, default=30)
    ap.add_argument('--out', default=os.path.join(REPO, 'results/hook'))
    a = ap.parse_args()
    texts = os.path.join(a.out, 'texts')
    shutil.rmtree(a.out, ignore_errors=True)
    os.makedirs(texts)
    cases = []
    for ds in DATASETS:
        with open(os.path.join(a.fixtures, ds + '.json')) as f:
            records = json.load(f)
        for n in ROWS:
            for style in ['compact', 'pretty']:
                body = records[:n]
                stdout = json.dumps(body, ensure_ascii=False, separators=(',', ':')) if style == 'compact' \
                    else json.dumps(body, ensure_ascii=False, indent=2)
                cases.append((ds, n, style, stdout))
    results = []
    for ds, n, style, stdout in cases:
        persisted = len(stdout) > INLINE_LIMIT
        base = f'{ds}.{n}.{style}'
        orig = os.path.join(texts, base + '.json')
        with open(orig, 'w') as f:
            f.write(stdout)
        for model in MODELS:
            tag = model or 'no-model'
            state = tempfile.mkdtemp(prefix='ldf-hook-bench-')
            try:
                first, _ = run_hook(state, model, payload('bench', stdout, persisted))
                ev_first = last_event(state)
                again, _ = run_hook(state, model, payload('bench', stdout, persisted))
                ev = last_event(state)
                times = [run_hook(state, model, payload('bench', stdout, persisted))[1] for _ in range(a.iters)]
                inside = [json.loads(l)['convert_us'] for l in open(os.path.join(state, 'events.jsonl')).read().splitlines()[2:]]
            finally:
                shutil.rmtree(state, ignore_errors=True)
            p_first = os.path.join(texts, f'{base}.{tag}.first.txt')
            p_again = os.path.join(texts, f'{base}.{tag}.again.txt')
            with open(p_first, 'w') as f:
                f.write(seen(stdout, first))
            with open(p_again, 'w') as f:
                f.write(seen(stdout, again))
            results.append({
                'dataset': ds, 'rows': n, 'style': style, 'model': model, 'json_bytes': len(stdout.encode()),
                'persisted': persisted,
                'first': {'outcome': ev_first['outcome'], 'reason': ev_first.get('reason'), 'bytes': ev_first['out_bytes']},
                'again': {'outcome': ev['outcome'], 'reason': ev.get('reason'), 'bytes': ev['out_bytes']},
                'wall_ms': {'p50': statistics.median(times), 'p95': sorted(times)[int(0.95 * (len(times) - 1))]},
                'inside_ms': {'p50': statistics.median(inside) / 1000, 'max': max(inside) / 1000},
                'files': {'json': orig, 'first': p_first, 'again': p_again},
            })
    tok = count(sorted({p for r in results for p in r['files'].values()}))
    for r in results:
        r['tokens'] = {k: tok[os.path.basename(p)] for k, p in r['files'].items()}
        r['files'] = {k: os.path.relpath(p, a.out) for k, p in r['files'].items()}
    sysinfo = json.loads(subprocess.run([BENCH, 'sysinfo'], capture_output=True, text=True, check=True).stdout)
    version = subprocess.run([MCP, '--version'], capture_output=True, text=True).stdout.strip()
    with open(os.path.join(a.out, 'hook.json'), 'w') as f:
        json.dump({'ldf_mcp': version, 'iters': a.iters, 'inline_limit': INLINE_LIMIT, 'sysinfo': sysinfo,
                   'results': results}, f, indent=2)
    with open(os.path.join(a.out, 'README.md'), 'w') as f:
        f.write(report(results, version, a.iters, sysinfo))
    print(os.path.join(a.out, 'README.md'))


def report(results, version, iters, sysinfo):
    o = [f'# Claude Code Bash hook, {version}\n',
         'Written by `eval/bench_hook.py`. Tokens are exact o200k_base counts (`ldf-bench count`) of the text '
         'the model reads: stdout, plus the hook\'s note when it converted. "first" is the first conversion in '
         'a session (the LDF primer rides along), "again" every later one (a two-line note). Claude counts are '
         'not here: its tokenizer is not public. Times are per invocation, process start included, '
         f'{iters} warm calls; "inside" is the hook\'s own time after start.\n',
         f'Machine: {sysinfo.get("cpu", "?")}, {sysinfo.get("os", "?")}.\n',
         '| dataset | rows | style | JSON bytes | model | first | again | JSON tok | first tok | again tok | again / JSON | wall p50 ms | inside p50 ms |',
         '|---|---:|---|---:|---|---|---|---:|---:|---:|---:|---:|---:|']
    for r in results:
        t = r['tokens']
        j, fi, ag = t['json']['o200k_base'], t['first']['o200k_base'], t['again']['o200k_base']
        out = lambda e: 'ldf' if e['outcome'] == 'ldf' else f'json ({e["reason"]})'
        o.append(f'| {r["dataset"]} | {r["rows"]} | {r["style"]} | {r["json_bytes"]} | {r["model"] or "none"} | '
                 f'{out(r["first"])} | {out(r["again"])} | {j} | {fi} | {ag} | {ag / j:.2f} | '
                 f'{r["wall_ms"]["p50"]:.1f} | {r["inside_ms"]["p50"]:.2f} |')
    return '\n'.join(o) + '\n'


if __name__ == '__main__':
    main()
