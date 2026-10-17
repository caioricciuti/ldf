# ldf-bench results

Started 20261017-211623 UTC. Command: `./target/release/ldf-bench perf --scale small --iters 7 --out results/small`

Machine: Apple M2 Pro (aarch64), 10 logical CPUs, 16.0 GiB RAM, Darwin 27.0.0. release profile, commit 41bf21d.

Toolchain: rustc 1.98.0 (88d9e12ae 2026-08-18). Crates: arrow 60.0.0, criterion 0.8.2, csv 1.4.0, parquet 60.0.0, serde_json 1.0.151, simd-json 0.18.1, tiktoken-rs 0.12.1.

Each operation runs in its own child process. Iteration 0 is the cold run; medians and p95 are over the remaining 6 warm iterations. MB = 1e6 bytes. Peak RSS is the child process high-water mark after the operation, as reported by getrusage, minus nothing: it includes the loaded input. See docs/benchmark-methodology.md.

## flat_users (20 records, 2.7 KiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 17.0 µs | 11.8 µs | 13.3 µs | 233.4 | - | 5.8 MiB |
| parse | serde_json | 31.1 µs | 10.4 µs | 12.7 µs | 265.3 | 1931621 | 6.0 MiB |
| parse | simd-json | 45.1 µs | 7.8 µs | 11.0 µs | 354.5 | 2580645 | 6.1 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 2.7 KiB | 1.00 | 11.5 µs | 4.1 µs | 676.2 | 11.1 µs | 9.7 µs | 282.9 | 6.2 MiB | 6.2 MiB |
| json-pretty | byte | 3.7 KiB | 1.39 | 13.2 µs | 5.0 µs | 765.6 | 13.0 µs | 10.4 µs | 369.7 | 6.2 MiB | 6.2 MiB |
| jsonl | byte | 2.7 KiB | 1.00 | 10.6 µs | 4.0 µs | 686.5 | 24.8 µs | 10.6 µs | 259.0 | 6.2 MiB | 6.3 MiB |
| csv | byte | 1.2 KiB | 0.46 | 41.6 µs | 10.4 µs | 121.4 | 44.7 µs | 26.6 µs | 47.2 | 6.2 MiB | 6.4 MiB |
| tsv | byte | 1.2 KiB | 0.46 | 27.2 µs | 10.6 µs | 118.1 | 40.0 µs | 24.6 µs | 51.1 | 6.2 MiB | 6.4 MiB |
| markdown | byte | 1.6 KiB | 0.61 | 31.8 µs | 11.5 µs | 146.3 | 34.2 µs | 20.8 µs | 81.0 | 6.2 MiB | 6.3 MiB |
| toon | byte | 1.3 KiB | 0.47 | 25.4 µs | 11.7 µs | 111.3 | 32.6 µs | 14.1 µs | 92.1 | 6.3 MiB | 6.4 MiB |
| toon-tab | byte | 1.3 KiB | 0.47 | 22.5 µs | 11.3 µs | 115.7 | 27.8 µs | 13.5 µs | 96.4 | 6.3 MiB | 6.4 MiB |
| yaml | byte | 2.6 KiB | 0.99 | 34.5 µs | 15.7 µs | 172.9 | 145.2 µs | 93.6 µs | 28.9 | 6.2 MiB | 6.6 MiB |

