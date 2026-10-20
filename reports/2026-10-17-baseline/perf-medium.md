# ldf-bench results

Started 20261017-222522 UTC. Command: `./target/release/ldf-bench perf --scale medium --iters 7 --out results/medium`

Machine: Apple M2 Pro (aarch64), 10 logical CPUs, 16.0 GiB RAM, Darwin 27.0.0. release profile, commit a7bcfcb.

Toolchain: rustc 1.98.0 (88d9e12ae 2026-08-18). Crates: arrow 60.0.0, criterion 0.8.2, csv 1.4.0, parquet 60.0.0, serde_json 1.0.151, simd-json 0.18.1, tiktoken-rs 0.12.1.

Each operation runs in its own child process. Iteration 0 is the cold run; medians and p95 are over the remaining 6 warm iterations. MB = 1e6 bytes. Peak RSS is the child process high-water mark after the operation, as reported by getrusage, minus nothing: it includes the loaded input. See site/src/content/docs/docs/methodology.md.

## flat_users (10000 records, 1.4 MiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 185.8 µs | 51.5 µs | 66.3 µs | 27511.0 | - | 7.2 MiB |
| parse | serde_json | 4.62 ms | 3.53 ms | 3.73 ms | 401.9 | 2835656 | 15.6 MiB |
| parse | simd-json | 3.73 ms | 2.59 ms | 2.80 ms | 547.1 | 3859887 | 22.8 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 1.4 MiB | 1.00 | 1.49 ms | 1.46 ms | 971.2 | 3.84 ms | 3.66 ms | 386.8 | 14.7 MiB | 17.7 MiB |
| json-pretty | byte | 1.9 MiB | 1.38 | 1.97 ms | 2.02 ms | 971.3 | 4.60 ms | 3.90 ms | 502.3 | 17.1 MiB | 19.3 MiB |
| jsonl | byte | 1.4 MiB | 1.00 | 1.52 ms | 1.52 ms | 935.2 | 4.48 ms | 4.01 ms | 353.7 | 16.1 MiB | 19.9 MiB |
| csv | semantic | 632.1 KiB | 0.46 | 4.28 ms | 4.25 ms | 152.1 | 5.26 ms | 3.30 ms | 196.2 | 15.3 MiB | 21.1 MiB |
| tsv | semantic | 632.1 KiB | 0.46 | 4.43 ms | 4.19 ms | 154.6 | 4.17 ms | 3.16 ms | 205.1 | 15.3 MiB | 21.4 MiB |
| markdown | semantic | 807.9 KiB | 0.58 | 4.61 ms | 4.38 ms | 188.7 | 9.13 ms | 7.98 ms | 103.6 | 16.9 MiB | 21.5 MiB |
| toon | semantic | 651.6 KiB | 0.47 | 4.56 ms | 3.98 ms | 167.7 | 5.57 ms | 4.80 ms | 139.0 | 16.1 MiB | 17.9 MiB |
| toon-tab | semantic | 651.6 KiB | 0.47 | 4.50 ms | 4.10 ms | 162.9 | 5.81 ms | 4.85 ms | 137.5 | 15.9 MiB | 22.2 MiB |
| yaml | byte | 1.3 MiB | 0.99 | 6.33 ms | 5.95 ms | 235.0 | 42.06 ms | 38.09 ms | 36.7 | 18.8 MiB | 41.2 MiB |
| ldf | byte | 632.3 KiB | 0.46 | 3.89 ms | 3.54 ms | 183.0 | 5.56 ms | 4.78 ms | 135.4 | 17.0 MiB | 19.4 MiB |

