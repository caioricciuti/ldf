# ldf-bench results

Started 20261017-220957 UTC. Command: `./target/release/ldf-bench tokens --scale small --out results/small`

Machine: Apple M2 Pro (aarch64), 10 logical CPUs, 16.0 GiB RAM, Darwin 27.0.0. release profile, commit a7bcfcb.

Toolchain: rustc 1.98.0 (88d9e12ae 2026-08-18). Crates: arrow 60.0.0, criterion 0.8.2, csv 1.4.0, parquet 60.0.0, serde_json 1.0.151, simd-json 0.18.1, tiktoken-rs 0.12.1.

Each operation runs in its own child process. Iteration 0 is the cold run; medians and p95 are over the remaining 6 warm iterations. MB = 1e6 bytes. Peak RSS is the child process high-water mark after the operation, as reported by getrusage, minus nothing: it includes the loaded input. See site/src/content/docs/docs/methodology.md.

## flat_users (20 records, 2.7 KiB as json)

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 2.7 KiB | 1.00 | - | - | - | - | - | - | - | - |
| json-pretty | byte | 3.7 KiB | 1.39 | - | - | - | - | - | - | - | - |
| jsonl | byte | 2.7 KiB | 1.00 | - | - | - | - | - | - | - | - |
| csv | byte | 1.2 KiB | 0.46 | - | - | - | - | - | - | - | - |
| tsv | byte | 1.2 KiB | 0.46 | - | - | - | - | - | - | - | - |
| markdown | byte | 1.6 KiB | 0.61 | - | - | - | - | - | - | - | - |
| toon | byte | 1.3 KiB | 0.47 | - | - | - | - | - | - | - | - |
| toon-tab | byte | 1.3 KiB | 0.47 | - | - | - | - | - | - | - | - |
| yaml | byte | 2.6 KiB | 0.99 | - | - | - | - | - | - | - | - |
| ldf | byte | 1.3 KiB | 0.47 | - | - | - | - | - | - | - | - |

### Tokens (20 records)

| format | bytes | cl100k_base | vs json | bytes/token | o200k_base | vs json | bytes/token |
|---|---:|---:|---:|---:|---:|---:|---:|
| json | 2.7 KiB | 948 | 1.00 | 2.90 | 944 | 1.00 | 2.91 |
| json-pretty | 3.7 KiB | 1487 | 1.57 | 2.57 | 1483 | 1.57 | 2.58 |
| jsonl | 2.7 KiB | 965 | 1.02 | 2.85 | 961 | 1.02 | 2.86 |
| csv | 1.2 KiB | 580 | 0.61 | 2.17 | 580 | 0.61 | 2.17 |
| tsv | 1.2 KiB | 576 | 0.61 | 2.18 | 578 | 0.61 | 2.17 |
| markdown | 1.6 KiB | 731 | 0.77 | 2.31 | 726 | 0.77 | 2.32 |
| toon | 1.3 KiB | 623 | 0.66 | 2.09 | 623 | 0.66 | 2.09 |
| toon-tab | 1.3 KiB | 620 | 0.65 | 2.10 | 622 | 0.66 | 2.10 |
| yaml | 2.6 KiB | 1177 | 1.24 | 2.30 | 1172 | 1.24 | 2.31 |
| ldf | 1.3 KiB | 590 | 0.62 | 2.17 | 590 | 0.62 | 2.17 |

Claude token counts are not in this table: Anthropic does not publish its tokenizer. Run `eval/count_tokens_claude.py` against the encoded files in `encoded/` to add exact counts from the count_tokens API.

