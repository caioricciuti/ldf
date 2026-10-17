//! Input readers. Every reader produces a stream of records so memory is
//! bounded by the largest record (JSONL, CSV, Parquet) or, where the
//! container forces it, by the whole document (a single JSON array).
//!
//! Readers never silently flatten or retype: CSV cells are strings unless
//! [`CsvOptions::infer_types`] is on, and then the rule is the conservative
//! JSON grammar in [`crate::number::parse_json_number`], plus `true`/`false`
//! and empty-cell-as-null. Both choices are recorded in the reader options so
//! benchmark reports can state them.

use std::fmt;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Read};
use std::path::Path;

use crate::number::parse_json_number;
use crate::value::Value;

#[derive(Debug)]
pub enum InputError {
    Io(io::Error),
    Json(serde_json::Error),
    #[cfg(feature = "simd")]
    SimdJson(simd_json::Error),
    Csv(csv::Error),
    #[cfg(feature = "parquet")]
    Parquet(String),
    Shape(String),
    UnknownFormat(String),
}

impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InputError::Io(e) => write!(f, "io: {e}"),
            InputError::Json(e) => write!(f, "json: {e}"),
            #[cfg(feature = "simd")]
            InputError::SimdJson(e) => write!(f, "simd-json: {e}"),
            InputError::Csv(e) => write!(f, "csv: {e}"),
            #[cfg(feature = "parquet")]
            InputError::Parquet(e) => write!(f, "parquet: {e}"),
            InputError::Shape(e) => write!(f, "shape: {e}"),
            InputError::UnknownFormat(e) => write!(f, "unknown input format: {e}"),
        }
    }
}

impl std::error::Error for InputError {}

impl From<io::Error> for InputError {
    fn from(e: io::Error) -> Self {
        InputError::Io(e)
    }
}
impl From<serde_json::Error> for InputError {
    fn from(e: serde_json::Error) -> Self {
        InputError::Json(e)
    }
}
impl From<csv::Error> for InputError {
    fn from(e: csv::Error) -> Self {
        InputError::Csv(e)
    }
}

/// Input container kinds. `Json` is a single JSON array (or a single object,
/// treated as a one-record dataset).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputKind {
    Json,
    Jsonl,
    Csv,
    Tsv,
    Parquet,
}

impl InputKind {
    pub fn from_path(path: &Path) -> Result<InputKind, InputError> {
        match path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| e.to_ascii_lowercase())
            .as_deref()
        {
            Some("json") => Ok(InputKind::Json),
            Some("jsonl" | "ndjson") => Ok(InputKind::Jsonl),
            Some("csv") => Ok(InputKind::Csv),
            Some("tsv") => Ok(InputKind::Tsv),
            Some("parquet") => Ok(InputKind::Parquet),
            other => Err(InputError::UnknownFormat(format!(
                "{} (extension {:?})",
                path.display(),
                other
            ))),
        }
    }
}

/// Which JSON parser to use for `.json` and `.jsonl`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum JsonEngine {
    #[default]
    SerdeJson,
    #[cfg(feature = "simd")]
    SimdJson,
}

#[derive(Debug, Clone)]
pub struct CsvOptions {
    pub delimiter: u8,
    pub has_headers: bool,
    /// Convert cells that match the JSON number grammar, `true`/`false`, to
    /// typed values and empty cells to null. Off means every cell is a string.
    pub infer_types: bool,
}

