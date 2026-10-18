//! Batch and streaming encoders.

use ldf_data::{Number, Object, Value};

use crate::ROOT;
use crate::plan::{ColType, Column, Join, Options, TablePlan, lookup, lookup_hint, plan_with};
use crate::text::{push_float, push_int, push_json_value, push_path, push_string_cell};

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
        self.write_table(&p, records);
    }

    fn write_table(&mut self, p: &TablePlan, records: &[Value]) {
        self.out.push_str(&p.header());
        self.out.push('\n');
        if p.scalar_rows {
            for r in records {
                push_json_value(r, &mut self.out);
                self.out.push('\n');
            }
            return;
        }
        let objs: Vec<&Object> = records
            .iter()
            .map(|r| r.as_object().expect("planned as objects"))
            .collect();
        for o in &objs {
            self.write_row(p, o, None);
        }
        self.write_children(p, &objs, None);
    }

    /// Writes one row. `join` is the join cell text for child rows.
    fn write_row(&mut self, p: &TablePlan, o: &Object, join: Option<&str>) {
        let mut first = true;
        if let Some(j) = join {
            self.out.push_str(j);
            first = false;
        }
        for c in &p.columns {
            if !first {
                self.out.push(',');
            }
            first = false;
            self.write_cell(c, lookup_hint(o, &c.path, c.hint));
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

    /// Child tables of `p`, in column order, each followed by its own children.
    fn write_children(
        &mut self,
        p: &TablePlan,
        parents: &[&Object],
        parent_join: Option<&[String]>,
    ) {
        let _ = parent_join;
        for c in &p.columns {
            let Some(child) = &c.child else { continue };
            self.out.push_str(&child.header());
            self.out.push('\n');
            let mut elements: Vec<&Object> = Vec::new();
            let mut joins: Vec<String> = Vec::new();
            for (idx, parent) in parents.iter().enumerate() {
                let Some(Value::Array(items)) = lookup_hint(parent, &c.path, c.hint) else {
                    continue;
                };
                let join_text = match child.join {
                    Join::Ordinal => (idx + 1).to_string(),
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
                    joins.push(join_text.clone());
                }
            }
            self.write_children(child, &elements, None);
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