## nested_orders (20 records, 5.9 KiB as json)

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 5.9 KiB | 1.00 | - | - | - | - | - | - | - | - |
| json-pretty | byte | 10.4 KiB | 1.77 | - | - | - | - | - | - | - | - |
| jsonl | byte | 5.9 KiB | 1.00 | - | - | - | - | - | - | - | - |
| csv | unsupported: record 0 field "customer" is a object (nested) | | | | | | | | | | |
| tsv | unsupported: record 0 field "customer" is a object (nested) | | | | | | | | | | |
| markdown | unsupported: record 0 field "customer" is a object (nested) | | | | | | | | | | |
| toon | semantic | 6.0 KiB | 1.03 | - | - | - | - | - | - | - | - |
| toon-tab | semantic | 6.1 KiB | 1.04 | - | - | - | - | - | - | - | - |
| yaml | byte | 6.6 KiB | 1.13 | - | - | - | - | - | - | - | - |
| ldf | byte | 2.9 KiB | 0.50 | - | - | - | - | - | - | - | - |

### Tokens (20 records)

| format | bytes | cl100k_base | vs json | bytes/token | o200k_base | vs json | bytes/token |
|---|---:|---:|---:|---:|---:|---:|---:|
| json | 5.9 KiB | 2022 | 1.00 | 2.97 | 2021 | 1.00 | 2.97 |
| json-pretty | 10.4 KiB | 3600 | 1.78 | 2.95 | 3579 | 1.77 | 2.97 |
| jsonl | 5.9 KiB | 2039 | 1.01 | 2.94 | 2038 | 1.01 | 2.94 |
| csv | unsupported: record 0 field "customer" is a object (nested) |
| tsv | unsupported: record 0 field "customer" is a object (nested) |
| markdown | unsupported: record 0 field "customer" is a object (nested) |
| toon | 6.0 KiB | 2177 | 1.08 | 2.84 | 2171 | 1.07 | 2.85 |
| toon-tab | 6.1 KiB | 2178 | 1.08 | 2.85 | 2172 | 1.07 | 2.86 |
| yaml | 6.6 KiB | 2539 | 1.26 | 2.67 | 2533 | 1.25 | 2.68 |
| ldf | 2.9 KiB | 1362 | 0.67 | 2.18 | 1357 | 0.67 | 2.19 |

Claude token counts are not in this table: Anthropic does not publish its tokenizer. Run `eval/count_tokens_claude.py` against the encoded files in `encoded/` to add exact counts from the count_tokens API.

## sparse_events (20 records, 2.9 KiB as json)

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 2.9 KiB | 1.00 | - | - | - | - | - | - | - | - |
| json-pretty | byte | 4.0 KiB | 1.36 | - | - | - | - | - | - | - | - |
| jsonl | byte | 2.9 KiB | 1.00 | - | - | - | - | - | - | - | - |
| csv | lossy (0: object size 15 vs 9) | 1.5 KiB | 0.50 | - | - | - | - | - | - | - | - |
| tsv | lossy (0: object size 15 vs 9) | 1.5 KiB | 0.50 | - | - | - | - | - | - | - | - |
| markdown | lossy (0: object size 15 vs 9) | 2.2 KiB | 0.75 | - | - | - | - | - | - | - | - |
| toon | byte | 3.3 KiB | 1.11 | - | - | - | - | - | - | - | - |
| toon-tab | byte | 3.3 KiB | 1.11 | - | - | - | - | - | - | - | - |
| yaml | byte | 2.9 KiB | 0.99 | - | - | - | - | - | - | - | - |
| ldf | semantic-unordered (object key order changed) | 1.6 KiB | 0.55 | - | - | - | - | - | - | - | - |

### Tokens (20 records)

| format | bytes | cl100k_base | vs json | bytes/token | o200k_base | vs json | bytes/token |
|---|---:|---:|---:|---:|---:|---:|---:|
| json | 2.9 KiB | 1087 | 1.00 | 2.76 | 1092 | 1.00 | 2.75 |
| json-pretty | 4.0 KiB | 1613 | 1.48 | 2.53 | 1618 | 1.48 | 2.52 |
| jsonl | 2.9 KiB | 1104 | 1.02 | 2.72 | 1109 | 1.02 | 2.71 |
| csv | 1.5 KiB | 712 | 0.66 | 2.11 | 711 | 0.65 | 2.11 |
| tsv | 1.5 KiB | 703 | 0.65 | 2.14 | 699 | 0.64 | 2.15 |
| markdown | 2.2 KiB | 1148 | 1.06 | 1.97 | 1143 | 1.05 | 1.98 |
| toon | 3.3 KiB | 1337 | 1.23 | 2.49 | 1332 | 1.22 | 2.50 |
| toon-tab | 3.3 KiB | 1338 | 1.23 | 2.49 | 1333 | 1.22 | 2.50 |
| yaml | 2.9 KiB | 1310 | 1.21 | 2.26 | 1305 | 1.20 | 2.27 |
| ldf | 1.6 KiB | 755 | 0.69 | 2.18 | 753 | 0.69 | 2.18 |