impl Default for CsvOptions {
    fn default() -> Self {
        CsvOptions {
            delimiter: b',',
            has_headers: true,
            infer_types: true,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ReadOptions {
    pub json_engine: JsonEngine,
    pub csv: CsvOptions,
    /// Row-group batch size for Parquet.
    pub parquet_batch_size: Option<usize>,
}

/// A streaming source of records.
pub type RecordStream<'a> = Box<dyn Iterator<Item = Result<Value, InputError>> + 'a>;

/// Opens `path` and returns a record stream, detecting the kind by extension.
pub fn open(path: &Path, opts: &ReadOptions) -> Result<RecordStream<'static>, InputError> {
    let kind = InputKind::from_path(path)?;
    open_kind(path, kind, opts)
}

pub fn open_kind(
    path: &Path,
    kind: InputKind,
    opts: &ReadOptions,
) -> Result<RecordStream<'static>, InputError> {
    match kind {
        InputKind::Json => {
            let bytes = std::fs::read(path)?;
            let records = parse_json_document(bytes, opts.json_engine)?;
            Ok(Box::new(records.into_iter().map(Ok)))
        }
        InputKind::Jsonl => {
            let file = BufReader::with_capacity(1 << 20, File::open(path)?);
            Ok(jsonl_stream(file, opts.json_engine))
        }
        InputKind::Csv => {
            let file = File::open(path)?;
            Ok(csv_stream(file, &opts.csv))
        }
        InputKind::Tsv => {
            let file = File::open(path)?;
            let mut o = opts.csv.clone();
            o.delimiter = b'\t';
            Ok(csv_stream(file, &o))
        }
        InputKind::Parquet => {
            #[cfg(feature = "parquet")]
            {
                parquet::stream(path, opts.parquet_batch_size)
            }
            #[cfg(not(feature = "parquet"))]
            {
                Err(InputError::UnknownFormat(
                    "parquet support not compiled in (feature `parquet`)".into(),
                ))
            }
        }
    }
}

/// Reads every record into memory.
pub fn read_all(path: &Path, opts: &ReadOptions) -> Result<Vec<Value>, InputError> {
    open(path, opts)?.collect()
}

// ---- JSON --------------------------------------------------------------

/// Parses a whole JSON document held in memory into records. An array is a
/// dataset; any other root is a single record.
pub fn parse_json_document(
    mut bytes: Vec<u8>,
    engine: JsonEngine,
) -> Result<Vec<Value>, InputError> {
    let root = parse_json_value(&mut bytes, engine)?;
    Ok(match root {
        Value::Array(items) => items,
        other => vec![other],
    })
}

/// Parses one JSON value from a mutable buffer (simd-json mutates in place).
pub fn parse_json_value(bytes: &mut [u8], engine: JsonEngine) -> Result<Value, InputError> {
    match engine {
        JsonEngine::SerdeJson => Ok(serde_json::from_slice(bytes)?),
        #[cfg(feature = "simd")]
        JsonEngine::SimdJson => simd_json::serde::from_slice(bytes).map_err(InputError::SimdJson),
    }
}

/// Streams newline-delimited JSON. Blank lines are skipped. Each line is
/// parsed independently so one bad line surfaces as one error item.
pub fn jsonl_stream<R: Read + 'static>(reader: R, engine: JsonEngine) -> RecordStream<'static> {
    let mut lines = BufReader::with_capacity(1 << 20, reader);
    let mut buf: Vec<u8> = Vec::with_capacity(4096);
    Box::new(std::iter::from_fn(move || {
        loop {
            buf.clear();
            match lines.read_until(b'\n', &mut buf) {
                Ok(0) => return None,
                Ok(_) => {}
                Err(e) => return Some(Err(e.into())),
            }
            while matches!(buf.last(), Some(b'\n' | b'\r')) {
                buf.pop();
            }
            if buf.iter().all(u8::is_ascii_whitespace) {
                continue;
            }
            return Some(parse_json_value(&mut buf, engine));
        }
    }))
}

// ---- CSV ---------------------------------------------------------------

/// Infers a typed value from a CSV cell under the documented rule.
pub fn csv_cell_value(cell: &str, infer: bool) -> Value {
    if !infer {
        return Value::String(cell.to_owned());
    }
    if cell.is_empty() {
        return Value::Null;
    }
    match cell {
        "true" => return Value::Bool(true),
        "false" => return Value::Bool(false),
        "null" => return Value::Null,
        _ => {}
    }
    if let Some(n) = parse_json_number(cell) {
        return Value::Number(n);
    }
    Value::String(cell.to_owned())
}

pub fn csv_stream<R: Read + 'static>(reader: R, opts: &CsvOptions) -> RecordStream<'static> {
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(opts.delimiter)
        .has_headers(opts.has_headers)
        .flexible(true)
        .buffer_capacity(1 << 20)
        .from_reader(reader);
    let infer = opts.infer_types;
    let has_headers = opts.has_headers;
    let mut headers: Option<Vec<String>> = None;
    let mut header_err: Option<InputError> = None;
    if has_headers {
        match rdr.headers() {
            Ok(h) => headers = Some(h.iter().map(str::to_owned).collect()),
            Err(e) => header_err = Some(e.into()),
        }
    }
    let mut record = csv::StringRecord::new();
    let mut index = 0usize;
    Box::new(std::iter::from_fn(move || {
        if let Some(e) = header_err.take() {
            return Some(Err(e));
        }
        match rdr.read_record(&mut record) {
            Ok(false) => None,
            Err(e) => Some(Err(e.into())),
            Ok(true) => {
                index += 1;
                let keys: Vec<String> = match &headers {
                    Some(h) => {
                        if h.len() != record.len() {
                            return Some(Err(InputError::Shape(format!(
                                "csv row {index} has {} cells, header has {}",
                                record.len(),
                                h.len()
                            ))));
                        }
                        h.clone()
                    }
                    None => (0..record.len()).map(|i| format!("c{i}")).collect(),
                };
                let fields = keys
                    .into_iter()
                    .zip(record.iter())
                    .map(|(k, c)| (k, csv_cell_value(c, infer)))
                    .collect();
                Some(Ok(Value::Object(fields)))
            }
        }
    }))
}

// ---- Parquet -----------------------------------------------------------

#[cfg(feature = "parquet")]
pub mod parquet {
    //! Parquet via the Arrow reader. Each Arrow row becomes a [`Value`]
    //! object. Primitive Arrow types map to the obvious JSON type; temporal,
    //! decimal and binary types are rendered through Arrow's own display
    //! formatter as strings (documented lossy mapping, same as Arrow's JSON
    //! writer behaviour for those types). Struct maps to object, List and
    //! LargeList to array, Map to object with stringified keys.