## nested_orders (20 records, 5.9 KiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 16.9 µs | 12.5 µs | 13.8 µs | 479.0 | - | 5.8 MiB |
| parse | serde_json | 49.0 µs | 22.9 µs | 27.1 µs | 261.5 | 871935 | 6.0 MiB |
| parse | simd-json | 47.6 µs | 18.8 µs | 28.0 µs | 319.2 | 1064311 | 6.2 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 5.9 KiB | 1.00 | 18.2 µs | 9.1 µs | 658.8 | 26.9 µs | 22.5 µs | 266.6 | 6.3 MiB | 6.3 MiB |
| json-pretty | byte | 10.4 KiB | 1.77 | 24.3 µs | 13.4 µs | 791.8 | 32.2 µs | 25.1 µs | 422.5 | 6.3 MiB | 6.3 MiB |
| jsonl | byte | 5.9 KiB | 1.00 | 19.1 µs | 9.2 µs | 651.3 | 33.0 µs | 24.3 µs | 246.7 | 6.2 MiB | 6.3 MiB |
| csv | unsupported: record 0 field "customer" is a object (nested) | | | | | | | | | | |
| tsv | unsupported: record 0 field "customer" is a object (nested) | | | | | | | | | | |
| markdown | unsupported: record 0 field "customer" is a object (nested) | | | | | | | | | | |
| toon | semantic | 6.0 KiB | 1.03 | 116.1 µs | 62.3 µs | 99.3 | 98.0 µs | 58.9 µs | 105.1 | 6.4 MiB | 6.5 MiB |
| toon-tab | semantic | 6.1 KiB | 1.04 | 64.5 µs | 41.4 µs | 150.1 | 100.9 µs | 63.5 µs | 97.8 | 6.4 MiB | 6.5 MiB |
| yaml | byte | 6.6 KiB | 1.13 | 59.3 µs | 38.3 µs | 177.3 | 298.0 µs | 234.1 µs | 29.0 | 6.3 MiB | 6.8 MiB |

## sparse_events (20 records, 2.9 KiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 17.2 µs | 13.3 µs | 16.1 µs | 226.1 | - | 5.8 MiB |
| parse | serde_json | 31.6 µs | 12.0 µs | 23.9 µs | 250.4 | 1666667 | 6.0 MiB |
| parse | simd-json | 28.1 µs | 8.5 µs | 13.4 µs | 354.4 | 2358769 | 6.1 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 2.9 KiB | 1.00 | 12.0 µs | 4.2 µs | 717.6 | 16.5 µs | 10.6 µs | 283.4 | 6.2 MiB | 6.2 MiB |
| json-pretty | byte | 4.0 KiB | 1.36 | 14.5 µs | 5.1 µs | 791.8 | 16.7 µs | 11.0 µs | 370.4 | 6.2 MiB | 6.2 MiB |
| jsonl | byte | 2.9 KiB | 1.00 | 13.2 µs | 3.7 µs | 819.3 | 23.9 µs | 14.1 µs | 213.0 | 6.1 MiB | 6.3 MiB |
| csv | lossy (0: object size 15 vs 9) | 1.5 KiB | 0.50 | 49.4 µs | 14.2 µs | 105.6 | 54.9 µs | 31.9 µs | 47.1 | 6.3 MiB | 6.5 MiB |
| tsv | lossy (0: object size 15 vs 9) | 1.5 KiB | 0.50 | 33.8 µs | 15.1 µs | 99.2 | 45.3 µs | 28.6 µs | 52.5 | 6.3 MiB | 6.5 MiB |
| markdown | lossy (0: object size 15 vs 9) | 2.2 KiB | 0.75 | 50.3 µs | 17.2 µs | 131.8 | 53.8 µs | 30.4 µs | 74.6 | 6.3 MiB | 6.4 MiB |
| toon | byte | 3.3 KiB | 1.11 | 35.5 µs | 18.1 µs | 184.2 | 68.7 µs | 31.4 µs | 106.2 | 6.2 MiB | 6.4 MiB |
| toon-tab | byte | 3.3 KiB | 1.11 | 34.9 µs | 18.2 µs | 183.0 | 50.1 µs | 32.3 µs | 103.1 | 6.2 MiB | 6.4 MiB |
| yaml | byte | 2.9 KiB | 0.99 | 33.8 µs | 18.1 µs | 163.5 | 145.6 µs | 95.2 µs | 31.1 | 6.3 MiB | 6.7 MiB |

