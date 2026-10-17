# ldf-bench results

Started 20261017-203951 UTC. Command: `./target/release/ldf-bench perf --scale small --iters 7 --out results/small`

Machine: Apple M2 Pro (aarch64), 10 logical CPUs, 16.0 GiB RAM, Darwin 27.0.0. release profile, commit unknown.

Toolchain: rustc 1.98.0 (88d9e12ae 2026-08-18). Crates: arrow 60.0.0, criterion 0.8.2, csv 1.4.0, parquet 60.0.0, serde_json 1.0.151, simd-json 0.18.1, tiktoken-rs 0.12.1.

Each operation runs in its own child process. Iteration 0 is the cold run; medians and p95 are over the remaining 6 warm iterations. MB = 1e6 bytes. Peak RSS is the child process high-water mark after the operation, as reported by getrusage, minus nothing: it includes the loaded input. See docs/benchmark-methodology.md.

## flat_users (20 records, 2.7 KiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 117.7 µs | 31.8 µs | 50.3 µs | 86.4 | - | 5.8 MiB |
| parse | serde_json | 61.9 µs | 23.8 µs | 28.1 µs | 115.6 | 841361 | 6.0 MiB |
| parse | simd-json | 95.5 µs | 19.2 µs | 22.1 µs | 143.2 | 1042318 | 6.1 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 2.7 KiB | 1.00 | 29.3 µs | 8.2 µs | 334.7 | 31.1 µs | 22.9 µs | 120.2 | 6.2 MiB | 6.2 MiB |
| json-pretty | byte | 3.7 KiB | 1.39 | 35.0 µs | 10.4 µs | 369.7 | 16.3 µs | 10.8 µs | 356.1 | 6.2 MiB | 6.2 MiB |
| jsonl | byte | 2.7 KiB | 1.00 | 29.9 µs | 7.7 µs | 355.3 | 50.4 µs | 24.9 µs | 110.4 | 6.2 MiB | 6.3 MiB |
| csv | byte | 1.2 KiB | 0.46 | 70.7 µs | 26.6 µs | 47.3 | 37.4 µs | 25.6 µs | 49.0 | 6.3 MiB | 6.4 MiB |
| tsv | byte | 1.2 KiB | 0.46 | 69.0 µs | 26.2 µs | 48.0 | 80.4 µs | 47.8 µs | 26.3 | 6.3 MiB | 6.4 MiB |
| markdown | byte | 1.6 KiB | 0.61 | 66.0 µs | 28.0 µs | 60.3 | 41.7 µs | 24.6 µs | 68.4 | 6.2 MiB | 6.4 MiB |
| toon | byte | 1.3 KiB | 0.47 | 49.0 µs | 23.9 µs | 54.6 | 66.9 µs | 30.6 µs | 42.6 | 6.2 MiB | 6.3 MiB |
| toon-tab | byte | 1.3 KiB | 0.47 | 47.3 µs | 25.1 µs | 51.9 | 64.3 µs | 32.1 µs | 40.6 | 6.2 MiB | 6.4 MiB |

## nested_orders (20 records, 5.9 KiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 132.3 µs | 33.7 µs | 53.4 µs | 177.8 | - | 5.8 MiB |
| parse | serde_json | 106.9 µs | 54.6 µs | 64.2 µs | 109.8 | 366270 | 6.1 MiB |
| parse | simd-json | 105.2 µs | 51.7 µs | 65.2 µs | 116.1 | 387094 | 6.2 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 5.9 KiB | 1.00 | 44.5 µs | 19.5 µs | 307.6 | 60.1 µs | 53.4 µs | 112.2 | 6.3 MiB | 6.3 MiB |
| json-pretty | byte | 10.4 KiB | 1.77 | 60.4 µs | 28.8 µs | 369.2 | 70.2 µs | 59.1 µs | 179.7 | 6.3 MiB | 6.3 MiB |
| jsonl | byte | 5.9 KiB | 1.00 | 44.7 µs | 18.5 µs | 324.9 | 81.5 µs | 57.4 µs | 104.5 | 6.3 MiB | 6.4 MiB |
| csv | unsupported: record 0 field "customer" is a object (nested) | | | | | | | | | | |
| tsv | unsupported: record 0 field "customer" is a object (nested) | | | | | | | | | | |
| markdown | unsupported: record 0 field "customer" is a object (nested) | | | | | | | | | | |
| toon | semantic | 6.0 KiB | 1.03 | 148.4 µs | 92.4 µs | 67.0 | 194.2 µs | 143.0 µs | 43.3 | 6.4 MiB | 6.5 MiB |
| toon-tab | semantic | 6.1 KiB | 1.04 | 155.4 µs | 93.8 µs | 66.2 | 197.6 µs | 143.9 µs | 43.2 | 6.3 MiB | 6.5 MiB |

