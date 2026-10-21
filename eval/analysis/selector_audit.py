"""Audit of the selector's acceptance rule against a non-inferiority reading.

The selector (`crates/ldf-select`) accepts an evidence row when the
accuracy drop against JSON is at most 3 points (point estimate) and the
exact sign test does not show a significant loss at 0.05. Both are tests
for harm: a row with too few questions to show anything passes. This
script asks the other question of every evidence row the policy counts
(n >= 50): how large a loss does the data still allow?

For each row, with w questions right only in the format and l right only in
JSON out of n, the loss is d = (l - w) / n. Its one-sided 95 percent upper
bound is taken as CP_upper(l, n) - CP_lower(w, n), each Clopper-Pearson at
97.5 percent (Bonferroni, exact, conservative; it does not assume anything
about the discordant pairs, so zero discordant pairs still gives a bound
above zero). A row is

- `fails` when the current policy rejects it,
- `non-inferior` when the upper bound is at most the margin,
- `inconclusive` otherwise: no detected harm, and not enough data to rule
  out a loss larger than the margin.

The score bound is the paired score (Tango) interval, the usual test for
non-inferiority on matched pairs. It is tighter than the exact bound and
is the one a pre-registered study should use; both are reported.

Standard library only.

    python3 -I eval/analysis/selector_audit.py reports/evidence/evidence.json > results/selector-audit.md
"""
import json, math, sys

ALPHA = 0.05
MIN_N = 50
MAX_DROP = 3.0
MARGINS = [3.0, 5.0]


def binom_cdf(k, n, p):
    if k < 0:
        return 0.0
    if k >= n:
        return 1.0
    if p <= 0:
        return 1.0
    if p >= 1:
        return 0.0
    lp, lq = math.log(p), math.log1p(-p)
    s, lc = 0.0, 0.0
    for i in range(k + 1):
        if i:
            lc += math.log(n - i + 1) - math.log(i)
        s += math.exp(lc + i * lp + (n - i) * lq)
    return min(s, 1.0)


def cp_upper(k, n, a):
    """Smallest p with P(X <= k) <= a."""
    if k >= n:
        return 1.0
    lo, hi = k / n, 1.0
    for _ in range(80):
        mid = (lo + hi) / 2
        if binom_cdf(k, n, mid) > a:
            lo = mid
        else:
            hi = mid
    return hi


def cp_lower(k, n, a):
    """Largest p with P(X >= k) <= a."""
    if k <= 0:
        return 0.0
    return 1.0 - cp_upper(n - k, n, a)


def loss_upper(w, l, n):
    return 100 * (cp_upper(l, n, ALPHA / 2) - cp_lower(w, n, ALPHA / 2))


def score_upper(w, l, n, z=1.6448536269514722):
    """One-sided 95 percent upper bound on the loss (p_l - p_w) from the
    paired score test (Tango 1998): the largest loss d whose restricted
    maximum-likelihood fit still gives Z >= -z for the observed counts."""
    t = n - w - l

    def zstat(d):
        # Restricted MLE of p_w with p_l = p_w + d, by golden section on the
        # multinomial log-likelihood.
        lo, hi = max(0.0, -d), (1.0 - d) / 2
        if hi <= lo:
            return float('-inf')

        def ll(pw):
            pl, pt = pw + d, 1 - 2 * pw - d
            if min(pw, pl, pt) < 0:
                return float('-inf')
            f = lambda k, p: 0.0 if k == 0 else (k * math.log(p) if p > 0 else float('-inf'))
            return f(w, pw) + f(l, pl) + f(t, pt)

        g = (math.sqrt(5) - 1) / 2
        a, b = lo, hi
        for _ in range(200):
            c, e = b - g * (b - a), a + g * (b - a)
            if ll(c) < ll(e):
                a = c
            else:
                b = e
        pw = (a + b) / 2
        var = (2 * pw + d - d * d) / n
        return ((l - w) / n - d) / math.sqrt(var) if var > 0 else float('-inf')

    lo, hi = (l - w) / n, 1.0
    for _ in range(100):
        mid = (lo + hi) / 2
        if zstat(mid) >= -z:
            lo = mid
        else:
            hi = mid
    return 100 * lo


def zero_loss_n(margin):
    """Questions needed for a perfect tie (w = l = 0) to show the margin."""
    n = 1
    while loss_upper(0, 0, n) > margin:
        n += 1
    return n


def score_n(margin):
    n = 1
    while score_upper(0, 0, n) > margin:
        n += 1
    return n


def policy_passes(e):
    drop = 100 * (e['json_k'] - e['k']) / e['n']
    return drop <= MAX_DROP and not (e['losses'] > e['wins'] and e['p'] < ALPHA)


def main():
    ev = [e for e in json.load(open(sys.argv[1])) if e['family'] in ('ldf', 'toon') and e['n'] >= MIN_N]
    print('# Selector acceptance audit\n')
    print(f'Evidence rows the policy counts (ldf and toon families, n >= {MIN_N}): {len(ev)}.')
    for m in MARGINS:
        print(f'A perfect tie (no question differs) shows a loss of at most {m:g} points only from n = {zero_loss_n(m)} questions '
              f'(exact bound) or n = {score_n(m)} (score bound).')
    print('\n| model | run | dataset | records | format | n | k / JSON | +w / -l | drop pp | policy | exact bound pp | score bound pp | exact at 3 pp | score at 3 pp | exact at 5 pp |')
    print('|---|---|---|---:|---|---:|---|---|---:|---|---:|---:|---|---|---|')
    tally = {}
    for e in sorted(ev, key=lambda e: (e['model'], e['dataset'], e['records'], e['format'])):
        drop = 100 * (e['json_k'] - e['k']) / e['n']
        ub = loss_upper(e['wins'], e['losses'], e['n'])
        sb = score_upper(e['wins'], e['losses'], e['n'])
        ok = policy_passes(e)
        cls = lambda m, b=ub: 'fails' if not ok else ('non-inferior' if b <= m else 'inconclusive')
        for m in MARGINS:
            tally[(m, cls(m))] = tally.get((m, cls(m)), 0) + 1
            tally[('score', m, cls(m, sb))] = tally.get(('score', m, cls(m, sb)), 0) + 1
        print(f"| {e['model']} | {e['run']} | {e['dataset']} | {e['records']} | {e['format']} | {e['n']} | "
              f"{e['k']} / {e['json_k']} | +{e['wins']} / -{e['losses']} | {drop:.1f} | {'pass' if ok else 'fail'} | "
              f"{ub:.1f} | {sb:.1f} | {cls(3.0)} | {cls(3.0, sb)} | {cls(5.0)} |")
    print()
    for m in MARGINS:
        parts = ', '.join(f'{c} {tally.get((m, c), 0)}' for c in ('non-inferior', 'inconclusive', 'fails'))
        print(f'- margin {m:g} pp, exact bound: {parts}')
        parts = ', '.join(f"{c} {tally.get(('score', m, c), 0)}" for c in ('non-inferior', 'inconclusive', 'fails'))
        print(f'- margin {m:g} pp, score bound: {parts}')


if __name__ == '__main__':
    main()