    use std::fs::File;
    use std::path::Path;
    use std::sync::Arc;

    use arrow::array::{Array, ArrayRef, AsArray, RecordBatch};
    use arrow::datatypes::*;
    use arrow::util::display::{ArrayFormatter, FormatOptions};
    use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;

    use super::{InputError, RecordStream};
    use crate::number::Number;
    use crate::value::Value;

    pub fn stream(
        path: &Path,
        batch_size: Option<usize>,
    ) -> Result<RecordStream<'static>, InputError> {
        let file = File::open(path)?;
        let mut builder = ParquetRecordBatchReaderBuilder::try_new(file)
            .map_err(|e| InputError::Parquet(e.to_string()))?;
        if let Some(n) = batch_size {
            builder = builder.with_batch_size(n);
        }
        let reader = builder
            .build()
            .map_err(|e| InputError::Parquet(e.to_string()))?;
        let mut pending: std::vec::IntoIter<Value> = Vec::new().into_iter();
        let mut reader = reader;
        Ok(Box::new(std::iter::from_fn(move || {
            loop {
                if let Some(v) = pending.next() {
                    return Some(Ok(v));
                }
                match reader.next() {
                    None => return None,
                    Some(Err(e)) => return Some(Err(InputError::Parquet(e.to_string()))),
                    Some(Ok(batch)) => match batch_to_values(&batch) {
                        Ok(vals) => pending = vals.into_iter(),
                        Err(e) => return Some(Err(e)),
                    },
                }
            }
        })))
    }

    pub fn batch_to_values(batch: &RecordBatch) -> Result<Vec<Value>, InputError> {
        let schema = batch.schema();
        let cols: Vec<ColumnReader> = batch
            .columns()
            .iter()
            .map(|c| ColumnReader::new(c.clone()))
            .collect::<Result<_, _>>()?;
        let names: Vec<&str> = schema.fields().iter().map(|f| f.name().as_str()).collect();
        let mut out = Vec::with_capacity(batch.num_rows());
        for row in 0..batch.num_rows() {
            let mut fields = Vec::with_capacity(cols.len());
            for (name, col) in names.iter().zip(&cols) {
                fields.push(((*name).to_owned(), col.value(row)?));
            }
            out.push(Value::Object(fields));
        }
        Ok(out)
    }

    enum ColumnReader {
        Typed(ArrayRef),
        Struct(Vec<(String, ColumnReader)>, ArrayRef),
        List(Box<ColumnReader>, ArrayRef),
        LargeList(Box<ColumnReader>, ArrayRef),
        Display(ArrayRef),
    }