## nested_orders (10000 records, 2.8 MiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 431.6 µs | 176.3 µs | 248.3 µs | 16794.9 | - | 8.7 MiB |
| parse | serde_json | 13.78 ms | 9.49 ms | 11.05 ms | 312.1 | 1054023 | 32.1 MiB |
| parse | simd-json | 10.69 ms | 7.95 ms | 9.70 ms | 372.5 | 1257898 | 68.2 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 2.8 MiB | 1.00 | 4.60 ms | 4.07 ms | 728.5 | 10.74 ms | 9.50 ms | 311.7 | 36.3 MiB | 34.6 MiB |
| json-pretty | byte | 5.0 MiB | 1.77 | 6.80 ms | 6.56 ms | 799.1 | 12.06 ms | 10.68 ms | 490.6 | 69.0 MiB | 39.0 MiB |
| jsonl | byte | 2.8 MiB | 1.00 | 4.61 ms | 4.03 ms | 735.0 | 10.57 ms | 9.76 ms | 303.6 | 36.8 MiB | 36.3 MiB |
| csv | unsupported: record 0 field "customer" is a object (nested) | | | | | | | | | | |
| tsv | unsupported: record 0 field "customer" is a object (nested) | | | | | | | | | | |
| markdown | unsupported: record 0 field "customer" is a object (nested) | | | | | | | | | | |
| toon | semantic | 2.9 MiB | 1.04 | 18.45 ms | 17.95 ms | 171.1 | 28.85 ms | 25.28 ms | 121.4 | 50.0 MiB | 93.7 MiB |
| toon-tab | semantic | 2.9 MiB | 1.04 | 24.99 ms | 19.20 ms | 160.5 | 28.39 ms | 25.27 ms | 121.9 | 49.1 MiB | 91.9 MiB |
| yaml | byte | 3.2 MiB | 1.13 | 15.46 ms | 14.77 ms | 226.9 | 101.08 ms | 96.50 ms | 34.7 | 40.4 MiB | 101.0 MiB |
| ldf | byte | 1.3 MiB | 0.46 | 12.05 ms | 12.12 ms | 111.9 | 19.62 ms | 17.11 ms | 79.2 | 36.2 MiB | 45.3 MiB |

## sparse_events (10000 records, 1.5 MiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 268.7 µs | 83.8 µs | 95.9 µs | 18236.2 | - | 7.3 MiB |
| parse | serde_json | 5.66 ms | 4.53 ms | 4.67 ms | 337.4 | 2206876 | 17.6 MiB |
| parse | simd-json | 4.57 ms | 3.39 ms | 3.45 ms | 450.7 | 2947787 | 23.5 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 1.5 MiB | 1.00 | 2.05 ms | 1.72 ms | 888.7 | 4.76 ms | 4.48 ms | 341.0 | 18.2 MiB | 18.1 MiB |
| json-pretty | byte | 2.0 MiB | 1.36 | 2.21 ms | 2.18 ms | 952.0 | 5.59 ms | 4.76 ms | 435.8 | 17.1 MiB | 21.5 MiB |
| jsonl | byte | 1.5 MiB | 1.00 | 1.73 ms | 1.72 ms | 886.5 | 5.13 ms | 4.94 ms | 309.6 | 16.6 MiB | 22.0 MiB |
| csv | lossy (0: object size 15 vs 9) | 685.5 KiB | 0.46 | 5.25 ms | 5.11 ms | 137.5 | 5.65 ms | 4.73 ms | 148.5 | 16.6 MiB | 27.9 MiB |
| tsv | lossy (0: object size 15 vs 9) | 685.5 KiB | 0.46 | 5.42 ms | 5.31 ms | 132.2 | 6.09 ms | 4.92 ms | 142.7 | 17.6 MiB | 28.2 MiB |
| markdown | lossy (0: object size 15 vs 9) | 998.1 KiB | 0.67 | 5.48 ms | 5.31 ms | 192.6 | 12.76 ms | 11.80 ms | 86.6 | 17.3 MiB | 28.4 MiB |
| toon | semantic | 1.6 MiB | 1.11 | 7.84 ms | 7.66 ms | 221.6 | 14.87 ms | 13.97 ms | 121.5 | 21.8 MiB | 49.6 MiB |
| toon-tab | semantic | 1.6 MiB | 1.11 | 7.82 ms | 7.50 ms | 226.2 | 15.45 ms | 14.27 ms | 119.0 | 18.6 MiB | 50.3 MiB |
| yaml | byte | 1.4 MiB | 0.99 | 6.70 ms | 6.12 ms | 246.8 | 43.48 ms | 39.34 ms | 38.4 | 18.1 MiB | 43.0 MiB |
| ldf | semantic-unordered (object key order changed) | 732.5 KiB | 0.49 | 8.49 ms | 7.92 ms | 94.7 | 8.50 ms | 7.33 ms | 102.4 | 19.1 MiB | 26.7 MiB |

