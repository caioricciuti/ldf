# Stage C: Opus 5.5 on held-out data

Run 2026-10-19, `eval/plan.sh stage-c10-build stage-c10-opus`. Per-bundle
scorer output in `<dataset>.opus.md`; the evidence rows are in
`docs/reports/evidence/` (runs `stage-c10/*`).

**Setup.** First 200 records of the held-out medium fixtures
(`ldf-gen --seed 20261118`), question seed 2, so neither the records nor
the questions were seen when the candidates were chosen. Four datasets,
10 questions per kind (57 to 95 per dataset; 8 per kind would have left
mixed_types at 47, under the selector's minimum of 50). Formats: JSON,
LDF v0.1 (sparse threshold 0.5), LDF at threshold 0.75 where it differs
(nested_orders, sparse_events) and `ldf-hoisted` on nested_orders (arrays
of objects inline, keys once in the header; id in `eval/plan.sh`). No TOON:
`ldf-mcp` never sends it. All LDF variants are shown to the model as `ldf`
with no explanation. Claude Opus 5.5, effort low, max_tokens 4096, data
block prompt-cached. 864 prompts, no API errors, no truncation; $5.96 from
the reported usage at list prices (cap $7).

**Results.** Accuracy, and questions one format got right and JSON wrong
(+) or the reverse (-), with an exact sign test. Tokens are Claude input
tokens per prompt from the usage, relative to JSON (o200k in brackets, from
the evidence table).

| dataset | format | tokens / JSON | accuracy | vs JSON |
|---|---|---:|---:|---|
| flat_users | json | 1.00 | 65/65 | |
| flat_users | ldf | 0.57 (0.61) | 65/65 | +0 / -0 |
| nested_orders | json | 1.00 | 94/95 | |
| nested_orders | ldf | 0.57 (0.65) | 94/95 | +1 / -1, p=1.00 |
| nested_orders | ldf at 0.75 | 0.62 (0.68) | 95/95 | +1 / -0, p=1.00 |
| nested_orders | ldf-hoisted | 0.47 (0.55) | 93/95 | +1 / -2, p=1.00 |
| sparse_events | json | 1.00 | 78/80 | |
| sparse_events | ldf | 0.76 (0.80) | 78/80 | +2 / -2, p=1.00 |
| sparse_events | ldf at 0.75 | 0.86 (0.89) | 76/80 | +2 / -4, p=0.69 |
| mixed_types | json | 1.00 | 57/57 | |
| mixed_types | ldf | 0.77 (0.73) | 57/57 | +0 / -0 |

Pooled, LDF v0.1 and JSON both get 294 of 297 questions right.

## Findings

1. **Opus reads LDF v0.1 as well as JSON on unseen data, at 57 to 77
   percent of JSON's input tokens.** Every cell has at least 57 paired
   questions, so the selector now counts them: `ldf-discover select` and
   `ldf-mcp --model claude-opus-5-5` pick LDF at threshold 0.5 for flat,
   nested (child tables with sparse keys), sparse and mixed-type data, on
   the held-out fixtures and on the small ones, where they used to pick
   JSON for lack of evidence.
2. **Threshold 0.75 brings Opus nothing.** On sparse_events it costs 12
   percent more tokens than 0.5 and lands two questions lower (not
   significant); on nested_orders it is one question higher (not
   significant). DeepSeek gained 14 points from it in the sparse sweep. The
   threshold is a per-model choice, which the selector already makes; the
   v0.1 default of 0.5 stays.
3. **Hoisted arrays hold up on a third model.** 17 percent fewer Claude
   tokens than v0.1 on nested_orders for one question fewer (+1 / -2,
   p=1.00; one of the two is a *bare* answer, the right value without its
   JSON quotes). With Haiku tied and DeepSeek two questions lower (not
   significant) in Stage B, no model has shown a significant loss. It is
   the strongest v0.2 candidate. The selector cannot pick it yet: it is not
   a v0.1 document (family `other`).
4. **Gap in the selector: exact trait match.** The held-out nested_orders
   has a top-level key filled in 72 percent of rows, so it counts as
   `child_tables` + `sparse`; the seed-1 small fixture's lowest fill is 80
   percent, so it is `child_tables` only, has no evidence, and Opus gets
   JSON there. Opus ties JSON on both shapes wherever measured, so this is
   the policy being conservative, not a measured risk.

**Limits.** One model, 200 records, 57 to 95 questions per cell (a single
answer moves a cell by 1 to 2 points). GPT still has no cell with 50
questions and gets JSON.