## high_card_strings (20 records, 4.6 KiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 21.0 µs | 12.1 µs | 20.1 µs | 387.0 | - | 5.8 MiB |
| parse | serde_json | 25.9 µs | 10.3 µs | 16.0 µs | 455.9 | 1939394 | 6.0 MiB |
| parse | simd-json | 33.8 µs | 7.2 µs | 10.7 µs | 648.4 | 2758621 | 6.1 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 4.6 KiB | 1.00 | 9.6 µs | 4.5 µs | 1049.4 | 11.9 µs | 9.1 µs | 514.0 | 6.2 MiB | 6.2 MiB |
| json-pretty | byte | 5.4 KiB | 1.18 | 14.8 µs | 4.8 µs | 1151.6 | 13.1 µs | 10.4 µs | 534.2 | 6.2 MiB | 6.2 MiB |
| jsonl | byte | 4.6 KiB | 1.00 | 12.7 µs | 5.0 µs | 936.1 | 17.6 µs | 10.0 µs | 469.0 | 6.2 MiB | 6.2 MiB |
| csv | byte | 3.6 KiB | 0.78 | 25.6 µs | 7.7 µs | 474.7 | 43.8 µs | 26.1 µs | 140.8 | 6.2 MiB | 6.5 MiB |
| tsv | byte | 3.6 KiB | 0.78 | 22.0 µs | 7.4 µs | 494.7 | 36.8 µs | 24.7 µs | 148.7 | 6.2 MiB | 6.5 MiB |
| markdown | byte | 3.9 KiB | 0.85 | 31.5 µs | 17.0 µs | 235.9 | 61.1 µs | 32.0 µs | 125.0 | 6.2 MiB | 6.4 MiB |
| toon | byte | 3.7 KiB | 0.80 | 28.7 µs | 14.3 µs | 262.4 | 42.7 µs | 18.5 µs | 203.2 | 6.3 MiB | 6.5 MiB |
| toon-tab | byte | 3.7 KiB | 0.80 | 25.4 µs | 13.8 µs | 272.7 | 41.2 µs | 19.3 µs | 194.9 | 6.3 MiB | 6.4 MiB |
| yaml | byte | 4.5 KiB | 0.97 | 43.7 µs | 16.6 µs | 275.5 | 135.2 µs | 95.7 µs | 47.8 | 6.3 MiB | 6.7 MiB |

## repetitive_logs (20 records, 2.8 KiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 15.5 µs | 11.8 µs | 13.5 µs | 244.9 | - | 5.8 MiB |
| parse | serde_json | 23.7 µs | 11.7 µs | 12.4 µs | 247.5 | 1714237 | 6.0 MiB |
| parse | simd-json | 25.8 µs | 7.0 µs | 10.9 µs | 415.0 | 2874183 | 6.1 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 2.8 KiB | 1.00 | 9.0 µs | 3.5 µs | 820.3 | 15.5 µs | 9.8 µs | 294.9 | 6.1 MiB | 6.2 MiB |
| json-pretty | byte | 3.8 KiB | 1.33 | 12.0 µs | 4.9 µs | 782.9 | 13.9 µs | 10.2 µs | 376.3 | 6.1 MiB | 6.2 MiB |
| jsonl | byte | 2.8 KiB | 1.00 | 18.5 µs | 3.4 µs | 839.9 | 25.4 µs | 9.9 µs | 291.7 | 6.1 MiB | 6.2 MiB |
| csv | byte | 1.4 KiB | 0.49 | 26.4 µs | 8.7 µs | 162.0 | 40.0 µs | 27.1 µs | 52.3 | 6.2 MiB | 6.4 MiB |
| tsv | byte | 1.4 KiB | 0.49 | 22.3 µs | 8.7 µs | 162.3 | 40.4 µs | 24.1 µs | 58.9 | 6.2 MiB | 6.4 MiB |
| markdown | byte | 1.8 KiB | 0.62 | 23.6 µs | 10.0 µs | 179.7 | 37.2 µs | 20.6 µs | 87.0 | 6.2 MiB | 6.3 MiB |
| toon | byte | 1.5 KiB | 0.52 | 22.2 µs | 12.2 µs | 122.9 | 37.8 µs | 14.9 µs | 100.9 | 6.3 MiB | 6.4 MiB |
| toon-tab | byte | 1.5 KiB | 0.52 | 21.1 µs | 12.1 µs | 124.3 | 28.1 µs | 15.1 µs | 99.7 | 6.3 MiB | 6.3 MiB |
| yaml | byte | 2.7 KiB | 0.97 | 29.5 µs | 14.3 µs | 194.7 | 121.5 µs | 84.2 µs | 33.1 | 6.2 MiB | 6.6 MiB |