Claude token counts are not in this table: Anthropic does not publish its tokenizer. Run `eval/count_tokens_claude.py` against the encoded files in `encoded/` to add exact counts from the count_tokens API.

## high_card_strings (20 records, 4.6 KiB as json)

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 4.6 KiB | 1.00 | - | - | - | - | - | - | - | - |
| json-pretty | byte | 5.4 KiB | 1.18 | - | - | - | - | - | - | - | - |
| jsonl | byte | 4.6 KiB | 1.00 | - | - | - | - | - | - | - | - |
| csv | byte | 3.6 KiB | 0.78 | - | - | - | - | - | - | - | - |
| tsv | byte | 3.6 KiB | 0.78 | - | - | - | - | - | - | - | - |
| markdown | byte | 3.9 KiB | 0.85 | - | - | - | - | - | - | - | - |
| toon | byte | 3.7 KiB | 0.80 | - | - | - | - | - | - | - | - |
| toon-tab | byte | 3.7 KiB | 0.80 | - | - | - | - | - | - | - | - |
| yaml | byte | 4.5 KiB | 0.97 | - | - | - | - | - | - | - | - |
| ldf | byte | 3.6 KiB | 0.78 | - | - | - | - | - | - | - | - |

### Tokens (20 records)

| format | bytes | cl100k_base | vs json | bytes/token | o200k_base | vs json | bytes/token |
|---|---:|---:|---:|---:|---:|---:|---:|
| json | 4.6 KiB | 2307 | 1.00 | 2.04 | 2288 | 1.00 | 2.05 |
| json-pretty | 5.4 KiB | 2746 | 1.19 | 2.02 | 2727 | 1.19 | 2.03 |
| jsonl | 4.6 KiB | 2324 | 1.01 | 2.02 | 2305 | 1.01 | 2.04 |
| csv | 3.6 KiB | 2042 | 0.89 | 1.80 | 2024 | 0.88 | 1.81 |
| tsv | 3.6 KiB | 2035 | 0.88 | 1.80 | 2020 | 0.88 | 1.82 |
| markdown | 3.9 KiB | 2161 | 0.94 | 1.85 | 2141 | 0.94 | 1.87 |
| toon | 3.7 KiB | 2094 | 0.91 | 1.79 | 2076 | 0.91 | 1.81 |
| toon-tab | 3.7 KiB | 2099 | 0.91 | 1.79 | 2084 | 0.91 | 1.80 |
| yaml | 4.5 KiB | 2475 | 1.07 | 1.85 | 2455 | 1.07 | 1.87 |
| ldf | 3.6 KiB | 2050 | 0.89 | 1.80 | 2032 | 0.89 | 1.82 |

Claude token counts are not in this table: Anthropic does not publish its tokenizer. Run `eval/count_tokens_claude.py` against the encoded files in `encoded/` to add exact counts from the count_tokens API.

## repetitive_logs (20 records, 2.8 KiB as json)

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 2.8 KiB | 1.00 | - | - | - | - | - | - | - | - |
| json-pretty | byte | 3.8 KiB | 1.33 | - | - | - | - | - | - | - | - |
| jsonl | byte | 2.8 KiB | 1.00 | - | - | - | - | - | - | - | - |
| csv | byte | 1.4 KiB | 0.49 | - | - | - | - | - | - | - | - |
| tsv | byte | 1.4 KiB | 0.49 | - | - | - | - | - | - | - | - |
| markdown | byte | 1.8 KiB | 0.62 | - | - | - | - | - | - | - | - |
| toon | byte | 1.5 KiB | 0.52 | - | - | - | - | - | - | - | - |
| toon-tab | byte | 1.5 KiB | 0.52 | - | - | - | - | - | - | - | - |
| yaml | byte | 2.7 KiB | 0.97 | - | - | - | - | - | - | - | - |
| ldf | byte | 1.4 KiB | 0.50 | - | - | - | - | - | - | - | - |

