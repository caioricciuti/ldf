"""Evidence table for the format selector (`ldf-discover select`).

Regrades every responses file under eval/out with `ldf-eval score`, and for
each (model, bundle, format) compares the format against JSON on the same
questions: accuracy, wins and losses, exact two-sided sign test. JSON
answers come from the same responses file, or from a twin bundle with the
identical task list (the sparse sweep reuses the Stage B questions).

The LDF layout in a bundle is identified by re-encoding the bundle's JSON
records with `ldf-mcp convert --sparse T` for T in 0, 0.25, 0.5, 0.75, 1
and comparing with the data block of the bundle's prompts, because older
bundles labelled `ldf` hold the v0 layout. A layout no threshold
reproduces (hoisted, tab, global candidates) is family `other`, which the
selector never picks. Traits come from `ldf-discover profile` on the same
records. Standard library only.

    python3 -I eval/analysis/evidence.py eval/out docs/reports/evidence
"""
import collections, glob, hashlib, json, math, os, shutil, subprocess, sys, tempfile

root, out = sys.argv[1], sys.argv[2]
os.makedirs(out, exist_ok=True)
REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
EVAL = os.path.join(REPO, 'target/release/ldf-eval')
MCP = os.path.join(REPO, 'target/release/ldf-mcp')
DISCOVER = os.path.join(REPO, 'target/release/ldf-discover')
THRESHOLDS = [0.0, 0.25, 0.5, 0.75, 1.0]


def sign_test(b, c):
    n = b + c
    if n == 0:
        return 1.0
    k = min(b, c)
    return min(1.0, 2 * sum(math.comb(n, i) for i in range(k + 1)) / 2 ** n)


def data_blocks(bundle):
    """First data block per format, from prompts.jsonl."""
    seen = {}
    with open(os.path.join(bundle, 'prompts.jsonl'), encoding='utf-8') as f:
        for line in f:
            p = json.loads(line)
            if p['format'] in seen:
                continue
            u = p['user']
            seen[p['format']] = u[u.index('<data>\n') + 7:u.index('\n</data>')]
    return seen


def run(*cmd):
    return subprocess.run(cmd, check=True, capture_output=True, text=True).stdout


bundles = sorted({os.path.dirname(p) for p in glob.glob(f'{root}/**/responses.*.jsonl', recursive=True)})
graded = {}      # (bundle, model_run) -> {format: {task_id: record}}
meta = {}
tmp = tempfile.mkdtemp()
for b in bundles:
    m = json.load(open(os.path.join(b, 'manifest.json')))
    ds = m['dataset']
    records_path = os.path.join(b, 'data', f'{ds}.json')
    if not os.path.exists(records_path):
        print(f'skipped {b}: no JSON records in the bundle to identify layouts against', file=sys.stderr)
        continue
    traits = json.loads(run(DISCOVER, 'profile', '--input', records_path))['traits']
    blocks = data_blocks(b)
    refs = {t: run(MCP, 'convert', records_path, '--sparse', str(t)).strip('\n') for t in THRESHOLDS}
    layout = {}
    for fmt, text in blocks.items():
        text = text.strip('\n')
        if fmt == 'json':
            layout[fmt] = ('json', [])
        elif fmt == 'toon':
            layout[fmt] = ('toon', [])
        elif fmt.startswith('ldf'):
            ts = [t for t in THRESHOLDS if refs[t] == text]
            layout[fmt] = ('ldf', ts) if ts else ('other', [])
        else:
            layout[fmt] = ('other', [])
    tasks_hash = hashlib.sha256(open(os.path.join(b, 'tasks.jsonl'), 'rb').read()).hexdigest()
    tokens = dict(map(tuple, m['data_tokens_o200k']))
    meta[b] = dict(dataset=ds, records=m['record_count'], traits=[k for k, v in traits.items() if v],
                   layout=layout, tasks_hash=tasks_hash, tokens=tokens)
    for r in glob.glob(os.path.join(b, 'responses.*.jsonl')):
        name = os.path.basename(r)[len('responses.'):-len('.jsonl')]
        provider, rest = name.split('.', 1)
        # model ids contain dots (gpt-6.1-sol); a run tag is the last
        # segment only when it is not part of a known id pattern.
        model, tag = rest, ''
        if rest.endswith('.cap4096'):
            model, tag = rest[:-len('.cap4096')], 'cap4096'
        md = os.path.join(tmp, 'x.md')
        run(EVAL, 'score', '--bundle', b, '--responses', r, '--out', md)
        rows = collections.defaultdict(dict)
        for line in open(os.path.join(b, 'graded.jsonl'), encoding='utf-8'):
            x = json.loads(line)
            rows[x['format']][x['task_id']] = x
        graded[(b, model, tag)] = rows