## unicode_strings (20 records, 2.5 KiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 15.5 µs | 11.8 µs | 13.2 µs | 215.6 | - | 5.8 MiB |
| parse | serde_json | 44.0 µs | 11.0 µs | 14.7 µs | 232.0 | 1821577 | 6.0 MiB |
| parse | simd-json | 27.8 µs | 8.2 µs | 12.5 µs | 310.3 | 2436647 | 6.1 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 2.5 KiB | 1.00 | 8.5 µs | 3.8 µs | 675.5 | 12.9 µs | 10.5 µs | 242.1 | 6.1 MiB | 6.2 MiB |
| json-pretty | byte | 3.4 KiB | 1.38 | 11.1 µs | 4.8 µs | 725.8 | 14.4 µs | 11.2 µs | 314.1 | 6.2 MiB | 6.2 MiB |
| jsonl | byte | 2.5 KiB | 1.00 | 8.6 µs | 3.7 µs | 679.0 | 28.0 µs | 11.3 µs | 225.1 | 6.1 MiB | 6.2 MiB |
| csv | lossy (0.empty: null vs "") | 1.2 KiB | 0.47 | 29.0 µs | 8.8 µs | 137.2 | 43.4 µs | 25.4 µs | 47.6 | 6.2 MiB | 6.4 MiB |
| tsv | lossy (0.empty: null vs "") | 1.2 KiB | 0.47 | 22.0 µs | 8.5 µs | 142.5 | 46.2 µs | 27.2 µs | 44.4 | 6.2 MiB | 6.4 MiB |
| markdown | lossy (0.empty: null vs "") | 1.5 KiB | 0.62 | 23.9 µs | 9.1 µs | 172.9 | 41.1 µs | 23.6 µs | 66.9 | 6.2 MiB | 6.4 MiB |
| toon | byte | 1.3 KiB | 0.54 | 25.8 µs | 10.6 µs | 128.2 | 37.6 µs | 16.2 µs | 84.1 | 6.3 MiB | 6.4 MiB |
| toon-tab | byte | 1.3 KiB | 0.53 | 24.5 µs | 12.0 µs | 113.5 | 36.0 µs | 16.7 µs | 81.5 | 6.3 MiB | 6.4 MiB |
| yaml | byte | 2.5 KiB | 0.99 | 31.8 µs | 15.5 µs | 163.1 | 128.3 µs | 81.4 µs | 31.0 | 6.3 MiB | 6.7 MiB |

