# ldf-bench results

Started 20261017-203954 UTC. Command: `./target/release/ldf-bench perf --scale medium --iters 7 --out results/medium`

Machine: Apple M2 Pro (aarch64), 10 logical CPUs, 16.0 GiB RAM, Darwin 27.0.0. release profile, commit unknown.

Toolchain: rustc 1.98.0 (88d9e12ae 2026-08-18). Crates: arrow 60.0.0, criterion 0.8.2, csv 1.4.0, parquet 60.0.0, serde_json 1.0.151, simd-json 0.18.1, tiktoken-rs 0.12.1.

Each operation runs in its own child process. Iteration 0 is the cold run; medians and p95 are over the remaining 6 warm iterations. MB = 1e6 bytes. Peak RSS is the child process high-water mark after the operation, as reported by getrusage, minus nothing: it includes the loaded input. See docs/benchmark-methodology.md.

## flat_users (10000 records, 1.4 MiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 424.4 µs | 398.6 µs | 537.9 µs | 3555.5 | - | 7.2 MiB |
| parse | serde_json | 7.42 ms | 3.86 ms | 6.44 ms | 367.6 | 2593389 | 17.2 MiB |
| parse | simd-json | 5.23 ms | 2.72 ms | 3.04 ms | 520.2 | 3669893 | 27.6 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 1.4 MiB | 1.00 | 1.62 ms | 1.52 ms | 935.1 | 3.97 ms | 3.68 ms | 385.3 | 16.1 MiB | 17.6 MiB |
| json-pretty | byte | 1.9 MiB | 1.38 | 2.15 ms | 1.99 ms | 982.4 | 5.83 ms | 3.99 ms | 490.1 | 16.6 MiB | 21.3 MiB |
| jsonl | byte | 1.4 MiB | 1.00 | 3.06 ms | 1.51 ms | 940.3 | 6.02 ms | 4.41 ms | 321.6 | 16.0 MiB | 22.4 MiB |
| csv | semantic | 632.1 KiB | 0.46 | 4.46 ms | 4.33 ms | 149.4 | 3.93 ms | 3.32 ms | 195.2 | 15.4 MiB | 20.0 MiB |
| tsv | semantic | 632.1 KiB | 0.46 | 4.38 ms | 5.99 ms | 108.0 | 4.66 ms | 3.23 ms | 200.1 | 15.4 MiB | 20.2 MiB |
| markdown | semantic | 807.9 KiB | 0.58 | 4.67 ms | 4.50 ms | 183.8 | 8.86 ms | 13.78 ms | 60.0 | 17.2 MiB | 23.2 MiB |
| toon | semantic | 651.6 KiB | 0.47 | 4.99 ms | 4.43 ms | 150.6 | 5.73 ms | 4.95 ms | 134.7 | 16.1 MiB | 17.9 MiB |
| toon-tab | semantic | 651.6 KiB | 0.47 | 4.95 ms | 4.14 ms | 161.1 | 6.04 ms | 4.90 ms | 136.1 | 15.0 MiB | 22.2 MiB |

## nested_orders (10000 records, 2.8 MiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 722.5 µs | 397.9 µs | 958.3 µs | 7442.6 | - | 8.6 MiB |
| parse | serde_json | 13.24 ms | 14.65 ms | 16.73 ms | 202.2 | 682594 | 32.5 MiB |
| parse | simd-json | 15.35 ms | 9.17 ms | 12.30 ms | 323.0 | 1090807 | 69.7 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 2.8 MiB | 1.00 | 7.92 ms | 4.10 ms | 722.0 | 13.37 ms | 9.76 ms | 303.3 | 36.6 MiB | 34.9 MiB |
| json-pretty | byte | 5.0 MiB | 1.77 | 6.57 ms | 6.69 ms | 783.2 | 18.72 ms | 10.85 ms | 483.1 | 68.9 MiB | 39.4 MiB |
| jsonl | byte | 2.8 MiB | 1.00 | 4.41 ms | 4.39 ms | 674.9 | 10.93 ms | 10.28 ms | 288.2 | 38.2 MiB | 36.1 MiB |
| csv | unsupported: record 0 field "customer" is a object (nested) | | | | | | | | | | |
| tsv | unsupported: record 0 field "customer" is a object (nested) | | | | | | | | | | |
| markdown | unsupported: record 0 field "customer" is a object (nested) | | | | | | | | | | |
| toon | semantic | 2.9 MiB | 1.04 | 19.72 ms | 20.06 ms | 153.1 | 27.97 ms | 27.07 ms | 113.5 | 51.0 MiB | 91.7 MiB |
| toon-tab | semantic | 2.9 MiB | 1.04 | 19.49 ms | 26.36 ms | 116.9 | 31.35 ms | 36.71 ms | 83.9 | 51.4 MiB | 91.6 MiB |

