//! Flat row formats: CSV, TSV and Markdown pipe tables.
//!
//! All three need a [`FlatSchema`]: every record an object of primitives.
//! Anything else is reported as unsupported. Known, documented losses:
//!
//! * cells are untyped text; decoding applies the conservative inference of
//!   [`ldf_data::input::csv_cell_value`], so the string `"42"` comes back as
//!   the number `42` and the string `"true"` as a boolean. The benchmark
//!   reports this as a semantic round-trip failure where it happens.
//! * an empty cell stands for both `null` and a missing field.
//! * Markdown cells cannot contain a newline; `\n` is written as the literal
//!   two characters `\n` and pipes are escaped as `\|`. Decoding reverses
//!   both, which makes a source string containing a literal backslash-n
//!   ambiguous. Leading and trailing spaces inside Markdown cells are lost.

use ldf_data::input::csv_cell_value;
use ldf_data::{FlatSchema, Number, Value};

use crate::{Capabilities, DecodeError, EncodeError, Format, StreamEncoder, unsupported};

const CAPS: Capabilities = Capabilities {
    nested: false,
    typed: false,
    missing_vs_null: false,
    streaming_encode: true,
    requires_count: false,
};

/// Text of a primitive cell shared by all three formats. Returns `None` for
/// nested values so callers can report them.
fn cell_text(v: &Value, buf: &mut String) -> bool {
    buf.clear();
    match v {
        Value::Null => {}
        Value::Bool(b) => buf.push_str(if *b { "true" } else { "false" }),
        Value::Number(n) => {
            if !n.write_canonical(buf) {
                buf.clear();
            }
        }
        Value::String(s) => buf.push_str(s),
        Value::Array(_) | Value::Object(_) => return false,
    }
    true
}

fn schema_or_unsupported(name: &'static str, records: &[Value]) -> Result<FlatSchema, EncodeError> {
    FlatSchema::infer(records).map_err(|e| unsupported(name, e.to_string()))
}

// ---- CSV / TSV ----------------------------------------------------------

fn encode_delimited(
    name: &'static str,
    delimiter: u8,
    records: &[Value],
    out: &mut Vec<u8>,
) -> Result<(), EncodeError> {
    let schema = schema_or_unsupported(name, records)?;
    let mut w = csv::WriterBuilder::new()
        .delimiter(delimiter)
        .from_writer(out);
    w.write_record(&schema.columns)
        .map_err(|e| EncodeError::Other(e.to_string()))?;
    let mut row: Vec<String> = vec![String::new(); schema.len()];
    let mut buf = String::new();
    for rec in records {
        for cell in row.iter_mut() {
            cell.clear();
        }
        if let Value::Object(fields) = rec {
            for (k, v) in fields {
                let i = schema.column_index(k).expect("schema covers every key");
                if !cell_text(v, &mut buf) {
                    return Err(unsupported(name, format!("nested value at {k}")));
                }
                row[i].clone_from(&buf);
            }
        }
        w.write_record(&row)
            .map_err(|e| EncodeError::Other(e.to_string()))?;
    }
    w.flush()?;
    Ok(())
}

fn decode_delimited(
    name: &'static str,
    delimiter: u8,
    bytes: &[u8],
) -> Result<Vec<Value>, DecodeError> {
    let mut rdr = csv::ReaderBuilder::new()
        .delimiter(delimiter)
        .has_headers(true)
        .from_reader(bytes);
    let headers: Vec<String> = rdr
        .headers()
        .map_err(|e| DecodeError {
            format: name,
            line: None,
            message: e.to_string(),
        })?
        .iter()
        .map(str::to_owned)
        .collect();
    let mut out = Vec::new();
    for (i, rec) in rdr.records().enumerate() {
        let rec = rec.map_err(|e| DecodeError {
            format: name,
            line: Some(i + 2),
            message: e.to_string(),
        })?;
        if rec.len() != headers.len() {
            return Err(DecodeError {
                format: name,
                line: Some(i + 2),
                message: format!("{} cells, header has {}", rec.len(), headers.len()),
            });
        }
        out.push(Value::Object(
            headers
                .iter()
                .zip(rec.iter())
                .map(|(k, c)| (k.clone(), csv_cell_value(c, true)))
                .collect(),
        ));
    }
    Ok(out)
}

/// Streaming CSV: the header comes from the first record, later records
/// must use exactly the same keys in the same order.
struct DelimitedStream {
    name: &'static str,
    delimiter: u8,
    columns: Option<Vec<String>>,
    buf: String,
    row: Vec<u8>,
}