## sparse_events (20 records, 2.9 KiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 113.3 µs | 31.9 µs | 49.9 µs | 94.1 | - | 5.8 MiB |
| parse | serde_json | 66.0 µs | 24.8 µs | 33.0 µs | 121.3 | 807396 | 6.0 MiB |
| parse | simd-json | 62.0 µs | 22.0 µs | 25.1 µs | 136.5 | 908245 | 6.1 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 2.9 KiB | 1.00 | 31.8 µs | 8.8 µs | 341.0 | 34.7 µs | 24.3 µs | 123.7 | 6.2 MiB | 6.2 MiB |
| json-pretty | byte | 4.0 KiB | 1.36 | 21.3 µs | 5.4 µs | 757.9 | 42.4 µs | 25.9 µs | 157.4 | 6.2 MiB | 6.3 MiB |
| jsonl | byte | 2.9 KiB | 1.00 | 32.2 µs | 8.8 µs | 340.9 | 55.8 µs | 28.1 µs | 107.0 | 6.2 MiB | 6.4 MiB |
| csv | lossy (0: object size 15 vs 9) | 1.5 KiB | 0.50 | 82.7 µs | 31.7 µs | 47.4 | 87.1 µs | 59.6 µs | 25.2 | 6.3 MiB | 6.5 MiB |
| tsv | lossy (0: object size 15 vs 9) | 1.5 KiB | 0.50 | 80.9 µs | 32.6 µs | 46.1 | 97.4 µs | 60.1 µs | 25.0 | 6.3 MiB | 6.5 MiB |
| markdown | lossy (0: object size 15 vs 9) | 2.2 KiB | 0.75 | 94.8 µs | 33.6 µs | 67.5 | 109.9 µs | 73.6 µs | 30.8 | 6.3 MiB | 6.4 MiB |
| toon | byte | 3.3 KiB | 1.11 | 76.7 µs | 41.8 µs | 79.8 | 119.1 µs | 76.2 µs | 43.7 | 6.2 MiB | 6.4 MiB |
| toon-tab | byte | 3.3 KiB | 1.11 | 84.0 µs | 43.0 µs | 77.5 | 108.0 µs | 75.6 µs | 44.1 | 6.2 MiB | 6.4 MiB |

## high_card_strings (20 records, 4.6 KiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 129.1 µs | 33.4 µs | 52.5 µs | 140.7 | - | 5.8 MiB |
| parse | serde_json | 31.6 µs | 10.0 µs | 13.2 µs | 470.1 | 2000000 | 6.0 MiB |
| parse | simd-json | 60.0 µs | 16.3 µs | 24.2 µs | 288.9 | 1229181 | 6.1 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 4.6 KiB | 1.00 | 25.0 µs | 8.6 µs | 547.7 | 29.8 µs | 22.1 µs | 213.1 | 6.2 MiB | 6.2 MiB |
| json-pretty | byte | 5.4 KiB | 1.18 | 56.0 µs | 10.4 µs | 532.0 | 28.6 µs | 22.7 µs | 244.5 | 6.2 MiB | 6.2 MiB |
| jsonl | byte | 4.6 KiB | 1.00 | 26.5 µs | 8.7 µs | 539.7 | 49.7 µs | 33.2 µs | 141.7 | 6.2 MiB | 6.3 MiB |
| csv | byte | 3.6 KiB | 0.78 | 58.7 µs | 20.1 µs | 182.1 | 80.5 µs | 51.6 µs | 71.1 | 6.3 MiB | 6.4 MiB |
| tsv | byte | 3.6 KiB | 0.78 | 60.4 µs | 18.9 µs | 194.0 | 83.1 µs | 50.7 µs | 72.3 | 6.3 MiB | 6.5 MiB |
| markdown | byte | 3.9 KiB | 0.85 | 70.0 µs | 36.3 µs | 110.3 | 119.2 µs | 81.5 µs | 49.1 | 6.2 MiB | 6.5 MiB |
| toon | byte | 3.7 KiB | 0.80 | 59.7 µs | 31.1 µs | 120.6 | 81.9 µs | 41.3 µs | 90.9 | 6.3 MiB | 6.4 MiB |
| toon-tab | byte | 3.7 KiB | 0.80 | 33.7 µs | 16.4 µs | 229.4 | 82.0 µs | 41.0 µs | 91.5 | 6.3 MiB | 6.4 MiB |

