//! TOON encoder (spec v4.4 sections 2, 3, 5 to 12).

use ldf_data::{Object, Value};

use super::EncodeOptions;
use super::text::{push_key, push_primitive};

/// A column header tree for tabular and keyed tabular forms (spec 9.3).
#[derive(Debug, Clone)]
pub struct FieldNode {
    pub name: String,
    pub children: Option<Vec<FieldNode>>,
}

impl FieldNode {
    pub fn leaf_count(nodes: &[FieldNode]) -> usize {
        nodes
            .iter()
            .map(|n| match &n.children {
                Some(c) => FieldNode::leaf_count(c),
                None => 1,
            })
            .sum()
    }
}

/// Where the head text of a field goes: a fresh line at `depth`, or appended
/// to the current line (the `- ` of a list item, spec 10).
#[derive(Clone, Copy)]
enum Head {
    Line(usize),
    Append,
}

pub struct Encoder {
    opts: EncodeOptions,
    delim: char,
    out: String,
}

impl Encoder {
    pub fn new(opts: EncodeOptions) -> Self {
        Encoder {
            opts,
            delim: opts.delimiter.ch(),
            out: String::new(),
        }
    }

    pub fn finish(self) -> String {
        self.out
    }

    pub fn encode_document(mut self, v: &Value) -> String {
        match v {
            Value::Object(o) => {
                if let Some(tree) = keyed_tabular_tree(o) {
                    self.keyed_header(Head::Line(0), None, o.len(), &tree);
                    self.entry_rows(o, &tree, 1);
                } else {
                    self.fields(o, 0);
                }
            }
            Value::Array(a) => self.encode_root_array(a),
            prim => {
                let mut line = String::new();
                if let Value::String(s) = prim
                    && s.starts_with('\u{feff}')
                {
                    super::text::push_quoted(s, &mut line);
                } else {
                    push_primitive(prim, self.delim, &mut line);
                }
                self.line(0, &line);
            }
        }
        self.out
    }

    /// Root array: `[N]: ...`, `[N]{...}:` + rows, or `[N]:` + items.
    pub fn encode_root_array(&mut self, items: &[Value]) {
        if items.is_empty() {
            self.line(0, "[]");
            return;
        }
        self.array(Head::Line(0), None, items, 1);
    }

    // ---- line plumbing ---------------------------------------------------

    fn line(&mut self, depth: usize, text: &str) {
        if !self.out.is_empty() {
            self.out.push('\n');
        }
        for _ in 0..depth * self.opts.indent {
            self.out.push(' ');
        }
        self.out.push_str(text);
    }

    fn emit(&mut self, head: Head, text: &str) {
        match head {
            Head::Line(d) => self.line(d, text),
            Head::Append => self.out.push_str(text),
        }
    }

    // ---- objects ---------------------------------------------------------

    fn fields(&mut self, o: &Object, depth: usize) {
        for (k, v) in o {
            self.field(Head::Line(depth), k, v, depth + 1);
        }
    }

    /// One object field. `scope` is the depth of any nested content.
    fn field(&mut self, head: Head, key: &str, v: &Value, scope: usize) {
        let mut text = String::new();
        push_key(key, &mut text);
        match v {
            Value::Object(o) => {
                if let Some(tree) = keyed_tabular_tree(o) {
                    self.keyed_header(head, Some(key), o.len(), &tree);
                    self.entry_rows(o, &tree, scope);
                } else {
                    text.push(':');
                    self.emit(head, &text);
                    self.fields(o, scope);
                }
            }
            Value::Array(a) => {
                if a.is_empty() {
                    text.push_str(": []");
                    self.emit(head, &text);
                } else {
                    self.array(head, Some(key), a, scope);
                }
            }
            prim => {
                text.push_str(": ");
                push_primitive(prim, self.delim, &mut text);
                self.emit(head, &text);
            }
        }
    }

    // ---- arrays ----------------------------------------------------------

