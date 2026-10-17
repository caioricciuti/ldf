# ldf-bench results

Started 20261017-211646 UTC. Command: `./target/release/ldf-bench tokens --scale medium --sample 1000 --out results/medium`

Machine: Apple M2 Pro (aarch64), 10 logical CPUs, 16.0 GiB RAM, Darwin 27.0.0. release profile, commit 41bf21d.

Toolchain: rustc 1.98.0 (88d9e12ae 2026-08-18). Crates: arrow 60.0.0, criterion 0.8.2, csv 1.4.0, parquet 60.0.0, serde_json 1.0.151, simd-json 0.18.1, tiktoken-rs 0.12.1.

Each operation runs in its own child process. Iteration 0 is the cold run; medians and p95 are over the remaining 6 warm iterations. MB = 1e6 bytes. Peak RSS is the child process high-water mark after the operation, as reported by getrusage, minus nothing: it includes the loaded input. See docs/benchmark-methodology.md.

## flat_users (1000 records, 1.4 MiB as json)

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 136.5 KiB | 1.00 | - | - | - | - | - | - | - | - |
| json-pretty | byte | 189.2 KiB | 1.39 | - | - | - | - | - | - | - | - |
| jsonl | byte | 136.5 KiB | 1.00 | - | - | - | - | - | - | - | - |
| csv | semantic | 61.3 KiB | 0.45 | - | - | - | - | - | - | - | - |
| tsv | semantic | 61.3 KiB | 0.45 | - | - | - | - | - | - | - | - |
| markdown | semantic | 79.0 KiB | 0.58 | - | - | - | - | - | - | - | - |
| toon | semantic | 63.3 KiB | 0.46 | - | - | - | - | - | - | - | - |
| toon-tab | semantic | 63.3 KiB | 0.46 | - | - | - | - | - | - | - | - |
| yaml | byte | 134.6 KiB | 0.99 | - | - | - | - | - | - | - | - |

### Tokens (1000 records)

| format | bytes | cl100k_base | vs json | bytes/token | o200k_base | vs json | bytes/token |
|---|---:|---:|---:|---:|---:|---:|---:|
| json | 136.5 KiB | 47109 | 1.00 | 2.97 | 46854 | 1.00 | 2.98 |
| json-pretty | 189.2 KiB | 74108 | 1.57 | 2.61 | 73853 | 1.58 | 2.62 |
| jsonl | 136.5 KiB | 48106 | 1.02 | 2.91 | 47851 | 1.02 | 2.92 |
| csv | 61.3 KiB | 28321 | 0.60 | 2.22 | 28345 | 0.60 | 2.22 |
| tsv | 61.3 KiB | 28227 | 0.60 | 2.23 | 28254 | 0.60 | 2.22 |
| markdown | 79.0 KiB | 34859 | 0.74 | 2.32 | 34473 | 0.74 | 2.35 |
| toon | 63.3 KiB | 30325 | 0.64 | 2.14 | 30349 | 0.65 | 2.14 |
| toon-tab | 63.3 KiB | 30232 | 0.64 | 2.14 | 30259 | 0.65 | 2.14 |
| yaml | 134.6 KiB | 58884 | 1.25 | 2.34 | 58498 | 1.25 | 2.36 |

Claude token counts are not in this table: Anthropic does not publish its tokenizer. Run `eval/count_tokens_claude.py` against the encoded files in `encoded/` to add exact counts from the count_tokens API.

## nested_orders (1000 records, 2.8 MiB as json)

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 289.2 KiB | 1.00 | - | - | - | - | - | - | - | - |
| json-pretty | byte | 512.0 KiB | 1.77 | - | - | - | - | - | - | - | - |
| jsonl | byte | 289.2 KiB | 1.00 | - | - | - | - | - | - | - | - |
| csv | unsupported: record 0 field "customer" is a object (nested) | | | | | | | | | | |
| tsv | unsupported: record 0 field "customer" is a object (nested) | | | | | | | | | | |
| markdown | unsupported: record 0 field "customer" is a object (nested) | | | | | | | | | | |
| toon | semantic | 299.8 KiB | 1.04 | - | - | - | - | - | - | - | - |
| toon-tab | semantic | 300.8 KiB | 1.04 | - | - | - | - | - | - | - | - |
| yaml | byte | 327.3 KiB | 1.13 | - | - | - | - | - | - | - | - |

