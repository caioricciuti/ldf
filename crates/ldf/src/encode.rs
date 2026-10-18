//! Batch, block and record-at-a-time encoders.

use std::collections::HashMap;

use ldf_data::{Number, Object, Value};

use crate::ROOT;
use crate::plan::{ColType, Column, Join, Options, TablePlan, lookup, lookup_hint, plan_with};
use crate::text::{
    push_float, push_int, push_json_string, push_json_value, push_path, push_string_cell,
};

/// Encodes a whole dataset (spec section 7 flattening rules apply).
pub fn encode(records: &[Value]) -> String {
    encode_with(records, &Options::default())
}

/// Encodes a whole dataset with planner options.
pub fn encode_with(records: &[Value], opts: &Options) -> String {
    let mut enc = Encoder::default();
    enc.encode_dataset_with(records, opts);
    enc.finish()
}

#[derive(Default)]
pub struct Encoder {
    out: String,
    /// Rows already written per table name, so ordinal joins keep counting
    /// across the blocks of a [`BlockEncoder`]. Empty for a batch encode.
    offsets: HashMap<String, usize>,
}

impl Encoder {
    pub fn finish(self) -> String {
        self.out
    }

    pub fn encode_dataset(&mut self, records: &[Value]) {
        self.encode_dataset_with(records, &Options::default());
    }

    pub fn encode_dataset_with(&mut self, records: &[Value], opts: &Options) {
        let p = plan_with(records, opts);
        self.write_table(&p, records, true);
    }

    /// Writes the root table of a plan: its header (when `header` is set),
    /// its rows, then its child tables. Ordinal joins start at the number of
    /// rows this encoder has already written for the table.
    fn write_table(&mut self, p: &TablePlan, records: &[Value], header: bool) {
        if header {
            self.out.push_str(&p.header());
            self.out.push('\n');
        }
        let base = *self.offsets.entry(p.name.clone()).or_insert(0);
        if p.scalar_rows {
            for r in records {
                push_json_value(r, &mut self.out);
                self.out.push('\n');
            }
            *self.offsets.get_mut(&p.name).expect("offset") += records.len();
            return;
        }
        let objs: Vec<&Object> = records
            .iter()
            .map(|r| r.as_object().expect("planned as objects"))
            .collect();
        for o in &objs {
            self.write_row(p, o, None);
        }
        *self.offsets.get_mut(&p.name).expect("offset") += objs.len();
        self.write_children(p, &objs, base);
    }

    /// Writes one row. `join` is the join cell text for child rows.
    fn write_row(&mut self, p: &TablePlan, o: &Object, join: Option<&str>) {
        let start = self.out.len();
        let mut first = true;
        if let Some(j) = join {
            self.out.push_str(j);
            quote_line_start(&mut self.out, start);
            first = false;
        }
        for c in &p.columns {
            if !first {
                self.out.push(',');
            }
            self.write_cell(c, lookup_hint(o, &c.path, c.hint));
            if first {
                quote_line_start(&mut self.out, start);
            }
            first = false;
        }
        if !p.rest_keys.is_empty() {
            if !first {
                self.out.push(',');
            }
            let rest: Object = o
                .iter()
                .filter(|(k, _)| p.rest_keys.iter().any(|r| r == k))
                .cloned()
                .collect();
            if !rest.is_empty() {
                push_json_value(&Value::Object(rest), &mut self.out);
            }
        }
        self.out.push('\n');
    }

    fn write_cell(&mut self, c: &Column, v: Option<&Value>) {
        let Some(v) = v else { return };
        if v.is_null() {
            self.out.push_str("null");
            return;
        }
        match c.ty {
            ColType::Int => match v {
                Value::Number(n) => push_int(*n, &mut self.out),
                other => push_json_value(other, &mut self.out),
            },
            ColType::Float => match v {
                Value::Number(Number::F64(f)) => push_float(*f, &mut self.out),
                Value::Number(n) => push_int(*n, &mut self.out),
                other => push_json_value(other, &mut self.out),
            },
            ColType::Str => match v {
                Value::String(s) => push_string_cell(s, &mut self.out),
                other => push_json_value(other, &mut self.out),
            },
            ColType::Bool => match v {
                Value::Bool(b) => self.out.push_str(if *b { "true" } else { "false" }),
                other => push_json_value(other, &mut self.out),
            },
            ColType::Json => push_json_value(v, &mut self.out),
            ColType::Table => match v {
                Value::Array(a) => self.out.push_str(&a.len().to_string()),
                other => push_json_value(other, &mut self.out),
            },
        }
    }

