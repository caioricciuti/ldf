# ldf-bench results

Started 20261017-211624 UTC. Command: `./target/release/ldf-bench perf --scale medium --iters 7 --out results/medium`

Machine: Apple M2 Pro (aarch64), 10 logical CPUs, 16.0 GiB RAM, Darwin 27.0.0. release profile, commit 41bf21d.

Toolchain: rustc 1.98.0 (88d9e12ae 2026-08-18). Crates: arrow 60.0.0, criterion 0.8.2, csv 1.4.0, parquet 60.0.0, serde_json 1.0.151, simd-json 0.18.1, tiktoken-rs 0.12.1.

Each operation runs in its own child process. Iteration 0 is the cold run; medians and p95 are over the remaining 6 warm iterations. MB = 1e6 bytes. Peak RSS is the child process high-water mark after the operation, as reported by getrusage, minus nothing: it includes the loaded input. See docs/benchmark-methodology.md.

## flat_users (10000 records, 1.4 MiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 177.4 µs | 53.4 µs | 62.6 µs | 26544.7 | - | 7.2 MiB |
| parse | serde_json | 4.39 ms | 3.56 ms | 3.69 ms | 397.9 | 2807165 | 15.5 MiB |
| parse | simd-json | 3.71 ms | 2.64 ms | 2.73 ms | 536.3 | 3783609 | 22.7 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 1.4 MiB | 1.00 | 1.46 ms | 1.43 ms | 989.0 | 3.87 ms | 3.58 ms | 396.3 | 14.5 MiB | 16.1 MiB |
| json-pretty | byte | 1.9 MiB | 1.38 | 1.98 ms | 1.92 ms | 1022.1 | 4.16 ms | 3.83 ms | 511.5 | 15.0 MiB | 17.0 MiB |
| jsonl | byte | 1.4 MiB | 1.00 | 1.47 ms | 1.43 ms | 992.8 | 4.12 ms | 3.91 ms | 362.4 | 14.5 MiB | 17.1 MiB |
| csv | semantic | 632.1 KiB | 0.46 | 4.23 ms | 4.11 ms | 157.5 | 4.19 ms | 3.09 ms | 209.2 | 14.4 MiB | 21.0 MiB |
| tsv | semantic | 632.1 KiB | 0.46 | 4.21 ms | 4.16 ms | 155.5 | 4.15 ms | 3.10 ms | 209.0 | 14.4 MiB | 21.0 MiB |
| markdown | semantic | 807.9 KiB | 0.58 | 4.33 ms | 4.36 ms | 189.9 | 8.99 ms | 7.78 ms | 106.4 | 15.1 MiB | 21.5 MiB |
| toon | semantic | 651.6 KiB | 0.47 | 4.34 ms | 4.17 ms | 160.0 | 5.85 ms | 4.74 ms | 140.7 | 16.4 MiB | 22.3 MiB |
| toon-tab | semantic | 651.6 KiB | 0.47 | 4.45 ms | 3.94 ms | 169.5 | 6.07 ms | 5.04 ms | 132.3 | 15.1 MiB | 21.8 MiB |
| yaml | byte | 1.3 MiB | 0.99 | 6.15 ms | 5.80 ms | 241.3 | 40.37 ms | 37.10 ms | 37.7 | 16.0 MiB | 34.1 MiB |

