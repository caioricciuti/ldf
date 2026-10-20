"""A/B of the Claude Code Bash hook on provider-reported usage: the same
headless Claude Code task with and without `ldf-mcp hook claude-code`.

Each run gets a fresh directory holding data.json (the first ROWS records of
a medium fixture, compact or pretty), only the Bash tool, no MCP servers,
and a prompt to run `cat data.json` once and answer two questions whose
answers are computed here from the data. Arm `hook` passes the hook through
--settings (state isolated per run, so the primer is paid in every run, as
in a fresh session); arm `json` passes nothing. Your own settings and hooks
apply to both arms alike.

Usage comes from the session transcript: per API request, input +
cache_creation + cache_read input tokens (the prompt the model processed)
and output tokens, as Anthropic reported them. The second request's prompt
minus the first request's prompt and output is what the tool call added:
the tool result, the hook's note, and Claude Code's own wrapping, in Claude
tokens. Cost is Claude Code's total_cost_usd for the run.

This spends money: nothing runs without --confirm-spend, and it stops
before a run would start past --max-cost (spent so far, as reported).
Finished runs are kept in runs.jsonl and skipped on a rerun.

    python3 -I eval/ab_hook.py --model claude-haiku-5-5 --reps 3 --max-cost 1.00 --confirm-spend
"""
import argparse, glob, json, os, re, shutil, subprocess, sys

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
MCP = os.path.join(REPO, 'target/release/ldf-mcp')
FIX = os.path.join(REPO, 'fixtures/medium')


def load(ds, n):
    with open(os.path.join(FIX, ds + '.json')) as f:
        return json.load(f)[:n]


def cases():
    """(name, dataset, rows, style, [(question, answer)])"""
    out = []
    r = load('flat_users', 100)
    out.append(('flat_users_100_pretty', r, 'pretty', [
        ('How many users have active true and country "DE"?', sum(1 for x in r if x['active'] and x['country'] == 'DE')),
        ('What is the email of the user with id 37?', next(x['email'] for x in r if x['id'] == 37))]))
    r = load('repetitive_logs', 100)
    out.append(('repetitive_logs_100_pretty', r, 'pretty', [
        ('How many records have level "ERROR"?', sum(1 for x in r if x['level'] == 'ERROR')),
        ('How many records have status 500?', sum(1 for x in r if x['status'] == 500))]))
    r = load('wide_table', 20)
    out.append(('wide_table_20_pretty', r, 'pretty', [
        ('What is the value of i07 for the record with id 13?', next(x['i07'] for x in r if x['id'] == 13)),
        ('What is the value of f03 for the record with id 5?', next(x['f03'] for x in r if x['id'] == 5))]))
    r = load('sparse_events', 100)
    out.append(('sparse_events_100_pretty', r, 'pretty', [
        ('How many events have type "purchase"?', sum(1 for x in r if x.get('type') == 'purchase')),
        ('What is the device of the event with event_id 42?', next(x.get('device') for x in r if x['event_id'] == 42))]))
    r = load('unicode_strings', 100)
    out.append(('unicode_strings_100_compact', r, 'compact', [
        ('What is the name of the record with id 58?', next(x['name'] for x in r if x['id'] == 58)),
        ('How many records have emoji exactly "✓"?', sum(1 for x in r if x.get('emoji') == '✓'))]))
    r = load('nested_orders', 20)
    out.append(('nested_orders_20_compact', r, 'compact', [
        ('What is the status of order ORD-000007?', next(x['status'] for x in r if x['order_id'] == 'ORD-000007')),
        ('How many orders have shipped true?', sum(1 for x in r if x.get('shipped') is True))]))
    return out


def prompt(questions):
    qs = '\n'.join(f'q{i + 1}: {q}' for i, (q, _) in enumerate(questions))
    return ('Run exactly `cat data.json` once with the Bash tool and nothing else. Then answer from its output:\n'
            f'{qs}\nEnd your reply with one line of JSON: {{"q1": ..., "q2": ...}}')


def hook_settings(path):
    h = {'type': 'command', 'command': MCP, 'args': ['hook', 'claude-code'], 'timeout': 10}
    s = {'hooks': {'PostToolUse': [{'matcher': 'Bash', 'hooks': [h]}],
                   'SessionStart': [{'hooks': [h]}], 'PostModelSwitch': [{'hooks': [h]}]}}
    with open(path, 'w') as f:
        json.dump(s, f)