## repetitive_logs (20 records, 2.8 KiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 42.5 µs | 13.8 µs | 17.9 µs | 209.1 | - | 5.8 MiB |
| parse | serde_json | 30.4 µs | 10.8 µs | 15.0 µs | 266.6 | 1846125 | 6.0 MiB |
| parse | simd-json | 61.8 µs | 17.0 µs | 21.1 µs | 170.1 | 1177891 | 6.1 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 2.8 KiB | 1.00 | 22.3 µs | 7.2 µs | 401.8 | 16.2 µs | 10.3 µs | 280.6 | 6.1 MiB | 6.2 MiB |
| json-pretty | byte | 3.8 KiB | 1.33 | 26.8 µs | 10.5 µs | 368.0 | 33.9 µs | 24.0 µs | 160.4 | 6.1 MiB | 6.2 MiB |
| jsonl | byte | 2.8 KiB | 1.00 | 24.2 µs | 7.6 µs | 379.7 | 26.6 µs | 10.3 µs | 281.6 | 6.1 MiB | 6.2 MiB |
| csv | byte | 1.4 KiB | 0.49 | 60.2 µs | 22.4 µs | 63.3 | 81.1 µs | 48.7 µs | 29.1 | 6.2 MiB | 6.4 MiB |
| tsv | byte | 1.4 KiB | 0.49 | 60.9 µs | 22.2 µs | 63.8 | 73.8 µs | 48.8 µs | 29.1 | 6.2 MiB | 6.4 MiB |
| markdown | byte | 1.8 KiB | 0.62 | 57.9 µs | 24.6 µs | 73.0 | 82.1 µs | 51.5 µs | 34.9 | 6.2 MiB | 6.3 MiB |
| toon | byte | 1.5 KiB | 0.52 | 48.6 µs | 25.2 µs | 59.7 | 71.3 µs | 32.5 µs | 46.2 | 6.2 MiB | 6.3 MiB |
| toon-tab | byte | 1.5 KiB | 0.52 | 48.5 µs | 25.8 µs | 58.3 | 78.1 µs | 32.6 µs | 46.1 | 6.2 MiB | 6.3 MiB |

## unicode_strings (20 records, 2.5 KiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 125.1 µs | 18.1 µs | 102.4 µs | 141.0 | - | 5.8 MiB |
| parse | serde_json | 35.5 µs | 11.5 µs | 16.6 µs | 220.7 | 1732802 | 6.0 MiB |
| parse | simd-json | 62.7 µs | 20.0 µs | 27.3 µs | 127.1 | 997929 | 6.1 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 2.5 KiB | 1.00 | 21.4 µs | 7.5 µs | 340.5 | 32.1 µs | 23.8 µs | 107.2 | 6.1 MiB | 6.2 MiB |
| json-pretty | byte | 3.4 KiB | 1.38 | 25.9 µs | 9.8 µs | 359.0 | 32.1 µs | 24.3 µs | 144.3 | 6.2 MiB | 6.2 MiB |
| jsonl | byte | 2.5 KiB | 1.00 | 24.0 µs | 8.0 µs | 317.4 | 55.2 µs | 28.4 µs | 89.6 | 6.2 MiB | 6.3 MiB |
| csv | lossy (0.empty: null vs "") | 1.2 KiB | 0.47 | 58.6 µs | 20.5 µs | 58.9 | 43.5 µs | 25.9 µs | 46.6 | 6.2 MiB | 6.4 MiB |
| tsv | lossy (0.empty: null vs "") | 1.2 KiB | 0.47 | 57.2 µs | 20.7 µs | 58.3 | 100.4 µs | 54.8 µs | 22.0 | 6.3 MiB | 6.4 MiB |
| markdown | lossy (0.empty: null vs "") | 1.5 KiB | 0.62 | 53.7 µs | 23.0 µs | 68.7 | 86.0 µs | 50.0 µs | 31.6 | 6.2 MiB | 6.4 MiB |
| toon | byte | 1.3 KiB | 0.54 | 48.0 µs | 24.8 µs | 55.0 | 75.6 µs | 38.7 µs | 35.3 | 6.3 MiB | 6.4 MiB |
| toon-tab | byte | 1.3 KiB | 0.53 | 26.7 µs | 10.8 µs | 126.2 | 72.7 µs | 41.1 µs | 33.2 | 6.2 MiB | 6.3 MiB |