### Tokens (1000 records)

| format | bytes | cl100k_base | vs json | bytes/token | o200k_base | vs json | bytes/token |
|---|---:|---:|---:|---:|---:|---:|---:|
| json | 289.2 KiB | 99403 | 1.00 | 2.98 | 99948 | 1.00 | 2.96 |
| json-pretty | 512.0 KiB | 177344 | 1.78 | 2.96 | 176889 | 1.77 | 2.96 |
| jsonl | 289.2 KiB | 100401 | 1.01 | 2.95 | 100946 | 1.01 | 2.93 |
| csv | unsupported: record 0 field "customer" is a object (nested) |
| tsv | unsupported: record 0 field "customer" is a object (nested) |
| markdown | unsupported: record 0 field "customer" is a object (nested) |
| toon | 299.8 KiB | 108171 | 1.09 | 2.84 | 108056 | 1.08 | 2.84 |
| toon-tab | 300.8 KiB | 108172 | 1.09 | 2.85 | 108057 | 1.08 | 2.85 |
| yaml | 327.3 KiB | 125581 | 1.26 | 2.67 | 125466 | 1.26 | 2.67 |

Claude token counts are not in this table: Anthropic does not publish its tokenizer. Run `eval/count_tokens_claude.py` against the encoded files in `encoded/` to add exact counts from the count_tokens API.

## sparse_events (1000 records, 1.5 MiB as json)

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 148.7 KiB | 1.00 | - | - | - | - | - | - | - | - |
| json-pretty | byte | 202.3 KiB | 1.36 | - | - | - | - | - | - | - | - |
| jsonl | byte | 148.7 KiB | 1.00 | - | - | - | - | - | - | - | - |
| csv | lossy (0: object size 15 vs 9) | 67.6 KiB | 0.45 | - | - | - | - | - | - | - | - |
| tsv | lossy (0: object size 15 vs 9) | 67.6 KiB | 0.45 | - | - | - | - | - | - | - | - |
| markdown | lossy (0: object size 15 vs 9) | 99.0 KiB | 0.67 | - | - | - | - | - | - | - | - |
| toon | semantic | 165.2 KiB | 1.11 | - | - | - | - | - | - | - | - |
| toon-tab | semantic | 165.2 KiB | 1.11 | - | - | - | - | - | - | - | - |
| yaml | byte | 146.8 KiB | 0.99 | - | - | - | - | - | - | - | - |

### Tokens (1000 records)

| format | bytes | cl100k_base | vs json | bytes/token | o200k_base | vs json | bytes/token |
|---|---:|---:|---:|---:|---:|---:|---:|
| json | 148.7 KiB | 54783 | 1.00 | 2.78 | 54975 | 1.00 | 2.77 |
| json-pretty | 202.3 KiB | 81779 | 1.49 | 2.53 | 81971 | 1.49 | 2.53 |
| jsonl | 148.7 KiB | 55781 | 1.02 | 2.73 | 55973 | 1.02 | 2.72 |
| csv | 67.6 KiB | 33410 | 0.61 | 2.07 | 33321 | 0.61 | 2.08 |
| tsv | 67.6 KiB | 33162 | 0.61 | 2.09 | 33046 | 0.60 | 2.09 |
| markdown | 99.0 KiB | 53460 | 0.98 | 1.90 | 53313 | 0.97 | 1.90 |
| toon | 165.2 KiB | 67434 | 1.23 | 2.51 | 67287 | 1.22 | 2.51 |
| toon-tab | 165.2 KiB | 67435 | 1.23 | 2.51 | 67288 | 1.22 | 2.51 |
| yaml | 146.8 KiB | 66165 | 1.21 | 2.27 | 66018 | 1.20 | 2.28 |

Claude token counts are not in this table: Anthropic does not publish its tokenizer. Run `eval/count_tokens_claude.py` against the encoded files in `encoded/` to add exact counts from the count_tokens API.

