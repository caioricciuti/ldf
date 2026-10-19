# Stage B: candidates in front of Haiku 5.5 and DeepSeek chat

Run 2026-10-18, `eval/plan.sh stage-b-build stage-b-haiku stage-b-deepseek`.
Tables in `summary.md` (written by `eval/analysis/stage_b.py`); per-bundle
scorer output in `<dataset>.<model>.md`.

**Setup.** First 200 records of the seed-1 medium fixtures, four datasets
(flat_users, nested_orders, sparse_events, mixed_types), 82 to 135 questions
per dataset, the same question in every format. Formats: JSON, CSV (flat
data only), TOON, LDF v0.1, LDF v0 (where it differs from v0.1), and three
`ldf-discover` candidates from Stage A, all shown to the model as `ldf`
with no format explanation:

| name | id | what differs from v0.1 |
|---|---|---|
| ldf-hoisted | `d=comma h=typed q?=1 o=dotted a=hoisted sp=50 rf=json n=null/empty qt=json k=once fl=keep b=words tr=0` | arrays of objects inline, keys once in the header |
| ldf-tab | `d=tab h=typed q?=1 o=dotted a=child-key sp=50 rf=json n=null/empty qt=json k=once fl=keep b=words tr=0` | tab delimiter |
| ldf-global | `d=comma h=untyped q?=0 o=dotted a=hoisted sp=0 rf=json n=dash/empty qt=csv k=once fl=keep b=words tr=1` | the best single configuration of Stage A by tokens |

A rendering identical to v0.1 on a dataset was not sent. 2 641 prompts per
model; Haiku with max_tokens 4096, DeepSeek with the default 1024. Spend
from the reported usage: Haiku $1.05, DeepSeek $0.96.

**Pooled over the four datasets** (formats present in all four, 421
questions each; "vs v0.1" counts questions one format got right and the
other wrong, with an exact sign test):

| format | tokens / JSON | Haiku | vs v0.1 | DeepSeek | vs v0.1 |
|---|---:|---:|---|---:|---|
| json | 1.00 | 96.0% | +15 / -9, p=0.31 | 70.1% | +26 / -18, p=0.29 |
| toon | 1.07 | 95.2% | +16 / -13, p=0.71 | 71.7% | +27 / -12, p=0.02 |
| ldf v0.1 | 0.69 | 94.5% | | 68.2% | |
| ldf-tab | 0.71 | 94.3% | +16 / -17, p=1.00 | 70.1% | +23 / -15, p=0.26 |
| ldf-global | 0.61 | 82.4% | +9 / -60, p<0.01 | 57.7% | +14 / -58, p<0.01 |

## Findings

1. **ldf-global is rejected.** It loses 12 points on Haiku and 10 on
   DeepSeek for up to 20 percent fewer tokens than v0.1 (0 on flat_users, 7 on
   mixed_types, 15 on nested_orders, 20 on sparse_events). The cause is visible in the
   answers: with `-` as the null marker, both models return the string
   `"-"` for null values (Haiku 19 of 20 missing_value misses, DeepSeek 31
   of 33); missing_value accuracy falls from 100 to 56 percent (Haiku) and
   93 to 27 percent (DeepSeek). Its other knobs were not isolated. Null
   must stay the word `null`.
2. **v0.1 holds against JSON** on both models at 0.69 of JSON's tokens: no
   significant difference pooled (Haiku 94.5 vs 96.0, DeepSeek 68.2 vs
   70.1).
3. **DeepSeek still reads sparse rows worse than keyed ones.** On
   sparse_events DeepSeek scores JSON 60.9, TOON 62.6 (list form, keys on
   every row), v0.1 52.2, v0 48.7, CSV 47.8; JSON and TOON beat v0.1 at
   p=0.04 and 0.02. This drives TOON's pooled lead. Haiku shows no such gap
   at the 0.5 threshold (v0.1 93.9, JSON 92.2). The sparse threshold is
   model-dependent; DeepSeek needs a sweep at 0.75 and 1.0.
4. **Tab delimiter: no effect** on either model, tokens equal. Comma stays.
5. **ldf-hoisted is a tie on comprehension at 15 percent fewer tokens on
   nested_orders**, once answer shape is separated (next point): Haiku 94.8
   against v0.1's 94.1 and TOON's 94.8, JSON 96.3. On DeepSeek 60.0 against
   62.2 (+5 / -8, p=0.58). It stays a candidate for nested arrays, to be
   confirmed on held-out data.
6. **Bare string answers are scored as wrong.** All 8 of ldf-hoisted's
   compare misses on Haiku name the right order but write it unquoted
   (`ORD-000050` instead of `"ORD-000050"`); v0.1 loses 3 and TOON 2 the same
   way, JSON none. Tabular data with bare strings primes bare answers. With
   a bare final string accepted, Haiku nested_orders moves: ldf-hoisted
   88.9 to 94.8, ldf-global 75.6 to 80.7, v0.1 91.9 to 94.1, TOON 92.6 to
   94.8. DeepSeek is unaffected. The scorer should count these as a
   separate outcome, as it does for wrapped answers.
7. **child_aggregate and aggregate_where are beyond DeepSeek at 1024 output
   tokens** (0 to 7 and 13 to 33 percent in every format): it answers in
   about 5 tokens without working. Those kinds carry no format signal for
   DeepSeek; Haiku reasons through them (53 to 80 percent).

Not measured here: Opus, GPT, held-out data. Each cell is one run
(DeepSeek at temperature 0; Anthropic models take no temperature);
run-to-run variance was not measured.
