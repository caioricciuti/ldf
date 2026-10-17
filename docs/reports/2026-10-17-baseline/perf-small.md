# ldf-bench results

Started 20261017-222521 UTC. Command: `./target/release/ldf-bench perf --scale small --iters 7 --out results/small`

Machine: Apple M2 Pro (aarch64), 10 logical CPUs, 16.0 GiB RAM, Darwin 27.0.0. release profile, commit a7bcfcb.

Toolchain: rustc 1.98.0 (88d9e12ae 2026-08-18). Crates: arrow 60.0.0, criterion 0.8.2, csv 1.4.0, parquet 60.0.0, serde_json 1.0.151, simd-json 0.18.1, tiktoken-rs 0.12.1.

Each operation runs in its own child process. Iteration 0 is the cold run; medians and p95 are over the remaining 6 warm iterations. MB = 1e6 bytes. Peak RSS is the child process high-water mark after the operation, as reported by getrusage, minus nothing: it includes the loaded input. See docs/benchmark-methodology.md.

## flat_users (20 records, 2.7 KiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 32.5 µs | 12.2 µs | 14.2 µs | 224.6 | - | 5.9 MiB |
| parse | serde_json | 39.3 µs | 9.9 µs | 10.8 µs | 276.4 | 2012579 | 6.1 MiB |
| parse | simd-json | 34.9 µs | 8.1 µs | 10.4 µs | 340.7 | 2480774 | 6.2 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 2.7 KiB | 1.00 | 12.4 µs | 3.8 µs | 720.4 | 19.0 µs | 12.3 µs | 223.5 | 6.3 MiB | 6.3 MiB |
| json-pretty | byte | 3.7 KiB | 1.39 | 14.9 µs | 4.7 µs | 816.6 | 18.6 µs | 11.1 µs | 344.1 | 6.3 MiB | 6.3 MiB |
| jsonl | byte | 2.7 KiB | 1.00 | 12.0 µs | 3.8 µs | 716.3 | 27.3 µs | 10.9 µs | 253.0 | 6.3 MiB | 6.4 MiB |
| csv | byte | 1.2 KiB | 0.46 | 39.9 µs | 11.9 µs | 106.0 | 39.3 µs | 24.8 µs | 50.7 | 6.4 MiB | 6.6 MiB |
| tsv | byte | 1.2 KiB | 0.46 | 32.4 µs | 10.8 µs | 116.7 | 39.3 µs | 25.6 µs | 49.2 | 6.4 MiB | 6.6 MiB |
| markdown | byte | 1.6 KiB | 0.61 | 29.5 µs | 11.6 µs | 145.5 | 37.1 µs | 21.3 µs | 79.2 | 6.4 MiB | 6.5 MiB |
| toon | byte | 1.3 KiB | 0.47 | 23.9 µs | 12.5 µs | 104.2 | 27.7 µs | 13.4 µs | 97.1 | 6.4 MiB | 6.4 MiB |
| toon-tab | byte | 1.3 KiB | 0.47 | 24.2 µs | 12.4 µs | 105.5 | 26.0 µs | 14.4 µs | 90.3 | 6.4 MiB | 6.4 MiB |
| yaml | byte | 2.6 KiB | 0.99 | 30.9 µs | 15.5 µs | 174.7 | 145.0 µs | 96.0 µs | 28.2 | 6.3 MiB | 6.7 MiB |
| ldf | byte | 1.3 KiB | 0.47 | 36.5 µs | 12.6 µs | 102.0 | 26.8 µs | 13.6 µs | 93.9 | 6.5 MiB | 6.4 MiB |

