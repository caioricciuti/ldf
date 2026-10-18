//! The search space. Every layout choice of a table encoding is one knob;
//! a [`Config`] picks one value per knob and is a complete, decodable
//! format. [`Config::LDF_V01`] and [`Config::LDF_V0`] are the released
//! layouts as points in the space.

use std::fmt::Write as _;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Delim {
    Comma,
    Tab,
    Pipe,
}

impl Delim {
    pub fn byte(self) -> u8 {
        match self {
            Delim::Comma => b',',
            Delim::Tab => b'\t',
            Delim::Pipe => b'|',
        }
    }
    pub fn char(self) -> char {
        self.byte() as char
    }
}

/// Scalar types in the header (`id:i`) or not (`id`, cells say what they are).
/// Structural types (`t`, hoisted key lists, the `~` column) are always written.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Typing {
    Letters,
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Objects {
    /// Uniform objects flatten into dotted columns.
    Dotted,
    /// Objects stay compact JSON cells.
    Json,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Arrays {
    /// Child table joined by the parent's first column when unique, else by ordinal.
    ChildKey,
    /// Child table always joined by the parent's 1-based ordinal.
    ChildOrdinal,
    /// Keys once in the header (`items:[sku,qty]`), each cell a JSON array
    /// of positional arrays.
    Hoisted,
    /// Compact JSON cell.
    Json,
}

/// How the sparse keys of a row are written.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RestForm {
    /// One `~` cell holding a JSON object (v0.1).
    Json,
    /// `key=value` cells after the dense cells, values self-describing.
    Pairs,
}

/// Null and absent markers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Nulls {
    /// `null` is null, an empty cell is absent (v0.1).
    NullEmpty,
    /// `-` is null, an empty cell is absent.
    DashEmpty,
    /// An empty cell is null, `?` is absent.
    EmptyQuestion,
}

impl Nulls {
    pub fn null(self) -> &'static str {
        match self {
            Nulls::NullEmpty => "null",
            Nulls::DashEmpty => "-",
            Nulls::EmptyQuestion => "",
        }
    }
    pub fn absent(self) -> &'static str {
        match self {
            Nulls::NullEmpty | Nulls::DashEmpty => "",
            Nulls::EmptyQuestion => "?",
        }
    }
}

