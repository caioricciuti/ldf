//! Direct Arrow path (feature `arrow`): encodes `RecordBatch`es as LDF
//! without building a [`Value`] per row.
//!
//! The output is byte-identical to converting the batches with
//! `ldf_data::input::parquet::batch_to_values` and encoding the result
//! with [`crate::encode_with`] (or [`crate::BlockEncoder`] for the block
//! form); the tests and `ldf-bench arrow` check that on every fixture.
//! The plan comes from the schema and Arrow's null counts; primitive,
//! string, boolean and display-formatted (temporal, decimal, binary)
//! columns are written straight from the arrays, struct columns flatten to
//! dotted paths under the same rule as the batch planner. Cells that are
//! JSON in the plan (lists, structs with a null, sparse `~` objects) go
//! through `ldf_data`'s own Arrow reader, so the mapping cannot drift.
//!
//! A list of structs would become a child table, which this path does not
//! write: [`Unsupported`] is returned and the caller falls back to the
//! `Value` path, which handles every schema.

use std::fmt::{self, Write as _};

use arrow::array::{Array, ArrayRef, AsArray, RecordBatch};
use arrow::buffer::NullBuffer;
use arrow::datatypes::*;
use arrow::util::display::{ArrayFormatter, FormatOptions};
use ldf_data::input::parquet::ColumnReader;
use ldf_data::{Object, Value};

use crate::ROOT;
use crate::encode::quote_line_start;
use crate::plan::{ColType, Column, Join, Options, TablePlan};
use crate::text::{push_float, push_json_value, push_string_cell};

/// The schema needs the `Value` path (a list of structs, or no columns).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Unsupported(pub String);

impl fmt::Display for Unsupported {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "not supported by the direct Arrow path: {}", self.0)
    }
}

impl std::error::Error for Unsupported {}

/// Encodes whole batches as one document, the output of [`crate::encode_with`].
pub fn encode_batches(batches: &[RecordBatch], opts: &Options) -> Result<String, Unsupported> {
    let mut out = String::new();
    let rows: usize = batches.iter().map(RecordBatch::num_rows).sum();
    if rows == 0 {
        out.push_str(&format!("@{ROOT} _:j\n"));
        return Ok(out);
    }
    let plan = plan_batches(batches, opts)?;
    out.push_str(&plan.table.header());
    out.push('\n');
    write_rows(&plan, batches, &mut out)?;
    Ok(out)
}

/// Block form, the output of [`crate::BlockEncoder`] with the same block
/// size: rows are cut into blocks of `block` regardless of how the reader
/// batched them (slices, no copies), each block planned on its own.
pub struct ArrowBlockEncoder {
    block: usize,
    opts: Options,
    pending: Vec<RecordBatch>,
    pending_rows: usize,
    last_header: Option<String>,
    any: bool,
}

impl ArrowBlockEncoder {
    pub fn new(block: usize, opts: Options) -> Self {
        ArrowBlockEncoder {
            block: block.max(1),
            opts: Options {
                ordinal_joins: true,
                ..opts
            },
            pending: Vec::new(),
            pending_rows: 0,
            last_header: None,
            any: false,
        }
    }

    /// Takes one batch; writes every block it completes to `out`.
    pub fn batch(&mut self, batch: &RecordBatch, out: &mut String) -> Result<(), Unsupported> {
        let mut offset = 0;
        while offset < batch.num_rows() {
            let take = (self.block - self.pending_rows).min(batch.num_rows() - offset);
            self.pending.push(batch.slice(offset, take));
            self.pending_rows += take;
            offset += take;
            if self.pending_rows == self.block {
                self.flush(out)?;
            }
        }
        Ok(())
    }