    /// Child tables of `p`, in column order, each followed by its own
    /// children. `base` is the ordinal of the row before `parents[0]` in the
    /// whole document, so ordinal joins are global.
    fn write_children(&mut self, p: &TablePlan, parents: &[&Object], base: usize) {
        for c in &p.columns {
            let Some(child) = &c.child else { continue };
            self.out.push_str(&child.header());
            self.out.push('\n');
            let child_base = *self.offsets.entry(child.name.clone()).or_insert(0);
            let mut elements: Vec<&Object> = Vec::new();
            for (idx, parent) in parents.iter().enumerate() {
                let Some(Value::Array(items)) = lookup_hint(parent, &c.path, c.hint) else {
                    continue;
                };
                let join_text = match child.join {
                    Join::Ordinal => (base + idx + 1).to_string(),
                    Join::ParentKey => {
                        let (path, ty) = child.join_col.as_ref().expect("join column");
                        let mut s = String::new();
                        let v = lookup(parent, path).expect("join key present");
                        let col = Column {
                            path: path.clone(),
                            ty: *ty,
                            nullable: false,
                            child: None,
                            hint: 0,
                        };
                        let mut tmp = Encoder::default();
                        tmp.write_cell(&col, Some(v));
                        s.push_str(&tmp.out);
                        s
                    }
                    Join::None => String::new(),
                };
                for item in items {
                    let o = item.as_object().expect("planned as objects");
                    self.write_row(child, o, Some(&join_text));
                    elements.push(o);
                }
            }
            *self.offsets.get_mut(&child.name).expect("offset") += elements.len();
            self.write_children(child, &elements, child_base);
        }
    }
}

/// Spec 5: a bare first cell starting with `@` would read as a header line.
/// Only a bare string can start with `@`, so the cell text is the string.
fn quote_line_start(out: &mut String, start: usize) {
    if out[start..].starts_with('@') {
        let s = out.split_off(start);
        push_json_string(&s, out);
    }
}

/// Block encoder (spec section 8): buffers up to `block` records, plans
/// each block with the batch rules and writes it: the root header unless
/// the previous block ended on the same root header (rows belong to the
/// most recent header, so a block that wrote child tables forces it), the
/// root rows, then the block's child tables. Child tables join by ordinal, numbered across the whole
/// document, so the decoder attaches them exactly as for a batch encoding.
/// Memory is bounded by the block; the output is the batch output when the
/// data is uniform, with a child header repeated per block.
pub struct BlockEncoder {
    block: usize,
    opts: Options,
    buf: Vec<Value>,
    enc: Encoder,
    last_header: Option<String>,
    any: bool,
}

impl BlockEncoder {
    pub fn new(block: usize) -> Self {
        BlockEncoder::with_options(block, Options::default())
    }

    pub fn with_options(block: usize, opts: Options) -> Self {
        BlockEncoder {
            block: block.max(1),
            opts: Options {
                ordinal_joins: true,
                ..opts
            },
            buf: Vec::new(),
            enc: Encoder::default(),
            last_header: None,
            any: false,
        }
    }

    /// Takes one record; writes a block to `out` when the buffer is full.
    pub fn record(&mut self, rec: Value, out: &mut String) {
        self.buf.push(rec);
        if self.buf.len() >= self.block {
            self.flush(out);
        }
    }

    /// Writes the buffered records as one block.
    pub fn flush(&mut self, out: &mut String) {
        if self.buf.is_empty() {
            return;
        }
        let p = plan_with(&self.buf, &self.opts);
        let header = p.header();
        let emit = self.last_header.as_deref() != Some(header.as_str());
        self.enc.write_table(&p, &self.buf, emit);
        // After a child table the most recent header is the child's, so the
        // next block must start with the root header again.
        let wrote_children = p.columns.iter().any(|c| c.child.is_some());
        self.last_header = if wrote_children { None } else { Some(header) };
        self.any = true;
        out.push_str(&self.enc.out);
        self.enc.out.clear();
        self.buf.clear();
    }

    /// Flushes the last block; an empty dataset is a header with no rows.
    pub fn finish(&mut self, out: &mut String) {
        self.flush(out);
        if !self.any {
            out.push_str(&format!("@{ROOT} _:j\n"));
            self.any = true;
        }
    }
}

/// Record-at-a-time encoder (spec section 8): no flattening, no child
/// tables, a new header whenever the key list or a column type changes.
#[derive(Default)]
pub struct StreamEncoder {
    header: Vec<(String, ColType)>,
    any: bool,
}