## high_card_strings (1000 records, 2.3 MiB as json)

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 230.9 KiB | 1.00 | - | - | - | - | - | - | - | - |
| json-pretty | byte | 271.9 KiB | 1.18 | - | - | - | - | - | - | - | - |
| jsonl | byte | 230.9 KiB | 1.00 | - | - | - | - | - | - | - | - |
| csv | byte | 179.1 KiB | 0.78 | - | - | - | - | - | - | - | - |
| tsv | byte | 179.1 KiB | 0.78 | - | - | - | - | - | - | - | - |
| markdown | byte | 192.9 KiB | 0.84 | - | - | - | - | - | - | - | - |
| toon | byte | 183.1 KiB | 0.79 | - | - | - | - | - | - | - | - |
| toon-tab | byte | 183.1 KiB | 0.79 | - | - | - | - | - | - | - | - |
| yaml | byte | 225.0 KiB | 0.97 | - | - | - | - | - | - | - | - |

### Tokens (1000 records)

| format | bytes | cl100k_base | vs json | bytes/token | o200k_base | vs json | bytes/token |
|---|---:|---:|---:|---:|---:|---:|---:|
| json | 230.9 KiB | 115761 | 1.00 | 2.04 | 114671 | 1.00 | 2.06 |
| json-pretty | 271.9 KiB | 137760 | 1.19 | 2.02 | 136670 | 1.19 | 2.04 |
| jsonl | 230.9 KiB | 116758 | 1.01 | 2.02 | 115668 | 1.01 | 2.04 |
| csv | 179.1 KiB | 102391 | 0.88 | 1.79 | 101408 | 0.88 | 1.81 |
| tsv | 179.1 KiB | 101995 | 0.88 | 1.80 | 101001 | 0.88 | 1.82 |
| markdown | 192.9 KiB | 107219 | 0.93 | 1.84 | 106130 | 0.93 | 1.86 |
| toon | 183.1 KiB | 104703 | 0.90 | 1.79 | 103656 | 0.90 | 1.81 |
| toon-tab | 183.1 KiB | 105000 | 0.91 | 1.79 | 104006 | 0.91 | 1.80 |
| yaml | 225.0 KiB | 124193 | 1.07 | 1.86 | 123104 | 1.07 | 1.87 |

Claude token counts are not in this table: Anthropic does not publish its tokenizer. Run `eval/count_tokens_claude.py` against the encoded files in `encoded/` to add exact counts from the count_tokens API.

## repetitive_logs (1000 records, 1.4 MiB as json)

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 141.9 KiB | 1.00 | - | - | - | - | - | - | - | - |
| json-pretty | byte | 188.7 KiB | 1.33 | - | - | - | - | - | - | - | - |
| jsonl | byte | 141.9 KiB | 1.00 | - | - | - | - | - | - | - | - |
| csv | byte | 67.7 KiB | 0.48 | - | - | - | - | - | - | - | - |
| tsv | byte | 67.7 KiB | 0.48 | - | - | - | - | - | - | - | - |
| markdown | byte | 83.4 KiB | 0.59 | - | - | - | - | - | - | - | - |
| toon | byte | 71.6 KiB | 0.50 | - | - | - | - | - | - | - | - |
| toon-tab | byte | 71.6 KiB | 0.50 | - | - | - | - | - | - | - | - |
| yaml | byte | 137.0 KiB | 0.97 | - | - | - | - | - | - | - | - |

### Tokens (1000 records)

| format | bytes | cl100k_base | vs json | bytes/token | o200k_base | vs json | bytes/token |
|---|---:|---:|---:|---:|---:|---:|---:|
| json | 141.9 KiB | 46047 | 1.00 | 3.15 | 46047 | 1.00 | 3.15 |
| json-pretty | 188.7 KiB | 71047 | 1.54 | 2.72 | 71047 | 1.54 | 2.72 |
| jsonl | 141.9 KiB | 47045 | 1.02 | 3.09 | 47045 | 1.02 | 3.09 |
| csv | 67.7 KiB | 29609 | 0.64 | 2.34 | 29720 | 0.65 | 2.33 |
| tsv | 67.7 KiB | 28114 | 0.61 | 2.47 | 28145 | 0.61 | 2.46 |
| markdown | 83.4 KiB | 34055 | 0.74 | 2.51 | 34055 | 0.74 | 2.51 |
| toon | 71.6 KiB | 31624 | 0.69 | 2.32 | 31735 | 0.69 | 2.31 |
| toon-tab | 71.6 KiB | 31119 | 0.68 | 2.36 | 31150 | 0.68 | 2.35 |
| yaml | 137.0 KiB | 55024 | 1.19 | 2.55 | 55024 | 1.19 | 2.55 |