    impl ColumnReader {
        fn new(array: ArrayRef) -> Result<ColumnReader, InputError> {
            Ok(match array.data_type() {
                DataType::Null
                | DataType::Boolean
                | DataType::Int8
                | DataType::Int16
                | DataType::Int32
                | DataType::Int64
                | DataType::UInt8
                | DataType::UInt16
                | DataType::UInt32
                | DataType::UInt64
                | DataType::Float32
                | DataType::Float64
                | DataType::Utf8
                | DataType::LargeUtf8
                | DataType::Utf8View => ColumnReader::Typed(array),
                DataType::Struct(_) => {
                    let s = array.as_struct();
                    let children = s
                        .fields()
                        .iter()
                        .zip(s.columns())
                        .map(|(f, c)| ColumnReader::new(c.clone()).map(|r| (f.name().clone(), r)))
                        .collect::<Result<Vec<_>, _>>()?;
                    ColumnReader::Struct(children, array)
                }
                DataType::List(_) => {
                    let inner = ColumnReader::new(array.as_list::<i32>().values().clone())?;
                    ColumnReader::List(Box::new(inner), array)
                }
                DataType::LargeList(_) => {
                    let inner = ColumnReader::new(array.as_list::<i64>().values().clone())?;
                    ColumnReader::LargeList(Box::new(inner), array)
                }
                _ => ColumnReader::Display(array),
            })
        }

        fn value(&self, row: usize) -> Result<Value, InputError> {
            match self {
                ColumnReader::Typed(a) => Ok(typed_value(a, row)),
                ColumnReader::Struct(children, a) => {
                    if a.is_null(row) {
                        return Ok(Value::Null);
                    }
                    let mut fields = Vec::with_capacity(children.len());
                    for (name, child) in children {
                        fields.push((name.clone(), child.value(row)?));
                    }
                    Ok(Value::Object(fields))
                }
                ColumnReader::List(inner, a) => {
                    if a.is_null(row) {
                        return Ok(Value::Null);
                    }
                    let l = a.as_list::<i32>();
                    let offs = l.value_offsets();
                    let (s, e) = (offs[row] as usize, offs[row + 1] as usize);
                    (s..e)
                        .map(|i| inner.value(i))
                        .collect::<Result<Vec<_>, _>>()
                        .map(Value::Array)
                }
                ColumnReader::LargeList(inner, a) => {
                    if a.is_null(row) {
                        return Ok(Value::Null);
                    }
                    let l = a.as_list::<i64>();
                    let offs = l.value_offsets();
                    let (s, e) = (offs[row] as usize, offs[row + 1] as usize);
                    (s..e)
                        .map(|i| inner.value(i))
                        .collect::<Result<Vec<_>, _>>()
                        .map(Value::Array)
                }
                ColumnReader::Display(a) => {
                    if a.is_null(row) {
                        return Ok(Value::Null);
                    }
                    let opts = FormatOptions::default();
                    let f = ArrayFormatter::try_new(a.as_ref(), &opts)
                        .map_err(|e| InputError::Parquet(e.to_string()))?;
                    Ok(Value::String(f.value(row).to_string()))
                }
            }
        }
    }

    fn typed_value(a: &ArrayRef, row: usize) -> Value {
        if a.is_null(row) {
            return Value::Null;
        }
        match a.data_type() {
            DataType::Null => Value::Null,
            DataType::Boolean => Value::Bool(a.as_boolean().value(row)),
            DataType::Int8 => Value::from(a.as_primitive::<Int8Type>().value(row) as i64),
            DataType::Int16 => Value::from(a.as_primitive::<Int16Type>().value(row) as i64),
            DataType::Int32 => Value::from(a.as_primitive::<Int32Type>().value(row) as i64),
            DataType::Int64 => Value::from(a.as_primitive::<Int64Type>().value(row)),
            DataType::UInt8 => Value::from(a.as_primitive::<UInt8Type>().value(row) as u64),
            DataType::UInt16 => Value::from(a.as_primitive::<UInt16Type>().value(row) as u64),
            DataType::UInt32 => Value::from(a.as_primitive::<UInt32Type>().value(row) as u64),
            DataType::UInt64 => Value::from(a.as_primitive::<UInt64Type>().value(row)),
            DataType::Float32 => {
                // Widen through the shortest decimal text so 0.1f32 becomes
                // 0.1, not 0.10000000149011612.
                let v = a.as_primitive::<Float32Type>().value(row);
                let s = format!("{v}");
                Value::Number(Number::F64(s.parse::<f64>().unwrap_or(v as f64)))
            }
            DataType::Float64 => Value::from(a.as_primitive::<Float64Type>().value(row)),
            DataType::Utf8 => Value::String(a.as_string::<i32>().value(row).to_owned()),
            DataType::LargeUtf8 => Value::String(a.as_string::<i64>().value(row).to_owned()),
            DataType::Utf8View => Value::String(a.as_string_view().value(row).to_owned()),
            _ => unreachable!("typed_value called on non-typed column"),
        }
    }