    fn flush(&mut self, out: &mut String) -> Result<(), Unsupported> {
        if self.pending_rows == 0 {
            return Ok(());
        }
        let plan = plan_batches(&self.pending, &self.opts)?;
        let header = plan.table.header();
        if self.last_header.as_deref() != Some(header.as_str()) {
            out.push_str(&header);
            out.push('\n');
        }
        write_rows(&plan, &self.pending, out)?;
        self.last_header = Some(header);
        self.any = true;
        self.pending.clear();
        self.pending_rows = 0;
        Ok(())
    }

    /// Writes the last block; an empty dataset is a header with no rows.
    pub fn finish(&mut self, out: &mut String) -> Result<(), Unsupported> {
        self.flush(out)?;
        if !self.any {
            out.push_str(&format!("@{ROOT} _:j\n"));
            self.any = true;
        }
        Ok(())
    }
}

/// Checks a schema up front, so a caller can pick the path before reading.
pub fn check_schema(schema: &Schema) -> Result<(), Unsupported> {
    if schema.fields().is_empty() {
        return Err(Unsupported("a schema with no columns".into()));
    }
    for f in schema.fields() {
        check_type(f.name(), f.data_type())?;
    }
    Ok(())
}

fn check_type(name: &str, ty: &DataType) -> Result<(), Unsupported> {
    match ty {
        DataType::List(item) | DataType::LargeList(item)
            if matches!(item.data_type(), DataType::Struct(_)) =>
        {
            Err(Unsupported(format!(
                "`{name}` is a list of structs (a child table)"
            )))
        }
        DataType::Struct(fields) => {
            for f in fields {
                check_type(f.name(), f.data_type())?;
            }
            Ok(())
        }
        _ => Ok(()),
    }
}

/// How a leaf column's cells are produced.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Source {
    /// Written straight from the array.
    Native,
    /// Converted through `ColumnReader` and written as a JSON literal.
    Json,
}

struct Leaf {
    /// Field indexes from the batch root to this column.
    indexes: Vec<usize>,
    source: Source,
}

struct Plan {
    table: TablePlan,
    leaves: Vec<Leaf>,
    /// Top-level field indexes that go into the `~` object.
    rest: Vec<usize>,
}

fn plan_batches(batches: &[RecordBatch], opts: &Options) -> Result<Plan, Unsupported> {
    let schema = batches[0].schema();
    check_schema(&schema)?;
    let n: usize = batches.iter().map(RecordBatch::num_rows).sum();
    let mut columns = Vec::new();
    let mut leaves = Vec::new();
    let mut rest = Vec::new();
    let mut rest_keys = Vec::new();
    for (i, field) in schema.fields().iter().enumerate() {
        let arrays: Vec<&ArrayRef> = batches.iter().map(|b| b.column(i)).collect();
        if opts.sparse_threshold > 0.0 {
            let filled = n - null_count(&arrays);
            if (filled as f64) < opts.sparse_threshold * n as f64 {
                rest.push(i);
                rest_keys.push(field.name().clone());
                continue;
            }
        }
        plan_field(
            vec![field.name().clone()],
            vec![i],
            &arrays,
            n,
            &mut columns,
            &mut leaves,
        );
    }
    Ok(Plan {
        table: TablePlan {
            name: ROOT.to_string(),
            join: Join::None,
            join_col: None,
            columns,
            scalar_rows: false,
            rest_keys,
        },
        leaves,
        rest,
    })
}

fn null_count(arrays: &[&ArrayRef]) -> usize {
    arrays.iter().map(|a| a.logical_null_count()).sum()
}