Claude token counts are not in this table: Anthropic does not publish its tokenizer. Run `eval/count_tokens_claude.py` against the encoded files in `encoded/` to add exact counts from the count_tokens API.

## unicode_strings (1000 records, 1.2 MiB as json)

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 126.3 KiB | 1.00 | - | - | - | - | - | - | - | - |
| json-pretty | byte | 173.2 KiB | 1.37 | - | - | - | - | - | - | - | - |
| jsonl | byte | 126.3 KiB | 1.00 | - | - | - | - | - | - | - | - |
| csv | lossy (0.empty: null vs "") | 58.8 KiB | 0.47 | - | - | - | - | - | - | - | - |
| tsv | lossy (0.empty: null vs "") | 58.8 KiB | 0.47 | - | - | - | - | - | - | - | - |
| markdown | lossy (0.empty: null vs "") | 74.0 KiB | 0.59 | - | - | - | - | - | - | - | - |
| toon | byte | 66.4 KiB | 0.53 | - | - | - | - | - | - | - | - |
| toon-tab | byte | 66.2 KiB | 0.52 | - | - | - | - | - | - | - | - |
| yaml | byte | 124.6 KiB | 0.99 | - | - | - | - | - | - | - | - |

### Tokens (1000 records)

| format | bytes | cl100k_base | vs json | bytes/token | o200k_base | vs json | bytes/token |
|---|---:|---:|---:|---:|---:|---:|---:|
| json | 126.3 KiB | 44398 | 1.00 | 2.91 | 42188 | 1.00 | 3.07 |
| json-pretty | 173.2 KiB | 67816 | 1.53 | 2.62 | 64668 | 1.53 | 2.74 |
| jsonl | 126.3 KiB | 45396 | 1.02 | 2.85 | 43186 | 1.02 | 3.00 |
| csv | 58.8 KiB | 24948 | 0.56 | 2.41 | 21820 | 0.52 | 2.76 |
| tsv | 58.8 KiB | 25954 | 0.58 | 2.32 | 23037 | 0.55 | 2.61 |
| markdown | 74.0 KiB | 30479 | 0.69 | 2.49 | 27577 | 0.65 | 2.75 |
| toon | 66.4 KiB | 29024 | 0.65 | 2.34 | 25884 | 0.61 | 2.63 |
| toon-tab | 66.2 KiB | 31734 | 0.71 | 2.14 | 29776 | 0.71 | 2.28 |
| yaml | 124.6 KiB | 51154 | 1.15 | 2.50 | 47794 | 1.13 | 2.67 |

Claude token counts are not in this table: Anthropic does not publish its tokenizer. Run `eval/count_tokens_claude.py` against the encoded files in `encoded/` to add exact counts from the count_tokens API.

## numeric_mix (1000 records, 2.3 MiB as json)

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 232.9 KiB | 1.00 | - | - | - | - | - | - | - | - |
| json-pretty | byte | 315.0 KiB | 1.35 | - | - | - | - | - | - | - | - |
| jsonl | byte | 232.9 KiB | 1.00 | - | - | - | - | - | - | - | - |
| csv | lossy (944.huge: 2167128000339913000 vs 2.167128000339913e+18) | 132.4 KiB | 0.57 | - | - | - | - | - | - | - | - |
| tsv | lossy (944.huge: 2167128000339913000 vs 2.167128000339913e+18) | 132.4 KiB | 0.57 | - | - | - | - | - | - | - | - |
| markdown | lossy (944.huge: 2167128000339913000 vs 2.167128000339913e+18) | 159.8 KiB | 0.69 | - | - | - | - | - | - | - | - |
| toon | lossy (944.huge: 2167128000339913000 vs 2.167128000339913e+18) | 134.3 KiB | 0.58 | - | - | - | - | - | - | - | - |
| toon-tab | lossy (944.huge: 2167128000339913000 vs 2.167128000339913e+18) | 134.3 KiB | 0.58 | - | - | - | - | - | - | - | - |
| yaml | byte | 242.6 KiB | 1.04 | - | - | - | - | - | - | - | - |

### Tokens (1000 records)