/// How a string that cannot be bare is written.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Quoting {
    /// JSON string literal (v0.1).
    Json,
    /// RFC 4180: double quotes, `""` inside, line breaks literal.
    Csv,
    /// No quotes: `\\`, `\n`, `\r`, `\t`, backslash before the delimiter,
    /// and `\_` (empty) in front of a string that would read as something else.
    Backslash,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Keys {
    /// One header per table.
    Once,
    /// The header is repeated before every `n` rows of a table.
    Every(u16),
    /// Every cell is `key=value`; absent keys are left out.
    Row,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FloatKind {
    /// `1.0` keeps its `.0` (v0.1).
    Keep,
    /// An `f` column says the kind, so integral floats are written `1`.
    Typed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Bools {
    Words,
    Digits,
    Letters,
}

impl Bools {
    pub fn text(self, b: bool) -> &'static str {
        match (self, b) {
            (Bools::Words, true) => "true",
            (Bools::Words, false) => "false",
            (Bools::Digits, true) => "1",
            (Bools::Digits, false) => "0",
            (Bools::Letters, true) => "t",
            (Bools::Letters, false) => "f",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Config {
    pub delim: Delim,
    pub typing: Typing,
    pub nullable_mark: bool,
    pub objects: Objects,
    pub arrays: Arrays,
    /// Sparse threshold in percent: a top-level key filled (present and
    /// non-null) in fewer than this share of rows goes to the rest of the row.
    pub sparse: u8,
    pub rest: RestForm,
    pub nulls: Nulls,
    pub quoting: Quoting,
    pub keys: Keys,
    pub float_kind: FloatKind,
    pub bools: Bools,
    /// Trailing absent cells are left out.
    pub trim: bool,
}

impl Config {
    pub const LDF_V01: Config = Config {
        delim: Delim::Comma,
        typing: Typing::Letters,
        nullable_mark: true,
        objects: Objects::Dotted,
        arrays: Arrays::ChildKey,
        sparse: 50,
        rest: RestForm::Json,
        nulls: Nulls::NullEmpty,
        quoting: Quoting::Json,
        keys: Keys::Once,
        float_kind: FloatKind::Keep,
        bools: Bools::Words,
        trim: false,
    };
    pub const LDF_V0: Config = Config {
        sparse: 0,
        ..Config::LDF_V01
    };

    /// Combinations the decoder could not tell apart, or that duplicate
    /// another point, are not part of the space.
    pub fn is_valid(&self) -> bool {
        if self.typing == Typing::None
            && (self.nullable_mark
                || self.float_kind != FloatKind::Keep
                || self.bools != Bools::Words)
        {
            return false;
        }
        if self.sparse == 0 && self.rest != RestForm::Json {
            return false;
        }
        if self.keys == Keys::Row && (self.sparse != 0 || self.trim) {
            return false;
        }
        true
    }

    /// Short stable id, knob by knob.
    pub fn id(&self) -> String {
        let mut s = String::with_capacity(96);
        let _ = write!(
            s,
            "d={} h={} q?={} o={} a={} sp={} rf={} n={} qt={} k={} fl={} b={} tr={}",
            match self.delim {
                Delim::Comma => "comma",
                Delim::Tab => "tab",
                Delim::Pipe => "pipe",
            },
            match self.typing {
                Typing::Letters => "typed",
                Typing::None => "untyped",
            },
            u8::from(self.nullable_mark),
            match self.objects {
                Objects::Dotted => "dotted",
                Objects::Json => "json",
            },
            match self.arrays {
                Arrays::ChildKey => "child-key",
                Arrays::ChildOrdinal => "child-ord",
                Arrays::Hoisted => "hoisted",
                Arrays::Json => "json",
            },
            self.sparse,
            match self.rest {
                RestForm::Json => "json",
                RestForm::Pairs => "pairs",
            },
            match self.nulls {
                Nulls::NullEmpty => "null/empty",
                Nulls::DashEmpty => "dash/empty",
                Nulls::EmptyQuestion => "empty/?",
            },
            match self.quoting {
                Quoting::Json => "json",
                Quoting::Csv => "csv",
                Quoting::Backslash => "backslash",
            },
            match self.keys {
                Keys::Once => "once".to_string(),
                Keys::Every(n) => format!("every{n}"),
                Keys::Row => "row".to_string(),
            },
            match self.float_kind {
                FloatKind::Keep => "keep",
                FloatKind::Typed => "typed",
            },
            match self.bools {
                Bools::Words => "words",
                Bools::Digits => "digits",
                Bools::Letters => "letters",
            },
            u8::from(self.trim),
        );
        s
    }

    /// The value of knob `k` (index into [`KNOBS`]) as text, for the
    /// per-knob analysis.
    pub fn knob_value(&self, k: usize) -> String {
        let id = self.id();
        id.split(' ')
            .nth(k)
            .and_then(|p| p.split_once('='))
            .map(|(_, v)| v.to_string())
            .unwrap_or_default()
    }
}

/// Knob names in [`Config::id`] order.
pub const KNOBS: [&str; 13] = [
    "delimiter",
    "header typing",
    "nullable mark",
    "objects",
    "arrays of objects",
    "sparse threshold",
    "sparse cell form",
    "null / absent",
    "quoting",
    "key repetition",
    "float kind",
    "booleans",
    "trim trailing absent",
];

/// Every valid configuration: the product of all knob values, starting
/// from v0.1 and expanding one knob at a time.
pub fn space() -> Vec<Config> {
    fn expand<T: Copy>(cs: Vec<Config>, vals: &[T], set: fn(&mut Config, T)) -> Vec<Config> {
        cs.into_iter()
            .flat_map(|c| {
                vals.iter().map(move |v| {
                    let mut c = c;
                    set(&mut c, *v);
                    c
                })
            })
            .collect()
    }
    let mut s = vec![Config::LDF_V01];
    s = expand(s, &[Delim::Comma, Delim::Tab, Delim::Pipe], |c, v| {
        c.delim = v
    });
    s = expand(s, &[Typing::Letters, Typing::None], |c, v| c.typing = v);
    s = expand(s, &[true, false], |c, v| c.nullable_mark = v);
    s = expand(s, &[Objects::Dotted, Objects::Json], |c, v| c.objects = v);
    s = expand(
        s,
        &[
            Arrays::ChildKey,
            Arrays::ChildOrdinal,
            Arrays::Hoisted,
            Arrays::Json,
        ],
        |c, v| c.arrays = v,
    );
    s = expand(s, &[0, 25, 50, 75, 100], |c, v| c.sparse = v);
    s = expand(s, &[RestForm::Json, RestForm::Pairs], |c, v| c.rest = v);
    s = expand(
        s,
        &[Nulls::NullEmpty, Nulls::DashEmpty, Nulls::EmptyQuestion],
        |c, v| c.nulls = v,
    );
    s = expand(
        s,
        &[Quoting::Json, Quoting::Csv, Quoting::Backslash],
        |c, v| c.quoting = v,
    );
    s = expand(
        s,
        &[Keys::Once, Keys::Every(50), Keys::Every(200), Keys::Row],
        |c, v| c.keys = v,
    );
    s = expand(s, &[FloatKind::Keep, FloatKind::Typed], |c, v| {
        c.float_kind = v
    });
    s = expand(s, &[Bools::Words, Bools::Digits, Bools::Letters], |c, v| {
        c.bools = v
    });
    s = expand(s, &[false, true], |c, v| c.trim = v);
    s.retain(Config::is_valid);
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn space_contains_the_released_layouts_once() {
        let s = space();
        assert_eq!(s.iter().filter(|c| **c == Config::LDF_V01).count(), 1);
        assert_eq!(s.iter().filter(|c| **c == Config::LDF_V0).count(), 1);
        let ids: std::collections::HashSet<String> = s.iter().map(Config::id).collect();
        assert_eq!(ids.len(), s.len());
    }

    #[test]
    fn knob_values_follow_the_id() {
        let c = Config::LDF_V01;
        assert_eq!(c.knob_value(0), "comma");
        assert_eq!(c.knob_value(5), "50");
        assert_eq!(c.knob_value(12), "0");
        assert_eq!(KNOBS.len(), c.id().split(' ').count());
    }
}
