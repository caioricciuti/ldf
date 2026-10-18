# ldf-bench results

Started 20261018-101009 UTC. Command: `./target/release/ldf-bench perf --input fixtures/medium/wide_table.json --iters 7 --out /private/tmp/claude-501/-Users-caioricciuti-Desktop-ldf/cb127970-93b2-428e-b513-1917cb064e51/scratchpad/perf-after`

Machine: Apple M2 Pro (aarch64), 10 logical CPUs, 16.0 GiB RAM, Darwin 27.0.0. release profile, commit 7684137.

Toolchain: rustc 1.98.0 (88d9e12ae 2026-08-18). Crates: arrow 60.0.0, criterion 0.8.2, csv 1.4.0, parquet 60.0.0, serde_json 1.0.151, simd-json 0.18.1, tiktoken-rs 0.12.1.

Each operation runs in its own child process. Iteration 0 is the cold run; medians and p95 are over the remaining 6 warm iterations. MB = 1e6 bytes. Peak RSS is the child process high-water mark after the operation, as reported by getrusage, minus nothing: it includes the loaded input. See docs/benchmark-methodology.md.

## wide_table (10000 records, 5.2 MiB as json)

### Input: I/O and parse

| step | engine | cold | warm median | warm p95 | MB/s (median) | records/s | peak RSS |
|---|---|---:|---:|---:|---:|---:|---:|
| read file | - | 621.8 µs | 262.0 µs | 331.7 µs | 20652.4 | - | 11.0 MiB |
| parse | serde_json | 20.63 ms | 17.68 ms | 19.71 ms | 306.1 | 565734 | 64.0 MiB |
| parse | simd-json | 16.58 ms | 12.77 ms | 15.38 ms | 423.8 | 783268 | 112.3 MiB |

### Formats

| format | round trip | bytes | vs json | encode cold | encode median | encode MB/s | decode cold | decode median | decode MB/s | peak RSS enc | peak RSS dec |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| json | byte | 5.2 MiB | 1.00 | 8.21 ms | 8.34 ms | 648.9 | 19.13 ms | 17.87 ms | 302.7 | 94.6 MiB | 67.1 MiB |
| json-pretty | byte | 7.6 MiB | 1.47 | 10.22 ms | 10.64 ms | 745.1 | 20.41 ms | 19.23 ms | 412.4 | 111.4 MiB | 71.9 MiB |
| jsonl | byte | 5.2 MiB | 1.00 | 7.98 ms | 8.25 ms | 656.1 | 19.98 ms | 18.77 ms | 288.2 | 94.6 MiB | 68.3 MiB |
| csv | lossy (0.s01: null vs "null") | 2.6 MiB | 0.51 | 27.11 ms | 26.53 ms | 103.3 | 15.73 ms | 13.71 ms | 199.9 | 63.7 MiB | 94.6 MiB |
| tsv | lossy (0.s01: null vs "null") | 2.6 MiB | 0.51 | 26.32 ms | 26.13 ms | 104.9 | 15.56 ms | 14.20 ms | 192.9 | 63.2 MiB | 94.1 MiB |
| markdown | lossy (0.s01: null vs "null") | 3.4 MiB | 0.66 | 25.60 ms | 25.55 ms | 140.1 | 36.37 ms | 34.37 ms | 104.2 | 65.0 MiB | 92.9 MiB |
| toon | semantic | 2.6 MiB | 0.51 | 27.76 ms | 25.60 ms | 108.3 | 55.54 ms | 50.67 ms | 54.7 | 63.7 MiB | 94.5 MiB |
| toon-tab | semantic | 2.6 MiB | 0.51 | 27.03 ms | 26.09 ms | 106.3 | 54.47 ms | 50.48 ms | 54.9 | 65.0 MiB | 94.4 MiB |
| yaml | byte | 5.4 MiB | 1.04 | 26.63 ms | 26.68 ms | 210.4 | 187.66 ms | 172.27 ms | 32.6 | 95.9 MiB | 175.6 MiB |
| ldf | byte | 2.6 MiB | 0.51 | 28.66 ms | 27.85 ms | 98.9 | 21.85 ms | 17.71 ms | 155.5 | 66.6 MiB | 109.5 MiB |