| format | bytes | cl100k_base | vs json | bytes/token | o200k_base | vs json | bytes/token |
|---|---:|---:|---:|---:|---:|---:|---:|
| json | 232.9 KiB | 97390 | 1.00 | 2.45 | 99863 | 1.00 | 2.39 |
| json-pretty | 315.0 KiB | 138390 | 1.42 | 2.33 | 139390 | 1.40 | 2.31 |
| jsonl | 232.9 KiB | 98388 | 1.01 | 2.42 | 100861 | 1.01 | 2.36 |
| csv | 132.4 KiB | 64894 | 0.67 | 2.09 | 64895 | 0.65 | 2.09 |
| tsv | 132.4 KiB | 66365 | 0.68 | 2.04 | 66366 | 0.66 | 2.04 |
| markdown | 159.8 KiB | 78923 | 0.81 | 2.07 | 78923 | 0.79 | 2.07 |
| toon | 134.3 KiB | 66898 | 0.69 | 2.06 | 66899 | 0.67 | 2.06 |
| toon-tab | 134.3 KiB | 68370 | 0.70 | 2.01 | 68371 | 0.68 | 2.01 |
| yaml | 242.6 KiB | 120446 | 1.24 | 2.06 | 120446 | 1.21 | 2.06 |

Claude token counts are not in this table: Anthropic does not publish its tokenizer. Run `eval/count_tokens_claude.py` against the encoded files in `encoded/` to add exact counts from the count_tokens API.

## mixed_types (1000 records, 736.1 KiB as json)

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 73.3 KiB | 1.00 | - | - | - | - | - | - | - | - |
| json-pretty | byte | 136.7 KiB | 1.86 | - | - | - | - | - | - | - | - |
| jsonl | byte | 73.3 KiB | 1.00 | - | - | - | - | - | - | - | - |
| csv | unsupported: record 0 field "value" is a object (nested) | | | | | | | | | | |
| tsv | unsupported: record 0 field "value" is a object (nested) | | | | | | | | | | |
| markdown | unsupported: record 0 field "value" is a object (nested) | | | | | | | | | | |
| toon | byte | 89.0 KiB | 1.21 | - | - | - | - | - | - | - | - |
| toon-tab | byte | 89.3 KiB | 1.22 | - | - | - | - | - | - | - | - |
| yaml | byte | 81.2 KiB | 1.11 | - | - | - | - | - | - | - | - |

### Tokens (1000 records)

| format | bytes | cl100k_base | vs json | bytes/token | o200k_base | vs json | bytes/token |
|---|---:|---:|---:|---:|---:|---:|---:|
| json | 73.3 KiB | 24184 | 1.00 | 3.10 | 24896 | 1.00 | 3.02 |
| json-pretty | 136.7 KiB | 49747 | 2.06 | 2.81 | 49747 | 2.00 | 2.81 |
| jsonl | 73.3 KiB | 24667 | 1.02 | 3.04 | 25228 | 1.01 | 2.98 |
| csv | unsupported: record 0 field "value" is a object (nested) |
| tsv | unsupported: record 0 field "value" is a object (nested) |
| markdown | unsupported: record 0 field "value" is a object (nested) |
| toon | 89.0 KiB | 33033 | 1.37 | 2.76 | 33045 | 1.33 | 2.76 |
| toon-tab | 89.3 KiB | 33333 | 1.38 | 2.74 | 33338 | 1.34 | 2.74 |
| yaml | 81.2 KiB | 33258 | 1.38 | 2.50 | 33258 | 1.34 | 2.50 |

Claude token counts are not in this table: Anthropic does not publish its tokenizer. Run `eval/count_tokens_claude.py` against the encoded files in `encoded/` to add exact counts from the count_tokens API.

## wide_table (1000 records, 5.2 MiB as json)

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 527.3 KiB | 1.00 | - | - | - | - | - | - | - | - |
| json-pretty | byte | 773.4 KiB | 1.47 | - | - | - | - | - | - | - | - |
| jsonl | byte | 527.3 KiB | 1.00 | - | - | - | - | - | - | - | - |
| csv | lossy (0.s01: null vs "null") | 266.7 KiB | 0.51 | - | - | - | - | - | - | - | - |
| tsv | lossy (0.s01: null vs "null") | 266.7 KiB | 0.51 | - | - | - | - | - | - | - | - |
| markdown | lossy (0.s01: null vs "null") | 349.1 KiB | 0.66 | - | - | - | - | - | - | - | - |
| toon | semantic | 270.0 KiB | 0.51 | - | - | - | - | - | - | - | - |
| toon-tab | semantic | 270.0 KiB | 0.51 | - | - | - | - | - | - | - | - |
| yaml | byte | 547.2 KiB | 1.04 | - | - | - | - | - | - | - | - |