## sparse_events (10000 records, 1.5 MiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 433.4 µs | 400.5 µs | 550.8 µs | 3817.4 | - | 7.3 MiB |
| parse | serde_json | 5.74 ms | 7.16 ms | 9.68 ms | 213.4 | 1396132 | 21.2 MiB |
| parse | simd-json | 6.40 ms | 3.49 ms | 3.82 ms | 437.5 | 2861657 | 28.9 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 1.5 MiB | 1.00 | 1.93 ms | 1.79 ms | 855.4 | 4.82 ms | 6.84 ms | 223.7 | 19.6 MiB | 21.5 MiB |
| json-pretty | byte | 2.0 MiB | 1.36 | 4.39 ms | 2.24 ms | 928.0 | 5.39 ms | 5.01 ms | 414.2 | 19.1 MiB | 23.1 MiB |
| jsonl | byte | 1.5 MiB | 1.00 | 1.90 ms | 1.76 ms | 866.3 | 5.37 ms | 5.17 ms | 295.7 | 18.1 MiB | 24.6 MiB |
| csv | lossy (0: object size 15 vs 9) | 685.5 KiB | 0.46 | 6.08 ms | 5.29 ms | 132.7 | 5.82 ms | 5.13 ms | 136.8 | 17.4 MiB | 28.4 MiB |
| tsv | lossy (0: object size 15 vs 9) | 685.5 KiB | 0.46 | 5.37 ms | 7.86 ms | 89.3 | 10.67 ms | 4.95 ms | 141.7 | 18.1 MiB | 27.8 MiB |
| markdown | lossy (0: object size 15 vs 9) | 998.1 KiB | 0.67 | 5.49 ms | 8.05 ms | 127.0 | 12.81 ms | 14.18 ms | 72.1 | 19.7 MiB | 29.9 MiB |
| toon | semantic | 1.6 MiB | 1.11 | 10.93 ms | 9.14 ms | 185.7 | 15.35 ms | 17.50 ms | 97.0 | 23.4 MiB | 48.7 MiB |
| toon-tab | semantic | 1.6 MiB | 1.11 | 14.08 ms | 7.95 ms | 213.6 | 14.31 ms | 13.88 ms | 122.3 | 21.7 MiB | 50.1 MiB |

## high_card_strings (10000 records, 2.3 MiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 317.7 µs | 104.8 µs | 130.2 µs | 22648.0 | - | 8.1 MiB |
| parse | serde_json | 4.55 ms | 3.44 ms | 3.63 ms | 689.8 | 2907188 | 18.6 MiB |
| parse | simd-json | 3.71 ms | 2.60 ms | 2.92 ms | 913.9 | 3851369 | 27.5 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 2.3 MiB | 1.00 | 2.07 ms | 1.68 ms | 1408.9 | 3.82 ms | 3.47 ms | 683.3 | 23.1 MiB | 20.9 MiB |
| json-pretty | byte | 2.7 MiB | 1.18 | 2.15 ms | 2.16 ms | 1291.7 | 3.95 ms | 3.64 ms | 766.8 | 35.3 MiB | 20.8 MiB |
| jsonl | byte | 2.3 MiB | 1.00 | 1.91 ms | 1.59 ms | 1488.4 | 3.97 ms | 3.75 ms | 632.8 | 22.2 MiB | 22.3 MiB |
| csv | byte | 1.8 MiB | 0.78 | 3.33 ms | 3.19 ms | 576.9 | 4.40 ms | 3.65 ms | 504.3 | 20.4 MiB | 24.9 MiB |
| tsv | byte | 1.8 MiB | 0.78 | 3.19 ms | 3.07 ms | 599.8 | 4.85 ms | 3.58 ms | 514.6 | 20.5 MiB | 25.9 MiB |
| markdown | byte | 1.9 MiB | 0.84 | 8.11 ms | 7.61 ms | 260.6 | 14.00 ms | 12.56 ms | 157.9 | 24.5 MiB | 25.6 MiB |
| toon | byte | 1.8 MiB | 0.79 | 7.27 ms | 6.29 ms | 299.4 | 9.24 ms | 7.92 ms | 237.7 | 22.4 MiB | 32.7 MiB |
| toon-tab | byte | 1.8 MiB | 0.79 | 6.70 ms | 6.18 ms | 304.8 | 8.91 ms | 7.66 ms | 245.9 | 24.1 MiB | 31.7 MiB |