/// Same decisions as `plan::plan_field` for values that come from Arrow:
/// every key is present in every row, so `nullable` is "has a null".
fn plan_field(
    path: Vec<String>,
    indexes: Vec<usize>,
    arrays: &[&ArrayRef],
    n: usize,
    columns: &mut Vec<Column>,
    leaves: &mut Vec<Leaf>,
) {
    let nulls = null_count(arrays);
    let present = n - nulls;
    let nullable = nulls > 0;
    let ty = arrays[0].data_type();
    if let DataType::Struct(fields) = ty
        && nulls == 0
        && present > 0
        && !fields.is_empty()
    {
        for (k, f) in fields.iter().enumerate() {
            let sub: Vec<&ArrayRef> = arrays.iter().map(|a| a.as_struct().column(k)).collect();
            let mut p = path.clone();
            p.push(f.name().clone());
            let mut ix = indexes.clone();
            ix.push(k);
            plan_field(p, ix, &sub, n, columns, leaves);
        }
        return;
    }
    let (col_ty, source) = if present == 0 {
        (ColType::Json, Source::Json)
    } else {
        match ty {
            DataType::Boolean => (ColType::Bool, Source::Native),
            DataType::Int8
            | DataType::Int16
            | DataType::Int32
            | DataType::Int64
            | DataType::UInt8
            | DataType::UInt16
            | DataType::UInt32
            | DataType::UInt64 => (ColType::Int, Source::Native),
            DataType::Float32 | DataType::Float64 => (ColType::Float, Source::Native),
            DataType::Utf8 | DataType::LargeUtf8 | DataType::Utf8View => {
                (ColType::Str, Source::Native)
            }
            // Arrays and objects (a struct with a null somewhere) are JSON
            // cells; everything else `ColumnReader` renders as a string.
            DataType::Struct(_) | DataType::List(_) | DataType::LargeList(_) | DataType::Null => {
                (ColType::Json, Source::Json)
            }
            _ => (ColType::Str, Source::Native),
        }
    };
    columns.push(Column {
        path,
        ty: col_ty,
        nullable,
        child: None,
        hint: 0,
        keys: Vec::new(),
    });
    leaves.push(Leaf { indexes, source });
}