## high_card_strings (10000 records, 2.3 MiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 400.3 µs | 241.5 µs | 423.9 µs | 9827.1 | - | 8.1 MiB |
| parse | serde_json | 4.97 ms | 3.47 ms | 4.04 ms | 683.4 | 2879960 | 18.7 MiB |
| parse | simd-json | 4.37 ms | 2.86 ms | 3.36 ms | 830.6 | 3500329 | 27.8 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 2.3 MiB | 1.00 | 1.84 ms | 1.66 ms | 1431.9 | 3.84 ms | 3.46 ms | 685.9 | 22.7 MiB | 21.0 MiB |
| json-pretty | byte | 2.7 MiB | 1.18 | 3.14 ms | 2.09 ms | 1333.7 | 4.38 ms | 3.70 ms | 754.2 | 35.3 MiB | 20.9 MiB |
| jsonl | byte | 2.3 MiB | 1.00 | 1.92 ms | 1.64 ms | 1446.0 | 3.85 ms | 3.72 ms | 637.9 | 20.6 MiB | 21.9 MiB |
| csv | byte | 1.8 MiB | 0.78 | 3.26 ms | 3.12 ms | 590.7 | 4.50 ms | 3.49 ms | 527.4 | 20.5 MiB | 23.6 MiB |
| tsv | byte | 1.8 MiB | 0.78 | 3.39 ms | 3.19 ms | 578.4 | 4.70 ms | 3.59 ms | 513.9 | 18.7 MiB | 23.9 MiB |
| markdown | byte | 1.9 MiB | 0.84 | 8.24 ms | 7.94 ms | 249.7 | 14.73 ms | 13.20 ms | 150.3 | 20.8 MiB | 26.3 MiB |
| toon | byte | 1.8 MiB | 0.79 | 6.57 ms | 6.12 ms | 307.5 | 9.00 ms | 7.61 ms | 247.4 | 22.5 MiB | 28.9 MiB |
| toon-tab | byte | 1.8 MiB | 0.79 | 6.74 ms | 5.88 ms | 320.0 | 9.14 ms | 7.62 ms | 247.2 | 20.6 MiB | 30.4 MiB |
| yaml | byte | 2.2 MiB | 0.97 | 7.04 ms | 6.27 ms | 368.9 | 41.02 ms | 39.24 ms | 58.9 | 22.4 MiB | 40.3 MiB |
| ldf | byte | 1.8 MiB | 0.78 | 4.80 ms | 4.15 ms | 443.6 | 5.92 ms | 4.84 ms | 380.7 | 22.6 MiB | 27.4 MiB |