## nested_orders (20 records, 5.9 KiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 21.6 µs | 13.3 µs | 15.3 µs | 449.8 | - | 5.9 MiB |
| parse | serde_json | 56.4 µs | 22.5 µs | 26.7 µs | 266.1 | 887252 | 6.2 MiB |
| parse | simd-json | 51.9 µs | 20.2 µs | 25.3 µs | 297.4 | 991719 | 6.3 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 5.9 KiB | 1.00 | 20.0 µs | 9.2 µs | 651.4 | 27.4 µs | 23.4 µs | 256.1 | 6.4 MiB | 6.4 MiB |
| json-pretty | byte | 10.4 KiB | 1.77 | 23.9 µs | 13.5 µs | 789.3 | 33.8 µs | 25.3 µs | 420.0 | 6.4 MiB | 6.4 MiB |
| jsonl | byte | 5.9 KiB | 1.00 | 18.5 µs | 9.1 µs | 660.2 | 40.2 µs | 23.3 µs | 257.5 | 6.4 MiB | 6.4 MiB |
| csv | unsupported: record 0 field "customer" is a object (nested) | | | | | | | | | | |
| tsv | unsupported: record 0 field "customer" is a object (nested) | | | | | | | | | | |
| markdown | unsupported: record 0 field "customer" is a object (nested) | | | | | | | | | | |
| toon | semantic | 6.0 KiB | 1.03 | 97.7 µs | 42.2 µs | 146.6 | 102.8 µs | 61.5 µs | 100.6 | 6.5 MiB | 6.6 MiB |
| toon-tab | semantic | 6.1 KiB | 1.04 | 65.4 µs | 40.3 µs | 154.2 | 328.3 µs | 66.8 µs | 93.0 | 6.5 MiB | 6.6 MiB |
| yaml | byte | 6.6 KiB | 1.13 | 54.6 µs | 36.2 µs | 187.3 | 290.9 µs | 240.7 µs | 28.2 | 6.4 MiB | 6.8 MiB |
| ldf | byte | 2.9 KiB | 0.50 | 64.7 µs | 35.4 µs | 84.1 | 72.2 µs | 48.8 µs | 61.0 | 6.7 MiB | 6.8 MiB |

## sparse_events (20 records, 2.9 KiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 22.4 µs | 16.2 µs | 19.8 µs | 185.9 | - | 5.9 MiB |
| parse | serde_json | 46.0 µs | 14.7 µs | 19.5 µs | 204.9 | 1363652 | 6.1 MiB |
| parse | simd-json | 38.7 µs | 9.7 µs | 13.4 µs | 308.9 | 2055604 | 6.2 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 2.9 KiB | 1.00 | 14.8 µs | 3.7 µs | 810.3 | 17.5 µs | 11.9 µs | 251.7 | 6.3 MiB | 6.4 MiB |
| json-pretty | byte | 4.0 KiB | 1.36 | 13.9 µs | 4.9 µs | 835.7 | 17.6 µs | 12.0 µs | 338.9 | 6.3 MiB | 6.3 MiB |
| jsonl | byte | 2.9 KiB | 1.00 | 12.5 µs | 3.9 µs | 771.0 | 24.2 µs | 11.5 µs | 260.7 | 6.3 MiB | 6.4 MiB |
| csv | lossy (0: object size 15 vs 9) | 1.5 KiB | 0.50 | 42.8 µs | 15.0 µs | 99.9 | 48.8 µs | 28.8 µs | 52.2 | 6.5 MiB | 6.6 MiB |
| tsv | lossy (0: object size 15 vs 9) | 1.5 KiB | 0.50 | 37.5 µs | 15.0 µs | 100.3 | 48.5 µs | 28.7 µs | 52.3 | 6.5 MiB | 6.6 MiB |
| markdown | lossy (0: object size 15 vs 9) | 2.2 KiB | 0.75 | 37.3 µs | 15.6 µs | 145.0 | 55.3 µs | 31.1 µs | 72.8 | 6.5 MiB | 6.6 MiB |
| toon | byte | 3.3 KiB | 1.11 | 30.2 µs | 17.1 µs | 194.5 | 56.8 µs | 32.3 µs | 103.0 | 6.3 MiB | 6.5 MiB |
| toon-tab | byte | 3.3 KiB | 1.11 | 34.1 µs | 18.8 µs | 177.5 | 57.2 µs | 32.8 µs | 101.5 | 6.4 MiB | 6.5 MiB |
| yaml | byte | 2.9 KiB | 0.99 | 31.1 µs | 15.5 µs | 191.2 | 148.5 µs | 96.6 µs | 30.6 | 6.3 MiB | 6.7 MiB |
| ldf | semantic-unordered (object key order changed) | 1.6 KiB | 0.55 | 50.0 µs | 19.1 µs | 86.1 | 35.8 µs | 17.6 µs | 93.4 | 6.5 MiB | 6.5 MiB |

