//! The direct Arrow path must write exactly what the Value path writes:
//! `batch_to_values` then `encode_with` / `BlockEncoder`.
#![cfg(feature = "arrow")]

use std::fs::File;
use std::path::PathBuf;
use std::sync::Arc;

use arrow::array::{
    ArrayRef, BooleanArray, Float32Array, Float64Array, Int32Array, Int64Array, ListArray,
    NullArray, RecordBatch, StringArray, StructArray, TimestampMillisecondArray, UInt64Array,
};
use arrow::buffer::NullBuffer;
use arrow::datatypes::{DataType, Field, Fields, Int32Type};
use ldf::arrow::{ArrowBlockEncoder, encode_batches};
use ldf::{BlockEncoder, Options, encode_with};
use ldf_data::Value;
use ldf_data::input::parquet::batch_to_values;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;

fn values(batches: &[RecordBatch]) -> Vec<Value> {
    batches
        .iter()
        .flat_map(|b| batch_to_values(b).expect("convert"))
        .collect()
}

fn via_values(batches: &[RecordBatch], opts: &Options) -> String {
    encode_with(&values(batches), opts)
}

fn blocks_via_values(batches: &[RecordBatch], block: usize, opts: Options) -> String {
    let mut out = String::new();
    let mut enc = BlockEncoder::with_options(block, opts);
    for v in values(batches) {
        enc.record(v, &mut out);
    }
    enc.finish(&mut out);
    out
}

fn blocks_direct(batches: &[RecordBatch], block: usize, opts: Options) -> String {
    let mut out = String::new();
    let mut enc = ArrowBlockEncoder::new(block, opts);
    for b in batches {
        enc.batch(b, &mut out).expect("supported");
    }
    enc.finish(&mut out).expect("supported");
    out
}

fn assert_same(batches: &[RecordBatch], what: &str) {
    for opts in [Options::default(), Options::V0] {
        let direct = encode_batches(batches, &opts).expect("supported");
        assert_eq!(direct, via_values(batches, &opts), "{what} batch {opts:?}");
        for block in [1, 3, 64, 1000] {
            assert_eq!(
                blocks_direct(batches, block, opts),
                blocks_via_values(batches, block, opts),
                "{what} block {block} {opts:?}"
            );
        }
    }
}

fn read_parquet(path: &PathBuf, batch: usize) -> Vec<RecordBatch> {
    let file = File::open(path).expect("open");
    ParquetRecordBatchReaderBuilder::try_new(file)
        .expect("parquet")
        .with_batch_size(batch)
        .build()
        .expect("reader")
        .collect::<Result<_, _>>()
        .expect("batches")
}

#[test]
fn every_small_parquet_fixture() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/small");
    let mut seen = 0;
    let mut unsupported = Vec::new();
    for entry in std::fs::read_dir(&dir).expect("fixtures/small") {
        let path = entry.expect("entry").path();
        if path.extension().and_then(|e| e.to_str()) != Some("parquet") {
            continue;
        }
        let batches = read_parquet(&path, 7);
        if ldf::arrow::check_schema(&batches[0].schema()).is_err() {
            unsupported.push(path.file_stem().unwrap().to_string_lossy().into_owned());
            assert!(encode_batches(&batches, &Options::default()).is_err());
            continue;
        }
        assert_same(&batches, &path.display().to_string());
        seen += 1;
    }
    assert!(seen >= 8, "only {seen} fixtures checked");
    // nested_orders has an items list of structs: a child table.
    assert_eq!(unsupported, vec!["nested_orders".to_string()]);
}

fn batch(cols: Vec<(&str, ArrayRef)>) -> RecordBatch {
    RecordBatch::try_from_iter(cols.into_iter().map(|(n, a)| (n.to_string(), a))).expect("batch")
}

#[test]
fn edge_cases() {
    let point = Fields::from(vec![
        Field::new("x", DataType::Int32, true),
        Field::new("label", DataType::Utf8, true),
    ]);
    let dense_struct = StructArray::new(
        point.clone(),
        vec![
            Arc::new(Int32Array::from(vec![Some(1), None, Some(3), Some(4)])) as ArrayRef,
            Arc::new(StringArray::from(vec!["a", "b,c", "", "null"])),
        ],
        None,
    );
    let null_struct = StructArray::new(
        point,
        vec![
            Arc::new(Int32Array::from(vec![1, 2, 3, 4])) as ArrayRef,
            Arc::new(StringArray::from(vec!["a", "b", "c", "d"])),
        ],
        Some(NullBuffer::from(vec![true, false, true, true])),
    );
    let list = ListArray::from_iter_primitive::<Int32Type, _, _>(vec![
        Some(vec![Some(1), Some(2)]),
        None,
        Some(vec![]),
        Some(vec![None]),
    ]);
    let b = batch(vec![
        (
            "name",
            Arc::new(StringArray::from(vec!["@at", "x", " y", "q\"q"])) as ArrayRef,
        ),
        ("id", Arc::new(Int64Array::from(vec![1, 2, 3, 4]))),
        ("big", Arc::new(UInt64Array::from(vec![u64::MAX, 0, 1, 2]))),
        (
            "f32",
            Arc::new(Float32Array::from(vec![0.1, 1.0, -0.0, 3.5])),
        ),
        (
            "f64",
            Arc::new(Float64Array::from(vec![
                Some(1.0),
                None,
                Some(f64::NAN),
                Some(1e21),
            ])),
        ),
        (
            "ok",
            Arc::new(BooleanArray::from(vec![
                Some(true),
                Some(false),
                None,
                Some(true),
            ])),
        ),
        (
            "sparse",
            Arc::new(Int64Array::from(vec![Some(9), None, None, None])),
        ),
        ("gone", Arc::new(NullArray::new(4))),
        (
            "ts",
            Arc::new(TimestampMillisecondArray::from(vec![
                Some(0),
                None,
                Some(1_700_000_000_000),
                Some(5),
            ])),
        ),
        ("p", Arc::new(dense_struct)),
        ("q", Arc::new(null_struct)),
        ("tags", Arc::new(list)),
    ]);
    assert_same(std::slice::from_ref(&b), "edge cases");
    // Split into uneven batches: the plan is over all rows, not per batch.
    let parts = vec![b.slice(0, 1), b.slice(1, 3)];
    assert_same(&parts, "edge cases split");
    // Spot check of the first cell rule and the null struct as JSON.
    let out = encode_batches(&parts, &Options::default()).unwrap();
    assert!(out.starts_with("@records name:s,id:i,big:i,f32:f,f64:f?,ok:b?,ts:s?,p.x:i?,p.label:s,q:j?,tags:j?,~:j?\n\"@at\","), "{out}");
}

#[test]
fn empty_and_unsupported() {
    let b = batch(vec![(
        "a",
        Arc::new(Int64Array::from(Vec::<i64>::new())) as ArrayRef,
    )]);
    assert_same(std::slice::from_ref(&b), "empty");
    assert_eq!(
        encode_batches(&[], &Options::default()).unwrap(),
        "@records _:j\n"
    );
    let items = Field::new(
        "item",
        DataType::Struct(Fields::from(vec![Field::new("sku", DataType::Utf8, false)])),
        true,
    );
    let schema = arrow::datatypes::Schema::new(vec![Field::new(
        "items",
        DataType::List(Arc::new(items)),
        true,
    )]);
    assert!(ldf::arrow::check_schema(&schema).is_err());
}