## repetitive_logs (10000 records, 1.4 MiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 213.0 µs | 63.6 µs | 82.1 µs | 22836.0 | - | 7.3 MiB |
| parse | serde_json | 4.93 ms | 3.74 ms | 3.90 ms | 388.8 | 2676257 | 17.5 MiB |
| parse | simd-json | 3.99 ms | 2.83 ms | 3.46 ms | 513.9 | 3536928 | 27.1 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 1.4 MiB | 1.00 | 1.59 ms | 1.53 ms | 946.9 | 4.12 ms | 3.86 ms | 376.1 | 14.8 MiB | 16.4 MiB |
| json-pretty | byte | 1.8 MiB | 1.33 | 2.08 ms | 2.00 ms | 968.5 | 4.37 ms | 4.14 ms | 467.3 | 15.2 MiB | 17.1 MiB |
| jsonl | byte | 1.4 MiB | 1.00 | 1.57 ms | 1.47 ms | 990.2 | 4.45 ms | 4.13 ms | 351.8 | 16.2 MiB | 17.5 MiB |
| csv | byte | 676.7 KiB | 0.48 | 3.85 ms | 3.80 ms | 182.4 | 3.84 ms | 3.28 ms | 211.4 | 14.7 MiB | 16.0 MiB |
| tsv | byte | 676.7 KiB | 0.48 | 3.65 ms | 3.67 ms | 189.1 | 4.20 ms | 3.21 ms | 215.9 | 14.7 MiB | 21.8 MiB |
| markdown | byte | 833.1 KiB | 0.59 | 4.00 ms | 3.87 ms | 220.1 | 8.83 ms | 8.26 ms | 103.3 | 15.3 MiB | 16.5 MiB |
| toon | byte | 715.8 KiB | 0.50 | 4.51 ms | 4.07 ms | 180.2 | 6.44 ms | 5.48 ms | 133.7 | 15.2 MiB | 21.6 MiB |
| toon-tab | byte | 715.8 KiB | 0.50 | 4.36 ms | 4.05 ms | 181.0 | 6.90 ms | 5.50 ms | 133.3 | 15.2 MiB | 21.6 MiB |
| yaml | byte | 1.3 MiB | 0.97 | 6.39 ms | 5.75 ms | 243.8 | 37.04 ms | 33.68 ms | 41.7 | 16.2 MiB | 38.4 MiB |
| ldf | byte | 676.8 KiB | 0.48 | 3.62 ms | 3.16 ms | 219.0 | 5.59 ms | 4.87 ms | 142.4 | 17.3 MiB | 21.1 MiB |

## unicode_strings (10000 records, 1.2 MiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 173.5 µs | 53.8 µs | 63.4 µs | 24205.5 | - | 7.1 MiB |
| parse | serde_json | 5.58 ms | 4.39 ms | 4.71 ms | 296.4 | 2276284 | 15.4 MiB |
| parse | simd-json | 4.34 ms | 3.02 ms | 3.44 ms | 430.4 | 3305876 | 21.2 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 1.2 MiB | 1.00 | 1.57 ms | 1.52 ms | 857.7 | 4.66 ms | 4.38 ms | 297.0 | 14.6 MiB | 16.1 MiB |
| json-pretty | byte | 1.7 MiB | 1.37 | 2.03 ms | 1.96 ms | 908.0 | 4.93 ms | 4.73 ms | 377.1 | 15.0 MiB | 18.8 MiB |
| jsonl | byte | 1.2 MiB | 1.00 | 1.61 ms | 1.52 ms | 856.1 | 5.49 ms | 5.02 ms | 259.4 | 14.5 MiB | 17.2 MiB |
| csv | lossy (0.empty: null vs "") | 596.1 KiB | 0.47 | 3.74 ms | 3.72 ms | 163.9 | 5.21 ms | 4.38 ms | 139.5 | 14.5 MiB | 20.2 MiB |
| tsv | lossy (0.empty: null vs "") | 596.1 KiB | 0.47 | 3.76 ms | 3.69 ms | 165.6 | 5.27 ms | 4.36 ms | 140.1 | 14.5 MiB | 20.2 MiB |
| markdown | lossy (0.empty: null vs "") | 747.6 KiB | 0.59 | 4.21 ms | 4.07 ms | 187.9 | 9.81 ms | 9.15 ms | 83.7 | 15.2 MiB | 16.4 MiB |
| toon | byte | 671.9 KiB | 0.53 | 4.79 ms | 4.22 ms | 162.9 | 8.42 ms | 7.17 ms | 96.0 | 15.1 MiB | 22.3 MiB |
| toon-tab | byte | 669.9 KiB | 0.53 | 4.85 ms | 4.30 ms | 159.7 | 8.50 ms | 7.39 ms | 92.8 | 15.1 MiB | 21.7 MiB |
| yaml | byte | 1.2 MiB | 0.99 | 6.79 ms | 6.31 ms | 203.6 | 38.75 ms | 34.56 ms | 37.2 | 17.2 MiB | 38.0 MiB |
| ldf | byte | 636.7 KiB | 0.50 | 4.21 ms | 3.63 ms | 179.5 | 7.12 ms | 6.53 ms | 99.9 | 15.4 MiB | 19.6 MiB |