## numeric_mix (20 records, 4.7 KiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 17.8 µs | 13.3 µs | 14.5 µs | 359.6 | - | 5.8 MiB |
| parse | serde_json | 33.2 µs | 15.1 µs | 17.6 µs | 318.3 | 1327801 | 6.0 MiB |
| parse | simd-json | 37.9 µs | 10.2 µs | 13.5 µs | 469.7 | 1959248 | 6.1 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 4.7 KiB | 1.00 | 16.9 µs | 7.4 µs | 644.7 | 17.7 µs | 14.8 µs | 324.6 | 6.2 MiB | 6.2 MiB |
| json-pretty | byte | 6.3 KiB | 1.35 | 18.9 µs | 9.1 µs | 709.7 | 20.0 µs | 16.0 µs | 405.3 | 6.2 MiB | 6.2 MiB |
| jsonl | byte | 4.7 KiB | 1.00 | 16.5 µs | 7.4 µs | 644.6 | 24.0 µs | 16.0 µs | 300.0 | 6.2 MiB | 6.3 MiB |
| csv | semantic | 2.7 KiB | 0.59 | 58.3 µs | 28.1 µs | 100.0 | 43.4 µs | 30.5 µs | 92.2 | 6.3 MiB | 6.4 MiB |
| tsv | semantic | 2.7 KiB | 0.59 | 52.9 µs | 28.7 µs | 98.0 | 43.8 µs | 29.6 µs | 95.1 | 6.3 MiB | 6.5 MiB |
| markdown | semantic | 3.4 KiB | 0.73 | 53.8 µs | 29.9 µs | 116.6 | 54.5 µs | 38.8 µs | 89.7 | 6.4 MiB | 6.4 MiB |
| toon | semantic | 2.8 KiB | 0.60 | 48.5 µs | 26.2 µs | 109.0 | 39.9 µs | 21.6 µs | 132.4 | 6.4 MiB | 6.4 MiB |
| toon-tab | semantic | 2.8 KiB | 0.60 | 46.0 µs | 25.1 µs | 114.1 | 37.2 µs | 22.5 µs | 127.1 | 6.4 MiB | 6.5 MiB |
| yaml | byte | 4.9 KiB | 1.04 | 42.9 µs | 24.0 µs | 208.3 | 192.5 µs | 152.7 µs | 32.7 | 6.3 MiB | 6.7 MiB |

## mixed_types (20 records, 1.4 KiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 15.5 µs | 11.6 µs | 12.4 µs | 122.7 | - | 5.8 MiB |
| parse | serde_json | 19.4 µs | 10.4 µs | 14.5 µs | 137.2 | 1923817 | 6.0 MiB |
| parse | simd-json | 42.7 µs | 9.7 µs | 20.2 µs | 146.9 | 2060157 | 6.1 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 1.4 KiB | 1.00 | 11.4 µs | 2.8 µs | 507.0 | 9.3 µs | 7.5 µs | 190.1 | 6.1 MiB | 6.2 MiB |
| json-pretty | byte | 2.6 KiB | 1.90 | 13.5 µs | 4.6 µs | 593.7 | 10.4 µs | 7.5 µs | 360.2 | 6.1 MiB | 6.2 MiB |
| jsonl | byte | 1.4 KiB | 1.00 | 10.3 µs | 2.8 µs | 506.6 | 16.8 µs | 8.8 µs | 161.7 | 6.1 MiB | 6.3 MiB |
| csv | unsupported: record 0 field "value" is a object (nested) | | | | | | | | | | |
| tsv | unsupported: record 0 field "value" is a object (nested) | | | | | | | | | | |
| markdown | unsupported: record 0 field "value" is a object (nested) | | | | | | | | | | |
| toon | byte | 1.7 KiB | 1.22 | 32.1 µs | 10.5 µs | 165.4 | 36.2 µs | 20.4 µs | 85.0 | 6.2 MiB | 6.3 MiB |
| toon-tab | byte | 1.7 KiB | 1.22 | 23.0 µs | 9.6 µs | 182.0 | 37.5 µs | 23.5 µs | 74.0 | 6.2 MiB | 6.4 MiB |
| yaml | byte | 1.6 KiB | 1.12 | 28.0 µs | 9.8 µs | 163.3 | 126.9 µs | 70.6 µs | 22.5 | 6.2 MiB | 6.6 MiB |