## nested_orders (10000 records, 2.8 MiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 386.2 µs | 109.7 µs | 143.6 µs | 26999.5 | - | 8.6 MiB |
| parse | serde_json | 11.21 ms | 9.64 ms | 11.15 ms | 307.2 | 1037275 | 32.2 MiB |
| parse | simd-json | 10.31 ms | 7.71 ms | 9.08 ms | 383.9 | 1296201 | 67.8 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 2.8 MiB | 1.00 | 4.57 ms | 3.88 ms | 763.2 | 9.98 ms | 9.19 ms | 322.1 | 34.6 MiB | 35.0 MiB |
| json-pretty | byte | 5.0 MiB | 1.77 | 6.74 ms | 6.33 ms | 828.5 | 11.46 ms | 10.36 ms | 506.0 | 66.9 MiB | 39.4 MiB |
| jsonl | byte | 2.8 MiB | 1.00 | 4.25 ms | 3.94 ms | 752.4 | 10.52 ms | 9.66 ms | 306.6 | 34.6 MiB | 34.8 MiB |
| csv | unsupported: record 0 field "customer" is a object (nested) | | | | | | | | | | |
| tsv | unsupported: record 0 field "customer" is a object (nested) | | | | | | | | | | |
| markdown | unsupported: record 0 field "customer" is a object (nested) | | | | | | | | | | |
| toon | semantic | 2.9 MiB | 1.04 | 19.36 ms | 17.80 ms | 172.6 | 29.27 ms | 26.69 ms | 115.0 | 49.5 MiB | 94.1 MiB |
| toon-tab | semantic | 2.9 MiB | 1.04 | 20.51 ms | 18.01 ms | 171.0 | 28.35 ms | 25.48 ms | 120.9 | 50.2 MiB | 94.7 MiB |
| yaml | byte | 3.2 MiB | 1.13 | 15.39 ms | 14.65 ms | 228.7 | 98.39 ms | 94.72 ms | 35.4 | 38.3 MiB | 91.9 MiB |

## sparse_events (10000 records, 1.5 MiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 222.3 µs | 66.5 µs | 94.0 µs | 22982.2 | - | 7.3 MiB |
| parse | serde_json | 5.66 ms | 4.54 ms | 4.88 ms | 336.5 | 2200906 | 17.5 MiB |
| parse | simd-json | 4.58 ms | 3.38 ms | 3.45 ms | 452.3 | 2958270 | 23.4 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 1.5 MiB | 1.00 | 1.77 ms | 1.67 ms | 917.9 | 5.12 ms | 4.61 ms | 331.6 | 16.5 MiB | 19.8 MiB |
| json-pretty | byte | 2.0 MiB | 1.36 | 2.21 ms | 2.14 ms | 968.7 | 4.93 ms | 4.74 ms | 438.0 | 17.0 MiB | 19.0 MiB |
| jsonl | byte | 1.5 MiB | 1.00 | 1.73 ms | 1.69 ms | 907.0 | 5.05 ms | 4.77 ms | 320.2 | 16.5 MiB | 19.0 MiB |
| csv | lossy (0: object size 15 vs 9) | 685.5 KiB | 0.46 | 5.24 ms | 5.12 ms | 137.2 | 5.76 ms | 4.64 ms | 151.2 | 16.3 MiB | 28.1 MiB |
| tsv | lossy (0: object size 15 vs 9) | 685.5 KiB | 0.46 | 5.11 ms | 5.13 ms | 136.8 | 5.56 ms | 4.58 ms | 153.2 | 16.4 MiB | 25.0 MiB |
| markdown | lossy (0: object size 15 vs 9) | 998.1 KiB | 0.67 | 5.11 ms | 5.02 ms | 203.7 | 12.41 ms | 11.26 ms | 90.7 | 17.1 MiB | 24.3 MiB |
| toon | semantic | 1.6 MiB | 1.11 | 7.59 ms | 7.12 ms | 238.4 | 14.33 ms | 13.39 ms | 126.7 | 18.4 MiB | 49.5 MiB |
| toon-tab | semantic | 1.6 MiB | 1.11 | 7.56 ms | 7.33 ms | 231.7 | 14.97 ms | 13.88 ms | 122.3 | 20.1 MiB | 50.7 MiB |
| yaml | byte | 1.4 MiB | 0.99 | 6.64 ms | 6.10 ms | 247.3 | 41.11 ms | 38.46 ms | 39.2 | 19.6 MiB | 38.8 MiB |