    /// Non-empty array under an optional key; content at `scope`.
    fn array(&mut self, head: Head, key: Option<&str>, items: &[Value], scope: usize) {
        if items.iter().all(Value::is_primitive) {
            let mut text = String::new();
            self.header_prefix(key, items.len(), &mut text);
            text.push_str("]: ");
            self.join_primitives(items, &mut text);
            self.emit(head, &text);
            return;
        }
        if let Some(tree) = tabular_tree(items) {
            let mut text = String::new();
            self.header_prefix(key, items.len(), &mut text);
            text.push(']');
            self.fields_segment(&tree, &mut text);
            text.push(':');
            self.emit(head, &text);
            for item in items {
                let mut row = String::new();
                self.row_cells(item.as_object().unwrap(), &tree, &mut row);
                self.line(scope, &row);
            }
            return;
        }
        let mut text = String::new();
        self.header_prefix(key, items.len(), &mut text);
        text.push_str("]:");
        self.emit(head, &text);
        self.list_items(items, scope);
    }

    /// `key[N` or `[N` plus the delimiter symbol (the caller closes it).
    fn header_prefix(&self, key: Option<&str>, n: usize, text: &mut String) {
        if let Some(k) = key {
            push_key(k, text);
        }
        text.push('[');
        text.push_str(&n.to_string());
        text.push_str(self.opts.delimiter.header_symbol());
    }

    fn fields_segment(&self, tree: &[FieldNode], text: &mut String) {
        text.push('{');
        for (i, node) in tree.iter().enumerate() {
            if i > 0 {
                text.push(self.delim);
            }
            push_key(&node.name, text);
            if let Some(children) = &node.children {
                self.fields_segment(children, text);
            }
        }
        text.push('}');
    }

    fn join_primitives(&self, items: &[Value], text: &mut String) {
        for (i, v) in items.iter().enumerate() {
            if i > 0 {
                text.push(self.delim);
            }
            push_primitive(v, self.delim, text);
        }
    }

    fn row_cells(&self, o: &Object, tree: &[FieldNode], row: &mut String) {
        fn walk(enc: &Encoder, o: &Object, tree: &[FieldNode], row: &mut String, first: &mut bool) {
            for (i, node) in tree.iter().enumerate() {
                // Fast path: most objects list keys in header order.
                let v = match o.get(i) {
                    Some((k, v)) if *k == node.name => v,
                    _ => lookup(o, &node.name).expect("tabular detection guarantees the key"),
                };
                match &node.children {
                    Some(children) => walk(enc, v.as_object().unwrap(), children, row, first),
                    None => {
                        if !*first {
                            row.push(enc.delim);
                        }
                        *first = false;
                        push_primitive(v, enc.delim, row);
                    }
                }
            }
        }
        let mut first = true;
        walk(self, o, tree, row, &mut first);
    }

    fn list_items(&mut self, items: &[Value], depth: usize) {
        for item in items {
            match item {
                Value::Array(a) if a.is_empty() => {
                    let mut text = String::from("- [0");
                    text.push_str(self.opts.delimiter.header_symbol());
                    text.push_str("]:");
                    self.line(depth, &text);
                }
                Value::Array(a) if a.iter().all(Value::is_primitive) => {
                    let mut text = String::from("- ");
                    self.header_prefix(None, a.len(), &mut text);
                    text.push_str("]: ");
                    self.join_primitives(a, &mut text);
                    self.line(depth, &text);
                }
                Value::Array(a) => {
                    // Tabular form is not available in list-item position.
                    let mut text = String::from("- ");
                    self.header_prefix(None, a.len(), &mut text);
                    text.push_str("]:");
                    self.line(depth, &text);
                    self.list_items(a, depth + 1);
                }
                Value::Object(o) if o.is_empty() => self.line(depth, "-"),
                Value::Object(o) => {
                    self.line(depth, "- ");
                    let (k0, v0) = &o[0];
                    self.field(Head::Append, k0, v0, depth + 2);
                    for (k, v) in &o[1..] {
                        self.field(Head::Line(depth + 1), k, v, depth + 2);
                    }
                }
                prim => {
                    let mut text = String::from("- ");
                    push_primitive(prim, self.delim, &mut text);
                    self.line(depth, &text);
                }
            }
        }
    }

    // ---- keyed tabular (9.5) ---------------------------------------------

    fn keyed_header(&mut self, head: Head, key: Option<&str>, n: usize, tree: &[FieldNode]) {
        let mut text = String::new();
        if let Some(k) = key {
            push_key(k, &mut text);
        }
        text.push('[');
        text.push_str(&n.to_string());
        text.push(':');
        text.push_str(self.opts.delimiter.header_symbol());
        text.push(']');
        self.fields_segment(tree, &mut text);
        text.push(':');
        self.emit(head, &text);
    }