### Tokens (20 records)

| format | bytes | cl100k_base | vs json | bytes/token | o200k_base | vs json | bytes/token |
|---|---:|---:|---:|---:|---:|---:|---:|
| json | 2.8 KiB | 918 | 1.00 | 3.15 | 918 | 1.00 | 3.15 |
| json-pretty | 3.8 KiB | 1418 | 1.54 | 2.71 | 1418 | 1.54 | 2.71 |
| jsonl | 2.8 KiB | 936 | 1.02 | 3.08 | 936 | 1.02 | 3.08 |
| csv | 1.4 KiB | 596 | 0.65 | 2.38 | 600 | 0.65 | 2.36 |
| tsv | 1.4 KiB | 566 | 0.62 | 2.50 | 566 | 0.62 | 2.50 |
| markdown | 1.8 KiB | 704 | 0.77 | 2.55 | 704 | 0.77 | 2.55 |
| toon | 1.5 KiB | 640 | 0.70 | 2.35 | 644 | 0.70 | 2.33 |
| toon-tab | 1.5 KiB | 630 | 0.69 | 2.39 | 630 | 0.69 | 2.39 |
| yaml | 2.7 KiB | 1093 | 1.19 | 2.55 | 1093 | 1.19 | 2.55 |
| ldf | 1.4 KiB | 605 | 0.66 | 2.38 | 609 | 0.66 | 2.36 |

Claude token counts are not in this table: Anthropic does not publish its tokenizer. Run `eval/count_tokens_claude.py` against the encoded files in `encoded/` to add exact counts from the count_tokens API.

## unicode_strings (20 records, 2.5 KiB as json)

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 2.5 KiB | 1.00 | - | - | - | - | - | - | - | - |
| json-pretty | byte | 3.4 KiB | 1.38 | - | - | - | - | - | - | - | - |
| jsonl | byte | 2.5 KiB | 1.00 | - | - | - | - | - | - | - | - |
| csv | lossy (0.empty: null vs "") | 1.2 KiB | 0.47 | - | - | - | - | - | - | - | - |
| tsv | lossy (0.empty: null vs "") | 1.2 KiB | 0.47 | - | - | - | - | - | - | - | - |
| markdown | lossy (0.empty: null vs "") | 1.5 KiB | 0.62 | - | - | - | - | - | - | - | - |
| toon | byte | 1.3 KiB | 0.54 | - | - | - | - | - | - | - | - |
| toon-tab | byte | 1.3 KiB | 0.53 | - | - | - | - | - | - | - | - |
| yaml | byte | 2.5 KiB | 0.99 | - | - | - | - | - | - | - | - |
| ldf | byte | 1.3 KiB | 0.51 | - | - | - | - | - | - | - | - |

### Tokens (20 records)

| format | bytes | cl100k_base | vs json | bytes/token | o200k_base | vs json | bytes/token |
|---|---:|---:|---:|---:|---:|---:|---:|
| json | 2.5 KiB | 895 | 1.00 | 2.85 | 845 | 1.00 | 3.01 |
| json-pretty | 3.4 KiB | 1366 | 1.53 | 2.57 | 1297 | 1.53 | 2.70 |
| jsonl | 2.5 KiB | 913 | 1.02 | 2.79 | 863 | 1.02 | 2.95 |
| csv | 1.2 KiB | 530 | 0.59 | 2.28 | 460 | 0.54 | 2.62 |
| tsv | 1.2 KiB | 552 | 0.62 | 2.19 | 490 | 0.58 | 2.47 |
| markdown | 1.5 KiB | 660 | 0.74 | 2.39 | 595 | 0.70 | 2.65 |
| toon | 1.3 KiB | 606 | 0.68 | 2.25 | 538 | 0.64 | 2.54 |
| toon-tab | 1.3 KiB | 665 | 0.74 | 2.05 | 620 | 0.73 | 2.20 |
| yaml | 2.5 KiB | 1037 | 1.16 | 2.43 | 965 | 1.14 | 2.61 |
| ldf | 1.3 KiB | 565 | 0.63 | 2.32 | 497 | 0.59 | 2.64 |