    /// Writes a Parquet file from a JSONL file, inferring the Arrow schema
    /// from the data (arrow-json). Used by the fixture generator so Parquet
    /// fixtures are derived from the same records as the JSON ones.
    pub fn write_from_jsonl(
        jsonl: &Path,
        out: &Path,
        batch_size: usize,
    ) -> Result<usize, InputError> {
        use arrow::json::ReaderBuilder;
        use arrow::json::reader::infer_json_schema_from_seekable;
        use parquet::arrow::ArrowWriter;
        use parquet::basic::Compression;
        use parquet::file::properties::WriterProperties;
        use std::io::BufReader;

        let mut probe = BufReader::new(File::open(jsonl)?);
        let (schema, _) = infer_json_schema_from_seekable(&mut probe, None)
            .map_err(|e| InputError::Parquet(e.to_string()))?;
        let schema = Arc::new(schema);
        let reader = ReaderBuilder::new(schema.clone())
            .with_batch_size(batch_size)
            .build(BufReader::new(File::open(jsonl)?))
            .map_err(|e| InputError::Parquet(e.to_string()))?;
        let props = WriterProperties::builder()
            .set_compression(Compression::ZSTD(Default::default()))
            .build();
        let mut writer = ArrowWriter::try_new(File::create(out)?, schema, Some(props))
            .map_err(|e| InputError::Parquet(e.to_string()))?;
        let mut rows = 0;
        for batch in reader {
            let batch = batch.map_err(|e| InputError::Parquet(e.to_string()))?;
            rows += batch.num_rows();
            writer
                .write(&batch)
                .map_err(|e| InputError::Parquet(e.to_string()))?;
        }
        writer
            .close()
            .map_err(|e| InputError::Parquet(e.to_string()))?;
        Ok(rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jsonl_streams_and_skips_blank_lines() {
        let data = "{\"a\":1}\n\n{\"a\":2}\r\n";
        let recs: Vec<Value> = jsonl_stream(
            io::Cursor::new(data.as_bytes().to_vec()),
            JsonEngine::SerdeJson,
        )
        .collect::<Result<_, _>>()
        .unwrap();
        assert_eq!(recs.len(), 2);
        assert_eq!(recs[1].to_string(), "{\"a\":2}");
    }

    #[test]
    fn csv_type_inference_is_conservative() {
        let data = "id,name,score,ok,note\n1,ann,2.5,true,\n05,\"x,y\",1e3,false,null\n";
        let recs: Vec<Value> = csv_stream(
            io::Cursor::new(data.as_bytes().to_vec()),
            &CsvOptions::default(),
        )
        .collect::<Result<_, _>>()
        .unwrap();
        assert_eq!(
            recs[0].to_string(),
            r#"{"id":1,"name":"ann","score":2.5,"ok":true,"note":null}"#
        );
        assert_eq!(
            recs[1].to_string(),
            r#"{"id":"05","name":"x,y","score":1000.0,"ok":false,"note":null}"#
        );
    }

    #[cfg(feature = "simd")]
    #[test]
    fn simd_and_serde_agree() {
        let src = r#"[{"a":1,"b":[1.5,"x",null,true],"c":{"d":"é"}}]"#.as_bytes();
        let a = parse_json_document(src.to_vec(), JsonEngine::SerdeJson).unwrap();
        let b = parse_json_document(src.to_vec(), JsonEngine::SimdJson).unwrap();
        assert_eq!(a, b);
    }
}
