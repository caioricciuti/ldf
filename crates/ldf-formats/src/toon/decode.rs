//! TOON decoder (spec v4.4 sections 4 to 14), strict by default with the
//! five non-strict recoveries of section 14.4.

use ldf_data::{Object, Value};

use super::DecodeOptions;
use super::encode::FieldNode;
use super::text::{
    decode_primitive, find_closing_quote, find_unquoted, split_unquoted, trim_spaces, unescape,
};

#[derive(Debug, Clone)]
pub struct Error {
    pub line: usize,
    pub message: String,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}
impl std::error::Error for Error {}

type Result<T> = std::result::Result<T, Error>;

fn err<T>(line: usize, message: impl Into<String>) -> Result<T> {
    Err(Error {
        line,
        message: message.into(),
    })
}

struct Line {
    no: usize,
    depth: usize,
    text: String,
    blank: bool,
}

struct Header {
    key: Option<String>,
    /// Declared count; `None` when it does not fit in usize (count unmet).
    n: Option<usize>,
    keyed: bool,
    delim: char,
    fields: Option<Vec<FieldNode>>,
    rest: String,
}

pub fn decode(text: &str, opts: &DecodeOptions) -> Result<Value> {
    let lines = prepare_lines(text, opts)?;
    let mut p = Parser {
        lines,
        pos: 0,
        strict: opts.strict,
    };
    p.document()
}

// ---- line preparation (spec 12, 5.1) --------------------------------------

fn prepare_lines(text: &str, opts: &DecodeOptions) -> Result<Vec<Line>> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let mut out = Vec::new();
    for (i, raw) in text.split('\n').enumerate() {
        let no = i + 1;
        let raw = raw.strip_suffix('\r').unwrap_or(raw);
        let raw = raw.trim_end_matches(' ');
        // Comment: first char after zero or more spaces is '#'.
        let after_spaces = raw.trim_start_matches(' ');
        if after_spaces.starts_with('#') {
            continue;
        }
        let indent_len = raw.len() - raw.trim_start_matches([' ', '\t']).len();
        let (indent, content) = raw.split_at(indent_len);
        if content.is_empty() {
            if opts.strict && indent.contains('\t') {
                return err(no, "tab in indentation");
            }
            out.push(Line {
                no,
                depth: 0,
                text: String::new(),
                blank: true,
            });
            continue;
        }
        let spaces = indent.bytes().filter(|&b| b == b' ').count();
        let tabs = indent.len() - spaces;
        let depth = if opts.strict {
            if tabs > 0 {
                return err(no, "tab in indentation");
            }
            if spaces % opts.indent != 0 {
                return err(
                    no,
                    format!(
                        "indentation of {spaces} spaces is not a multiple of {}",
                        opts.indent
                    ),
                );
            }
            spaces / opts.indent
        } else {
            tabs + spaces / opts.indent
        };
        out.push(Line {
            no,
            depth,
            text: content.to_owned(),
            blank: false,
        });
    }
    Ok(out)
}

// ---- header syntax (spec 6) -----------------------------------------------