## numeric_mix (20 records, 4.7 KiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 28.9 µs | 13.3 µs | 16.1 µs | 360.8 | - | 5.8 MiB |
| parse | serde_json | 48.3 µs | 15.8 µs | 18.9 µs | 304.0 | 1268191 | 6.0 MiB |
| parse | simd-json | 37.3 µs | 10.2 µs | 14.0 µs | 467.8 | 1951220 | 6.1 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 4.7 KiB | 1.00 | 21.5 µs | 7.7 µs | 622.0 | 38.5 µs | 35.4 µs | 135.3 | 6.2 MiB | 6.3 MiB |
| json-pretty | byte | 6.3 KiB | 1.35 | 43.1 µs | 19.3 µs | 335.3 | 42.7 µs | 35.7 µs | 181.4 | 6.2 MiB | 6.2 MiB |
| jsonl | byte | 4.7 KiB | 1.00 | 39.3 µs | 15.4 µs | 311.8 | 63.1 µs | 38.0 µs | 126.2 | 6.2 MiB | 6.4 MiB |
| csv | semantic | 2.7 KiB | 0.59 | 68.1 µs | 28.6 µs | 98.3 | 93.7 µs | 61.6 µs | 45.6 | 6.4 MiB | 6.5 MiB |
| tsv | semantic | 2.7 KiB | 0.59 | 129.2 µs | 68.3 µs | 41.2 | 90.5 µs | 60.1 µs | 46.8 | 6.4 MiB | 6.5 MiB |
| markdown | semantic | 3.4 KiB | 0.73 | 128.5 µs | 76.3 µs | 45.6 | 139.3 µs | 92.6 µs | 37.6 | 6.4 MiB | 6.4 MiB |
| toon | semantic | 2.8 KiB | 0.60 | 105.5 µs | 61.2 µs | 46.7 | 93.4 µs | 50.3 µs | 56.8 | 6.4 MiB | 6.5 MiB |
| toon-tab | semantic | 2.8 KiB | 0.60 | 108.6 µs | 59.4 µs | 48.1 | 91.2 µs | 49.4 µs | 57.9 | 6.4 MiB | 6.5 MiB |

## mixed_types (20 records, 1.4 KiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 114.3 µs | 33.2 µs | 47.8 µs | 42.9 | - | 5.8 MiB |
| parse | serde_json | 45.0 µs | 17.2 µs | 20.3 µs | 83.1 | 1165060 | 6.0 MiB |
| parse | simd-json | 60.9 µs | 16.6 µs | 22.6 µs | 85.8 | 1203008 | 6.1 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 1.4 KiB | 1.00 | 29.1 µs | 5.8 µs | 245.3 | 21.8 µs | 17.9 µs | 79.8 | 6.1 MiB | 6.2 MiB |
| json-pretty | byte | 2.6 KiB | 1.90 | 32.8 µs | 8.7 µs | 312.6 | 13.5 µs | 9.0 µs | 300.3 | 6.2 MiB | 6.2 MiB |
| jsonl | byte | 1.4 KiB | 1.00 | 30.5 µs | 5.8 µs | 246.9 | 41.7 µs | 19.6 µs | 72.8 | 6.1 MiB | 6.3 MiB |
| csv | unsupported: record 0 field "value" is a object (nested) | | | | | | | | | | |
| tsv | unsupported: record 0 field "value" is a object (nested) | | | | | | | | | | |
| markdown | unsupported: record 0 field "value" is a object (nested) | | | | | | | | | | |
| toon | byte | 1.7 KiB | 1.22 | 56.8 µs | 22.6 µs | 76.8 | 79.8 µs | 50.8 µs | 34.1 | 6.2 MiB | 6.4 MiB |
| toon-tab | byte | 1.7 KiB | 1.22 | 53.8 µs | 21.0 µs | 82.8 | 87.4 µs | 50.7 µs | 34.3 | 6.1 MiB | 6.4 MiB |