## numeric_mix (10000 records, 2.3 MiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 300.4 µs | 123.5 µs | 140.2 µs | 19397.7 | - | 8.2 MiB |
| parse | serde_json | 8.17 ms | 6.59 ms | 6.76 ms | 363.3 | 1516592 | 22.3 MiB |
| parse | simd-json | 7.18 ms | 4.79 ms | 5.68 ms | 500.2 | 2088519 | 36.2 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 2.3 MiB | 1.00 | 3.80 ms | 3.42 ms | 699.5 | 6.66 ms | 6.55 ms | 365.9 | 26.6 MiB | 24.7 MiB |
| json-pretty | byte | 3.1 MiB | 1.35 | 4.48 ms | 4.27 ms | 757.1 | 7.83 ms | 6.93 ms | 467.1 | 27.5 MiB | 28.9 MiB |
| jsonl | byte | 2.3 MiB | 1.00 | 3.81 ms | 3.61 ms | 664.3 | 7.25 ms | 7.00 ms | 342.2 | 26.6 MiB | 25.7 MiB |
| csv | lossy (944.huge: 2167128000339913000 vs 2.167128000339913e+18) | 1.3 MiB | 0.57 | 12.73 ms | 11.91 ms | 114.5 | 6.62 ms | 5.79 ms | 235.7 | 22.8 MiB | 31.3 MiB |
| tsv | lossy (944.huge: 2167128000339913000 vs 2.167128000339913e+18) | 1.3 MiB | 0.57 | 13.77 ms | 12.09 ms | 112.8 | 6.59 ms | 5.65 ms | 241.4 | 23.2 MiB | 31.7 MiB |
| markdown | lossy (944.huge: 2167128000339913000 vs 2.167128000339913e+18) | 1.6 MiB | 0.69 | 13.73 ms | 12.54 ms | 131.2 | 16.95 ms | 15.85 ms | 103.7 | 23.6 MiB | 33.0 MiB |
| toon | lossy (944.huge: 2167128000339913000 vs 2.167128000339913e+18) | 1.3 MiB | 0.58 | 12.48 ms | 10.31 ms | 134.3 | 10.56 ms | 8.85 ms | 156.5 | 24.4 MiB | 34.7 MiB |
| toon-tab | lossy (944.huge: 2167128000339913000 vs 2.167128000339913e+18) | 1.3 MiB | 0.58 | 11.81 ms | 10.44 ms | 132.5 | 10.24 ms | 8.97 ms | 154.3 | 23.3 MiB | 34.8 MiB |
| yaml | byte | 2.4 MiB | 1.04 | 11.43 ms | 10.33 ms | 241.5 | 67.96 ms | 60.33 ms | 41.3 | 29.3 MiB | 63.6 MiB |
| ldf | byte | 1.3 MiB | 0.57 | 9.12 ms | 8.56 ms | 159.4 | 8.06 ms | 7.39 ms | 184.7 | 25.0 MiB | 28.1 MiB |