Claude token counts are not in this table: Anthropic does not publish its tokenizer. Run `eval/count_tokens_claude.py` against the encoded files in `encoded/` to add exact counts from the count_tokens API.

## numeric_mix (20 records, 4.7 KiB as json)

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 4.7 KiB | 1.00 | - | - | - | - | - | - | - | - |
| json-pretty | byte | 6.3 KiB | 1.35 | - | - | - | - | - | - | - | - |
| jsonl | byte | 4.7 KiB | 1.00 | - | - | - | - | - | - | - | - |
| csv | semantic | 2.7 KiB | 0.59 | - | - | - | - | - | - | - | - |
| tsv | semantic | 2.7 KiB | 0.59 | - | - | - | - | - | - | - | - |
| markdown | semantic | 3.4 KiB | 0.73 | - | - | - | - | - | - | - | - |
| toon | semantic | 2.8 KiB | 0.60 | - | - | - | - | - | - | - | - |
| toon-tab | semantic | 2.8 KiB | 0.60 | - | - | - | - | - | - | - | - |
| yaml | byte | 4.9 KiB | 1.04 | - | - | - | - | - | - | - | - |
| ldf | byte | 2.8 KiB | 0.59 | - | - | - | - | - | - | - | - |

### Tokens (20 records)

| format | bytes | cl100k_base | vs json | bytes/token | o200k_base | vs json | bytes/token |
|---|---:|---:|---:|---:|---:|---:|---:|
| json | 4.7 KiB | 1959 | 1.00 | 2.45 | 2013 | 1.00 | 2.38 |
| json-pretty | 6.3 KiB | 2779 | 1.42 | 2.33 | 2799 | 1.39 | 2.31 |
| jsonl | 4.7 KiB | 1977 | 1.01 | 2.42 | 2031 | 1.01 | 2.36 |
| csv | 2.7 KiB | 1349 | 0.69 | 2.08 | 1350 | 0.67 | 2.08 |
| tsv | 2.7 KiB | 1381 | 0.70 | 2.04 | 1382 | 0.69 | 2.03 |
| markdown | 3.4 KiB | 1658 | 0.85 | 2.10 | 1658 | 0.82 | 2.10 |
| toon | 2.8 KiB | 1392 | 0.71 | 2.05 | 1393 | 0.69 | 2.05 |
| toon-tab | 2.8 KiB | 1425 | 0.73 | 2.01 | 1426 | 0.71 | 2.00 |
| yaml | 4.9 KiB | 2421 | 1.24 | 2.06 | 2421 | 1.20 | 2.06 |
| ldf | 2.8 KiB | 1364 | 0.70 | 2.08 | 1365 | 0.68 | 2.08 |

Claude token counts are not in this table: Anthropic does not publish its tokenizer. Run `eval/count_tokens_claude.py` against the encoded files in `encoded/` to add exact counts from the count_tokens API.

## mixed_types (20 records, 1.4 KiB as json)

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 1.4 KiB | 1.00 | - | - | - | - | - | - | - | - |
| json-pretty | byte | 2.6 KiB | 1.90 | - | - | - | - | - | - | - | - |
| jsonl | byte | 1.4 KiB | 1.00 | - | - | - | - | - | - | - | - |
| csv | unsupported: record 0 field "value" is a object (nested) | | | | | | | | | | |
| tsv | unsupported: record 0 field "value" is a object (nested) | | | | | | | | | | |
| markdown | unsupported: record 0 field "value" is a object (nested) | | | | | | | | | | |
| toon | byte | 1.7 KiB | 1.22 | - | - | - | - | - | - | - | - |
| toon-tab | byte | 1.7 KiB | 1.22 | - | - | - | - | - | - | - | - |
| yaml | byte | 1.6 KiB | 1.12 | - | - | - | - | - | - | - | - |
| ldf | byte | 914 B | 0.64 | - | - | - | - | - | - | - | - |