## high_card_strings (10000 records, 2.3 MiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 346.0 µs | 82.2 µs | 104.6 µs | 28871.1 | - | 8.1 MiB |
| parse | serde_json | 4.49 ms | 3.37 ms | 3.54 ms | 704.3 | 2968039 | 18.3 MiB |
| parse | simd-json | 3.88 ms | 2.43 ms | 2.69 ms | 976.9 | 4117062 | 26.2 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 2.3 MiB | 1.00 | 1.90 ms | 1.56 ms | 1524.6 | 3.52 ms | 3.33 ms | 713.4 | 21.0 MiB | 21.1 MiB |
| json-pretty | byte | 2.7 MiB | 1.18 | 2.09 ms | 2.16 ms | 1295.4 | 4.07 ms | 3.59 ms | 777.4 | 34.2 MiB | 20.8 MiB |
| jsonl | byte | 2.3 MiB | 1.00 | 1.88 ms | 1.60 ms | 1486.9 | 3.98 ms | 3.69 ms | 643.2 | 20.5 MiB | 20.9 MiB |
| csv | byte | 1.8 MiB | 0.78 | 3.33 ms | 3.01 ms | 612.5 | 4.48 ms | 3.39 ms | 542.9 | 18.6 MiB | 25.5 MiB |
| tsv | byte | 1.8 MiB | 0.78 | 3.24 ms | 2.99 ms | 616.1 | 4.48 ms | 3.44 ms | 535.8 | 18.6 MiB | 25.5 MiB |
| markdown | byte | 1.9 MiB | 0.84 | 7.88 ms | 7.68 ms | 258.2 | 14.02 ms | 12.65 ms | 156.7 | 20.6 MiB | 26.9 MiB |
| toon | byte | 1.8 MiB | 0.79 | 6.17 ms | 5.50 ms | 342.1 | 8.22 ms | 7.18 ms | 262.1 | 20.5 MiB | 28.4 MiB |
| toon-tab | byte | 1.8 MiB | 0.79 | 6.21 ms | 5.56 ms | 338.7 | 8.38 ms | 7.20 ms | 261.4 | 20.6 MiB | 29.3 MiB |
| yaml | byte | 2.2 MiB | 0.97 | 7.15 ms | 6.26 ms | 369.5 | 40.35 ms | 38.01 ms | 60.9 | 20.7 MiB | 39.3 MiB |

## repetitive_logs (10000 records, 1.4 MiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 212.2 µs | 56.7 µs | 67.8 µs | 25630.7 | - | 7.2 MiB |
| parse | serde_json | 4.54 ms | 3.62 ms | 3.71 ms | 401.8 | 2765295 | 15.6 MiB |
| parse | simd-json | 3.71 ms | 2.54 ms | 2.58 ms | 571.1 | 3930335 | 22.0 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 1.4 MiB | 1.00 | 1.44 ms | 1.44 ms | 1010.6 | 3.80 ms | 3.58 ms | 405.4 | 14.7 MiB | 16.4 MiB |
| json-pretty | byte | 1.8 MiB | 1.33 | 1.90 ms | 1.87 ms | 1035.1 | 4.10 ms | 3.90 ms | 495.9 | 15.1 MiB | 17.0 MiB |
| jsonl | byte | 1.4 MiB | 1.00 | 1.46 ms | 1.45 ms | 1001.9 | 4.23 ms | 3.85 ms | 377.6 | 14.7 MiB | 17.4 MiB |
| csv | byte | 676.7 KiB | 0.48 | 3.58 ms | 3.55 ms | 195.5 | 4.08 ms | 3.06 ms | 226.7 | 14.6 MiB | 21.2 MiB |
| tsv | byte | 676.7 KiB | 0.48 | 3.57 ms | 3.60 ms | 192.3 | 3.82 ms | 3.13 ms | 221.1 | 14.5 MiB | 16.9 MiB |
| markdown | byte | 833.1 KiB | 0.59 | 3.86 ms | 3.78 ms | 225.9 | 8.73 ms | 8.10 ms | 105.3 | 15.2 MiB | 16.8 MiB |
| toon | byte | 715.8 KiB | 0.50 | 4.45 ms | 3.99 ms | 183.8 | 6.23 ms | 5.36 ms | 136.8 | 15.1 MiB | 22.3 MiB |
| toon-tab | byte | 715.8 KiB | 0.50 | 4.47 ms | 3.97 ms | 184.6 | 6.38 ms | 5.35 ms | 137.0 | 15.1 MiB | 21.6 MiB |
| yaml | byte | 1.3 MiB | 0.97 | 6.07 ms | 5.62 ms | 249.4 | 34.34 ms | 33.25 ms | 42.2 | 16.1 MiB | 33.2 MiB |