def requests(transcript):
    """Usage per API request, in order, one entry per message id."""
    seen, out = set(), []
    for line in open(transcript):
        try:
            ev = json.loads(line)
        except ValueError:
            continue
        m = ev.get('message') or {}
        u = m.get('usage')
        if ev.get('type') != 'assistant' or not u or m.get('id') in seen:
            continue
        seen.add(m.get('id'))
        out.append({'prompt': u.get('input_tokens', 0) + u.get('cache_creation_input_tokens', 0)
                    + u.get('cache_read_input_tokens', 0),
                    'output': u.get('output_tokens', 0), 'cache_creation': u.get('cache_creation_input_tokens', 0)})
    return out


def answer(text):
    for m in reversed(list(re.finditer(r'\{[^{}]*\}', text or ''))):
        try:
            return json.loads(m.group(0))
        except ValueError:
            continue
    return None


def same(got, want):
    if isinstance(want, (int, float)) and not isinstance(want, bool):
        try:
            return float(got) == float(want)
        except (TypeError, ValueError):
            return False
    return got == want


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--model', default='claude-haiku-5-5')
    ap.add_argument('--reps', type=int, default=3)
    ap.add_argument('--max-cost', type=float, required=True)
    ap.add_argument('--confirm-spend', action='store_true')
    ap.add_argument('--out', default=os.path.join(REPO, 'results/ab-hook'))
    a = ap.parse_args()
    if not a.confirm_spend:
        sys.exit('this runs paid model calls; pass --confirm-spend')
    os.makedirs(a.out, exist_ok=True)
    log = os.path.join(a.out, 'runs.jsonl')
    done = {}
    if os.path.exists(log):
        for line in open(log):
            r = json.loads(line)
            done[(r['case'], r['arm'], r['rep'])] = r
    spent = sum(r['cost_usd'] for r in done.values())
    work = os.path.join(a.out, 'work')
    for rep in range(a.reps):
        for name, rows, style, qs in cases():
            for arm in ['json', 'hook']:
                key = (name, arm, rep)
                if key in done:
                    continue
                if spent >= a.max_cost:
                    sys.exit(f'stopped: ${spent:.4f} spent, cap ${a.max_cost:.2f}')
                d = os.path.join(work, f'{name}.{arm}.{rep}')
                shutil.rmtree(d, ignore_errors=True)
                os.makedirs(os.path.join(d, 'cwd'))
                with open(os.path.join(d, 'cwd', 'data.json'), 'w') as f:
                    if style == 'pretty':
                        json.dump(rows, f, ensure_ascii=False, indent=2)
                    else:
                        json.dump(rows, f, ensure_ascii=False, separators=(',', ':'))
                argv = ['claude', '-p', '--model', a.model, '--tools', 'Bash', '--strict-mcp-config',
                        '--allowedTools', 'Bash(cat data.json)', '--max-turns', '4',
                        '--max-budget-usd', '0.25', '--output-format', 'json']
                env = dict(os.environ, LDF_STATE_DIR=os.path.join(d, 'state'),
                           LDF_STATS_FILE=os.path.join(d, 'state', 'events.jsonl'))
                if arm == 'hook':
                    hook_settings(os.path.join(d, 'settings.json'))
                    argv += ['--settings', os.path.join(d, 'settings.json')]
                p = subprocess.run(argv + [prompt(qs)], cwd=os.path.join(d, 'cwd'), env=env,
                                   capture_output=True, text=True, timeout=600)
                try:
                    res = json.loads(p.stdout)
                except ValueError:
                    sys.exit(f'{name} {arm}: no JSON result: {p.stdout[-500:]} {p.stderr[-500:]}')
                sid = res.get('session_id', '')
                tr = glob.glob(os.path.expanduser(f'~/.claude/projects/*/{sid}.jsonl'))
                reqs = requests(tr[0]) if tr else []
                got = answer(res.get('result'))
                ok = [same((got or {}).get(f'q{i + 1}'), want) for i, (_, want) in enumerate(qs)]
                hook_events = []
                ev_path = os.path.join(d, 'state', 'events.jsonl')
                if os.path.exists(ev_path):
                    hook_events = [json.loads(l) for l in open(ev_path)]
                row = {'case': name, 'arm': arm, 'rep': rep, 'model': a.model,
                       'data_bytes': os.path.getsize(os.path.join(d, 'cwd', 'data.json')),
                       'cost_usd': res.get('total_cost_usd') or 0.0, 'turns': res.get('num_turns'),
                       'requests': reqs, 'correct': ok, 'answer': got,
                       'hook': [{k: e.get(k) for k in ('outcome', 'reason', 'in_bytes', 'out_bytes')} for e in hook_events],
                       'is_error': res.get('is_error')}
                with open(log, 'a') as f:
                    f.write(json.dumps(row) + '\n')
                done[key] = row
                spent += row['cost_usd']
                added = reqs[1]['prompt'] - reqs[0]['prompt'] - reqs[0]['output'] if len(reqs) >= 2 else None
                print(f'{name:30} {arm:4} rep {rep}: ${row["cost_usd"]:.4f}, requests {len(reqs)}, '
                      f'tool call added {added}, correct {ok}, total ${spent:.4f}', flush=True)
    report(done, a)