## repetitive_logs (10000 records, 1.4 MiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 190.2 µs | 52.3 µs | 62.5 µs | 27763.1 | - | 7.2 MiB |
| parse | serde_json | 4.61 ms | 3.78 ms | 3.81 ms | 384.7 | 2648013 | 17.4 MiB |
| parse | simd-json | 5.99 ms | 2.81 ms | 5.93 ms | 516.2 | 3552477 | 32.1 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 1.4 MiB | 1.00 | 1.59 ms | 2.14 ms | 680.3 | 3.91 ms | 3.73 ms | 389.1 | 16.3 MiB | 18.1 MiB |
| json-pretty | byte | 1.8 MiB | 1.33 | 2.02 ms | 1.94 ms | 995.7 | 4.28 ms | 3.99 ms | 483.9 | 15.5 MiB | 18.7 MiB |
| jsonl | byte | 1.4 MiB | 1.00 | 1.53 ms | 1.51 ms | 962.3 | 4.35 ms | 4.05 ms | 358.3 | 14.7 MiB | 20.2 MiB |
| csv | byte | 676.7 KiB | 0.48 | 3.92 ms | 3.79 ms | 182.8 | 3.71 ms | 3.17 ms | 218.4 | 15.2 MiB | 20.2 MiB |
| tsv | byte | 676.7 KiB | 0.48 | 3.83 ms | 3.75 ms | 185.0 | 4.20 ms | 3.25 ms | 212.9 | 15.3 MiB | 21.7 MiB |
| markdown | byte | 833.1 KiB | 0.59 | 4.10 ms | 3.98 ms | 214.4 | 8.83 ms | 8.03 ms | 106.3 | 16.9 MiB | 20.7 MiB |
| toon | byte | 715.8 KiB | 0.50 | 4.86 ms | 4.23 ms | 173.4 | 6.15 ms | 5.46 ms | 134.3 | 16.5 MiB | 18.6 MiB |
| toon-tab | byte | 715.8 KiB | 0.50 | 4.94 ms | 4.15 ms | 176.5 | 6.20 ms | 5.50 ms | 133.3 | 15.7 MiB | 18.8 MiB |

## unicode_strings (10000 records, 1.2 MiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 181.6 µs | 61.2 µs | 74.3 µs | 21287.2 | - | 7.0 MiB |
| parse | serde_json | 5.08 ms | 4.39 ms | 4.65 ms | 296.4 | 2276543 | 16.9 MiB |
| parse | simd-json | 3.92 ms | 3.11 ms | 3.23 ms | 419.1 | 3218949 | 25.2 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 1.2 MiB | 1.00 | 1.59 ms | 1.52 ms | 856.9 | 4.36 ms | 4.31 ms | 302.1 | 15.8 MiB | 17.4 MiB |
| json-pretty | byte | 1.7 MiB | 1.37 | 2.01 ms | 1.90 ms | 937.5 | 4.90 ms | 4.56 ms | 390.5 | 14.9 MiB | 18.7 MiB |
| jsonl | byte | 1.2 MiB | 1.00 | 1.57 ms | 1.48 ms | 877.7 | 5.21 ms | 4.91 ms | 264.9 | 15.8 MiB | 19.7 MiB |
| csv | lossy (0.empty: null vs "") | 596.1 KiB | 0.47 | 3.87 ms | 3.67 ms | 166.3 | 5.18 ms | 4.31 ms | 141.7 | 15.0 MiB | 21.3 MiB |
| tsv | lossy (0.empty: null vs "") | 596.1 KiB | 0.47 | 3.88 ms | 3.66 ms | 166.6 | 5.17 ms | 4.34 ms | 140.5 | 15.0 MiB | 19.9 MiB |
| markdown | lossy (0.empty: null vs "") | 747.6 KiB | 0.59 | 4.24 ms | 4.08 ms | 187.6 | 9.90 ms | 8.89 ms | 86.1 | 16.5 MiB | 21.0 MiB |
| toon | byte | 671.9 KiB | 0.53 | 4.82 ms | 4.32 ms | 159.4 | 7.85 ms | 7.12 ms | 96.6 | 16.3 MiB | 18.6 MiB |
| toon-tab | byte | 669.9 KiB | 0.53 | 5.16 ms | 4.23 ms | 162.1 | 7.74 ms | 7.11 ms | 96.4 | 16.4 MiB | 18.5 MiB |