## mixed_types (10000 records, 736.1 KiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 220.4 µs | 33.8 µs | 41.6 µs | 22306.7 | - | 6.6 MiB |
| parse | serde_json | 3.62 ms | 2.76 ms | 2.80 ms | 272.9 | 3620182 | 15.0 MiB |
| parse | simd-json | 3.60 ms | 2.65 ms | 2.76 ms | 284.6 | 3775574 | 18.3 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 736.1 KiB | 1.00 | 1.33 ms | 1.28 ms | 586.7 | 2.81 ms | 2.71 ms | 278.4 | 14.8 MiB | 15.5 MiB |
| json-pretty | byte | 1.3 MiB | 1.85 | 1.98 ms | 1.90 ms | 734.5 | 3.19 ms | 3.02 ms | 461.3 | 15.4 MiB | 16.9 MiB |
| jsonl | byte | 736.1 KiB | 1.00 | 1.33 ms | 1.29 ms | 584.0 | 3.20 ms | 3.00 ms | 251.0 | 14.8 MiB | 16.3 MiB |
| csv | unsupported: record 0 field "value" is a object (nested) | | | | | | | | | | |
| tsv | unsupported: record 0 field "value" is a object (nested) | | | | | | | | | | |
| markdown | unsupported: record 0 field "value" is a object (nested) | | | | | | | | | | |
| toon | semantic | 890.0 KiB | 1.21 | 3.99 ms | 3.84 ms | 237.1 | 8.58 ms | 7.86 ms | 116.0 | 15.9 MiB | 21.0 MiB |
| toon-tab | semantic | 893.3 KiB | 1.21 | 4.11 ms | 3.93 ms | 232.9 | 8.82 ms | 7.82 ms | 117.0 | 15.9 MiB | 22.7 MiB |
| yaml | byte | 812.4 KiB | 1.10 | 3.98 ms | 3.77 ms | 220.8 | 33.75 ms | 29.57 ms | 28.1 | 17.3 MiB | 38.2 MiB |
| ldf | byte | 472.9 KiB | 0.64 | 3.07 ms | 2.82 ms | 171.5 | 5.60 ms | 4.76 ms | 101.7 | 16.5 MiB | 20.5 MiB |

## wide_table (10000 records, 5.2 MiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 865.9 µs | 556.8 µs | 709.3 µs | 9717.2 | - | 11.0 MiB |
| parse | serde_json | 22.80 ms | 18.02 ms | 19.83 ms | 300.2 | 554983 | 64.4 MiB |
| parse | simd-json | 17.66 ms | 14.17 ms | 17.25 ms | 381.9 | 705893 | 112.3 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 5.2 MiB | 1.00 | 8.68 ms | 8.23 ms | 657.8 | 20.37 ms | 17.36 ms | 311.7 | 92.7 MiB | 66.8 MiB |
| json-pretty | byte | 7.6 MiB | 1.47 | 12.87 ms | 10.75 ms | 737.7 | 22.30 ms | 19.21 ms | 412.8 | 111.0 MiB | 71.4 MiB |
| jsonl | byte | 5.2 MiB | 1.00 | 9.10 ms | 8.25 ms | 655.4 | 19.69 ms | 18.71 ms | 289.2 | 94.2 MiB | 67.3 MiB |
| csv | lossy (0.s01: null vs "null") | 2.6 MiB | 0.51 | 26.94 ms | 26.37 ms | 103.9 | 15.79 ms | 13.31 ms | 205.9 | 63.7 MiB | 94.4 MiB |
| tsv | lossy (0.s01: null vs "null") | 2.6 MiB | 0.51 | 26.20 ms | 26.27 ms | 104.3 | 15.87 ms | 13.48 ms | 203.3 | 63.7 MiB | 94.7 MiB |
| markdown | lossy (0.s01: null vs "null") | 3.4 MiB | 0.66 | 25.83 ms | 25.38 ms | 141.1 | 37.46 ms | 34.68 ms | 103.2 | 65.2 MiB | 92.9 MiB |
| toon | semantic | 2.6 MiB | 0.51 | 25.11 ms | 24.35 ms | 113.9 | 52.43 ms | 49.29 ms | 56.3 | 65.1 MiB | 89.8 MiB |
| toon-tab | semantic | 2.6 MiB | 0.51 | 24.74 ms | 23.85 ms | 116.3 | 54.38 ms | 50.07 ms | 55.4 | 65.2 MiB | 93.9 MiB |
| yaml | byte | 5.4 MiB | 1.04 | 28.09 ms | 26.62 ms | 210.8 | 193.71 ms | 167.48 ms | 33.5 | 93.8 MiB | 175.6 MiB |
| ldf | byte | 2.6 MiB | 0.51 | 57.70 ms | 56.15 ms | 49.0 | 60.50 ms | 52.00 ms | 52.9 | 64.5 MiB | 83.7 MiB |