### Tokens (20 records)

| format | bytes | cl100k_base | vs json | bytes/token | o200k_base | vs json | bytes/token |
|---|---:|---:|---:|---:|---:|---:|---:|
| json | 1.4 KiB | 479 | 1.00 | 2.98 | 492 | 1.00 | 2.90 |
| json-pretty | 2.6 KiB | 991 | 2.07 | 2.73 | 991 | 2.01 | 2.73 |
| jsonl | 1.4 KiB | 485 | 1.01 | 2.94 | 500 | 1.02 | 2.85 |
| csv | unsupported: record 0 field "value" is a object (nested) |
| tsv | unsupported: record 0 field "value" is a object (nested) |
| markdown | unsupported: record 0 field "value" is a object (nested) |
| toon | 1.7 KiB | 653 | 1.36 | 2.65 | 655 | 1.33 | 2.65 |
| toon-tab | 1.7 KiB | 661 | 1.38 | 2.63 | 661 | 1.34 | 2.63 |
| yaml | 1.6 KiB | 668 | 1.39 | 2.38 | 668 | 1.36 | 2.38 |
| ldf | 914 B | 334 | 0.70 | 2.74 | 346 | 0.70 | 2.64 |

Claude token counts are not in this table: Anthropic does not publish its tokenizer. Run `eval/count_tokens_claude.py` against the encoded files in `encoded/` to add exact counts from the count_tokens API.

## wide_table (20 records, 10.5 KiB as json)

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 10.5 KiB | 1.00 | - | - | - | - | - | - | - | - |
| json-pretty | byte | 15.4 KiB | 1.47 | - | - | - | - | - | - | - | - |
| jsonl | byte | 10.5 KiB | 1.00 | - | - | - | - | - | - | - | - |
| csv | lossy (0.s01: null vs "null") | 5.5 KiB | 0.52 | - | - | - | - | - | - | - | - |
| tsv | lossy (0.s01: null vs "null") | 5.5 KiB | 0.52 | - | - | - | - | - | - | - | - |
| markdown | lossy (0.s01: null vs "null") | 7.4 KiB | 0.71 | - | - | - | - | - | - | - | - |
| toon | semantic | 5.5 KiB | 0.53 | - | - | - | - | - | - | - | - |
| toon-tab | semantic | 5.5 KiB | 0.53 | - | - | - | - | - | - | - | - |
| yaml | byte | 10.9 KiB | 1.04 | - | - | - | - | - | - | - | - |
| ldf | byte | 5.6 KiB | 0.53 | - | - | - | - | - | - | - | - |

### Tokens (20 records)

| format | bytes | cl100k_base | vs json | bytes/token | o200k_base | vs json | bytes/token |
|---|---:|---:|---:|---:|---:|---:|---:|
| json | 10.5 KiB | 4686 | 1.00 | 2.30 | 4686 | 1.00 | 2.30 |
| json-pretty | 15.4 KiB | 7026 | 1.50 | 2.25 | 7026 | 1.50 | 2.25 |
| jsonl | 10.5 KiB | 4704 | 1.00 | 2.29 | 4704 | 1.00 | 2.29 |
| csv | 5.5 KiB | 2069 | 0.44 | 2.70 | 2078 | 0.44 | 2.69 |
| tsv | 5.5 KiB | 2014 | 0.43 | 2.77 | 2018 | 0.43 | 2.77 |
| markdown | 7.4 KiB | 2882 | 0.62 | 2.64 | 2882 | 0.62 | 2.64 |
| toon | 5.5 KiB | 2130 | 0.45 | 2.66 | 2138 | 0.46 | 2.65 |
| toon-tab | 5.5 KiB | 2109 | 0.45 | 2.69 | 2113 | 0.45 | 2.68 |
| yaml | 10.9 KiB | 5937 | 1.27 | 1.88 | 5937 | 1.27 | 1.88 |
| ldf | 5.6 KiB | 2134 | 0.46 | 2.68 | 2142 | 0.46 | 2.67 |

