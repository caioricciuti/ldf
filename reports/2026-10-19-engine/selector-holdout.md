# Selector decisions on held-out data (generated)

fixtures/holdout/medium (seed 20261118), first 200 records, default policy, evidence reports/evidence/evidence.json. Command: `ldf-discover select --model M --input fixtures/holdout/medium/D.json --sample 200`.

| dataset | traits | claude-haiku-5-5 | deepseek-chat | claude-opus-5-5 | gpt-6.1-sol | unknown model |
|---|---|---|---|---|---|---|
| flat_users | flat | ldf (sparse threshold 0.5) | ldf (sparse threshold 0.5) | json | json | json |
| nested_orders | child_tables,sparse | json | ldf (sparse threshold 0.5) | json | json | json |
| sparse_events | sparse | ldf (sparse threshold 0.5) | ldf (sparse threshold 0.75) | json | json | json |
| mixed_types | json_cells,sparse | ldf (sparse threshold 0.5) | ldf (sparse threshold 0.5) | json | json | json |

Full reasoning for each decision:

```
model claude-haiku-5-5, 200 rows, traits {} (min fill 1.00)
  json                                 9381 o200k  accepted  round trip byte
      baseline and fallback
  toon                                 6086 o200k  accepted  round trip semantic
      5 evidence rows with under 50 questions ignored
      ok: toon on flat_users (200 records, stage-b/flat_users): 86/89 vs JSON 87/89, +2 / -3, p=1.00
  ldf (sparse threshold 0.5)           5701 o200k  accepted  round trip byte
      2 evidence rows with under 50 questions ignored
      ok: ldf on flat_users (200 records, stage-b/flat_users): 87/89 vs JSON 87/89, +2 / -2, p=1.00
  ldf (sparse threshold 0.75)          5701 o200k  accepted  round trip byte
      2 evidence rows with under 50 questions ignored
      ok: ldf on flat_users (200 records, stage-b/flat_users): 87/89 vs JSON 87/89, +2 / -2, p=1.00
choice: ldf (sparse threshold 0.5)
```
```
model claude-haiku-5-5, 200 rows, traits {"child_tables", "sparse"} (min fill 0.72)
  json                                20069 o200k  accepted  round trip byte
      baseline and fallback
  toon                                21665 o200k  rejected  round trip semantic
      5 evidence rows with under 50 questions ignored
      toon on nested_orders (200 records): 3.7 points below JSON (125/135 vs 130/135), tolerance 3
  ldf (sparse threshold 0.5)          12989 o200k  rejected  round trip byte
      2 evidence rows with under 50 questions ignored
      ldf on nested_orders (200 records): 4.4 points below JSON (124/135 vs 130/135), tolerance 3
  ldf (sparse threshold 0.75)         13677 o200k  rejected  round trip byte
      2 evidence rows with under 50 questions ignored
      no evidence for ldf (sparse threshold 0.75) on a measured dataset with traits {"child_tables", "sparse"}
choice: json
```
```
model claude-haiku-5-5, 200 rows, traits {"sparse"} (min fill 0.10)
  json                                10898 o200k  accepted  round trip byte
      baseline and fallback
  toon                                13328 o200k  rejected  round trip byte
      5 evidence rows with under 50 questions ignored
      ok: toon on sparse_events (200 records, stage-b/sparse_events): 108/115 vs JSON 106/115, +6 / -4, p=0.75
      1.22 of JSON's tokens here, saving under 5 percent
  ldf (sparse threshold 0.5)           8726 o200k  accepted  round trip semantic-unordered
      2 evidence rows with under 50 questions ignored
      ok: ldf on sparse_events (200 records, stage-b/sparse_events): 108/115 vs JSON 106/115, +6 / -4, p=0.75
  ldf (sparse threshold 0.75)          9677 o200k  rejected  round trip byte
      2 evidence rows with under 50 questions ignored
      no evidence for ldf (sparse threshold 0.75) on a measured dataset with traits {"sparse"}
choice: ldf (sparse threshold 0.5)
```
```
model claude-haiku-5-5, 200 rows, traits {"json_cells", "sparse"} (min fill 0.49)
  json                                 4951 o200k  accepted  round trip byte
      baseline and fallback
  toon                                 6604 o200k  rejected  round trip byte
      5 evidence rows with under 50 questions ignored
      ok: toon on mixed_types (200 records, stage-b/mixed_types): 82/82 vs JSON 81/82, +1 / -0, p=1.00
      1.33 of JSON's tokens here, saving under 5 percent
  ldf (sparse threshold 0.5)           3616 o200k  accepted  round trip semantic-unordered
      2 evidence rows with under 50 questions ignored
      ok: ldf on mixed_types (200 records, stage-b/mixed_types): 79/82 vs JSON 81/82, +0 / -2, p=0.50
  ldf (sparse threshold 0.75)          3616 o200k  accepted  round trip semantic-unordered
      2 evidence rows with under 50 questions ignored
      ok: ldf on mixed_types (200 records, stage-b/mixed_types): 79/82 vs JSON 81/82, +0 / -2, p=0.50
choice: ldf (sparse threshold 0.5)
```
```
model deepseek-chat, 200 rows, traits {} (min fill 1.00)
  json                                 9381 o200k  accepted  round trip byte
      baseline and fallback
  toon                                 6086 o200k  accepted  round trip semantic
      4 evidence rows with under 50 questions ignored
      ok: toon on flat_users (200 records, stage-b/flat_users): 61/89 vs JSON 59/89, +7 / -5, p=0.77
  ldf (sparse threshold 0.5)           5701 o200k  accepted  round trip byte
      2 evidence rows with under 50 questions ignored
      ok: ldf on flat_users (200 records, stage-b/flat_users): 61/89 vs JSON 59/89, +7 / -5, p=0.77
  ldf (sparse threshold 0.75)          5701 o200k  accepted  round trip byte
      2 evidence rows with under 50 questions ignored
      ok: ldf on flat_users (200 records, stage-b/flat_users): 61/89 vs JSON 59/89, +7 / -5, p=0.77
choice: ldf (sparse threshold 0.5)
```
```
model deepseek-chat, 200 rows, traits {"child_tables", "sparse"} (min fill 0.72)
  json                                20069 o200k  accepted  round trip byte
      baseline and fallback
  toon                                21665 o200k  rejected  round trip semantic
      4 evidence rows with under 50 questions ignored
      ok: toon on nested_orders (200 records, stage-b/nested_orders): 87/135 vs JSON 84/135, +6 / -3, p=0.51
      1.08 of JSON's tokens here, saving under 5 percent
  ldf (sparse threshold 0.5)          12989 o200k  accepted  round trip byte
      2 evidence rows with under 50 questions ignored
      ok: ldf on nested_orders (200 records, stage-b/nested_orders): 84/135 vs JSON 84/135, +6 / -6, p=1.00
  ldf (sparse threshold 0.75)         13677 o200k  rejected  round trip byte
      2 evidence rows with under 50 questions ignored
      no evidence for ldf (sparse threshold 0.75) on a measured dataset with traits {"child_tables", "sparse"}
choice: ldf (sparse threshold 0.5)
```
```
model deepseek-chat, 200 rows, traits {"sparse"} (min fill 0.10)
  json                                10898 o200k  accepted  round trip byte
      baseline and fallback
  toon                                13328 o200k  rejected  round trip byte
      4 evidence rows with under 50 questions ignored
      ok: toon on sparse_events (200 records, stage-b/sparse_events): 72/115 vs JSON 70/115, +6 / -4, p=0.75
      1.22 of JSON's tokens here, saving under 5 percent
  ldf (sparse threshold 0.5)           8726 o200k  rejected  round trip semantic-unordered
      2 evidence rows with under 50 questions ignored
      ldf on sparse_events (200 records): 8.7 points below JSON (60/115 vs 70/115), tolerance 3
  ldf (sparse threshold 0.75)          9677 o200k  accepted  round trip byte
      2 evidence rows with under 50 questions ignored
      ok: ldf-s75 on sparse_events (200 records, sparse-sweep/sparse_events, json from stage-b/sparse_events): 76/115 vs JSON 70/115, +8 / -2, p=0.11
choice: ldf (sparse threshold 0.75)
```
```
model deepseek-chat, 200 rows, traits {"json_cells", "sparse"} (min fill 0.49)
  json                                 4951 o200k  accepted  round trip byte
      baseline and fallback
  toon                                 6604 o200k  rejected  round trip byte
      4 evidence rows with under 50 questions ignored
      ok: toon on mixed_types (200 records, stage-b/mixed_types): 82/82 vs JSON 82/82, +0 / -0, p=1.00
      1.33 of JSON's tokens here, saving under 5 percent
  ldf (sparse threshold 0.5)           3616 o200k  accepted  round trip semantic-unordered
      2 evidence rows with under 50 questions ignored
      ok: ldf on mixed_types (200 records, stage-b/mixed_types): 82/82 vs JSON 82/82, +0 / -0, p=1.00
  ldf (sparse threshold 0.75)          3616 o200k  accepted  round trip semantic-unordered
      2 evidence rows with under 50 questions ignored
      ok: ldf on mixed_types (200 records, stage-b/mixed_types): 82/82 vs JSON 82/82, +0 / -0, p=1.00
choice: ldf (sparse threshold 0.5)
```
```
model claude-opus-5-5, 200 rows, traits {} (min fill 1.00)
  json                                 9381 o200k  accepted  round trip byte
      baseline and fallback
  toon                                 6086 o200k  rejected  round trip semantic
      4 evidence rows with under 50 questions ignored
      no evidence for model claude-opus-5-5 with at least 50 questions
  ldf (sparse threshold 0.5)           5701 o200k  rejected  round trip byte
      3 evidence rows with under 50 questions ignored
      no evidence for model claude-opus-5-5 with at least 50 questions
  ldf (sparse threshold 0.75)          5701 o200k  rejected  round trip byte
      2 evidence rows with under 50 questions ignored
      no evidence for model claude-opus-5-5 with at least 50 questions
choice: json
```
```
model claude-opus-5-5, 200 rows, traits {"child_tables", "sparse"} (min fill 0.72)
  json                                20069 o200k  accepted  round trip byte
      baseline and fallback
  toon                                21665 o200k  rejected  round trip semantic
      4 evidence rows with under 50 questions ignored
      no evidence for model claude-opus-5-5 with at least 50 questions
  ldf (sparse threshold 0.5)          12989 o200k  rejected  round trip byte
      3 evidence rows with under 50 questions ignored
      no evidence for model claude-opus-5-5 with at least 50 questions
  ldf (sparse threshold 0.75)         13677 o200k  rejected  round trip byte
      2 evidence rows with under 50 questions ignored
      no evidence for model claude-opus-5-5 with at least 50 questions
choice: json
```
```
model claude-opus-5-5, 200 rows, traits {"sparse"} (min fill 0.10)
  json                                10898 o200k  accepted  round trip byte
      baseline and fallback
  toon                                13328 o200k  rejected  round trip byte
      4 evidence rows with under 50 questions ignored
      no evidence for model claude-opus-5-5 with at least 50 questions
  ldf (sparse threshold 0.5)           8726 o200k  rejected  round trip semantic-unordered
      3 evidence rows with under 50 questions ignored
      no evidence for model claude-opus-5-5 with at least 50 questions
  ldf (sparse threshold 0.75)          9677 o200k  rejected  round trip byte
      2 evidence rows with under 50 questions ignored
      no evidence for model claude-opus-5-5 with at least 50 questions
choice: json
```
```
model claude-opus-5-5, 200 rows, traits {"json_cells", "sparse"} (min fill 0.49)
  json                                 4951 o200k  accepted  round trip byte
      baseline and fallback
  toon                                 6604 o200k  rejected  round trip byte
      4 evidence rows with under 50 questions ignored
      no evidence for model claude-opus-5-5 with at least 50 questions
  ldf (sparse threshold 0.5)           3616 o200k  rejected  round trip semantic-unordered
      3 evidence rows with under 50 questions ignored
      no evidence for model claude-opus-5-5 with at least 50 questions
  ldf (sparse threshold 0.75)          3616 o200k  rejected  round trip semantic-unordered
      2 evidence rows with under 50 questions ignored
      no evidence for model claude-opus-5-5 with at least 50 questions
choice: json
```