## deep_nesting (2000 records, 625.2 KiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 110.3 µs | 29.0 µs | 36.8 µs | 22077.3 | - | 6.5 MiB |
| parse | serde_json | 3.45 ms | 2.57 ms | 2.72 ms | 248.7 | 776837 | 16.5 MiB |
| parse | simd-json | 3.56 ms | 2.45 ms | 2.51 ms | 261.2 | 815938 | 19.8 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 625.2 KiB | 1.00 | 1.08 ms | 1.06 ms | 604.4 | 2.83 ms | 2.57 ms | 248.9 | 16.2 MiB | 16.8 MiB |
| json-pretty | byte | 2.5 MiB | 4.18 | 3.43 ms | 3.18 ms | 840.9 | 3.98 ms | 3.75 ms | 712.3 | 20.1 MiB | 22.6 MiB |
| jsonl | byte | 625.2 KiB | 1.00 | 1.13 ms | 1.05 ms | 610.9 | 2.81 ms | 2.62 ms | 244.1 | 16.1 MiB | 17.4 MiB |
| csv | unsupported: record 0 field "tree" is a object (nested) | | | | | | | | | | |
| tsv | unsupported: record 0 field "tree" is a object (nested) | | | | | | | | | | |
| markdown | unsupported: record 0 field "tree" is a object (nested) | | | | | | | | | | |
| toon | byte | 1.5 MiB | 2.50 | 5.47 ms | 4.92 ms | 326.1 | 11.40 ms | 10.03 ms | 159.9 | 18.9 MiB | 46.7 MiB |
| toon-tab | byte | 1.6 MiB | 2.54 | 5.30 ms | 4.97 ms | 327.8 | 11.04 ms | 10.07 ms | 161.7 | 18.9 MiB | 47.1 MiB |
| yaml | byte | 1.6 MiB | 2.64 | 3.92 ms | 3.72 ms | 455.2 | 36.46 ms | 32.26 ms | 52.4 | 22.1 MiB | 45.9 MiB |
| ldf | byte | 424.2 KiB | 0.68 | 1.95 ms | 1.65 ms | 263.2 | 5.07 ms | 4.53 ms | 96.0 | 17.0 MiB | 22.4 MiB |