Claude token counts are not in this table: Anthropic does not publish its tokenizer. Run `eval/count_tokens_claude.py` against the encoded files in `encoded/` to add exact counts from the count_tokens API.

## deep_nesting (10 records, 3.3 KiB as json)

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 3.3 KiB | 1.00 | - | - | - | - | - | - | - | - |
| json-pretty | byte | 14.2 KiB | 4.27 | - | - | - | - | - | - | - | - |
| jsonl | byte | 3.3 KiB | 1.00 | - | - | - | - | - | - | - | - |
| csv | unsupported: record 0 field "tree" is a object (nested) | | | | | | | | | | |
| tsv | unsupported: record 0 field "tree" is a object (nested) | | | | | | | | | | |
| markdown | unsupported: record 0 field "tree" is a object (nested) | | | | | | | | | | |
| toon | byte | 8.6 KiB | 2.59 | - | - | - | - | - | - | - | - |
| toon-tab | byte | 8.7 KiB | 2.63 | - | - | - | - | - | - | - | - |
| yaml | byte | 9.0 KiB | 2.70 | - | - | - | - | - | - | - | - |
| ldf | byte | 2.5 KiB | 0.74 | - | - | - | - | - | - | - | - |

### Tokens (10 records)

| format | bytes | cl100k_base | vs json | bytes/token | o200k_base | vs json | bytes/token |
|---|---:|---:|---:|---:|---:|---:|---:|
| json | 3.3 KiB | 1338 | 1.00 | 2.55 | 1309 | 1.00 | 2.60 |
| json-pretty | 14.2 KiB | 3175 | 2.37 | 4.58 | 3175 | 2.43 | 4.58 |
| jsonl | 3.3 KiB | 1336 | 1.00 | 2.55 | 1316 | 1.01 | 2.59 |
| csv | unsupported: record 0 field "tree" is a object (nested) |
| tsv | unsupported: record 0 field "tree" is a object (nested) |
| markdown | unsupported: record 0 field "tree" is a object (nested) |
| toon | 8.6 KiB | 2345 | 1.75 | 3.76 | 2345 | 1.79 | 3.76 |
| toon-tab | 8.7 KiB | 2476 | 1.85 | 3.61 | 2476 | 1.89 | 3.61 |
| yaml | 9.0 KiB | 2413 | 1.80 | 3.82 | 2413 | 1.84 | 3.82 |
| ldf | 2.5 KiB | 1110 | 0.83 | 2.27 | 1100 | 0.84 | 2.30 |

Claude token counts are not in this table: Anthropic does not publish its tokenizer. Run `eval/count_tokens_claude.py` against the encoded files in `encoded/` to add exact counts from the count_tokens API.

## empty_values (20 records, 3.8 KiB as json)

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 3.8 KiB | 1.00 | - | - | - | - | - | - | - | - |
| json-pretty | byte | 5.6 KiB | 1.48 | - | - | - | - | - | - | - | - |
| jsonl | byte | 3.8 KiB | 1.00 | - | - | - | - | - | - | - | - |
| csv | unsupported: record 0 field "empty_a" is a array (nested) | | | | | | | | | | |
| tsv | unsupported: record 0 field "empty_a" is a array (nested) | | | | | | | | | | |
| markdown | unsupported: record 0 field "empty_a" is a array (nested) | | | | | | | | | | |
| toon | byte | 4.5 KiB | 1.19 | - | - | - | - | - | - | - | - |
| toon-tab | byte | 4.5 KiB | 1.19 | - | - | - | - | - | - | - | - |
| yaml | byte | 3.9 KiB | 1.04 | - | - | - | - | - | - | - | - |
| ldf | byte | 1.3 KiB | 0.35 | - | - | - | - | - | - | - | - |

### Tokens (20 records)