## high_card_strings (20 records, 4.6 KiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 23.2 µs | 13.1 µs | 14.4 µs | 359.3 | - | 5.9 MiB |
| parse | serde_json | 28.7 µs | 10.9 µs | 13.0 µs | 432.3 | 1839080 | 6.1 MiB |
| parse | simd-json | 30.7 µs | 6.6 µs | 14.9 µs | 716.3 | 3047619 | 6.2 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 4.6 KiB | 1.00 | 9.8 µs | 3.8 µs | 1239.7 | 12.1 µs | 10.0 µs | 470.1 | 6.3 MiB | 6.3 MiB |
| json-pretty | byte | 5.4 KiB | 1.18 | 12.2 µs | 4.7 µs | 1171.9 | 13.9 µs | 10.2 µs | 541.8 | 6.3 MiB | 6.3 MiB |
| jsonl | byte | 4.6 KiB | 1.00 | 10.5 µs | 4.0 µs | 1187.3 | 29.3 µs | 10.4 µs | 450.3 | 6.2 MiB | 6.4 MiB |
| csv | byte | 3.6 KiB | 0.78 | 33.0 µs | 7.4 µs | 496.1 | 46.6 µs | 24.7 µs | 148.7 | 6.4 MiB | 6.6 MiB |
| tsv | byte | 3.6 KiB | 0.78 | 26.8 µs | 7.4 µs | 496.0 | 43.2 µs | 25.0 µs | 147.0 | 6.4 MiB | 6.6 MiB |
| markdown | byte | 3.9 KiB | 0.85 | 36.5 µs | 15.0 µs | 266.4 | 62.3 µs | 30.6 µs | 130.9 | 6.3 MiB | 6.5 MiB |
| toon | byte | 3.7 KiB | 0.80 | 32.0 µs | 14.6 µs | 257.1 | 39.8 µs | 18.7 µs | 200.5 | 6.4 MiB | 6.5 MiB |
| toon-tab | byte | 3.7 KiB | 0.80 | 25.3 µs | 14.6 µs | 257.2 | 35.3 µs | 18.3 µs | 205.1 | 6.4 MiB | 6.5 MiB |
| yaml | byte | 4.5 KiB | 0.97 | 32.3 µs | 18.6 µs | 246.7 | 134.5 µs | 90.8 µs | 50.4 | 6.4 MiB | 6.7 MiB |
| ldf | byte | 3.6 KiB | 0.78 | 29.2 µs | 12.9 µs | 286.6 | 30.8 µs | 15.0 µs | 245.3 | 6.5 MiB | 6.5 MiB |

## repetitive_logs (20 records, 2.8 KiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 29.4 µs | 12.8 µs | 18.4 µs | 225.8 | - | 5.9 MiB |
| parse | serde_json | 29.3 µs | 10.1 µs | 16.0 µs | 284.6 | 1971220 | 6.0 MiB |
| parse | simd-json | 33.0 µs | 6.9 µs | 10.6 µs | 416.3 | 2882883 | 6.2 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 2.8 KiB | 1.00 | 10.2 µs | 3.4 µs | 840.1 | 14.5 µs | 9.9 µs | 290.6 | 6.2 MiB | 6.3 MiB |
| json-pretty | byte | 3.8 KiB | 1.33 | 10.8 µs | 4.4 µs | 883.9 | 16.5 µs | 10.9 µs | 354.6 | 6.2 MiB | 6.2 MiB |
| jsonl | byte | 2.8 KiB | 1.00 | 13.0 µs | 3.3 µs | 871.5 | 23.4 µs | 9.4 µs | 305.9 | 6.2 MiB | 6.3 MiB |
| csv | byte | 1.4 KiB | 0.49 | 42.2 µs | 15.3 µs | 92.8 | 42.1 µs | 22.8 µs | 62.1 | 6.4 MiB | 6.5 MiB |
| tsv | byte | 1.4 KiB | 0.49 | 26.0 µs | 9.5 µs | 149.8 | 40.1 µs | 24.6 µs | 57.7 | 6.3 MiB | 6.5 MiB |
| markdown | byte | 1.8 KiB | 0.62 | 25.9 µs | 9.5 µs | 189.6 | 42.3 µs | 20.5 µs | 87.5 | 6.3 MiB | 6.4 MiB |
| toon | byte | 1.5 KiB | 0.52 | 22.3 µs | 10.9 µs | 137.9 | 32.0 µs | 17.6 µs | 85.4 | 6.3 MiB | 6.4 MiB |
| toon-tab | byte | 1.5 KiB | 0.52 | 25.9 µs | 11.1 µs | 134.9 | 33.0 µs | 17.0 µs | 88.7 | 6.3 MiB | 6.4 MiB |
| yaml | byte | 2.7 KiB | 0.97 | 28.8 µs | 14.6 µs | 191.4 | 127.1 µs | 82.9 µs | 33.6 | 6.3 MiB | 6.6 MiB |
| ldf | byte | 1.4 KiB | 0.50 | 29.1 µs | 10.9 µs | 131.7 | 27.1 µs | 13.9 µs | 103.8 | 6.4 MiB | 6.4 MiB |