## empty_values (10000 records, 1.9 MiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 296.8 µs | 150.8 µs | 225.9 µs | 13032.9 | - | 7.8 MiB |
| parse | serde_json | 7.56 ms | 6.09 ms | 6.33 ms | 322.7 | 1641693 | 26.1 MiB |
| parse | simd-json | 6.36 ms | 4.10 ms | 4.80 ms | 479.5 | 2439260 | 35.6 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 1.9 MiB | 1.00 | 2.24 ms | 1.95 ms | 1010.4 | 5.94 ms | 6.20 ms | 317.1 | 24.2 MiB | 26.4 MiB |
| json-pretty | byte | 2.8 MiB | 1.47 | 3.38 ms | 2.95 ms | 981.8 | 6.59 ms | 6.52 ms | 444.2 | 26.1 MiB | 29.2 MiB |
| jsonl | byte | 1.9 MiB | 1.00 | 2.12 ms | 1.95 ms | 1009.8 | 6.32 ms | 6.23 ms | 315.6 | 23.3 MiB | 25.1 MiB |
| csv | unsupported: record 0 field "empty_a" is a array (nested) | | | | | | | | | | |
| tsv | unsupported: record 0 field "empty_a" is a array (nested) | | | | | | | | | | |
| markdown | unsupported: record 0 field "empty_a" is a array (nested) | | | | | | | | | | |
| toon | byte | 2.2 MiB | 1.18 | 10.06 ms | 9.33 ms | 249.4 | 21.28 ms | 20.40 ms | 114.0 | 27.8 MiB | 52.9 MiB |
| toon-tab | byte | 2.2 MiB | 1.18 | 10.23 ms | 9.50 ms | 244.8 | 21.03 ms | 20.56 ms | 113.1 | 29.2 MiB | 66.6 MiB |
| yaml | byte | 1.9 MiB | 1.04 | 8.31 ms | 7.70 ms | 264.8 | 61.07 ms | 55.54 ms | 36.7 | 25.3 MiB | 64.7 MiB |
| ldf | byte | 613.7 KiB | 0.32 | 6.07 ms | 5.24 ms | 119.9 | 8.25 ms | 7.86 ms | 80.0 | 23.5 MiB | 27.7 MiB |

## long_text (2000 records, 2.3 MiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 314.2 µs | 110.7 µs | 133.3 µs | 22026.2 | - | 8.2 MiB |
| parse | serde_json | 1.61 ms | 1.16 ms | 1.29 ms | 2107.8 | 1729076 | 14.2 MiB |
| parse | simd-json | 2.06 ms | 1.25 ms | 1.47 ms | 1953.7 | 1602671 | 19.5 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 2.3 MiB | 1.00 | 1.51 ms | 1.17 ms | 2078.7 | 1.15 ms | 1.10 ms | 2216.4 | 16.4 MiB | 16.5 MiB |
| json-pretty | byte | 2.4 MiB | 1.02 | 1.41 ms | 1.20 ms | 2079.8 | 1.37 ms | 1.16 ms | 2155.4 | 13.2 MiB | 15.7 MiB |
| jsonl | byte | 2.3 MiB | 1.00 | 1.66 ms | 1.18 ms | 2074.3 | 2.05 ms | 1.80 ms | 1355.3 | 16.4 MiB | 17.4 MiB |
| csv | byte | 2.3 MiB | 0.97 | 1.18 ms | 817.1 µs | 2888.3 | 2.44 ms | 2.29 ms | 1029.6 | 16.5 MiB | 16.9 MiB |
| tsv | byte | 2.3 MiB | 0.97 | 1.27 ms | 890.9 µs | 2648.9 | 2.13 ms | 2.07 ms | 1141.9 | 16.5 MiB | 16.9 MiB |
| markdown | byte | 2.3 MiB | 0.98 | 4.99 ms | 4.95 ms | 480.5 | 7.27 ms | 6.95 ms | 342.0 | 27.0 MiB | 16.4 MiB |
| toon | byte | 2.3 MiB | 0.97 | 4.88 ms | 4.56 ms | 519.7 | 11.00 ms | 10.13 ms | 234.1 | 16.6 MiB | 22.9 MiB |
| toon-tab | byte | 2.3 MiB | 0.97 | 5.32 ms | 4.95 ms | 479.3 | 10.36 ms | 9.92 ms | 239.1 | 16.8 MiB | 22.5 MiB |
| yaml | byte | 2.3 MiB | 1.00 | 7.00 ms | 6.79 ms | 358.1 | 17.49 ms | 16.23 ms | 149.9 | 18.8 MiB | 24.0 MiB |
| ldf | byte | 2.3 MiB | 0.97 | 2.53 ms | 2.11 ms | 1121.1 | 3.62 ms | 3.10 ms | 762.9 | 16.7 MiB | 22.4 MiB |