## numeric_mix (10000 records, 2.3 MiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 319.9 µs | 109.7 µs | 165.6 µs | 21836.8 | - | 8.1 MiB |
| parse | serde_json | 7.46 ms | 6.57 ms | 6.76 ms | 364.8 | 1523157 | 22.2 MiB |
| parse | simd-json | 6.35 ms | 4.65 ms | 4.89 ms | 514.7 | 2148891 | 36.2 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 2.3 MiB | 1.00 | 3.55 ms | 3.28 ms | 729.9 | 6.77 ms | 6.60 ms | 363.2 | 24.5 MiB | 24.5 MiB |
| json-pretty | byte | 3.1 MiB | 1.35 | 4.21 ms | 3.98 ms | 812.0 | 7.28 ms | 7.02 ms | 461.1 | 26.9 MiB | 28.4 MiB |
| jsonl | byte | 2.3 MiB | 1.00 | 3.92 ms | 3.31 ms | 723.6 | 7.37 ms | 6.91 ms | 346.8 | 26.1 MiB | 26.2 MiB |
| csv | lossy (944.huge: 2167128000339913000 vs 2.167128000339913e+18) | 1.3 MiB | 0.57 | 12.63 ms | 12.06 ms | 113.2 | 6.60 ms | 5.45 ms | 250.5 | 22.7 MiB | 29.6 MiB |
| tsv | lossy (944.huge: 2167128000339913000 vs 2.167128000339913e+18) | 1.3 MiB | 0.57 | 13.25 ms | 11.87 ms | 114.9 | 6.31 ms | 5.55 ms | 246.0 | 23.1 MiB | 29.7 MiB |
| markdown | lossy (944.huge: 2167128000339913000 vs 2.167128000339913e+18) | 1.6 MiB | 0.69 | 13.22 ms | 12.52 ms | 131.3 | 16.17 ms | 15.01 ms | 109.6 | 26.8 MiB | 33.1 MiB |
| toon | lossy (944.huge: 2167128000339913000 vs 2.167128000339913e+18) | 1.3 MiB | 0.58 | 12.00 ms | 10.42 ms | 132.8 | 9.96 ms | 8.66 ms | 159.9 | 24.2 MiB | 35.9 MiB |
| toon-tab | lossy (944.huge: 2167128000339913000 vs 2.167128000339913e+18) | 1.3 MiB | 0.58 | 12.73 ms | 10.66 ms | 129.8 | 10.04 ms | 8.75 ms | 158.1 | 24.6 MiB | 34.8 MiB |

## mixed_types (10000 records, 736.1 KiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 135.3 µs | 35.4 µs | 41.0 µs | 21308.2 | - | 6.5 MiB |
| parse | serde_json | 3.53 ms | 2.78 ms | 2.87 ms | 271.6 | 3602657 | 16.0 MiB |
| parse | simd-json | 3.49 ms | 2.66 ms | 2.74 ms | 283.3 | 3758604 | 18.3 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 736.1 KiB | 1.00 | 1.29 ms | 1.29 ms | 583.6 | 2.82 ms | 2.76 ms | 273.2 | 14.6 MiB | 16.3 MiB |
| json-pretty | byte | 1.3 MiB | 1.85 | 1.96 ms | 1.90 ms | 734.1 | 3.47 ms | 3.01 ms | 462.7 | 16.7 MiB | 18.5 MiB |
| jsonl | byte | 736.1 KiB | 1.00 | 1.32 ms | 1.29 ms | 582.4 | 3.21 ms | 3.02 ms | 249.4 | 15.5 MiB | 18.1 MiB |
| csv | unsupported: record 0 field "value" is a object (nested) | | | | | | | | | | |
| tsv | unsupported: record 0 field "value" is a object (nested) | | | | | | | | | | |
| markdown | unsupported: record 0 field "value" is a object (nested) | | | | | | | | | | |
| toon | semantic | 890.0 KiB | 1.21 | 4.09 ms | 3.88 ms | 234.7 | 8.23 ms | 7.87 ms | 115.8 | 17.5 MiB | 22.6 MiB |
| toon-tab | semantic | 893.3 KiB | 1.21 | 4.19 ms | 3.95 ms | 231.7 | 8.51 ms | 7.75 ms | 118.0 | 17.6 MiB | 21.7 MiB |