/// A leaf column of one batch, ready to write.
enum Cell<'a> {
    Bool(&'a arrow::array::BooleanArray),
    I8(&'a arrow::array::Int8Array),
    I16(&'a arrow::array::Int16Array),
    I32(&'a arrow::array::Int32Array),
    I64(&'a arrow::array::Int64Array),
    U8(&'a arrow::array::UInt8Array),
    U16(&'a arrow::array::UInt16Array),
    U32(&'a arrow::array::UInt32Array),
    U64(&'a arrow::array::UInt64Array),
    F32(&'a arrow::array::Float32Array),
    F64(&'a arrow::array::Float64Array),
    Utf8(&'a arrow::array::StringArray),
    LargeUtf8(&'a arrow::array::LargeStringArray),
    View(&'a arrow::array::StringViewArray),
    Display(ArrayFormatter<'a>),
    Json(ColumnReader),
}

struct Writer<'a> {
    nulls: Option<NullBuffer>,
    cell: Cell<'a>,
}

fn leaf_array<'a>(batch: &'a RecordBatch, indexes: &[usize]) -> &'a ArrayRef {
    let mut a = batch.column(indexes[0]);
    for &k in &indexes[1..] {
        a = a.as_struct().column(k);
    }
    a
}

fn writer<'a>(
    a: &'a ArrayRef,
    source: Source,
    fmt: &'a FormatOptions,
) -> Result<Writer<'a>, Unsupported> {
    let err = |e: &dyn fmt::Display| Unsupported(e.to_string());
    let cell = if source == Source::Json {
        Cell::Json(ColumnReader::new(a.clone()).map_err(|e| err(&e))?)
    } else {
        match a.data_type() {
            DataType::Boolean => Cell::Bool(a.as_boolean()),
            DataType::Int8 => Cell::I8(a.as_primitive()),
            DataType::Int16 => Cell::I16(a.as_primitive()),
            DataType::Int32 => Cell::I32(a.as_primitive()),
            DataType::Int64 => Cell::I64(a.as_primitive()),
            DataType::UInt8 => Cell::U8(a.as_primitive()),
            DataType::UInt16 => Cell::U16(a.as_primitive()),
            DataType::UInt32 => Cell::U32(a.as_primitive()),
            DataType::UInt64 => Cell::U64(a.as_primitive()),
            DataType::Float32 => Cell::F32(a.as_primitive()),
            DataType::Float64 => Cell::F64(a.as_primitive()),
            DataType::Utf8 => Cell::Utf8(a.as_string()),
            DataType::LargeUtf8 => Cell::LargeUtf8(a.as_string()),
            DataType::Utf8View => Cell::View(a.as_string_view()),
            _ => Cell::Display(ArrayFormatter::try_new(a.as_ref(), fmt).map_err(|e| err(&e))?),
        }
    };
    Ok(Writer {
        nulls: a.logical_nulls(),
        cell,
    })
}

impl Writer<'_> {
    fn write(&self, row: usize, out: &mut String) -> Result<(), Unsupported> {
        if self.nulls.as_ref().is_some_and(|n| n.is_null(row)) {
            out.push_str("null");
            return Ok(());
        }
        match &self.cell {
            Cell::Bool(a) => out.push_str(if a.value(row) { "true" } else { "false" }),
            Cell::I8(a) => push_display(a.value(row), out),
            Cell::I16(a) => push_display(a.value(row), out),
            Cell::I32(a) => push_display(a.value(row), out),
            Cell::I64(a) => push_display(a.value(row), out),
            Cell::U8(a) => push_display(a.value(row), out),
            Cell::U16(a) => push_display(a.value(row), out),
            Cell::U32(a) => push_display(a.value(row), out),
            Cell::U64(a) => push_display(a.value(row), out),
            Cell::F32(a) => {
                // ldf-data widens through the shortest decimal text.
                let v = a.value(row);
                push_float(format!("{v}").parse::<f64>().unwrap_or(v as f64), out);
            }
            Cell::F64(a) => push_float(a.value(row), out),
            Cell::Utf8(a) => push_string_cell(a.value(row), out),
            Cell::LargeUtf8(a) => push_string_cell(a.value(row), out),
            Cell::View(a) => push_string_cell(a.value(row), out),
            Cell::Display(f) => push_string_cell(&f.value(row).to_string(), out),
            Cell::Json(r) => {
                let v = r.value(row).map_err(|e| Unsupported(e.to_string()))?;
                push_json_value(&v, out);
            }
        }
        Ok(())
    }
}

fn push_display(v: impl fmt::Display, out: &mut String) {
    let _ = write!(out, "{v}");
}

fn write_rows(plan: &Plan, batches: &[RecordBatch], out: &mut String) -> Result<(), Unsupported> {
    let fmt = FormatOptions::default();
    let names: Vec<String> = plan.table.rest_keys.clone();
    for batch in batches {
        let writers: Vec<Writer> = plan
            .leaves
            .iter()
            .map(|l| writer(leaf_array(batch, &l.indexes), l.source, &fmt))
            .collect::<Result<_, _>>()?;
        let rest: Vec<ColumnReader> = plan
            .rest
            .iter()
            .map(|&i| ColumnReader::new(batch.column(i).clone()))
            .collect::<Result<_, _>>()
            .map_err(|e| Unsupported(e.to_string()))?;
        for row in 0..batch.num_rows() {
            let start = out.len();
            for (i, w) in writers.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                w.write(row, out)?;
                if i == 0 {
                    quote_line_start(out, start);
                }
            }
            if !rest.is_empty() {
                if !writers.is_empty() {
                    out.push(',');
                }
                let mut obj: Object = Vec::with_capacity(rest.len());
                for (name, r) in names.iter().zip(&rest) {
                    let v = r.value(row).map_err(|e| Unsupported(e.to_string()))?;
                    obj.push((name.clone(), v));
                }
                push_json_value(&Value::Object(obj), out);
            }
            out.push('\n');
        }
    }
    Ok(())
}
