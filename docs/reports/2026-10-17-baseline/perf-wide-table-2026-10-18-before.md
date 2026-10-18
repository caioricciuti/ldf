# ldf-bench results

Started 20261018-100715 UTC. Command: `./target/release/ldf-bench perf --input fixtures/medium/wide_table.json --iters 5 --out /private/tmp/claude-501/-Users-caioricciuti-Desktop-ldf/cb127970-93b2-428e-b513-1917cb064e51/scratchpad/perf-before`

Machine: Apple M2 Pro (aarch64), 10 logical CPUs, 16.0 GiB RAM, Darwin 27.0.0. release profile, commit 7684137.

Toolchain: rustc 1.98.0 (88d9e12ae 2026-08-18). Crates: arrow 60.0.0, criterion 0.8.2, csv 1.4.0, parquet 60.0.0, serde_json 1.0.151, simd-json 0.18.1, tiktoken-rs 0.12.1.

Each operation runs in its own child process. Iteration 0 is the cold run; medians and p95 are over the remaining 4 warm iterations. MB = 1e6 bytes. Peak RSS is the child process high-water mark after the operation, as reported by getrusage, minus nothing: it includes the loaded input. See docs/benchmark-methodology.md.

## wide_table (10000 records, 5.2 MiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 928.2 µs | 375.7 µs | 572.4 µs | 14401.2 | - | 11.0 MiB |
| parse | serde_json | 21.95 ms | 18.67 ms | 19.54 ms | 289.8 | 535750 | 64.4 MiB |
| parse | simd-json | 18.84 ms | 15.52 ms | 17.43 ms | 348.6 | 644363 | 99.6 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 5.2 MiB | 1.00 | 8.59 ms | 8.03 ms | 673.3 | 20.28 ms | 17.72 ms | 305.2 | 82.2 MiB | 66.6 MiB |
| json-pretty | byte | 7.6 MiB | 1.47 | 11.26 ms | 10.38 ms | 763.7 | 22.13 ms | 18.85 ms | 420.7 | 96.3 MiB | 71.4 MiB |
| jsonl | byte | 5.2 MiB | 1.00 | 8.40 ms | 8.14 ms | 664.8 | 20.47 ms | 17.74 ms | 305.0 | 82.2 MiB | 68.0 MiB |
| csv | lossy (0.s01: null vs "null") | 2.6 MiB | 0.51 | 26.04 ms | 24.98 ms | 109.7 | 16.80 ms | 12.79 ms | 214.3 | 61.7 MiB | 94.2 MiB |
| tsv | lossy (0.s01: null vs "null") | 2.6 MiB | 0.51 | 26.25 ms | 24.86 ms | 110.2 | 15.40 ms | 12.86 ms | 213.0 | 61.7 MiB | 94.4 MiB |
| markdown | lossy (0.s01: null vs "null") | 3.4 MiB | 0.66 | 25.30 ms | 24.55 ms | 145.8 | 36.32 ms | 34.11 ms | 105.0 | 65.1 MiB | 92.3 MiB |
| toon | semantic | 2.6 MiB | 0.51 | 25.95 ms | 22.73 ms | 122.0 | 50.67 ms | 48.05 ms | 57.7 | 63.6 MiB | 88.3 MiB |
| toon-tab | semantic | 2.6 MiB | 0.51 | 24.50 ms | 23.25 ms | 119.3 | 51.13 ms | 48.52 ms | 57.1 | 63.9 MiB | 89.1 MiB |
| yaml | byte | 5.4 MiB | 1.04 | 26.17 ms | 25.28 ms | 222.1 | 181.25 ms | 163.44 ms | 34.3 | 83.2 MiB | 177.1 MiB |
| ldf | byte | 2.6 MiB | 0.51 | 56.17 ms | 54.53 ms | 50.5 | 54.96 ms | 49.70 ms | 55.4 | 64.4 MiB | 83.5 MiB |