### Tokens (1000 records)

| format | bytes | cl100k_base | vs json | bytes/token | o200k_base | vs json | bytes/token |
|---|---:|---:|---:|---:|---:|---:|---:|
| json | 527.3 KiB | 234201 | 1.00 | 2.31 | 234201 | 1.00 | 2.31 |
| json-pretty | 773.4 KiB | 351201 | 1.50 | 2.26 | 351201 | 1.50 | 2.26 |
| jsonl | 527.3 KiB | 235199 | 1.00 | 2.30 | 235199 | 1.00 | 2.30 |
| csv | 266.7 KiB | 99099 | 0.42 | 2.76 | 99704 | 0.43 | 2.74 |
| tsv | 266.7 KiB | 96319 | 0.41 | 2.84 | 96524 | 0.41 | 2.83 |
| markdown | 349.1 KiB | 134177 | 0.57 | 2.66 | 134177 | 0.57 | 2.66 |
| toon | 270.0 KiB | 101983 | 0.44 | 2.71 | 102546 | 0.44 | 2.70 |
| toon-tab | 270.0 KiB | 100286 | 0.43 | 2.76 | 100491 | 0.43 | 2.75 |
| yaml | 547.2 KiB | 296643 | 1.27 | 1.89 | 296643 | 1.27 | 1.89 |

Claude token counts are not in this table: Anthropic does not publish its tokenizer. Run `eval/count_tokens_claude.py` against the encoded files in `encoded/` to add exact counts from the count_tokens API.

## deep_nesting (1000 records, 625.2 KiB as json)

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 312.6 KiB | 1.00 | - | - | - | - | - | - | - | - |
| json-pretty | byte | 1.3 MiB | 4.18 | - | - | - | - | - | - | - | - |
| jsonl | byte | 312.6 KiB | 1.00 | - | - | - | - | - | - | - | - |
| csv | unsupported: record 0 field "tree" is a object (nested) | | | | | | | | | | |
| tsv | unsupported: record 0 field "tree" is a object (nested) | | | | | | | | | | |
| markdown | unsupported: record 0 field "tree" is a object (nested) | | | | | | | | | | |
| toon | byte | 783.2 KiB | 2.51 | - | - | - | - | - | - | - | - |
| toon-tab | byte | 795.9 KiB | 2.55 | - | - | - | - | - | - | - | - |
| yaml | byte | 826.3 KiB | 2.64 | - | - | - | - | - | - | - | - |

### Tokens (1000 records)

| format | bytes | cl100k_base | vs json | bytes/token | o200k_base | vs json | bytes/token |
|---|---:|---:|---:|---:|---:|---:|---:|
| json | 312.6 KiB | 126454 | 1.00 | 2.53 | 123455 | 1.00 | 2.59 |
| json-pretty | 1.3 MiB | 302155 | 2.39 | 4.43 | 302155 | 2.45 | 4.43 |
| jsonl | 312.6 KiB | 126452 | 1.00 | 2.53 | 124452 | 1.01 | 2.57 |
| csv | unsupported: record 0 field "tree" is a object (nested) |
| tsv | unsupported: record 0 field "tree" is a object (nested) |
| markdown | unsupported: record 0 field "tree" is a object (nested) |
| toon | 783.2 KiB | 224576 | 1.78 | 3.57 | 224576 | 1.82 | 3.57 |
| toon-tab | 795.9 KiB | 237577 | 1.88 | 3.43 | 237577 | 1.92 | 3.43 |
| yaml | 826.3 KiB | 231573 | 1.83 | 3.65 | 231573 | 1.88 | 3.65 |

Claude token counts are not in this table: Anthropic does not publish its tokenizer. Run `eval/count_tokens_claude.py` against the encoded files in `encoded/` to add exact counts from the count_tokens API.