shutil.rmtree(tmp)

json_by_twin = {}
for (b, model, tag), rows in sorted(graded.items()):
    if 'json' in rows:
        json_by_twin.setdefault((meta[b]['tasks_hash'], model, tag), (b, rows['json']))

evidence = []
for (b, model, tag), rows in sorted(graded.items()):
    mb = meta[b]
    rel = os.path.relpath(b, root)
    base = rows.get('json')
    base_from = rel
    if base is None:
        twin = json_by_twin.get((mb['tasks_hash'], model, tag))
        if twin is None:
            continue
        base_from, base = os.path.relpath(twin[0], root), twin[1]
    for fmt, r in sorted(rows.items()):
        if fmt == 'json':
            continue
        family, ts = mb['layout'].get(fmt, ('other', []))
        common = sorted(set(r) & set(base))
        ok = lambda x: x['outcome'] == 'correct'
        k = sum(ok(r[t]) for t in common)
        jk = sum(ok(base[t]) for t in common)
        wins = sum(ok(r[t]) and not ok(base[t]) for t in common)
        losses = sum(ok(base[t]) and not ok(r[t]) for t in common)
        lat = [r[t]['latency_ms'] for t in common if r[t].get('latency_ms')]
        jlat = [base[t]['latency_ms'] for t in common if base[t].get('latency_ms')]
        otok = [r[t]['output_tokens'] for t in common if r[t].get('output_tokens') is not None]
        evidence.append(dict(
            model=model, run=rel + (f' ({tag})' if tag else '') + ('' if base_from == rel else f', json from {base_from}'),
            dataset=mb['dataset'], records=mb['records'], traits=mb['traits'], family=family, thresholds=ts,
            format=fmt, n=len(common), k=k, json_k=jk, wins=wins, losses=losses, p=round(sign_test(wins, losses), 4),
            tokens_ratio=round(mb['tokens'][fmt] / mb['tokens']['json'], 3) if fmt in mb['tokens'] and 'json' in mb['tokens'] else None,
            mean_latency_ms=round(sum(lat) / len(lat)) if lat else None,
            json_mean_latency_ms=round(sum(jlat) / len(jlat)) if jlat else None,
            mean_output_tokens=round(sum(otok) / len(otok)) if otok else None,
        ))

json.dump(evidence, open(os.path.join(out, 'evidence.json'), 'w'), indent=1)
L = ['# Selector evidence (generated by eval/analysis/evidence.py)', '',
     'Each row: one model, one bundle, one format, against JSON on the same questions. '
     '`family` is what the data block really is (`ldf` with the sparse thresholds that reproduce it '
     'byte for byte, `other` for layouts outside LDF v0.1). The selector uses rows with n >= 50 by default.', '',
     '| model | run | records | traits | format | family | thresholds | n | accuracy | JSON | +/- | p | tokens / JSON | latency ms (JSON) |',
     '|---|---|---:|---|---|---|---|---:|---:|---:|---|---:|---:|---|']
for e in evidence:
    L.append(f"| {e['model']} | {e['run']} | {e['records']} | {','.join(e['traits']) or 'flat'} | {e['format']} | {e['family']} | "
             f"{','.join(str(t) for t in e['thresholds'])} | {e['n']} | {100*e['k']/max(e['n'],1):.1f}% | {100*e['json_k']/max(e['n'],1):.1f}% | "
             f"+{e['wins']} / -{e['losses']} | {e['p']:.2f} | {e['tokens_ratio'] if e['tokens_ratio'] is not None else ''} | "
             f"{e['mean_latency_ms']} ({e['json_mean_latency_ms']}) |")
open(os.path.join(out, 'evidence.md'), 'w').write('\n'.join(L) + '\n')
print('\n'.join(L))