impl DelimitedStream {
    fn write_row(
        &mut self,
        cells: impl Iterator<Item = String>,
        out: &mut Vec<u8>,
    ) -> Result<(), EncodeError> {
        self.row.clear();
        {
            let mut w = csv::WriterBuilder::new()
                .delimiter(self.delimiter)
                .from_writer(&mut self.row);
            w.write_record(cells)
                .map_err(|e| EncodeError::Other(e.to_string()))?;
            w.flush()?;
        }
        out.extend_from_slice(&self.row);
        Ok(())
    }
}

impl StreamEncoder for DelimitedStream {
    fn record(&mut self, rec: &Value, out: &mut Vec<u8>) -> Result<(), EncodeError> {
        let Value::Object(fields) = rec else {
            return Err(unsupported(
                self.name,
                format!("record is a {}", rec.type_name()),
            ));
        };
        if self.columns.is_none() {
            let cols: Vec<String> = fields.iter().map(|(k, _)| k.clone()).collect();
            self.write_row(cols.iter().cloned(), out)?;
            self.columns = Some(cols);
        }
        let cols = self.columns.as_ref().unwrap();
        if cols.len() != fields.len() || cols.iter().zip(fields).any(|(c, (k, _))| c != k) {
            return Err(unsupported(
                self.name,
                "streaming mode requires every record to have the first record's keys in order",
            ));
        }
        let mut cells = Vec::with_capacity(fields.len());
        for (k, v) in fields {
            if !cell_text(v, &mut self.buf) {
                return Err(unsupported(self.name, format!("nested value at {k}")));
            }
            cells.push(self.buf.clone());
        }
        self.write_row(cells.into_iter(), out)
    }
    fn finish(&mut self, _out: &mut Vec<u8>) -> Result<(), EncodeError> {
        Ok(())
    }
}

pub struct Csv;

impl Format for Csv {
    fn name(&self) -> &'static str {
        "csv"
    }
    fn extension(&self) -> &'static str {
        "csv"
    }
    fn capabilities(&self) -> Capabilities {
        CAPS
    }
    fn encode(&self, records: &[Value], out: &mut Vec<u8>) -> Result<(), EncodeError> {
        encode_delimited("csv", b',', records, out)
    }
    fn decode(&self, bytes: &[u8]) -> Result<Vec<Value>, DecodeError> {
        decode_delimited("csv", b',', bytes)
    }
    fn stream_encoder(&self) -> Option<Box<dyn StreamEncoder>> {
        Some(Box::new(DelimitedStream {
            name: "csv",
            delimiter: b',',
            columns: None,
            buf: String::new(),
            row: Vec::new(),
        }))
    }
}

pub struct Tsv;

impl Format for Tsv {
    fn name(&self) -> &'static str {
        "tsv"
    }
    fn extension(&self) -> &'static str {
        "tsv"
    }
    fn capabilities(&self) -> Capabilities {
        CAPS
    }
    fn encode(&self, records: &[Value], out: &mut Vec<u8>) -> Result<(), EncodeError> {
        encode_delimited("tsv", b'\t', records, out)
    }
    fn decode(&self, bytes: &[u8]) -> Result<Vec<Value>, DecodeError> {
        decode_delimited("tsv", b'\t', bytes)
    }
    fn stream_encoder(&self) -> Option<Box<dyn StreamEncoder>> {
        Some(Box::new(DelimitedStream {
            name: "tsv",
            delimiter: b'\t',
            columns: None,
            buf: String::new(),
            row: Vec::new(),
        }))
    }
}

// ---- Markdown -----------------------------------------------------------

fn md_escape(cell: &str, out: &mut String) {
    for ch in cell.chars() {
        match ch {
            '|' => out.push_str("\\|"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\\' => out.push_str("\\\\"),
            c => out.push(c),
        }
    }
}

fn md_unescape(cell: &str) -> String {
    let mut out = String::with_capacity(cell.len());
    let mut chars = cell.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('|') => out.push('|'),
                Some('n') => out.push('\n'),
                Some('r') => out.push('\r'),
                Some('\\') => out.push('\\'),
                Some(o) => {
                    out.push('\\');
                    out.push(o);
                }
                None => out.push('\\'),
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// Splits a Markdown table row on unescaped pipes, trimming the outer pipes.
fn md_split_row(line: &str) -> Vec<String> {
    let line = line.trim();
    let line = line.strip_prefix('|').unwrap_or(line);
    let line = line.strip_suffix('|').unwrap_or(line);
    let mut cells = Vec::new();
    let mut cur = String::new();
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            cur.push(c);
            if let Some(n) = chars.next() {
                cur.push(n);
            }
        } else if c == '|' {
            cells.push(std::mem::take(&mut cur));
        } else {
            cur.push(c);
        }
    }
    cells.push(cur);
    cells.into_iter().map(|c| md_unescape(c.trim())).collect()
}

pub struct Markdown;