## empty_values (1000 records, 1.9 MiB as json)

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 191.2 KiB | 1.00 | - | - | - | - | - | - | - | - |
| json-pretty | byte | 282.0 KiB | 1.48 | - | - | - | - | - | - | - | - |
| jsonl | byte | 191.2 KiB | 1.00 | - | - | - | - | - | - | - | - |
| csv | unsupported: record 0 field "empty_a" is a array (nested) | | | | | | | | | | |
| tsv | unsupported: record 0 field "empty_a" is a array (nested) | | | | | | | | | | |
| markdown | unsupported: record 0 field "empty_a" is a array (nested) | | | | | | | | | | |
| toon | byte | 226.3 KiB | 1.18 | - | - | - | - | - | - | - | - |
| toon-tab | byte | 226.3 KiB | 1.18 | - | - | - | - | - | - | - | - |
| yaml | byte | 198.2 KiB | 1.04 | - | - | - | - | - | - | - | - |

### Tokens (1000 records)

| format | bytes | cl100k_base | vs json | bytes/token | o200k_base | vs json | bytes/token |
|---|---:|---:|---:|---:|---:|---:|---:|
| json | 191.2 KiB | 60812 | 1.00 | 3.22 | 59714 | 1.00 | 3.28 |
| json-pretty | 282.0 KiB | 97812 | 1.61 | 2.95 | 96714 | 1.62 | 2.99 |
| jsonl | 191.2 KiB | 61810 | 1.02 | 3.17 | 60712 | 1.02 | 3.22 |
| csv | unsupported: record 0 field "empty_a" is a array (nested) |
| tsv | unsupported: record 0 field "empty_a" is a array (nested) |
| markdown | unsupported: record 0 field "empty_a" is a array (nested) |
| toon | 226.3 KiB | 77049 | 1.27 | 3.01 | 75907 | 1.27 | 3.05 |
| toon-tab | 226.3 KiB | 77050 | 1.27 | 3.01 | 75908 | 1.27 | 3.05 |
| yaml | 198.2 KiB | 73795 | 1.21 | 2.75 | 72653 | 1.22 | 2.79 |

Claude token counts are not in this table: Anthropic does not publish its tokenizer. Run `eval/count_tokens_claude.py` against the encoded files in `encoded/` to add exact counts from the count_tokens API.

## long_text (1000 records, 2.3 MiB as json)

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 1.2 MiB | 1.00 | - | - | - | - | - | - | - | - |
| json-pretty | byte | 1.2 MiB | 1.02 | - | - | - | - | - | - | - | - |
| jsonl | byte | 1.2 MiB | 1.00 | - | - | - | - | - | - | - | - |
| csv | byte | 1.1 MiB | 0.97 | - | - | - | - | - | - | - | - |
| tsv | byte | 1.1 MiB | 0.97 | - | - | - | - | - | - | - | - |
| markdown | byte | 1.2 MiB | 0.98 | - | - | - | - | - | - | - | - |
| toon | byte | 1.2 MiB | 0.97 | - | - | - | - | - | - | - | - |
| toon-tab | byte | 1.2 MiB | 0.97 | - | - | - | - | - | - | - | - |
| yaml | byte | 1.2 MiB | 1.00 | - | - | - | - | - | - | - | - |

### Tokens (1000 records)

| format | bytes | cl100k_base | vs json | bytes/token | o200k_base | vs json | bytes/token |
|---|---:|---:|---:|---:|---:|---:|---:|
| json | 1.2 MiB | 224521 | 1.00 | 5.54 | 224271 | 1.00 | 5.54 |
| json-pretty | 1.2 MiB | 239419 | 1.07 | 5.32 | 239169 | 1.07 | 5.32 |
| jsonl | 1.2 MiB | 225519 | 1.00 | 5.51 | 225269 | 1.00 | 5.52 |
| csv | 1.1 MiB | 209865 | 0.93 | 5.74 | 210605 | 0.94 | 5.72 |
| tsv | 1.1 MiB | 210393 | 0.94 | 5.72 | 211016 | 0.94 | 5.71 |
| markdown | 1.2 MiB | 217070 | 0.97 | 5.59 | 216628 | 0.97 | 5.60 |
| toon | 1.2 MiB | 216679 | 0.97 | 5.59 | 216648 | 0.97 | 5.59 |
| toon-tab | 1.2 MiB | 217310 | 0.97 | 5.57 | 217162 | 0.97 | 5.57 |
| yaml | 1.2 MiB | 228707 | 1.02 | 5.43 | 228446 | 1.02 | 5.43 |

Claude token counts are not in this table: Anthropic does not publish its tokenizer. Run `eval/count_tokens_claude.py` against the encoded files in `encoded/` to add exact counts from the count_tokens API.