## wide_table (20 records, 10.5 KiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 20.0 µs | 11.6 µs | 12.3 µs | 929.7 | - | 5.8 MiB |
| parse | serde_json | 66.5 µs | 39.7 µs | 41.4 µs | 271.3 | 503937 | 6.1 MiB |
| parse | simd-json | 60.5 µs | 30.6 µs | 37.2 µs | 352.1 | 653947 | 6.3 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 10.5 KiB | 1.00 | 28.2 µs | 16.1 µs | 669.6 | 43.1 µs | 38.9 µs | 276.6 | 6.3 MiB | 6.3 MiB |
| json-pretty | byte | 15.4 KiB | 1.47 | 30.8 µs | 20.5 µs | 771.2 | 46.1 µs | 41.9 µs | 377.0 | 6.3 MiB | 6.3 MiB |
| jsonl | byte | 10.5 KiB | 1.00 | 25.6 µs | 16.4 µs | 655.9 | 49.2 µs | 41.2 µs | 261.3 | 6.3 MiB | 6.4 MiB |
| csv | lossy (0.s01: null vs "null") | 5.5 KiB | 0.52 | 93.0 µs | 57.3 µs | 97.5 | 72.5 µs | 49.2 µs | 113.5 | 6.5 MiB | 6.6 MiB |
| tsv | lossy (0.s01: null vs "null") | 5.5 KiB | 0.52 | 75.9 µs | 58.1 µs | 96.1 | 74.1 µs | 48.4 µs | 115.5 | 6.5 MiB | 6.6 MiB |
| markdown | lossy (0.s01: null vs "null") | 7.4 KiB | 0.71 | 77.7 µs | 61.7 µs | 123.1 | 107.7 µs | 88.3 µs | 86.1 | 6.5 MiB | 6.5 MiB |
| toon | semantic | 5.5 KiB | 0.53 | 61.1 µs | 39.5 µs | 143.3 | 134.8 µs | 108.1 µs | 52.4 | 6.5 MiB | 6.7 MiB |
| toon-tab | semantic | 5.5 KiB | 0.53 | 70.4 µs | 39.4 µs | 144.0 | 134.3 µs | 108.5 µs | 52.3 | 6.5 MiB | 6.7 MiB |
| yaml | byte | 10.9 KiB | 1.04 | 80.0 µs | 65.1 µs | 171.7 | 499.9 µs | 431.5 µs | 25.9 | 6.4 MiB | 7.0 MiB |

## deep_nesting (10 records, 3.3 KiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 18.8 µs | 11.9 µs | 20.0 µs | 286.8 | - | 5.8 MiB |
| parse | serde_json | 30.5 µs | 17.1 µs | 18.6 µs | 198.6 | 583226 | 6.0 MiB |
| parse | simd-json | 43.8 µs | 15.6 µs | 23.0 µs | 218.3 | 640841 | 6.2 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 3.3 KiB | 1.00 | 12.4 µs | 6.1 µs | 556.1 | 18.3 µs | 16.7 µs | 204.4 | 6.1 MiB | 6.2 MiB |
| json-pretty | byte | 14.2 KiB | 4.27 | 26.7 µs | 17.7 µs | 822.0 | 28.6 µs | 23.0 µs | 633.5 | 6.2 MiB | 6.2 MiB |
| jsonl | byte | 3.3 KiB | 1.00 | 12.3 µs | 6.0 µs | 567.5 | 28.6 µs | 17.7 µs | 192.5 | 6.1 MiB | 6.2 MiB |
| csv | unsupported: record 0 field "tree" is a object (nested) | | | | | | | | | | |
| tsv | unsupported: record 0 field "tree" is a object (nested) | | | | | | | | | | |
| markdown | unsupported: record 0 field "tree" is a object (nested) | | | | | | | | | | |
| toon | byte | 8.6 KiB | 2.59 | 55.8 µs | 33.4 µs | 263.6 | 89.3 µs | 61.5 µs | 143.2 | 6.3 MiB | 6.5 MiB |
| toon-tab | byte | 8.7 KiB | 2.63 | 53.6 µs | 34.6 µs | 258.6 | 87.2 µs | 60.2 µs | 148.6 | 6.3 MiB | 6.5 MiB |
| yaml | byte | 9.0 KiB | 2.70 | 36.2 µs | 23.0 µs | 399.9 | 298.3 µs | 204.8 µs | 45.0 | 6.2 MiB | 6.8 MiB |