## wide_table (20 records, 10.5 KiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 121.8 µs | 33.0 µs | 48.9 µs | 325.9 | - | 5.8 MiB |
| parse | serde_json | 76.1 µs | 41.3 µs | 44.1 µs | 260.8 | 484355 | 6.1 MiB |
| parse | simd-json | 65.0 µs | 30.6 µs | 36.0 µs | 351.9 | 653509 | 6.3 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 10.5 KiB | 1.00 | 30.8 µs | 16.5 µs | 652.7 | 96.3 µs | 93.8 µs | 114.8 | 6.3 MiB | 6.3 MiB |
| json-pretty | byte | 15.4 KiB | 1.47 | 74.3 µs | 45.1 µs | 350.5 | 104.0 µs | 103.0 µs | 153.6 | 6.4 MiB | 6.3 MiB |
| jsonl | byte | 10.5 KiB | 1.00 | 33.6 µs | 17.1 µs | 628.8 | 126.5 µs | 99.4 µs | 108.3 | 6.3 MiB | 6.5 MiB |
| csv | lossy (0.s01: null vs "null") | 5.5 KiB | 0.52 | 176.0 µs | 130.4 µs | 42.8 | 76.5 µs | 51.3 µs | 109.0 | 6.5 MiB | 6.6 MiB |
| tsv | lossy (0.s01: null vs "null") | 5.5 KiB | 0.52 | 181.0 µs | 130.4 µs | 42.9 | 146.2 µs | 107.5 µs | 52.0 | 6.5 MiB | 6.6 MiB |
| markdown | lossy (0.s01: null vs "null") | 7.4 KiB | 0.71 | 175.8 µs | 134.8 µs | 56.4 | 254.5 µs | 210.0 µs | 36.2 | 6.5 MiB | 6.6 MiB |
| toon | semantic | 5.5 KiB | 0.53 | 136.8 µs | 90.5 µs | 62.6 | 272.6 µs | 204.2 µs | 27.7 | 6.5 MiB | 6.6 MiB |
| toon-tab | semantic | 5.5 KiB | 0.53 | 130.7 µs | 86.4 µs | 65.6 | 148.6 µs | 113.7 µs | 49.9 | 6.5 MiB | 6.7 MiB |

## deep_nesting (10 records, 3.3 KiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 120.2 µs | 32.6 µs | 50.9 µs | 104.4 | - | 5.8 MiB |
| parse | serde_json | 72.8 µs | 40.9 µs | 45.1 µs | 83.4 | 244771 | 6.0 MiB |
| parse | simd-json | 106.0 µs | 41.6 µs | 53.4 µs | 81.9 | 240362 | 6.2 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 3.3 KiB | 1.00 | 28.7 µs | 12.1 µs | 280.9 | 48.8 µs | 40.3 µs | 84.4 | 6.1 MiB | 6.2 MiB |
| json-pretty | byte | 14.2 KiB | 4.27 | 31.4 µs | 18.4 µs | 793.1 | 64.2 µs | 53.3 µs | 272.9 | 6.2 MiB | 6.2 MiB |
| jsonl | byte | 3.3 KiB | 1.00 | 29.8 µs | 12.5 µs | 271.5 | 69.3 µs | 43.0 µs | 79.1 | 6.1 MiB | 6.2 MiB |
| csv | unsupported: record 0 field "tree" is a object (nested) | | | | | | | | | | |
| tsv | unsupported: record 0 field "tree" is a object (nested) | | | | | | | | | | |
| markdown | unsupported: record 0 field "tree" is a object (nested) | | | | | | | | | | |
| toon | byte | 8.6 KiB | 2.59 | 131.2 µs | 82.2 µs | 107.2 | 93.0 µs | 62.7 µs | 140.6 | 6.3 MiB | 6.4 MiB |
| toon-tab | byte | 8.7 KiB | 2.63 | 62.5 µs | 35.5 µs | 252.3 | 200.3 µs | 149.9 µs | 59.7 | 6.3 MiB | 6.4 MiB |