/// Returns `Ok(None)` when the content is not an array-header line.
fn parse_header(content: &str, no: usize, strict: bool) -> Result<Option<Header>> {
    let Some(colon) = find_unquoted(content, |c| c == ':') else {
        return Ok(None);
    };
    let Some(lb) = find_unquoted(content, |c| c == '[') else {
        return Ok(None);
    };
    if lb > colon {
        return Ok(None);
    }
    let key_part = &content[..lb];
    let key = if key_part.is_empty() {
        None
    } else {
        if key_part.ends_with([' ', '\t']) {
            return err(no, "whitespace between key and bracket segment");
        }
        Some(decode_key(key_part, no)?)
    };
    let b = content.as_bytes();
    let mut i = lb + 1;
    let digits_start = i;
    while i < b.len() && b[i].is_ascii_digit() {
        i += 1;
    }
    let digits = &content[digits_start..i];
    if digits.is_empty() || (digits.len() > 1 && digits.starts_with('0')) {
        return err(no, format!("malformed array length in header {content:?}"));
    }
    let n = digits.parse::<usize>().ok();
    let mut keyed = false;
    if i < b.len() && b[i] == b':' {
        keyed = true;
        i += 1;
    }
    let mut delim = ',';
    if i < b.len() && (b[i] == b'\t' || b[i] == b'|') {
        delim = b[i] as char;
        i += 1;
    }
    if i >= b.len() || b[i] != b']' {
        return err(
            no,
            format!("malformed bracket segment in header {content:?}"),
        );
    }
    i += 1;
    let mut fields = None;
    if i < b.len() && b[i] == b'{' {
        let close = matching_brace(content, i).ok_or_else(|| Error {
            line: no,
            message: format!("unmatched brace in header {content:?}"),
        })?;
        let inner = &content[i + 1..close];
        fields = Some(parse_fields(inner, delim, no, strict)?);
        i = close + 1;
    }
    if i >= b.len() || b[i] != b':' {
        return err(
            no,
            format!("header {content:?} must end its bracket/field segment with a colon"),
        );
    }
    if keyed && fields.is_none() {
        return err(no, format!("keyed header {content:?} needs a field list"));
    }
    let rest = trim_spaces(&content[i + 1..]).to_owned();
    if fields.is_some() && !rest.is_empty() {
        return err(
            no,
            format!("content after a fields-bearing header: {rest:?}"),
        );
    }
    Ok(Some(Header {
        key,
        n,
        keyed,
        delim,
        fields,
        rest,
    }))
}