## unicode_strings (10000 records, 1.2 MiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 196.7 µs | 53.9 µs | 64.6 µs | 24140.2 | - | 7.0 MiB |
| parse | serde_json | 5.37 ms | 4.20 ms | 4.35 ms | 310.1 | 2381272 | 15.3 MiB |
| parse | simd-json | 4.31 ms | 2.96 ms | 3.23 ms | 440.2 | 3380425 | 25.3 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 1.2 MiB | 1.00 | 1.52 ms | 1.47 ms | 885.9 | 4.45 ms | 4.20 ms | 310.2 | 14.4 MiB | 16.0 MiB |
| json-pretty | byte | 1.7 MiB | 1.37 | 2.00 ms | 1.89 ms | 943.9 | 4.76 ms | 4.47 ms | 398.5 | 14.9 MiB | 16.7 MiB |
| jsonl | byte | 1.2 MiB | 1.00 | 1.49 ms | 1.46 ms | 890.7 | 5.18 ms | 4.76 ms | 273.5 | 14.4 MiB | 17.1 MiB |
| csv | lossy (0.empty: null vs "") | 596.1 KiB | 0.47 | 3.59 ms | 3.57 ms | 170.9 | 5.50 ms | 4.22 ms | 144.6 | 14.3 MiB | 21.9 MiB |
| tsv | lossy (0.empty: null vs "") | 596.1 KiB | 0.47 | 3.59 ms | 3.57 ms | 170.9 | 5.10 ms | 4.22 ms | 144.6 | 14.3 MiB | 20.9 MiB |
| markdown | lossy (0.empty: null vs "") | 747.6 KiB | 0.59 | 4.04 ms | 4.01 ms | 190.9 | 9.62 ms | 8.93 ms | 85.7 | 15.0 MiB | 16.5 MiB |
| toon | byte | 671.9 KiB | 0.53 | 4.50 ms | 4.11 ms | 167.5 | 8.04 ms | 6.98 ms | 98.6 | 15.0 MiB | 21.9 MiB |
| toon-tab | byte | 669.9 KiB | 0.53 | 4.49 ms | 4.31 ms | 159.3 | 7.82 ms | 6.87 ms | 99.9 | 16.3 MiB | 22.1 MiB |
| yaml | byte | 1.2 MiB | 0.99 | 6.44 ms | 5.91 ms | 217.5 | 35.22 ms | 33.57 ms | 38.3 | 15.8 MiB | 32.8 MiB |

