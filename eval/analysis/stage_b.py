"""Stage B analysis: grades every responses file under a Stage B bundle
directory with `ldf-eval score`, then writes per-dataset accuracy with
Wilson intervals, paired comparisons against LDF v0.1 (exact sign test on
the questions where exactly one of the two formats is right), pooled
results and accuracy by task kind. Standard library only.

    python3 -I eval/analysis/stage_b.py eval/out/stage-b results/stage-b
"""
import json, sys, os, math, subprocess, itertools, collections
base, out = sys.argv[1], sys.argv[2]
os.makedirs(out, exist_ok=True)
DATASETS = ['flat_users', 'nested_orders', 'sparse_events', 'mixed_types']
MODELS = {'anthropic.claude-haiku-5-5': 'haiku', 'deepseek.deepseek-chat': 'deepseek'}
rows = []  # (dataset, model, format, task_id, kind, correct, out_tokens, outcome)
data_tokens = {}
for d in DATASETS:
    m = json.load(open(f'{base}/{d}/manifest.json'))
    for f, t in m['data_tokens_o200k']:
        data_tokens[(d, f)] = t
    for key, model in MODELS.items():
        resp = f'{base}/{d}/responses.{key}.jsonl'
        if not os.path.exists(resp): continue
        g = f'{out}/{d}.{model}.graded.jsonl'
        subprocess.run(['./target/release/ldf-eval', 'score', '--bundle', f'{base}/{d}', '--responses', resp, '--out', f'{out}/{d}.{model}.md'], check=True, capture_output=True)
        os.replace(f'{base}/{d}/graded.jsonl', g)
        for l in open(g):
            x = json.loads(l)
            rows.append((d, model, x['format'], x['task_id'], x['kind'], x['outcome'] == 'correct', x.get('output_tokens') or 0, x['outcome']))

def wilson(k, n, z=1.96):
    if n == 0: return (0, 0)
    p = k / n; den = 1 + z*z/n; c = (p + z*z/(2*n)) / den; h = z*math.sqrt(p*(1-p)/n + z*z/(4*n*n)) / den
    return (c - h, c + h)

def binom_two_sided(b, c):
    # exact sign test on discordant pairs
    n = b + c
    if n == 0: return 1.0
    k = min(b, c)
    p = sum(math.comb(n, i) for i in range(0, k + 1)) / 2**n
    return min(1.0, 2 * p)

idx = collections.defaultdict(dict)
for d, m, f, t, k, ok, ot, oc in rows:
    idx[(d, m, f)][t] = (ok, ot, k, oc)

lines = []
P = lines.append
P('# Stage B results (generated)\n')
for model in ['haiku', 'deepseek']:
    P(f'\n## {model}\n')
    P('| dataset | format | data tokens | / json | accuracy | 95% CI | mean out | vs ldf v0.1: +/- (p) | truncated |')
    P('|---|---|---:|---:|---:|---|---:|---|---:|')
    for d in DATASETS:
        fmts = sorted({f for (dd, mm, f) in idx if dd == d and mm == model}, key=lambda f: ['json','csv','toon','ldf','ldf-v0','ldf-hoisted','ldf-tab','ldf-global'].index(f))
        base_ldf = idx.get((d, model, 'ldf'), {})
        for f in fmts:
            r = idx[(d, model, f)]
            n = len(r); k = sum(v[0] for v in r.values())
            lo, hi = wilson(k, n)
            mo = sum(v[1] for v in r.values()) / max(n, 1)
            tr = sum(1 for v in r.values() if v[3] == 'truncated')
            cmp = ''
            if f != 'ldf' and base_ldf:
                common = set(r) & set(base_ldf)
                b = sum(1 for t in common if r[t][0] and not base_ldf[t][0])
                c = sum(1 for t in common if not r[t][0] and base_ldf[t][0])
                cmp = f'+{b} / -{c} (p={binom_two_sided(b, c):.2f})'
            P(f'| {d} | {f} | {data_tokens[(d,f)]} | {data_tokens[(d,f)]/data_tokens[(d,"json")]:.2f} | {100*k/n:.1f}% ({k}/{n}) | {100*lo:.0f}-{100*hi:.0f} | {mo:.0f} | {cmp} | {tr} |')
    # pooled over datasets for formats present everywhere
    P(f'\n### {model}: pooled over the four datasets (formats present in all)\n')
    P('| format | accuracy | n | vs ldf: +/- (p) | mean out tokens | data tokens / json (mean) |')
    P('|---|---:|---:|---|---:|---:|')
    allf = ['json', 'toon', 'ldf', 'ldf-tab', 'ldf-global']
    for f in allf:
        ks = ns = b = c = ot = 0; ratios = []
        for d in DATASETS:
            r = idx.get((d, model, f), {}); L = idx.get((d, model, 'ldf'), {})
            ns += len(r); ks += sum(v[0] for v in r.values()); ot += sum(v[1] for v in r.values())
            ratios.append(data_tokens[(d, f)] / data_tokens[(d, 'json')])
            if f != 'ldf':
                for t in set(r) & set(L):
                    b += r[t][0] and not L[t][0]; c += (not r[t][0]) and L[t][0]
        cmp = '' if f == 'ldf' else f'+{b} / -{c} (p={binom_two_sided(b, c):.2f})'
        P(f'| {f} | {100*ks/ns:.1f}% | {ns} | {cmp} | {ot/ns:.0f} | {sum(ratios)/len(ratios):.2f} |')
    # by kind, pooled, for key comparisons
    P(f'\n### {model}: accuracy by task kind, pooled over datasets\n')
    kinds = ['lookup','filter','aggregate','compare','missing_value','nested_extract','multi_record','child_count','child_aggregate','aggregate_where','rank']
    fl = ['json','csv','toon','ldf','ldf-v0','ldf-hoisted','ldf-tab','ldf-global']
    P('| kind | ' + ' | '.join(fl) + ' |'); P('|---|' + '---:|'*len(fl))
    for kd in kinds:
        cells = []
        for f in fl:
            v = [x for (d, mm, ff), r in idx.items() if mm == model and ff == f for x in r.values() if x[2] == kd]
            cells.append(f'{100*sum(x[0] for x in v)/len(v):.0f}% ({len(v)})' if v else '-')
        P(f'| {kd} | ' + ' | '.join(cells) + ' |')
open(f'{out}/summary.md', 'w').write('\n'.join(lines) + '\n')
print('\n'.join(lines))