| format | bytes | cl100k_base | vs json | bytes/token | o200k_base | vs json | bytes/token |
|---|---:|---:|---:|---:|---:|---:|---:|
| json | 3.8 KiB | 1208 | 1.00 | 3.20 | 1191 | 1.00 | 3.25 |
| json-pretty | 5.6 KiB | 1948 | 1.61 | 2.94 | 1931 | 1.62 | 2.97 |
| jsonl | 3.8 KiB | 1226 | 1.01 | 3.16 | 1209 | 1.02 | 3.20 |
| csv | unsupported: record 0 field "empty_a" is a array (nested) |
| tsv | unsupported: record 0 field "empty_a" is a array (nested) |
| markdown | unsupported: record 0 field "empty_a" is a array (nested) |
| toon | 4.5 KiB | 1533 | 1.27 | 2.99 | 1515 | 1.27 | 3.03 |
| toon-tab | 4.5 KiB | 1534 | 1.27 | 2.99 | 1516 | 1.27 | 3.03 |
| yaml | 3.9 KiB | 1465 | 1.21 | 2.74 | 1447 | 1.21 | 2.77 |
| ldf | 1.3 KiB | 463 | 0.38 | 2.91 | 483 | 0.41 | 2.79 |

Claude token counts are not in this table: Anthropic does not publish its tokenizer. Run `eval/count_tokens_claude.py` against the encoded files in `encoded/` to add exact counts from the count_tokens API.

## long_text (10 records, 11.9 KiB as json)

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 11.9 KiB | 1.00 | - | - | - | - | - | - | - | - |
| json-pretty | byte | 12.2 KiB | 1.02 | - | - | - | - | - | - | - | - |
| jsonl | byte | 11.9 KiB | 1.00 | - | - | - | - | - | - | - | - |
| csv | byte | 11.6 KiB | 0.97 | - | - | - | - | - | - | - | - |
| tsv | byte | 11.6 KiB | 0.97 | - | - | - | - | - | - | - | - |
| markdown | byte | 11.7 KiB | 0.98 | - | - | - | - | - | - | - | - |
| toon | byte | 11.6 KiB | 0.98 | - | - | - | - | - | - | - | - |
| toon-tab | byte | 11.6 KiB | 0.98 | - | - | - | - | - | - | - | - |
| yaml | byte | 11.9 KiB | 1.00 | - | - | - | - | - | - | - | - |
| ldf | byte | 11.6 KiB | 0.97 | - | - | - | - | - | - | - | - |

### Tokens (10 records)

| format | bytes | cl100k_base | vs json | bytes/token | o200k_base | vs json | bytes/token |
|---|---:|---:|---:|---:|---:|---:|---:|
| json | 11.9 KiB | 2218 | 1.00 | 5.51 | 2216 | 1.00 | 5.52 |
| json-pretty | 12.2 KiB | 2365 | 1.07 | 5.30 | 2363 | 1.07 | 5.30 |
| jsonl | 11.9 KiB | 2226 | 1.00 | 5.49 | 2224 | 1.00 | 5.50 |
| csv | 11.6 KiB | 2085 | 0.94 | 5.69 | 2090 | 0.94 | 5.68 |
| tsv | 11.6 KiB | 2090 | 0.94 | 5.68 | 2093 | 0.94 | 5.67 |
| markdown | 11.7 KiB | 2164 | 0.98 | 5.54 | 2159 | 0.97 | 5.55 |
| toon | 11.6 KiB | 2148 | 0.97 | 5.55 | 2148 | 0.97 | 5.55 |
| toon-tab | 11.6 KiB | 2157 | 0.97 | 5.53 | 2155 | 0.97 | 5.53 |
| yaml | 11.9 KiB | 2254 | 1.02 | 5.41 | 2250 | 1.02 | 5.42 |
| ldf | 11.6 KiB | 2130 | 0.96 | 5.59 | 2130 | 0.96 | 5.59 |

Claude token counts are not in this table: Anthropic does not publish its tokenizer. Run `eval/count_tokens_claude.py` against the encoded files in `encoded/` to add exact counts from the count_tokens API.