## unicode_strings (20 records, 2.5 KiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 23.1 µs | 11.9 µs | 13.7 µs | 214.1 | - | 5.9 MiB |
| parse | serde_json | 31.5 µs | 12.0 µs | 14.7 µs | 213.0 | 1672520 | 6.1 MiB |
| parse | simd-json | 35.6 µs | 9.5 µs | 12.8 µs | 266.9 | 2096106 | 6.2 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 2.5 KiB | 1.00 | 8.6 µs | 3.7 µs | 690.8 | 16.8 µs | 11.0 µs | 231.1 | 6.2 MiB | 6.3 MiB |
| json-pretty | byte | 3.4 KiB | 1.38 | 11.2 µs | 4.8 µs | 735.3 | 22.2 µs | 13.2 µs | 265.6 | 6.2 MiB | 6.3 MiB |
| jsonl | byte | 2.5 KiB | 1.00 | 8.0 µs | 3.4 µs | 745.2 | 25.2 µs | 11.8 µs | 215.9 | 6.2 MiB | 6.3 MiB |
| csv | lossy (0.empty: null vs "") | 1.2 KiB | 0.47 | 32.5 µs | 8.4 µs | 144.0 | 45.5 µs | 25.0 µs | 48.2 | 6.4 MiB | 6.5 MiB |
| tsv | lossy (0.empty: null vs "") | 1.2 KiB | 0.47 | 24.4 µs | 8.6 µs | 140.7 | 44.6 µs | 26.0 µs | 46.5 | 6.4 MiB | 6.6 MiB |
| markdown | lossy (0.empty: null vs "") | 1.5 KiB | 0.62 | 24.3 µs | 10.1 µs | 156.5 | 40.8 µs | 24.6 µs | 64.2 | 6.4 MiB | 6.5 MiB |
| toon | byte | 1.3 KiB | 0.54 | 25.7 µs | 11.4 µs | 120.0 | 40.2 µs | 19.1 µs | 71.5 | 6.3 MiB | 6.5 MiB |
| toon-tab | byte | 1.3 KiB | 0.53 | 23.5 µs | 11.0 µs | 124.3 | 38.2 µs | 21.7 µs | 62.7 | 6.3 MiB | 6.5 MiB |
| yaml | byte | 2.5 KiB | 0.99 | 38.1 µs | 21.6 µs | 116.5 | 126.7 µs | 80.2 µs | 31.4 | 6.3 MiB | 6.6 MiB |
| ldf | byte | 1.3 KiB | 0.51 | 56.1 µs | 17.0 µs | 77.2 | 32.3 µs | 20.2 µs | 64.8 | 6.5 MiB | 6.4 MiB |

