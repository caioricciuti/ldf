# Sparse threshold sweep, DeepSeek chat

Run 2026-10-19, `eval/plan.sh sparse-sweep-build sparse-sweep-deepseek`
($0.044). Same 200 records and 115 questions as the Stage B sparse_events
bundle; LDF v0.1 with the sparse threshold at 0.75 (`ldf-s75`). Threshold
1.0 gives the identical document on this dataset (no key between 60 and 100
percent fill) and was not sent. Other rows are DeepSeek's Stage B answers
(`docs/reports/2026-10-18-stage-b`), graded with the same scorer. The 0.75
answers come from a separate run; run-to-run variance is not measured.

| format | threshold | tokens / JSON | accuracy | vs v0.1 (0.5) | vs JSON |
|---|---|---:|---:|---|---|
| ldf-v0 | 0 | 0.64 | 48.7% | +5 / -9, p=0.42 | +3 / -17, p<0.01 |
| ldf v0.1 | 0.5 | 0.78 | 52.2% | | +5 / -15, p=0.04 |
| ldf-s75 | 0.75 | 0.89 | 66.1% | +19 / -3, p<0.01 | +8 / -2, p=0.11 |
| json | | 1.00 | 60.9% | +15 / -5, p=0.04 | |
| toon | | 1.22 | 62.6% | +17 / -5, p=0.02 | +6 / -4, p=0.75 |
| csv | | 0.61 | 47.8% (lossy) | +4 / -9, p=0.27 | +1 / -16, p<0.01 |

By kind (accuracy):

| kind | ldf-v0 | ldf | ldf-s75 | json | toon |
|---|---:|---:|---:|---:|---:|
| lookup | 100% | 100% | 100% | 100% | 100% |
| filter | 13% | 13% | 7% | 7% | 7% |
| aggregate | 27% | 33% | 47% | 33% | 33% |
| compare | 93% | 87% | 87% | 87% | 93% |
| missing_value | 93% | 80% | 93% | 87% | 93% |
| multi_record | 20% | 30% | 90% | 60% | 80% |
| aggregate_where | 20% | 27% | 47% | 47% | 27% |
| rank | 13% | 40% | 67% | 67% | 73% |

Reading: keys next to their values matter for DeepSeek on sparse rows. At
0.75 every key filled in under 75 percent of rows moves to the `~` object,
and DeepSeek goes from below JSON to above it at 0.89 of JSON's tokens. The
gain is in questions that rank or aggregate one field across all rows.
Haiku showed the same direction earlier (0.75 got 18 of 18 on the older
bundle). Candidate for the v0.2 default or for per-model choice in the
optimizer; Stage C (held-out data) not run yet.
