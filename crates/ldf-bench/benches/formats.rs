//! Criterion microbenchmarks: encode and decode of every baseline format on
//! in-memory records from two generated datasets. Complements the
//! process-isolated `ldf-bench perf` runs with statistically tighter
//! per-operation timings.

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use ldf_fixtures::{DEFAULT_SEED, find};
use ldf_formats::all_formats;

fn bench_formats(c: &mut Criterion) {
    for (dataset, rows) in [
        ("flat_users", 2_000u64),
        ("nested_orders", 1_000),
        ("wide_table", 500),
    ] {
        let spec = find(dataset).unwrap();
        let records: Vec<_> = spec.records(DEFAULT_SEED, rows).collect();
        let mut g = c.benchmark_group(format!("encode/{dataset}"));
        for f in all_formats() {
            let mut out = Vec::new();
            if f.encode(&records, &mut out).is_err() {
                continue;
            }
            g.throughput(Throughput::Bytes(out.len() as u64));
            g.bench_with_input(
                BenchmarkId::from_parameter(f.name()),
                &records,
                |b, recs| {
                    b.iter(|| {
                        let mut buf = Vec::new();
                        f.encode(recs, &mut buf).unwrap();
                        buf
                    })
                },
            );
        }
        g.finish();

        let mut g = c.benchmark_group(format!("decode/{dataset}"));
        for f in all_formats() {
            let mut out = Vec::new();
            if f.encode(&records, &mut out).is_err() {
                continue;
            }
            g.throughput(Throughput::Bytes(out.len() as u64));
            g.bench_with_input(BenchmarkId::from_parameter(f.name()), &out, |b, bytes| {
                b.iter(|| f.decode(bytes).unwrap())
            });
        }
        g.finish();
    }
}

criterion_group!(benches, bench_formats);
criterion_main!(benches);