## empty_values (20 records, 3.8 KiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 16.9 µs | 12.1 µs | 13.8 µs | 319.2 | - | 5.8 MiB |
| parse | serde_json | 38.5 µs | 17.3 µs | 21.5 µs | 223.8 | 1156638 | 6.0 MiB |
| parse | simd-json | 32.6 µs | 10.9 µs | 14.8 µs | 355.2 | 1835452 | 6.1 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 3.8 KiB | 1.00 | 10.7 µs | 4.6 µs | 836.8 | 22.4 µs | 14.4 µs | 268.8 | 6.2 MiB | 6.2 MiB |
| json-pretty | byte | 5.6 KiB | 1.48 | 15.9 µs | 7.3 µs | 783.7 | 22.4 µs | 15.6 µs | 366.8 | 6.2 MiB | 6.2 MiB |
| jsonl | byte | 3.8 KiB | 1.00 | 10.1 µs | 4.8 µs | 814.5 | 26.2 µs | 14.8 µs | 261.9 | 6.1 MiB | 6.2 MiB |
| csv | unsupported: record 0 field "empty_a" is a array (nested) | | | | | | | | | | |
| tsv | unsupported: record 0 field "empty_a" is a array (nested) | | | | | | | | | | |
| markdown | unsupported: record 0 field "empty_a" is a array (nested) | | | | | | | | | | |
| toon | byte | 4.5 KiB | 1.19 | 41.6 µs | 23.4 µs | 196.6 | 80.2 µs | 46.5 µs | 98.7 | 6.3 MiB | 6.4 MiB |
| toon-tab | byte | 4.5 KiB | 1.19 | 42.5 µs | 26.1 µs | 175.8 | 76.3 µs | 45.7 µs | 100.5 | 6.3 MiB | 6.4 MiB |
| yaml | byte | 3.9 KiB | 1.04 | 35.0 µs | 19.4 µs | 206.6 | 184.4 µs | 135.2 µs | 29.7 | 6.2 MiB | 6.6 MiB |

## long_text (10 records, 11.9 KiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 20.1 µs | 11.8 µs | 12.4 µs | 1036.8 | - | 5.8 MiB |
| parse | serde_json | 18.0 µs | 7.7 µs | 9.0 µs | 1594.5 | 1304291 | 6.1 MiB |
| parse | simd-json | 25.1 µs | 7.5 µs | 10.1 µs | 1629.9 | 1333244 | 6.2 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 11.9 KiB | 1.00 | 14.0 µs | 7.1 µs | 1731.0 | 8.9 µs | 6.9 µs | 1762.0 | 6.2 MiB | 6.3 MiB |
| json-pretty | byte | 12.2 KiB | 1.02 | 18.0 µs | 7.6 µs | 1651.9 | 9.6 µs | 7.6 µs | 1642.8 | 6.2 MiB | 6.3 MiB |
| jsonl | byte | 11.9 KiB | 1.00 | 14.7 µs | 7.4 µs | 1643.6 | 22.5 µs | 11.8 µs | 1033.0 | 6.2 MiB | 6.5 MiB |
| csv | byte | 11.6 KiB | 0.97 | 24.9 µs | 5.6 µs | 2116.3 | 38.7 µs | 31.4 µs | 377.3 | 6.3 MiB | 6.6 MiB |
| tsv | byte | 11.6 KiB | 0.97 | 20.1 µs | 5.3 µs | 2250.5 | 34.4 µs | 28.4 µs | 417.4 | 6.3 MiB | 6.6 MiB |
| markdown | byte | 11.7 KiB | 0.98 | 45.0 µs | 25.4 µs | 472.6 | 49.0 µs | 37.2 µs | 322.5 | 6.3 MiB | 6.5 MiB |
| toon | byte | 11.6 KiB | 0.98 | 41.0 µs | 27.4 µs | 435.8 | 64.2 µs | 52.5 µs | 227.2 | 6.5 MiB | 6.5 MiB |
| toon-tab | byte | 11.6 KiB | 0.98 | 41.3 µs | 33.6 µs | 354.8 | 63.6 µs | 52.7 µs | 226.1 | 6.4 MiB | 6.5 MiB |
| yaml | byte | 11.9 KiB | 1.00 | 45.0 µs | 31.8 µs | 383.5 | 134.8 µs | 87.0 µs | 140.1 | 6.3 MiB | 6.8 MiB |