## numeric_mix (20 records, 4.7 KiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 55.6 µs | 18.8 µs | 47.3 µs | 254.9 | - | 5.9 MiB |
| parse | serde_json | 38.6 µs | 14.9 µs | 17.5 µs | 321.9 | 1342642 | 6.1 MiB |
| parse | simd-json | 41.5 µs | 10.8 µs | 14.5 µs | 445.2 | 1856838 | 6.2 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 4.7 KiB | 1.00 | 23.8 µs | 7.8 µs | 613.7 | 18.0 µs | 15.6 µs | 308.1 | 6.3 MiB | 6.3 MiB |
| json-pretty | byte | 6.3 KiB | 1.35 | 30.4 µs | 9.3 µs | 697.0 | 20.4 µs | 15.9 µs | 407.4 | 6.3 MiB | 6.3 MiB |
| jsonl | byte | 4.7 KiB | 1.00 | 19.1 µs | 7.4 µs | 651.8 | 62.2 µs | 18.7 µs | 256.2 | 6.3 MiB | 6.4 MiB |
| csv | semantic | 2.7 KiB | 0.59 | 72.8 µs | 29.1 µs | 96.7 | 49.4 µs | 37.5 µs | 74.9 | 6.5 MiB | 6.7 MiB |
| tsv | semantic | 2.7 KiB | 0.59 | 164.2 µs | 64.1 µs | 43.9 | 48.8 µs | 35.2 µs | 79.8 | 6.6 MiB | 6.7 MiB |
| markdown | semantic | 3.4 KiB | 0.73 | 70.0 µs | 31.8 µs | 109.6 | 66.2 µs | 40.4 µs | 86.1 | 6.5 MiB | 6.6 MiB |
| toon | semantic | 2.8 KiB | 0.60 | 48.9 µs | 27.0 µs | 105.8 | 43.7 µs | 23.8 µs | 119.9 | 6.5 MiB | 6.5 MiB |
| toon-tab | semantic | 2.8 KiB | 0.60 | 71.0 µs | 32.6 µs | 87.8 | 41.3 µs | 24.8 µs | 115.4 | 6.5 MiB | 6.5 MiB |
| yaml | byte | 4.9 KiB | 1.04 | 46.7 µs | 24.8 µs | 201.6 | 224.4 µs | 174.8 µs | 28.6 | 6.3 MiB | 6.7 MiB |
| ldf | byte | 2.8 KiB | 0.59 | 59.9 µs | 27.5 µs | 103.1 | 37.5 µs | 19.7 µs | 144.0 | 6.6 MiB | 6.5 MiB |

## mixed_types (20 records, 1.4 KiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 22.6 µs | 13.6 µs | 14.3 µs | 104.7 | - | 5.9 MiB |
| parse | serde_json | 20.9 µs | 8.5 µs | 10.3 µs | 168.6 | 2364625 | 6.1 MiB |
| parse | simd-json | 29.6 µs | 7.0 µs | 9.1 µs | 203.1 | 2848597 | 6.2 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 1.4 KiB | 1.00 | 12.9 µs | 2.9 µs | 488.9 | 9.3 µs | 8.6 µs | 165.7 | 6.2 MiB | 6.3 MiB |
| json-pretty | byte | 2.6 KiB | 1.90 | 12.1 µs | 4.4 µs | 619.3 | 11.4 µs | 8.9 µs | 303.8 | 6.2 MiB | 6.3 MiB |
| jsonl | byte | 1.4 KiB | 1.00 | 10.7 µs | 3.0 µs | 468.5 | 18.3 µs | 8.8 µs | 161.7 | 6.2 MiB | 6.3 MiB |
| csv | unsupported: record 0 field "value" is a object (nested) | | | | | | | | | | |
| tsv | unsupported: record 0 field "value" is a object (nested) | | | | | | | | | | |
| markdown | unsupported: record 0 field "value" is a object (nested) | | | | | | | | | | |
| toon | byte | 1.7 KiB | 1.22 | 27.0 µs | 11.0 µs | 157.0 | 45.8 µs | 20.8 µs | 83.2 | 6.3 MiB | 6.3 MiB |
| toon-tab | byte | 1.7 KiB | 1.22 | 24.1 µs | 9.1 µs | 190.2 | 59.1 µs | 27.1 µs | 64.2 | 6.2 MiB | 6.4 MiB |
| yaml | byte | 1.6 KiB | 1.12 | 26.8 µs | 9.9 µs | 161.6 | 117.0 µs | 70.7 µs | 22.5 | 6.2 MiB | 6.6 MiB |
| ldf | byte | 914 B | 0.64 | 30.2 µs | 10.0 µs | 91.8 | 28.6 µs | 12.9 µs | 70.6 | 6.4 MiB | 6.5 MiB |