    fn entry_rows(&mut self, o: &Object, tree: &[FieldNode], depth: usize) {
        for (k, v) in o {
            let mut row = String::new();
            push_key(k, &mut row);
            row.push_str(": ");
            self.row_cells(v.as_object().unwrap(), tree, &mut row);
            self.line(depth, &row);
        }
    }
}

fn lookup<'a>(o: &'a Object, key: &str) -> Option<&'a Value> {
    o.iter().find(|(k, _)| k == key).map(|(_, v)| v)
}

/// Spec 9.3 tabular detection over array elements.
pub fn tabular_tree(items: &[Value]) -> Option<Vec<FieldNode>> {
    let objs: Vec<&Object> = items.iter().map(Value::as_object).collect::<Option<_>>()?;
    field_tree(&objs)
}

/// Spec 9.5 keyed tabular detection over an object's entry values.
pub fn keyed_tabular_tree(o: &Object) -> Option<Vec<FieldNode>> {
    if o.len() < 2 {
        return None;
    }
    let objs: Vec<&Object> = o
        .iter()
        .map(|(_, v)| v.as_object())
        .collect::<Option<_>>()?;
    field_tree(&objs)
}

/// Shared column classification: all objects non-empty with the same key
/// set, every column uniform-primitive or nested-uniform (recursively).
fn field_tree(objs: &[&Object]) -> Option<Vec<FieldNode>> {
    let first = objs.first()?;
    if first.is_empty() {
        return None;
    }
    if objs.iter().any(|o| o.len() != first.len()) {
        return None;
    }
    let mut tree = Vec::with_capacity(first.len());
    for (i, (key, _)) in first.iter().enumerate() {
        let mut column: Vec<&Value> = Vec::with_capacity(objs.len());
        for o in objs {
            // Fast path: same key at the same position; fall back to a scan.
            let v = match o.get(i) {
                Some((k, v)) if k == key => v,
                _ => lookup(o, key)?,
            };
            column.push(v);
        }
        if column.iter().all(|v| v.is_primitive()) {
            tree.push(FieldNode {
                name: key.clone(),
                children: None,
            });
            continue;
        }
        let sub: Vec<&Object> = column
            .iter()
            .map(|v| v.as_object().filter(|o| !o.is_empty()))
            .collect::<Option<_>>()?;
        let children = field_tree(&sub)?;
        tree.push(FieldNode {
            name: key.clone(),
            children: Some(children),
        });
    }
    Some(tree)
}

#[cfg(test)]
mod tests {
    use super::super::{Delimiter, EncodeOptions, encode_value};
    use ldf_data::{Value, obj};

    #[test]
    fn spec_example_shapes() {
        let v = obj! {
            "items" => vec![obj! {"sku" => "A1", "qty" => 2, "price" => 9.99}, obj! {"sku" => "B2", "qty" => 1, "price" => 14.5}]
        };
        assert_eq!(
            encode_value(&v, &EncodeOptions::default()),
            "items[2]{sku,qty,price}:\n  A1,2,9.99\n  B2,1,14.5"
        );
        let tags = || vec![Value::from("a"), Value::from("b,c")];
        let v = obj! {"tags" => tags(), "n" => obj! {}, "e" => Vec::<Value>::new()};
        assert_eq!(
            encode_value(&v, &EncodeOptions::default()),
            "tags[2]: a,\"b,c\"\nn:\ne: []"
        );
        let v = obj! {"tags" => tags()};
        assert_eq!(
            encode_value(
                &v,
                &EncodeOptions {
                    indent: 2,
                    delimiter: Delimiter::Pipe
                }
            ),
            "tags[2|]: a|b,c"
        );
    }

    #[test]
    fn list_items_and_keyed_tabular() {
        let v = obj! {
            "mixed" => vec![Value::from(1), obj! {"a" => 1, "b" => obj! {"c" => 2}}, obj! {}, Value::Array(vec![Value::from(1), Value::from(2)])],
            "users" => obj! {"ann" => obj! {"age" => 30}, "bob" => obj! {"age" => 41}}
        };
        assert_eq!(
            encode_value(&v, &EncodeOptions::default()),
            "mixed[4]:\n  - 1\n  - a: 1\n    b:\n      c: 2\n  -\n  - [2]: 1,2\nusers[2:]{age}:\n  ann: 30\n  bob: 41"
        );
    }
}