## numeric_mix (10000 records, 2.3 MiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 332.2 µs | 87.1 µs | 107.2 µs | 27505.0 | - | 8.1 MiB |
| parse | serde_json | 7.43 ms | 6.39 ms | 6.43 ms | 375.1 | 1566089 | 21.8 MiB |
| parse | simd-json | 6.35 ms | 4.65 ms | 4.78 ms | 515.5 | 2152196 | 34.7 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 2.3 MiB | 1.00 | 3.58 ms | 3.19 ms | 749.8 | 6.40 ms | 6.31 ms | 379.4 | 24.5 MiB | 24.6 MiB |
| json-pretty | byte | 3.1 MiB | 1.35 | 4.28 ms | 3.91 ms | 827.9 | 7.18 ms | 6.80 ms | 475.9 | 25.3 MiB | 28.5 MiB |
| jsonl | byte | 2.3 MiB | 1.00 | 3.58 ms | 3.24 ms | 739.7 | 6.84 ms | 6.74 ms | 355.4 | 24.5 MiB | 24.8 MiB |
| csv | lossy (944.huge: 2167128000339913000 vs 2.167128000339913e+18) | 1.3 MiB | 0.57 | 12.40 ms | 11.75 ms | 116.2 | 6.42 ms | 5.49 ms | 248.7 | 21.8 MiB | 31.2 MiB |
| tsv | lossy (944.huge: 2167128000339913000 vs 2.167128000339913e+18) | 1.3 MiB | 0.57 | 12.32 ms | 12.06 ms | 113.2 | 6.63 ms | 5.48 ms | 249.1 | 23.0 MiB | 31.3 MiB |
| markdown | lossy (944.huge: 2167128000339913000 vs 2.167128000339913e+18) | 1.6 MiB | 0.69 | 12.99 ms | 12.26 ms | 134.1 | 15.89 ms | 15.21 ms | 108.1 | 23.6 MiB | 31.5 MiB |
| toon | lossy (944.huge: 2167128000339913000 vs 2.167128000339913e+18) | 1.3 MiB | 0.58 | 11.48 ms | 10.22 ms | 135.5 | 9.80 ms | 8.62 ms | 160.6 | 23.3 MiB | 34.8 MiB |
| toon-tab | lossy (944.huge: 2167128000339913000 vs 2.167128000339913e+18) | 1.3 MiB | 0.58 | 11.29 ms | 10.05 ms | 137.7 | 9.87 ms | 8.55 ms | 161.8 | 23.2 MiB | 34.8 MiB |
| yaml | byte | 2.4 MiB | 1.04 | 10.77 ms | 9.97 ms | 250.2 | 64.62 ms | 60.19 ms | 41.4 | 27.2 MiB | 63.0 MiB |

## mixed_types (10000 records, 736.1 KiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 124.2 µs | 31.6 µs | 35.8 µs | 23819.8 | - | 6.5 MiB |
| parse | serde_json | 3.51 ms | 2.73 ms | 2.80 ms | 276.4 | 3667174 | 14.9 MiB |
| parse | simd-json | 3.59 ms | 2.62 ms | 2.74 ms | 287.8 | 3818282 | 18.3 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 736.1 KiB | 1.00 | 1.29 ms | 1.27 ms | 592.9 | 2.80 ms | 2.71 ms | 278.6 | 14.6 MiB | 15.4 MiB |
| json-pretty | byte | 1.3 MiB | 1.85 | 1.92 ms | 1.90 ms | 734.4 | 3.14 ms | 2.98 ms | 468.5 | 15.3 MiB | 16.8 MiB |
| jsonl | byte | 736.1 KiB | 1.00 | 1.28 ms | 1.25 ms | 601.9 | 3.06 ms | 2.99 ms | 252.4 | 14.6 MiB | 16.2 MiB |
| csv | unsupported: record 0 field "value" is a object (nested) | | | | | | | | | | |
| tsv | unsupported: record 0 field "value" is a object (nested) | | | | | | | | | | |
| markdown | unsupported: record 0 field "value" is a object (nested) | | | | | | | | | | |
| toon | semantic | 890.0 KiB | 1.21 | 4.03 ms | 3.85 ms | 236.7 | 8.15 ms | 7.70 ms | 118.4 | 15.7 MiB | 20.4 MiB |
| toon-tab | semantic | 893.3 KiB | 1.21 | 4.09 ms | 3.85 ms | 237.8 | 8.39 ms | 7.71 ms | 118.6 | 15.8 MiB | 21.8 MiB |
| yaml | byte | 812.4 KiB | 1.10 | 3.98 ms | 3.73 ms | 223.2 | 30.61 ms | 27.85 ms | 29.9 | 15.6 MiB | 37.5 MiB |