## wide_table (20 records, 10.5 KiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 31.4 µs | 12.6 µs | 13.8 µs | 855.8 | - | 5.9 MiB |
| parse | serde_json | 79.7 µs | 41.4 µs | 45.2 µs | 260.1 | 483138 | 6.2 MiB |
| parse | simd-json | 60.6 µs | 29.8 µs | 33.8 µs | 361.2 | 670848 | 6.3 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 10.5 KiB | 1.00 | 29.3 µs | 16.1 µs | 667.0 | 41.2 µs | 38.6 µs | 279.0 | 6.4 MiB | 6.4 MiB |
| json-pretty | byte | 15.4 KiB | 1.47 | 31.3 µs | 21.8 µs | 726.2 | 45.4 µs | 41.0 µs | 385.6 | 6.4 MiB | 6.4 MiB |
| jsonl | byte | 10.5 KiB | 1.00 | 25.9 µs | 16.0 µs | 674.8 | 53.2 µs | 40.5 µs | 265.7 | 6.4 MiB | 6.5 MiB |
| csv | lossy (0.s01: null vs "null") | 5.5 KiB | 0.52 | 98.3 µs | 61.0 µs | 91.6 | 86.5 µs | 53.4 µs | 104.7 | 6.7 MiB | 6.8 MiB |
| tsv | lossy (0.s01: null vs "null") | 5.5 KiB | 0.52 | 107.1 µs | 63.7 µs | 87.7 | 69.5 µs | 50.6 µs | 110.4 | 6.6 MiB | 6.8 MiB |
| markdown | lossy (0.s01: null vs "null") | 7.4 KiB | 0.71 | 75.8 µs | 61.1 µs | 124.4 | 135.5 µs | 97.4 µs | 78.0 | 6.7 MiB | 6.8 MiB |
| toon | semantic | 5.5 KiB | 0.53 | 63.0 µs | 39.1 µs | 145.1 | 133.7 µs | 108.7 µs | 52.1 | 6.6 MiB | 6.7 MiB |
| toon-tab | semantic | 5.5 KiB | 0.53 | 69.1 µs | 42.4 µs | 133.7 | 147.2 µs | 105.0 µs | 54.0 | 6.6 MiB | 6.7 MiB |
| yaml | byte | 10.9 KiB | 1.04 | 83.2 µs | 61.1 µs | 183.1 | 509.1 µs | 444.4 µs | 25.2 | 6.5 MiB | 7.1 MiB |
| ldf | byte | 5.6 KiB | 0.53 | 144.7 µs | 110.5 µs | 51.7 | 131.0 µs | 114.4 µs | 50.0 | 6.7 MiB | 6.8 MiB |

## deep_nesting (10 records, 3.3 KiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 23.8 µs | 12.0 µs | 13.3 µs | 284.3 | - | 5.9 MiB |
| parse | serde_json | 45.3 µs | 16.8 µs | 17.8 µs | 202.6 | 594796 | 6.1 MiB |
| parse | simd-json | 48.7 µs | 15.2 µs | 20.3 µs | 223.7 | 656642 | 6.3 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 3.3 KiB | 1.00 | 13.2 µs | 6.0 µs | 569.6 | 18.9 µs | 17.2 µs | 197.7 | 6.2 MiB | 6.3 MiB |
| json-pretty | byte | 14.2 KiB | 4.27 | 26.3 µs | 17.9 µs | 811.5 | 28.9 µs | 23.0 µs | 631.8 | 6.2 MiB | 6.2 MiB |
| jsonl | byte | 3.3 KiB | 1.00 | 11.4 µs | 6.1 µs | 557.8 | 25.9 µs | 17.4 µs | 195.3 | 6.2 MiB | 6.3 MiB |
| csv | unsupported: record 0 field "tree" is a object (nested) | | | | | | | | | | |
| tsv | unsupported: record 0 field "tree" is a object (nested) | | | | | | | | | | |
| markdown | unsupported: record 0 field "tree" is a object (nested) | | | | | | | | | | |
| toon | byte | 8.6 KiB | 2.59 | 57.1 µs | 33.2 µs | 265.7 | 95.9 µs | 61.4 µs | 143.6 | 6.4 MiB | 6.5 MiB |
| toon-tab | byte | 8.7 KiB | 2.63 | 54.5 µs | 33.8 µs | 264.5 | 89.0 µs | 63.3 µs | 141.4 | 6.4 MiB | 6.4 MiB |
| yaml | byte | 9.0 KiB | 2.70 | 35.5 µs | 21.9 µs | 420.1 | 268.8 µs | 209.1 µs | 44.0 | 6.3 MiB | 6.8 MiB |
| ldf | byte | 2.5 KiB | 0.74 | 41.8 µs | 17.3 µs | 146.2 | 54.9 µs | 31.2 µs | 80.8 | 6.5 MiB | 6.5 MiB |