impl Format for Markdown {
    fn name(&self) -> &'static str {
        "markdown"
    }
    fn extension(&self) -> &'static str {
        "md"
    }
    fn capabilities(&self) -> Capabilities {
        CAPS
    }
    fn encode(&self, records: &[Value], out: &mut Vec<u8>) -> Result<(), EncodeError> {
        let schema = schema_or_unsupported("markdown", records)?;
        let mut s = String::new();
        s.push('|');
        for c in &schema.columns {
            s.push(' ');
            md_escape(c, &mut s);
            s.push_str(" |");
        }
        s.push('\n');
        s.push('|');
        for _ in &schema.columns {
            s.push_str(" --- |");
        }
        s.push('\n');
        let mut row: Vec<String> = vec![String::new(); schema.len()];
        let mut buf = String::new();
        for rec in records {
            for cell in row.iter_mut() {
                cell.clear();
            }
            if let Value::Object(fields) = rec {
                for (k, v) in fields {
                    let i = schema.column_index(k).expect("schema covers every key");
                    if !cell_text(v, &mut buf) {
                        return Err(unsupported("markdown", format!("nested value at {k}")));
                    }
                    row[i].clear();
                    md_escape(&buf, &mut row[i]);
                }
            }
            s.push('|');
            for cell in &row {
                s.push(' ');
                s.push_str(cell);
                s.push_str(" |");
            }
            s.push('\n');
        }
        out.extend_from_slice(s.as_bytes());
        Ok(())
    }
    fn decode(&self, bytes: &[u8]) -> Result<Vec<Value>, DecodeError> {
        let text = std::str::from_utf8(bytes).map_err(|e| DecodeError {
            format: "markdown",
            line: None,
            message: e.to_string(),
        })?;
        let mut lines = text.lines().enumerate();
        let Some((_, header)) = lines.next() else {
            return Ok(Vec::new());
        };
        let headers = md_split_row(header);
        match lines.next() {
            Some((_, sep)) if sep.contains("---") => {}
            _ => {
                return Err(DecodeError {
                    format: "markdown",
                    line: Some(2),
                    message: "missing separator row".into(),
                });
            }
        }
        let mut out = Vec::new();
        for (i, line) in lines {
            if line.trim().is_empty() {
                continue;
            }
            let cells = md_split_row(line);
            if cells.len() != headers.len() {
                return Err(DecodeError {
                    format: "markdown",
                    line: Some(i + 1),
                    message: format!("{} cells, header has {}", cells.len(), headers.len()),
                });
            }
            out.push(Value::Object(
                headers
                    .iter()
                    .zip(cells)
                    .map(|(k, c)| (k.clone(), csv_cell_value(&c, true)))
                    .collect(),
            ));
        }
        Ok(out)
    }
}

#[allow(dead_code)]
fn _number_is_used(n: Number) -> f64 {
    n.as_f64()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ldf_data::obj;

    fn sample() -> Vec<Value> {
        vec![
            obj! {"id" => 1, "name" => "Ann, \"A\"", "ok" => true, "score" => 2.5, "note" => Value::Null},
            obj! {"id" => 2, "name" => "Bob|x\ny", "ok" => false, "score" => 1e21, "note" => "n"},
        ]
    }

    #[test]
    fn csv_and_tsv_round_trip_semantically() {
        for f in [&Csv as &dyn Format, &Tsv] {
            let s = f.encode_to_string(&sample()).unwrap();
            let back = f.decode(s.as_bytes()).unwrap();
            assert_eq!(back.len(), 2);
            assert!(back[0].semantic_eq(&sample()[0]), "{}: {s}", f.name());
            assert!(back[1].semantic_eq(&sample()[1]), "{}: {s}", f.name());
        }
    }

    #[test]
    fn markdown_round_trip_semantically() {
        let s = Markdown.encode_to_string(&sample()).unwrap();
        assert!(s.starts_with("| id | name | ok | score | note |\n| --- |"));
        let back = Markdown.decode(s.as_bytes()).unwrap();
        assert!(back[0].semantic_eq(&sample()[0]), "{s}");
        assert!(back[1].semantic_eq(&sample()[1]), "{s}");
    }

    #[test]
    fn nested_is_unsupported_not_flattened() {
        let recs = vec![obj! {"a" => obj! {"b" => 1}}];
        for f in [&Csv as &dyn Format, &Tsv, &Markdown] {
            match f.encode(&recs, &mut Vec::new()) {
                Err(EncodeError::Unsupported(u)) => assert!(u.reason.contains("nested")),
                other => panic!("{}: expected unsupported, got {other:?}", f.name()),
            }
        }
    }

    #[test]
    fn typed_strings_are_a_known_loss() {
        let recs = vec![obj! {"x" => "42"}];
        let s = Csv.encode_to_string(&recs).unwrap();
        let back = Csv.decode(s.as_bytes()).unwrap();
        assert!(!back[0].semantic_eq(&recs[0]));
    }
}