## wide_table (10000 records, 5.2 MiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 664.9 µs | 213.2 µs | 258.3 µs | 25372.0 | - | 11.0 MiB |
| parse | serde_json | 20.82 ms | 17.40 ms | 19.25 ms | 310.9 | 574596 | 63.9 MiB |
| parse | simd-json | 16.96 ms | 12.88 ms | 15.78 ms | 420.0 | 776410 | 112.2 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 5.2 MiB | 1.00 | 9.38 ms | 8.66 ms | 624.4 | 20.58 ms | 18.85 ms | 287.0 | 94.5 MiB | 67.0 MiB |
| json-pretty | byte | 7.6 MiB | 1.47 | 11.62 ms | 10.41 ms | 761.7 | 20.41 ms | 18.90 ms | 419.5 | 110.9 MiB | 71.5 MiB |
| jsonl | byte | 5.2 MiB | 1.00 | 8.70 ms | 8.25 ms | 655.9 | 19.12 ms | 18.08 ms | 299.2 | 92.5 MiB | 68.2 MiB |
| csv | lossy (0.s01: null vs "null") | 2.6 MiB | 0.51 | 26.26 ms | 25.90 ms | 105.8 | 15.76 ms | 13.16 ms | 208.2 | 61.5 MiB | 95.6 MiB |
| tsv | lossy (0.s01: null vs "null") | 2.6 MiB | 0.51 | 26.72 ms | 25.56 ms | 107.2 | 15.92 ms | 13.16 ms | 208.3 | 61.6 MiB | 95.6 MiB |
| markdown | lossy (0.s01: null vs "null") | 3.4 MiB | 0.66 | 26.73 ms | 25.07 ms | 142.8 | 37.38 ms | 34.49 ms | 103.8 | 65.0 MiB | 93.8 MiB |
| toon | semantic | 2.6 MiB | 0.51 | 24.68 ms | 22.32 ms | 124.2 | 52.14 ms | 49.48 ms | 56.0 | 63.7 MiB | 89.6 MiB |
| toon-tab | semantic | 2.6 MiB | 0.51 | 24.40 ms | 25.23 ms | 109.9 | 51.53 ms | 49.65 ms | 55.9 | 65.0 MiB | 90.1 MiB |
| yaml | byte | 5.4 MiB | 1.04 | 26.71 ms | 26.53 ms | 211.6 | 180.92 ms | 166.69 ms | 33.7 | 93.9 MiB | 168.7 MiB |

## deep_nesting (2000 records, 625.2 KiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 103.8 µs | 31.6 µs | 38.3 µs | 20231.7 | - | 6.4 MiB |
| parse | serde_json | 3.32 ms | 2.71 ms | 2.81 ms | 236.6 | 739115 | 17.1 MiB |
| parse | simd-json | 3.55 ms | 2.51 ms | 2.60 ms | 255.3 | 797408 | 22.4 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 625.2 KiB | 1.00 | 1.16 ms | 1.07 ms | 600.6 | 2.73 ms | 2.61 ms | 245.5 | 16.8 MiB | 17.4 MiB |
| json-pretty | byte | 2.5 MiB | 4.18 | 3.55 ms | 3.22 ms | 831.4 | 4.04 ms | 3.75 ms | 712.6 | 22.0 MiB | 22.7 MiB |
| jsonl | byte | 625.2 KiB | 1.00 | 1.06 ms | 1.05 ms | 609.8 | 2.81 ms | 2.65 ms | 241.3 | 16.0 MiB | 17.3 MiB |
| csv | unsupported: record 0 field "tree" is a object (nested) | | | | | | | | | | |
| tsv | unsupported: record 0 field "tree" is a object (nested) | | | | | | | | | | |
| markdown | unsupported: record 0 field "tree" is a object (nested) | | | | | | | | | | |
| toon | byte | 1.5 MiB | 2.50 | 5.42 ms | 4.91 ms | 326.4 | 10.37 ms | 9.90 ms | 162.0 | 18.9 MiB | 47.4 MiB |
| toon-tab | byte | 1.6 MiB | 2.54 | 5.38 ms | 4.93 ms | 330.4 | 10.25 ms | 9.83 ms | 165.7 | 18.8 MiB | 45.9 MiB |
| yaml | byte | 1.6 MiB | 2.64 | 3.73 ms | 3.41 ms | 496.3 | 35.28 ms | 31.69 ms | 53.4 | 18.7 MiB | 42.0 MiB |