## wide_table (10000 records, 5.2 MiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 635.9 µs | 235.0 µs | 305.7 µs | 23019.5 | - | 11.0 MiB |
| parse | serde_json | 21.27 ms | 18.17 ms | 19.54 ms | 297.8 | 550391 | 64.3 MiB |
| parse | simd-json | 16.15 ms | 13.19 ms | 16.39 ms | 410.2 | 758136 | 112.2 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 5.2 MiB | 1.00 | 7.96 ms | 8.54 ms | 633.6 | 19.08 ms | 17.56 ms | 308.1 | 94.1 MiB | 67.1 MiB |
| json-pretty | byte | 7.6 MiB | 1.47 | 10.53 ms | 10.63 ms | 745.7 | 21.37 ms | 19.52 ms | 406.2 | 111.0 MiB | 71.9 MiB |
| jsonl | byte | 5.2 MiB | 1.00 | 8.15 ms | 8.40 ms | 644.4 | 19.60 ms | 18.96 ms | 285.4 | 94.1 MiB | 67.8 MiB |
| csv | lossy (0.s01: null vs "null") | 2.6 MiB | 0.51 | 26.82 ms | 25.95 ms | 105.6 | 15.24 ms | 13.51 ms | 202.8 | 63.6 MiB | 88.3 MiB |
| tsv | lossy (0.s01: null vs "null") | 2.6 MiB | 0.51 | 26.39 ms | 26.51 ms | 103.4 | 15.56 ms | 13.36 ms | 205.1 | 63.6 MiB | 94.6 MiB |
| markdown | lossy (0.s01: null vs "null") | 3.4 MiB | 0.66 | 26.51 ms | 26.14 ms | 137.0 | 36.88 ms | 34.27 ms | 104.5 | 67.0 MiB | 93.7 MiB |
| toon | semantic | 2.6 MiB | 0.51 | 25.51 ms | 24.37 ms | 113.8 | 54.11 ms | 49.77 ms | 55.7 | 64.9 MiB | 94.6 MiB |
| toon-tab | semantic | 2.6 MiB | 0.51 | 24.43 ms | 24.24 ms | 114.4 | 53.94 ms | 49.54 ms | 56.0 | 64.5 MiB | 96.3 MiB |

## deep_nesting (2000 records, 625.2 KiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 112.9 µs | 29.5 µs | 38.9 µs | 21672.6 | - | 6.4 MiB |
| parse | serde_json | 3.48 ms | 2.58 ms | 2.61 ms | 247.7 | 773757 | 17.1 MiB |
| parse | simd-json | 3.46 ms | 2.40 ms | 2.45 ms | 266.7 | 833051 | 19.7 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 625.2 KiB | 1.00 | 1.11 ms | 1.05 ms | 606.9 | 2.65 ms | 2.57 ms | 249.0 | 16.0 MiB | 16.7 MiB |
| json-pretty | byte | 2.5 MiB | 4.18 | 3.37 ms | 3.25 ms | 822.3 | 3.99 ms | 3.74 ms | 715.7 | 22.0 MiB | 22.7 MiB |
| jsonl | byte | 625.2 KiB | 1.00 | 1.08 ms | 1.08 ms | 590.7 | 2.75 ms | 2.64 ms | 242.5 | 16.1 MiB | 17.3 MiB |
| csv | unsupported: record 0 field "tree" is a object (nested) | | | | | | | | | | |
| tsv | unsupported: record 0 field "tree" is a object (nested) | | | | | | | | | | |
| markdown | unsupported: record 0 field "tree" is a object (nested) | | | | | | | | | | |
| toon | byte | 1.5 MiB | 2.50 | 5.49 ms | 4.92 ms | 325.7 | 10.39 ms | 9.77 ms | 164.1 | 21.9 MiB | 48.1 MiB |
| toon-tab | byte | 1.6 MiB | 2.54 | 5.54 ms | 5.24 ms | 310.7 | 10.89 ms | 9.87 ms | 165.0 | 22.0 MiB | 47.4 MiB |