/// Index of the `}` matching the `{` at `open`, ignoring quoted spans.
fn matching_brace(s: &str, open: usize) -> Option<usize> {
    let b = s.as_bytes();
    let mut depth = 0i32;
    let mut i = open;
    while i < b.len() {
        match b[i] {
            b'"' => {
                let end = find_closing_quote(&s[i + 1..])?;
                i += 1 + end;
            }
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

fn parse_fields(inner: &str, delim: char, no: usize, strict: bool) -> Result<Vec<FieldNode>> {
    // Split on the delimiter at brace depth 0, outside quotes.
    let mut entries: Vec<&str> = Vec::new();
    let b = inner.as_bytes();
    let (mut start, mut depth, mut i) = (0usize, 0i32, 0usize);
    while i < b.len() {
        match b[i] {
            b'"' => {
                let end = find_closing_quote(&inner[i + 1..]).ok_or_else(|| Error {
                    line: no,
                    message: "unterminated quoted field name".into(),
                })?;
                i += 1 + end;
            }
            b'{' => depth += 1,
            b'}' => depth -= 1,
            c if depth == 0 && c as char == delim => {
                entries.push(&inner[start..i]);
                start = i + 1;
            }
            c if depth == 0 && (c == b',' || c == b'|' || c == b'\t') && c as char != delim => {
                return err(
                    no,
                    format!(
                        "field list uses delimiter {:?} but header declares {delim:?}",
                        c as char
                    ),
                );
            }
            _ => {}
        }
        i += 1;
    }
    entries.push(&inner[start..]);
    let mut nodes = Vec::with_capacity(entries.len());
    for entry in entries {
        let entry = trim_spaces(entry);
        if entry.is_empty() {
            return err(no, "empty field entry in header");
        }
        let (name_tok, children) = match find_unquoted(entry, |c| c == '{') {
            Some(ob) => {
                let close = matching_brace(entry, ob).ok_or_else(|| Error {
                    line: no,
                    message: "unmatched brace in nested field group".into(),
                })?;
                if close + 1 != entry.len() {
                    return err(
                        no,
                        format!("unexpected content after nested field group in {entry:?}"),
                    );
                }
                let name = &entry[..ob];
                if name.is_empty() {
                    return err(no, "nameless nested field group");
                }
                if name.ends_with([' ', '\t']) {
                    return err(no, "whitespace between field name and nested field group");
                }
                let sub = parse_fields(&entry[ob + 1..close], delim, no, strict)?;
                (name, Some(sub))
            }
            None => (entry, None),
        };
        let name = decode_key(name_tok, no)?;
        if strict && nodes.iter().any(|n: &FieldNode| n.name == name) {
            return err(no, format!("duplicate field name {name:?} in header"));
        }
        nodes.push(FieldNode { name, children });
    }
    Ok(nodes)
}

fn decode_key(tok: &str, no: usize) -> Result<String> {
    if let Some(rest) = tok.strip_prefix('"') {
        let end = find_closing_quote(rest).ok_or_else(|| Error {
            line: no,
            message: "unterminated quoted key".into(),
        })?;
        if end + 1 != rest.len() {
            return err(no, "unexpected characters after quoted key");
        }
        return unescape(&rest[..end]).map_err(|m| Error {
            line: no,
            message: m,
        });
    }
    Ok(tok.to_owned())
}

fn primitive(tok: &str, no: usize) -> Result<Value> {
    decode_primitive(tok).map_err(|m| Error {
        line: no,
        message: m,
    })
}

fn is_list_item(text: &str) -> bool {
    text == "-" || text.starts_with("- ")
}

// ---- parser -----------------------------------------------------------------

struct Parser {
    lines: Vec<Line>,
    pos: usize,
    strict: bool,
}

impl Parser {
    fn peek(&self) -> Option<&Line> {
        self.lines[self.pos..].iter().find(|l| !l.blank)
    }

    /// Index of the next non-blank line, and whether blanks precede it.
    fn peek_idx(&self) -> Option<(usize, bool)> {
        let mut blanks = false;
        for (i, l) in self.lines.iter().enumerate().skip(self.pos) {
            if l.blank {
                blanks = true;
            } else {
                return Some((i, blanks));
            }
        }
        None
    }

    /// Advances past the next non-blank line and returns it. Inside a header
    /// span (`span`), blank lines before a continuing line are an error in
    /// strict mode (spec 12).
    fn take(&mut self, span: bool) -> Result<&Line> {
        let (i, blanks) = self.peek_idx().expect("caller peeked");
        if span && blanks && self.strict {
            return err(self.lines[i].no, "blank line inside a header span");
        }
        self.pos = i + 1;
        Ok(&self.lines[i])
    }

    fn insert(obj: &mut Object, key: String, value: Value, no: usize, strict: bool) -> Result<()> {
        if let Some(slot) = obj.iter_mut().find(|(k, _)| *k == key) {
            if strict {
                return err(no, format!("duplicate key {key:?}"));
            }
            slot.1 = value;
        } else {
            obj.push((key, value));
        }
        Ok(())
    }

    fn document(&mut self) -> Result<Value> {
        let Some(first) = self.peek() else {
            return Ok(Value::Object(Vec::new()));
        };
        let (no, depth, text) = (first.no, first.depth, first.text.clone());
        if depth != 0 {
            return err(no, "first line must be at depth 0");
        }
        let total_nonblank = self.lines.iter().filter(|l| !l.blank).count();
        if text == "[]" {
            self.take(false)?;
            self.ensure_no_trailing()?;
            return Ok(Value::Array(Vec::new()));
        }
        if let Some(h) = parse_header(&text, no, self.strict)? {
            if h.key.is_none() {
                self.take(false)?;
                let v = self.array_or_keyed(h, no, 1, true)?;
                self.ensure_no_trailing()?;
                return Ok(v);
            }
        } else if find_unquoted(&text, |c| c == ':').is_none() {
            if total_nonblank == 1 {
                self.take(false)?;
                return primitive(&text, no);
            }
            return err(
                no,
                format!("scalar line {text:?} outside root primitive position"),
            );
        }
        let obj = self.object_body(0, false)?;
        if let Some(l) = self.peek() {
            return err(l.no, "unexpected trailing content");
        }
        Ok(Value::Object(obj))
    }

    fn ensure_no_trailing(&self) -> Result<()> {
        match self.peek() {
            Some(l) => err(l.no, "trailing content after a completed root form"),
            None => Ok(()),
        }
    }

    /// Fields of an object whose content depth is `depth`.
    fn object_body(&mut self, depth: usize, span: bool) -> Result<Object> {
        let mut obj = Object::new();
        while let Some(l) = self.peek() {
            if l.depth < depth {
                break;
            }
            if l.depth > depth {
                return err(l.no, "over-indented line");
            }
            let l = self.take(span)?;
            let (no, text) = (l.no, l.text.clone());
            let (k, v) = self.field_line(&text, no, depth, depth + 1, span)?;
            Parser::insert(&mut obj, k, v, no, self.strict)?;
        }
        Ok(obj)
    }

    /// Parses one key-value or header line. `key_depth` is where sibling
    /// fields live, `scope` where nested content lives.
    fn field_line(
        &mut self,
        text: &str,
        no: usize,
        key_depth: usize,
        scope: usize,
        span: bool,
    ) -> Result<(String, Value)> {
        if let Some(h) = parse_header(text, no, self.strict)? {
            let Some(key) = h.key.clone() else {
                return err(no, "keyless header in object-field position");
            };
            let v = self.array_or_keyed(h, no, scope, span)?;
            return Ok((key, v));
        }
        let Some(ci) = find_unquoted(text, |c| c == ':') else {
            return err(
                no,
                format!("expected key: value, found scalar line {text:?}"),
            );
        };
        let key = decode_key(trim_spaces(&text[..ci]), no)?;
        let rest = trim_spaces(&text[ci + 1..]);
        if rest.is_empty() {
            let v = match self.peek() {
                Some(l) if l.depth > key_depth => {
                    let c = self.adopt_depth(scope)?;
                    Value::Object(self.object_body(c, span)?)
                }
                _ => Value::Object(Vec::new()),
            };
            return Ok((key, v));
        }
        if rest == "[]" {
            return Ok((key, Value::Array(Vec::new())));
        }
        Ok((key, primitive(rest, no)?))
    }

    /// Content depth of a nested scope expected at `c`: strict errors on a
    /// depth jump, non-strict adopts the deeper depth (spec 14.4 rule 5).
    fn adopt_depth(&mut self, c: usize) -> Result<usize> {
        match self.peek() {
            Some(l) if l.depth > c => {
                if self.strict {
                    err(
                        l.no,
                        format!("depth jump: expected depth {c}, found {}", l.depth),
                    )
                } else {
                    Ok(l.depth)
                }
            }
            _ => Ok(c),
        }
    }

    fn check_count(
        &self,
        declared: Option<usize>,
        actual: usize,
        no: usize,
        what: &str,
    ) -> Result<()> {
        if !self.strict {
            return Ok(());
        }
        match declared {
            Some(n) if n == actual => Ok(()),
            Some(n) => err(no, format!("declared {n} {what}, found {actual}")),
            None => err(no, "declared length exceeds the representable range"),
        }
    }

    fn array_or_keyed(&mut self, h: Header, no: usize, scope: usize, span: bool) -> Result<Value> {
        if h.keyed {
            return self.keyed_rows(h, no, scope, span);
        }
        if h.fields.is_some() {
            return self.rows(h, no, scope, span);
        }
        if !h.rest.is_empty() {
            let toks = split_unquoted(&h.rest, h.delim);
            let mut vals = Vec::with_capacity(toks.len());
            for t in toks {
                vals.push(primitive(t, no)?);
            }
            self.check_count(h.n, vals.len(), no, "inline values")?;
            return Ok(Value::Array(vals));
        }
        self.list_items(h, no, scope, span)
    }

    fn rows(&mut self, h: Header, no: usize, scope: usize, span: bool) -> Result<Value> {
        let fields = h.fields.as_ref().unwrap();
        let leaves = FieldNode::leaf_count(fields);
        let depth = self.adopt_depth(scope)?;
        let mut rows = Vec::new();
        while let Some(l) = self.peek() {
            if l.depth < depth {
                break;
            }
            if l.depth > depth {
                return err(l.no, "over-indented line in tabular rows");
            }
            if !is_row(&l.text, h.delim) {
                break;
            }
            let l = self.take(span || !rows.is_empty())?;
            let (rno, text) = (l.no, l.text.clone());
            rows.push(self.row_object(&text, rno, fields, leaves, h.delim)?);
        }
        self.check_count(h.n, rows.len(), no, "rows")?;
        Ok(Value::Array(rows))
    }

    fn row_object(
        &self,
        text: &str,
        no: usize,
        fields: &[FieldNode],
        leaves: usize,
        delim: char,
    ) -> Result<Value> {
        let cells = split_unquoted(text, delim);
        if cells.len() != leaves {
            return err(
                no,
                format!("row has {} cells, header declares {leaves}", cells.len()),
            );
        }
        let mut idx = 0;
        self.build_row(fields, &cells, &mut idx, no)
    }

    fn build_row(
        &self,
        fields: &[FieldNode],
        cells: &[&str],
        idx: &mut usize,
        no: usize,
    ) -> Result<Value> {
        let mut obj = Object::with_capacity(fields.len());
        for f in fields {
            let v = match &f.children {
                Some(children) => self.build_row(children, cells, idx, no)?,
                None => {
                    let v = primitive(cells[*idx], no)?;
                    *idx += 1;
                    v
                }
            };
            Parser::insert(&mut obj, f.name.clone(), v, no, false)?;
        }
        Ok(Value::Object(obj))
    }

    fn keyed_rows(&mut self, h: Header, no: usize, scope: usize, span: bool) -> Result<Value> {
        let fields = h.fields.as_ref().unwrap();
        let leaves = FieldNode::leaf_count(fields);
        let depth = self.adopt_depth(scope)?;
        let mut obj = Object::new();
        let mut count = 0;
        while let Some(l) = self.peek() {
            if l.depth < depth {
                break;
            }
            if l.depth > depth {
                return err(l.no, "over-indented line in keyed entry rows");
            }
            let l = self.take(span || count > 0)?;
            let (rno, text) = (l.no, l.text.clone());
            let Some(ci) = find_unquoted(&text, |c| c == ':') else {
                return err(rno, "entry row without a colon");
            };
            let key = decode_key(trim_spaces(&text[..ci]), rno)?;
            let rest = trim_spaces(&text[ci + 1..]);
            let value = if rest.is_empty() {
                return err(
                    rno,
                    format!("entry row has 0 cells, header declares {leaves}"),
                );
            } else {
                self.row_object(rest, rno, fields, leaves, h.delim)?
            };
            Parser::insert(&mut obj, key, value, rno, self.strict)?;
            count += 1;
        }
        self.check_count(h.n, count, no, "entry rows")?;
        Ok(Value::Object(obj))
    }

    fn list_items(&mut self, h: Header, no: usize, scope: usize, span: bool) -> Result<Value> {
        let depth = self.adopt_depth(scope)?;
        let mut items = Vec::new();
        while let Some(l) = self.peek() {
            if l.depth < depth {
                break;
            }
            if l.depth > depth {
                return err(l.no, "over-indented line in list items");
            }
            if !is_list_item(&l.text) {
                break;
            }
            let l = self.take(span || !items.is_empty())?;
            let (ino, text) = (l.no, l.text.clone());
            items.push(self.list_item(&text, ino, depth)?);
        }
        self.check_count(h.n, items.len(), no, "list items")?;
        Ok(Value::Array(items))
    }

    /// One list item whose hyphen stands at `depth`.
    fn list_item(&mut self, text: &str, no: usize, depth: usize) -> Result<Value> {
        let r = if text == "-" {
            ""
        } else {
            text[1..].trim_start_matches(' ')
        };
        if r.is_empty() {
            return Ok(Value::Object(Vec::new()));
        }
        if r == "[]" {
            return Ok(Value::Array(Vec::new()));
        }
        if let Some(h) = parse_header(r, no, self.strict)? {
            return match h.key.clone() {
                None => {
                    if h.fields.is_some() {
                        return err(
                            no,
                            "keyless fields-bearing header outside the document root",
                        );
                    }
                    // Keyless header: the item itself is an array with its
                    // items at depth + 1.
                    self.array_or_keyed(h, no, depth + 1, true)
                }
                Some(key) => {
                    let first = self.array_or_keyed(h, no, depth + 2, true)?;
                    self.list_item_object(key, first, no, depth)
                }
            };
        }
        if find_unquoted(r, |c| c == ':').is_some() {
            let (key, first) = self.field_line(r, no, depth + 1, depth + 2, true)?;
            return self.list_item_object(key, first, no, depth);
        }
        primitive(r, no)
    }

    fn list_item_object(
        &mut self,
        key: String,
        first: Value,
        no: usize,
        depth: usize,
    ) -> Result<Value> {
        let mut obj = Object::new();
        obj.push((key, first));
        let rest = self.object_body(depth + 1, true)?;
        for (k, v) in rest {
            Parser::insert(&mut obj, k, v, no, self.strict)?;
        }
        Ok(Value::Object(obj))
    }
}

/// Spec 9.3 row disambiguation.
fn is_row(text: &str, delim: char) -> bool {
    let colon = find_unquoted(text, |c| c == ':');
    let d = find_unquoted(text, |c| c == delim);
    match (colon, d) {
        (None, _) => true,
        (Some(c), Some(d)) => d < c,
        (Some(_), None) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::super::{DecodeOptions, decode_value};

    fn d(s: &str) -> String {
        decode_value(s, &DecodeOptions::default())
            .unwrap()
            .to_string()
    }

    #[test]
    fn basics() {
        assert_eq!(
            d("a: 1\nb: x\nc:\n  d: true"),
            r#"{"a":1,"b":"x","c":{"d":true}}"#
        );
        assert_eq!(
            d("[2]{id,n{x}}:\n  1,a\n  2,b"),
            r#"[{"id":1,"n":{"x":"a"}},{"id":2,"n":{"x":"b"}}]"#
        );
        assert_eq!(
            d("items[3]:\n  - 1\n  - a: 1\n    b: 2\n  - [2]: x,y"),
            r#"{"items":[1,{"a":1,"b":2},["x","y"]]}"#
        );
        assert_eq!(
            d("m[2:]{v}:\n  a: 1\n  b: 2"),
            r#"{"m":{"a":{"v":1},"b":{"v":2}}}"#
        );
        assert_eq!(d("\"a:b\"[2|]: x|y,z"), r#"{"a:b":["x","y,z"]}"#);
        assert_eq!(d("hello"), r#""hello""#);
        assert_eq!(d(""), "{}");
        assert_eq!(d("[]"), "[]");
    }

    #[test]
    fn strict_errors() {
        for bad in [
            "tags[3]: a,b",
            "a: 1\n  b: 2",
            "a[2]{x}: 1",
            "a:\n    b: 1",
            "a: 1\na: 2",
            "[2]{a,a}:\n  1,2\n  3,4",
        ] {
            assert!(
                decode_value(bad, &DecodeOptions::default()).is_err(),
                "{bad:?}"
            );
        }
    }
}