impl StreamEncoder {
    pub fn record(&mut self, rec: &Value, out: &mut String) {
        let Some(o) = rec.as_object() else {
            if self.header.len() != 1 || self.header[0].0 != "_" {
                self.header = vec![("_".into(), ColType::Json)];
                out.push_str(&format!("@{ROOT} _:j\n"));
            }
            self.any = true;
            push_json_value(rec, out);
            out.push('\n');
            return;
        };
        let sig: Vec<(String, ColType)> = o
            .iter()
            .map(|(k, v)| {
                let ty = match v {
                    Value::Null => ColType::Json,
                    Value::Bool(_) => ColType::Bool,
                    Value::Number(Number::F64(_)) => ColType::Float,
                    Value::Number(_) => ColType::Int,
                    Value::String(_) => ColType::Str,
                    _ => ColType::Json,
                };
                (k.clone(), ty)
            })
            .collect();
        let compatible = self.header.len() == sig.len()
            && self.header.iter().zip(&sig).all(|((hk, ht), (k, t))| {
                hk == k && (ht == t || *ht == ColType::Json && matches!(t, ColType::Json))
            });
        if !compatible || !self.any {
            self.header = sig;
            out.push('@');
            out.push_str(ROOT);
            out.push(' ');
            for (i, (k, t)) in self.header.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                push_path(std::slice::from_ref(k), out);
                out.push(':');
                out.push(t.code());
                if *t == ColType::Json {
                    out.push('?');
                }
            }
            out.push('\n');
        }
        self.any = true;
        let mut tmp = Encoder::default();
        for (i, ((_, ty), (_, v))) in self.header.iter().zip(o.iter()).enumerate() {
            if i > 0 {
                tmp.out.push(',');
            }
            let col = Column {
                path: vec![],
                ty: *ty,
                nullable: true,
                child: None,
                hint: 0,
            };
            tmp.write_cell(&col, Some(v));
            if i == 0 {
                quote_line_start(&mut tmp.out, 0);
            }
        }
        out.push_str(&tmp.out);
        out.push('\n');
    }

    pub fn finish(&mut self, out: &mut String) {
        if !self.any {
            out.push_str(&format!("@{ROOT} _:j\n"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ldf_data::obj;

    #[test]
    fn spec_example() {
        let recs = vec![
            obj! {"order_id" => "ORD-000001", "customer" => obj! {"id" => 4654, "name" => "Val Thompson", "address" => obj! {"city" => "São Paulo"}},
            "items" => vec![obj! {"sku" => "E5", "qty" => 3, "price" => 28.78}, obj! {"sku" => "C3", "qty" => 4, "price" => 157.4}],
            "total" => 715.94, "status" => "shipped", "note" => "context compare"},
            obj! {"order_id" => "ORD-000002", "customer" => obj! {"id" => 50, "name" => "Sol Ritchie", "address" => obj! {"city" => "Buenos Aires"}},
            "items" => vec![obj! {"sku" => "A1", "qty" => 1, "price" => 2757.03}],
            "total" => 2757.03, "status" => "cancelled", "note" => Value::Null},
        ];
        assert_eq!(
            encode(&recs),
            "@records order_id:s,customer.id:i,customer.name:s,customer.address.city:s,items:t,total:f,status:s,note:s?\n\
             ORD-000001,4654,Val Thompson,São Paulo,2,715.94,shipped,context compare\n\
             ORD-000002,50,Sol Ritchie,Buenos Aires,1,2757.03,cancelled,null\n\
             @records.items order_id:s,sku:s,qty:i,price:f\n\
             ORD-000001,E5,3,28.78\n\
             ORD-000001,C3,4,157.4\n\
             ORD-000002,A1,1,2757.03\n"
        );
    }

    #[test]
    fn sparse_and_quoting() {
        let recs = vec![
            obj! {"id" => 1, "s" => "a,b", "n" => Value::Null, "e" => ""},
            obj! {"id" => 2, "s" => "007", "k" => "true"},
        ];
        // `n` is null in its only row (fill 0) and moves to the `~` object;
        // `e` and `k` are filled in half the rows and stay columns.
        assert_eq!(
            encode(&recs),
            "@records id:i,s:s,e:s?,k:s?,~:j?\n1,\"a,b\",\"\",,{\"n\":null}\n2,007,,\"true\",\n"
        );
        assert_eq!(
            encode_with(&recs, &Options::V0),
            "@records id:i,s:s,n:j?,e:s?,k:s?\n1,\"a,b\",null,\"\",\n2,007,,,\"true\"\n"
        );
    }

    #[test]
    fn sparse_keys_move_to_the_rest_object() {
        let recs = vec![
            obj! {"id" => 1, "t" => "a", "retries" => 2, "country" => "MX"},
            obj! {"id" => 2, "t" => "b"},
            obj! {"id" => 3, "t" => "c", "country" => Value::Null},
            obj! {"id" => 4, "t" => "d", "note" => "x"},
        ];
        let opts = Options {
            sparse_threshold: 0.5,
            ..Options::default()
        };
        let out = encode_with(&recs, &opts);
        assert_eq!(
            out,
            "@records id:i,t:s,~:j?\n1,a,{\"retries\":2,\"country\":\"MX\"}\n2,b,\n3,c,{\"country\":null}\n4,d,{\"note\":\"x\"}\n"
        );
        assert_eq!(encode(&recs), out, "0.5 is the default");
        assert_eq!(crate::decode(&out).unwrap(), recs);
        // Threshold off: the v0 layout, byte for byte.
        assert_eq!(
            encode_with(&recs, &Options::V0),
            "@records id:i,t:s,retries:i?,country:s?,note:s?\n1,a,2,MX,\n2,b,,,\n3,c,,null,\n4,d,,,x\n"
        );
    }

    #[test]
    fn block_encoder_matches_batch_up_to_the_join_column() {
        let recs: Vec<Value> = (1..=5)
            .map(|i| {
                obj! {"id" => i, "name" => format!("n{i}"),
                      "items" => (0..i % 3).map(|k| obj! {"sku" => format!("S{i}{k}"), "qty" => k}).collect::<Vec<_>>()}
            })
            .collect();
        let batch = encode(&recs);
        let mut out = String::new();
        let mut enc = BlockEncoder::new(2);
        for r in &recs {
            enc.record(r.clone(), &mut out);
        }
        enc.finish(&mut out);
        // Three blocks (2, 2, 1), each with child rows, so each block opens
        // with the root header again (rows belong to the most recent
        // header) and the ordinals keep counting: parents 1..5 across blocks.
        assert_eq!(out.matches("@records id:i").count(), 3, "{out}");
        assert_eq!(out.matches("@records.items _:i").count(), 3, "{out}");
        assert!(
            out.contains("\n5,S50,0\n"),
            "ordinal of the 5th parent: {out}"
        );
        assert_eq!(crate::decode(&out).unwrap(), recs);
        assert_eq!(crate::decode(&batch).unwrap(), recs);
        // The batch encoder joins on `id` (unique); only the join column differs.
        assert!(batch.contains("@records.items id:i"), "{batch}");
        // A block larger than the data gives one block: batch output with ordinals.
        let mut one = String::new();
        let mut enc = BlockEncoder::new(100);
        for r in &recs {
            enc.record(r.clone(), &mut one);
        }
        enc.finish(&mut one);
        assert_eq!(
            one,
            batch.replace("@records.items id:i", "@records.items _:i")
        );
    }

    #[test]
    fn block_encoder_reemits_the_root_header_when_the_plan_changes() {
        let mut out = String::new();
        let mut enc = BlockEncoder::new(2);
        enc.record(obj! {"a" => 1}, &mut out);
        enc.record(obj! {"a" => 2}, &mut out);
        enc.record(obj! {"a" => 3, "b" => "x"}, &mut out);
        enc.record(obj! {"a" => 4, "b" => "y"}, &mut out);
        enc.record(Value::from(7), &mut out);
        enc.finish(&mut out);
        assert_eq!(
            out,
            "@records a:i\n1\n2\n@records a:i,b:s\n3,x\n4,y\n@records _:j\n7\n"
        );
        let mut empty = String::new();
        BlockEncoder::new(3).finish(&mut empty);
        assert_eq!(empty, "@records _:j\n");
    }

    #[test]
    fn streaming_reemits_header_on_change() {
        let mut enc = StreamEncoder::default();
        let mut out = String::new();
        enc.record(&obj! {"a" => 1, "b" => "x"}, &mut out);
        enc.record(&obj! {"a" => 2, "b" => "y"}, &mut out);
        enc.record(&obj! {"a" => 3}, &mut out);
        enc.finish(&mut out);
        assert_eq!(out, "@records a:i,b:s\n1,x\n2,y\n@records a:i\n3\n");
    }
}