## empty_values (20 records, 3.8 KiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 119.3 µs | 31.9 µs | 46.2 µs | 121.2 | - | 5.8 MiB |
| parse | serde_json | 78.6 µs | 34.3 µs | 40.6 µs | 112.9 | 583235 | 6.0 MiB |
| parse | simd-json | 37.5 µs | 11.1 µs | 14.4 µs | 347.2 | 1794366 | 6.1 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 3.8 KiB | 1.00 | 25.0 µs | 9.9 µs | 389.4 | 43.2 µs | 35.6 µs | 108.8 | 6.2 MiB | 6.2 MiB |
| json-pretty | byte | 5.6 KiB | 1.48 | 34.5 µs | 15.3 µs | 374.8 | 46.2 µs | 36.8 µs | 155.7 | 6.2 MiB | 6.3 MiB |
| jsonl | byte | 3.8 KiB | 1.00 | 27.9 µs | 11.0 µs | 350.4 | 63.8 µs | 38.8 µs | 99.7 | 6.2 MiB | 6.3 MiB |
| csv | unsupported: record 0 field "empty_a" is a array (nested) | | | | | | | | | | |
| tsv | unsupported: record 0 field "empty_a" is a array (nested) | | | | | | | | | | |
| markdown | unsupported: record 0 field "empty_a" is a array (nested) | | | | | | | | | | |
| toon | byte | 4.5 KiB | 1.19 | 93.2 µs | 56.3 µs | 81.6 | 148.7 µs | 110.1 µs | 41.7 | 6.3 MiB | 6.4 MiB |
| toon-tab | byte | 4.5 KiB | 1.19 | 90.9 µs | 56.4 µs | 81.4 | 154.8 µs | 110.2 µs | 41.7 | 6.2 MiB | 6.4 MiB |

## long_text (10 records, 11.9 KiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 117.0 µs | 42.6 µs | 56.7 µs | 287.2 | - | 5.8 MiB |
| parse | serde_json | 56.6 µs | 15.5 µs | 22.4 µs | 787.7 | 644309 | 6.1 MiB |
| parse | simd-json | 56.2 µs | 18.7 µs | 28.3 µs | 654.2 | 535103 | 6.2 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 11.9 KiB | 1.00 | 31.9 µs | 14.3 µs | 857.9 | 20.8 µs | 17.4 µs | 704.4 | 6.2 MiB | 6.3 MiB |
| json-pretty | byte | 12.2 KiB | 1.02 | 38.8 µs | 13.6 µs | 919.3 | 19.8 µs | 17.0 µs | 735.9 | 6.2 MiB | 6.3 MiB |
| jsonl | byte | 11.9 KiB | 1.00 | 34.6 µs | 13.5 µs | 905.5 | 25.0 µs | 16.5 µs | 739.9 | 6.2 MiB | 6.6 MiB |
| csv | byte | 11.6 KiB | 0.97 | 62.0 µs | 12.6 µs | 942.6 | 73.1 µs | 56.4 µs | 210.3 | 6.3 MiB | 6.5 MiB |
| tsv | byte | 11.6 KiB | 0.97 | 53.8 µs | 14.0 µs | 847.2 | 67.6 µs | 57.5 µs | 206.4 | 6.3 MiB | 6.5 MiB |
| markdown | byte | 11.7 KiB | 0.98 | 93.1 µs | 53.2 µs | 225.4 | 100.5 µs | 79.8 µs | 150.3 | 6.3 MiB | 6.5 MiB |
| toon | byte | 11.6 KiB | 0.98 | 91.7 µs | 63.5 µs | 187.7 | 129.6 µs | 114.5 µs | 104.1 | 6.4 MiB | 6.6 MiB |
| toon-tab | byte | 11.6 KiB | 0.98 | 52.4 µs | 36.7 µs | 324.6 | 129.9 µs | 114.6 µs | 104.1 | 6.4 MiB | 6.6 MiB |