## empty_values (10000 records, 1.9 MiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 255.7 µs | 76.7 µs | 94.7 µs | 25633.9 | - | 7.7 MiB |
| parse | serde_json | 7.19 ms | 5.95 ms | 6.09 ms | 330.4 | 1680613 | 24.1 MiB |
| parse | simd-json | 6.25 ms | 4.14 ms | 4.66 ms | 474.7 | 2414888 | 42.5 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 1.9 MiB | 1.00 | 2.27 ms | 1.92 ms | 1023.5 | 6.37 ms | 6.16 ms | 319.3 | 25.1 MiB | 26.2 MiB |
| json-pretty | byte | 2.8 MiB | 1.47 | 3.36 ms | 3.01 ms | 962.0 | 6.73 ms | 6.42 ms | 451.0 | 26.0 MiB | 29.8 MiB |
| jsonl | byte | 1.9 MiB | 1.00 | 2.30 ms | 1.94 ms | 1013.6 | 6.30 ms | 6.22 ms | 316.0 | 25.1 MiB | 28.3 MiB |
| csv | unsupported: record 0 field "empty_a" is a array (nested) | | | | | | | | | | |
| tsv | unsupported: record 0 field "empty_a" is a array (nested) | | | | | | | | | | |
| markdown | unsupported: record 0 field "empty_a" is a array (nested) | | | | | | | | | | |
| toon | byte | 2.2 MiB | 1.18 | 10.40 ms | 9.76 ms | 238.2 | 20.43 ms | 20.60 ms | 112.9 | 29.8 MiB | 54.1 MiB |
| toon-tab | byte | 2.2 MiB | 1.18 | 10.33 ms | 9.44 ms | 246.5 | 20.75 ms | 20.76 ms | 112.1 | 29.1 MiB | 60.5 MiB |

## long_text (2000 records, 2.3 MiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 321.2 µs | 92.2 µs | 114.0 µs | 26446.4 | - | 8.1 MiB |
| parse | serde_json | 1.54 ms | 1.15 ms | 1.21 ms | 2126.2 | 1744186 | 14.3 MiB |
| parse | simd-json | 1.85 ms | 1.26 ms | 1.43 ms | 1934.9 | 1587249 | 19.4 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 2.3 MiB | 1.00 | 1.50 ms | 1.20 ms | 2032.7 | 1.16 ms | 1.09 ms | 2226.5 | 16.4 MiB | 16.6 MiB |
| json-pretty | byte | 2.4 MiB | 1.02 | 1.33 ms | 1.25 ms | 1997.3 | 1.30 ms | 1.12 ms | 2225.7 | 14.1 MiB | 15.8 MiB |
| jsonl | byte | 2.3 MiB | 1.00 | 1.54 ms | 1.25 ms | 1955.4 | 1.91 ms | 1.80 ms | 1350.9 | 16.4 MiB | 17.5 MiB |
| csv | byte | 2.3 MiB | 0.97 | 1.10 ms | 803.9 µs | 2935.9 | 2.73 ms | 2.34 ms | 1008.7 | 16.3 MiB | 19.5 MiB |
| tsv | byte | 2.3 MiB | 0.97 | 1.28 ms | 866.6 µs | 2723.1 | 2.29 ms | 2.05 ms | 1150.1 | 16.4 MiB | 19.4 MiB |
| markdown | byte | 2.3 MiB | 0.98 | 4.92 ms | 5.01 ms | 475.1 | 7.52 ms | 7.04 ms | 338.0 | 28.1 MiB | 18.7 MiB |
| toon | byte | 2.3 MiB | 0.97 | 6.21 ms | 5.87 ms | 403.8 | 11.35 ms | 11.16 ms | 212.5 | 18.5 MiB | 20.4 MiB |
| toon-tab | byte | 2.3 MiB | 0.97 | 6.59 ms | 6.30 ms | 376.4 | 11.26 ms | 10.80 ms | 219.7 | 18.6 MiB | 20.4 MiB |

