"""Grades the non-inferiority study (reports/2026-10-21-ni-study/PREREGISTRATION.md).

For every bundle under ROOT and every responses file in it: `ldf-eval score`
(strict, unchanged), then for each format other than JSON the paired
comparison on questions answered in both arms (an API error or a context
overflow is not an answer; anything the model returned, truncated or not,
is). Cell verdict as registered: `pass` when the one-sided 95 percent paired
score bound on the loss is at most 3 points, `fails` when the harm rule
rejects it (drop over 3 points, or a sign-test loss at p < 0.05), else
`inconclusive`. Standard library only.

    python3 -I eval/analysis/ni_study.py eval/out/ni reports/2026-10-21-ni-study
"""
import collections, glob, json, math, os, subprocess, sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from selector_audit import score_upper  # noqa: E402

REPO = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
EVAL = os.path.join(REPO, 'target/release/ldf-eval')
MARGIN = 3.0
NOT_ANSWERED = {'api_error', 'context_exceeded'}


def sign_test(w, l):
    n, k = w + l, min(w, l)
    if n == 0:
        return 1.0
    return min(1.0, 2 * sum(math.comb(n, i) for i in range(k + 1)) / 2 ** n)


def main():
    root, out = sys.argv[1], sys.argv[2]
    os.makedirs(out, exist_ok=True)
    cells = []
    for bundle in sorted(glob.glob(os.path.join(root, '*', 'manifest.json'))):
        bdir = os.path.dirname(bundle)
        ds = os.path.basename(bdir)
        for resp in sorted(glob.glob(os.path.join(bdir, 'responses.*.jsonl'))):
            model = os.path.basename(resp)[len('responses.'):-len('.jsonl')].split('.', 1)[1]
            subprocess.run([EVAL, 'score', '--bundle', bdir, '--responses', resp,
                            '--out', os.path.join(out, f'{ds}.{model}.md')], check=True, capture_output=True)
            graded = [json.loads(l) for l in open(os.path.join(bdir, 'graded.jsonl'))]
            by = collections.defaultdict(dict)
            kinds = {}
            for g in graded:
                by[g['format']][g['task_id']] = g['outcome']
                kinds[g['task_id']] = g['kind']
            if 'json' not in by:
                continue
            for fmt in sorted(f for f in by if f != 'json'):
                tasks = [t for t in by[fmt] if t in by['json']
                         and by[fmt][t] not in NOT_ANSWERED and by['json'][t] not in NOT_ANSWERED]
                n = len(tasks)
                k = sum(by[fmt][t] == 'correct' for t in tasks)
                jk = sum(by['json'][t] == 'correct' for t in tasks)
                w = sum(by[fmt][t] == 'correct' and by['json'][t] != 'correct' for t in tasks)
                l = sum(by['json'][t] == 'correct' and by[fmt][t] != 'correct' for t in tasks)
                drop = 100 * (jk - k) / n if n else 0.0
                p = sign_test(w, l)
                bound = score_upper(w, l, n) if n else 100.0
                harm = drop > MARGIN or (l > w and p < 0.05)
                verdict = 'fails' if harm else ('pass' if bound <= MARGIN else 'inconclusive')
                lost = collections.Counter(kinds[t] for t in tasks if by['json'][t] == 'correct' and by[fmt][t] != 'correct')
                shapes = collections.Counter(by[fmt][t] for t in tasks if by[fmt][t] not in ('correct', 'wrong'))
                cells.append({'model': model, 'dataset': ds, 'format': fmt, 'n': n, 'k': k, 'json_k': jk,
                              'wins': w, 'losses': l, 'drop_pp': round(drop, 2), 'p': round(p, 4),
                              'loss_bound_pp': round(bound, 2), 'verdict': verdict,
                              'losses_by_kind': dict(lost), 'other_outcomes': dict(shapes),
                              'unanswered': len(by[fmt]) - n})
    json.dump(cells, open(os.path.join(out, 'cells.json'), 'w'), indent=1)
    L = ['# Non-inferiority study: cells', '',
         f'Pass: one-sided 95 percent paired score bound on the loss at most {MARGIN:g} points. '
         'Fails: drop over 3 points or a sign-test loss at p < 0.05. Otherwise inconclusive. '
         'Grading strict (`ldf-eval score`); per-bundle scorer output in `<dataset>.<model>.md`.', '',
         '| model | dataset | format | n | correct / JSON | +w / -l | drop pp | p | loss bound pp | verdict | losses by kind | non-wrong misses |',
         '|---|---|---|---:|---|---|---:|---:|---:|---|---|---|']
    for c in sorted(cells, key=lambda c: (c['model'], c['dataset'])):
        L.append(f"| {c['model']} | {c['dataset']} | {c['format']} | {c['n']} | {c['k']} / {c['json_k']} | "
                 f"+{c['wins']} / -{c['losses']} | {c['drop_pp']:.1f} | {c['p']:.2f} | {c['loss_bound_pp']:.1f} | "
                 f"**{c['verdict']}** | {', '.join(f'{k} {v}' for k, v in sorted(c['losses_by_kind'].items())) or '-'} | "
                 f"{', '.join(f'{k} {v}' for k, v in sorted(c['other_outcomes'].items())) or '-'} |")
    open(os.path.join(out, 'cells.md'), 'w').write('\n'.join(L) + '\n')
    print('\n'.join(L))


if __name__ == '__main__':
    main()