def report(done, a):
    rows = list(done.values())
    names = []
    for r in rows:
        if r['case'] not in names:
            names.append(r['case'])
    o = [f'# Bash hook A/B on provider usage, {a.model}\n',
         'Written by `eval/ab_hook.py`. Same headless Claude Code task per case, with the hook (`hook`) and without '
         '(`json`). Tokens are Anthropic-reported usage from the session transcripts, in Claude tokens. "added" is '
         'the second request\'s prompt minus the first request\'s prompt and output: what the tool call put in the '
         'context (tool result, hook note and primer, Claude Code\'s wrapping). Medians over reps.\n',
         '| case | data bytes | hook outcome | added json | added hook | hook / json | total prompt json | total prompt hook | cost json | cost hook | correct json | correct hook |',
         '|---|---:|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|']
    med = lambda v: sorted(v)[len(v) // 2] if v else 0
    tot = {'json': [0, 0, 0.0, 0, 0], 'hook': [0, 0, 0.0, 0, 0]}
    for n in names:
        by = {arm: [r for r in rows if r['case'] == n and r['arm'] == arm] for arm in ['json', 'hook']}
        def added(r):
            q = r['requests']
            return q[1]['prompt'] - q[0]['prompt'] - q[0]['output'] if len(q) >= 2 else 0
        cell = {}
        for arm, rs in by.items():
            cell[arm] = (med([added(r) for r in rs]), med([sum(q['prompt'] for q in r['requests']) for r in rs]),
                         sum(r['cost_usd'] for r in rs) / max(len(rs), 1),
                         sum(sum(r['correct']) for r in rs), sum(len(r['correct']) for r in rs))
            t = tot[arm]
            t[0] += cell[arm][0]; t[1] += cell[arm][1]; t[2] += cell[arm][2]; t[3] += cell[arm][3]; t[4] += cell[arm][4]
        outcomes = sorted({h['outcome'] + ('' if h['outcome'] == 'ldf' else f' ({h["reason"]})')
                           for r in by['hook'] for h in r['hook']}) or ['none']
        j, h = cell['json'], cell['hook']
        o.append(f'| {n} | {by["json"][0]["data_bytes"] if by["json"] else "?"} | {", ".join(outcomes)} | {j[0]} | {h[0]} | '
                 f'{(h[0] / j[0]) if j[0] else 0:.2f} | {j[1]} | {h[1]} | ${j[2]:.4f} | ${h[2]:.4f} | {j[3]}/{j[4]} | {h[3]}/{h[4]} |')
    j, h = tot['json'], tot['hook']
    o.append(f'| **all** | | | {j[0]} | {h[0]} | {(h[0] / j[0]) if j[0] else 0:.2f} | {j[1]} | {h[1]} | ${j[2]:.4f} | ${h[2]:.4f} | {j[3]}/{j[4]} | {h[3]}/{h[4]} |')
    o.append(f'\nRuns: {len(rows)}, total cost ${sum(r["cost_usd"] for r in rows):.4f} (Claude Code total_cost_usd). '
             'Costs are mean per run; cache reads and writes make cost depend on run order, the token columns do not.\n')
    with open(os.path.join(a.out, 'README.md'), 'w') as f:
        f.write('\n'.join(o) + '\n')
    print(os.path.join(a.out, 'README.md'))


if __name__ == '__main__':
    main()