## empty_values (10000 records, 1.9 MiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 272.3 µs | 66.8 µs | 83.3 µs | 29422.4 | - | 7.7 MiB |
| parse | serde_json | 7.49 ms | 5.96 ms | 6.12 ms | 329.8 | 1677929 | 23.8 MiB |
| parse | simd-json | 5.91 ms | 4.13 ms | 4.38 ms | 475.5 | 2418672 | 35.5 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 1.9 MiB | 1.00 | 2.10 ms | 1.91 ms | 1026.7 | 6.01 ms | 5.95 ms | 330.3 | 23.1 MiB | 24.0 MiB |
| json-pretty | byte | 2.8 MiB | 1.47 | 3.35 ms | 3.02 ms | 959.5 | 6.68 ms | 6.41 ms | 451.4 | 26.0 MiB | 28.8 MiB |
| jsonl | byte | 1.9 MiB | 1.00 | 2.13 ms | 1.93 ms | 1018.3 | 6.56 ms | 6.16 ms | 319.2 | 23.1 MiB | 25.0 MiB |
| csv | unsupported: record 0 field "empty_a" is a array (nested) | | | | | | | | | | |
| tsv | unsupported: record 0 field "empty_a" is a array (nested) | | | | | | | | | | |
| markdown | unsupported: record 0 field "empty_a" is a array (nested) | | | | | | | | | | |
| toon | byte | 2.2 MiB | 1.18 | 10.17 ms | 9.34 ms | 248.9 | 21.26 ms | 20.23 ms | 114.9 | 27.8 MiB | 70.4 MiB |
| toon-tab | byte | 2.2 MiB | 1.18 | 10.09 ms | 9.28 ms | 250.6 | 21.15 ms | 20.29 ms | 114.6 | 28.0 MiB | 66.1 MiB |
| yaml | byte | 1.9 MiB | 1.04 | 8.11 ms | 7.60 ms | 268.0 | 59.59 ms | 56.81 ms | 35.9 | 25.2 MiB | 64.1 MiB |

## long_text (2000 records, 2.3 MiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 299.5 µs | 98.4 µs | 117.9 µs | 24782.9 | - | 8.1 MiB |
| parse | serde_json | 1.49 ms | 1.09 ms | 1.13 ms | 2241.6 | 1838834 | 14.1 MiB |
| parse | simd-json | 1.85 ms | 1.27 ms | 1.68 ms | 1914.6 | 1570603 | 20.0 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 2.3 MiB | 1.00 | 1.43 ms | 1.19 ms | 2055.5 | 1.15 ms | 1.07 ms | 2274.2 | 16.3 MiB | 16.5 MiB |
| json-pretty | byte | 2.4 MiB | 1.02 | 1.30 ms | 1.21 ms | 2062.3 | 1.35 ms | 1.11 ms | 2251.7 | 13.1 MiB | 15.7 MiB |
| jsonl | byte | 2.3 MiB | 1.00 | 1.50 ms | 1.17 ms | 2075.8 | 1.85 ms | 1.73 ms | 1410.3 | 16.3 MiB | 16.9 MiB |
| csv | byte | 2.3 MiB | 0.97 | 1.05 ms | 778.9 µs | 3030.1 | 2.53 ms | 2.26 ms | 1042.1 | 16.4 MiB | 19.3 MiB |
| tsv | byte | 2.3 MiB | 0.97 | 1.14 ms | 873.0 µs | 2703.0 | 2.26 ms | 2.00 ms | 1178.7 | 16.3 MiB | 19.5 MiB |
| markdown | byte | 2.3 MiB | 0.98 | 4.90 ms | 4.90 ms | 485.4 | 7.21 ms | 6.83 ms | 348.5 | 26.9 MiB | 18.5 MiB |
| toon | byte | 2.3 MiB | 0.97 | 4.84 ms | 4.48 ms | 529.2 | 10.37 ms | 10.02 ms | 236.8 | 16.7 MiB | 22.7 MiB |
| toon-tab | byte | 2.3 MiB | 0.97 | 5.34 ms | 4.89 ms | 484.7 | 10.24 ms | 9.76 ms | 243.1 | 16.6 MiB | 22.6 MiB |
| yaml | byte | 2.3 MiB | 1.00 | 7.00 ms | 6.49 ms | 374.9 | 17.73 ms | 17.07 ms | 142.5 | 16.7 MiB | 24.7 MiB |