## empty_values (20 records, 3.8 KiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 17.8 µs | 11.6 µs | 13.3 µs | 334.7 | - | 5.9 MiB |
| parse | serde_json | 43.3 µs | 14.8 µs | 18.5 µs | 262.0 | 1354004 | 6.1 MiB |
| parse | simd-json | 31.0 µs | 10.2 µs | 13.2 µs | 378.3 | 1955225 | 6.1 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 3.8 KiB | 1.00 | 10.1 µs | 4.7 µs | 818.4 | 23.3 µs | 15.0 µs | 257.3 | 6.3 MiB | 6.3 MiB |
| json-pretty | byte | 5.6 KiB | 1.48 | 12.8 µs | 6.9 µs | 828.5 | 22.0 µs | 15.2 µs | 376.8 | 6.3 MiB | 6.3 MiB |
| jsonl | byte | 3.8 KiB | 1.00 | 11.7 µs | 4.7 µs | 829.0 | 27.1 µs | 19.9 µs | 194.1 | 6.2 MiB | 6.4 MiB |
| csv | unsupported: record 0 field "empty_a" is a array (nested) | | | | | | | | | | |
| tsv | unsupported: record 0 field "empty_a" is a array (nested) | | | | | | | | | | |
| markdown | unsupported: record 0 field "empty_a" is a array (nested) | | | | | | | | | | |
| toon | byte | 4.5 KiB | 1.19 | 40.0 µs | 25.4 µs | 180.8 | 86.8 µs | 46.0 µs | 99.8 | 6.3 MiB | 6.5 MiB |
| toon-tab | byte | 4.5 KiB | 1.19 | 38.4 µs | 23.5 µs | 195.7 | 77.1 µs | 47.2 µs | 97.2 | 6.3 MiB | 6.5 MiB |
| yaml | byte | 3.9 KiB | 1.04 | 36.1 µs | 18.8 µs | 213.4 | 187.9 µs | 133.4 µs | 30.1 | 6.3 MiB | 6.6 MiB |
| ldf | byte | 1.3 KiB | 0.35 | 45.2 µs | 18.2 µs | 73.9 | 41.7 µs | 21.7 µs | 62.2 | 6.5 MiB | 6.5 MiB |

## long_text (10 records, 11.9 KiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 20.0 µs | 12.0 µs | 12.9 µs | 1018.8 | - | 5.9 MiB |
| parse | serde_json | 22.8 µs | 8.4 µs | 10.1 µs | 1463.4 | 1197031 | 6.2 MiB |
| parse | simd-json | 25.4 µs | 8.0 µs | 11.5 µs | 1532.1 | 1253290 | 6.2 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 11.9 KiB | 1.00 | 13.7 µs | 6.6 µs | 1839.5 | 10.7 µs | 7.4 µs | 1657.6 | 6.3 MiB | 6.4 MiB |
| json-pretty | byte | 12.2 KiB | 1.02 | 15.0 µs | 6.6 µs | 1896.6 | 9.9 µs | 7.4 µs | 1684.2 | 6.3 MiB | 6.4 MiB |
| jsonl | byte | 11.9 KiB | 1.00 | 14.1 µs | 6.8 µs | 1799.8 | 26.0 µs | 12.6 µs | 968.2 | 6.3 MiB | 6.5 MiB |
| csv | byte | 11.6 KiB | 0.97 | 28.4 µs | 4.6 µs | 2588.0 | 44.4 µs | 28.9 µs | 411.1 | 6.4 MiB | 6.6 MiB |
| tsv | byte | 11.6 KiB | 0.97 | 22.8 µs | 4.8 µs | 2453.7 | 42.0 µs | 28.5 µs | 416.8 | 6.4 MiB | 6.6 MiB |
| markdown | byte | 11.7 KiB | 0.98 | 57.3 µs | 25.2 µs | 476.5 | 49.8 µs | 40.6 µs | 295.5 | 6.5 MiB | 6.7 MiB |
| toon | byte | 11.6 KiB | 0.98 | 44.8 µs | 26.5 µs | 450.6 | 66.4 µs | 55.6 µs | 214.6 | 6.5 MiB | 6.6 MiB |
| toon-tab | byte | 11.6 KiB | 0.98 | 42.2 µs | 28.6 µs | 417.1 | 60.2 µs | 54.7 µs | 218.1 | 6.5 MiB | 6.6 MiB |
| yaml | byte | 11.9 KiB | 1.00 | 44.4 µs | 31.6 µs | 385.3 | 143.0 µs | 86.1 µs | 141.6 | 6.4 MiB | 6.8 MiB |
| ldf | byte | 11.6 KiB | 0.97 | 37.5 µs | 13.9 µs | 857.2 | 28.0 µs | 22.9 µs | 521.2 | 6.5 MiB | 6.8 MiB |

