//! The dataset catalog. Each generator receives a per-record RNG and the
//! record index and returns one record. Generators must be pure: no shared
//! state, so that any row can be regenerated independently.

use ldf_data::{Value, obj};

use crate::rng::Rng;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scale {
    Small,
    Medium,
    Large,
}

impl Scale {
    pub fn parse(s: &str) -> Option<Scale> {
        match s {
            "small" => Some(Scale::Small),
            "medium" => Some(Scale::Medium),
            "large" => Some(Scale::Large),
            _ => None,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Scale::Small => "small",
            Scale::Medium => "medium",
            Scale::Large => "large",
        }
    }
}

pub struct DatasetSpec {
    pub name: &'static str,
    pub description: &'static str,
    /// Flat means every record is an object of primitives, so CSV, TSV and
    /// Markdown can represent it. Uniform means every record has the same
    /// keys in the same order.
    pub flat: bool,
    pub uniform: bool,
    /// Rows for small, medium and large scale.
    pub rows: [u64; 3],
    pub generate: fn(&mut Rng, u64) -> Value,
}

impl DatasetSpec {
    pub fn rows_for(&self, scale: Scale) -> u64 {
        match scale {
            Scale::Small => self.rows[0],
            Scale::Medium => self.rows[1],
            Scale::Large => self.rows[2],
        }
    }

    /// Record `index` under `seed`.
    pub fn record(&self, seed: u64, index: u64) -> Value {
        let mut rng = Rng::for_record(seed ^ fnv(self.name), index);
        (self.generate)(&mut rng, index)
    }

    pub fn records(&self, seed: u64, rows: u64) -> impl Iterator<Item = Value> + '_ {
        (0..rows).map(move |i| self.record(seed, i))
    }
}

fn fnv(s: &str) -> u64 {
    s.bytes().fold(0xcbf2_9ce4_8422_2325u64, |h, b| {
        (h ^ b as u64).wrapping_mul(0x0100_0000_01b3)
    })
}

pub fn catalog() -> Vec<DatasetSpec> {
    vec![
        DatasetSpec {
            name: "flat_users",
            description: "Flat homogeneous user records: int id, strings, int age, bool, 2-decimal float, ISO date.",
            flat: true,
            uniform: true,
            rows: [20, 10_000, 1_000_000],
            generate: flat_users,
        },
        DatasetSpec {
            name: "nested_orders",
            description: "Orders with a nested customer object (two levels), a 1-4 element items array of objects, and a nullable note.",
            flat: false,
            uniform: true,
            rows: [20, 10_000, 500_000],
            generate: nested_orders,
        },
        DatasetSpec {
            name: "sparse_events",
            description: "Events with 3 required and 12 optional fields; optional fields are missing, explicitly null, or present with varying probabilities. Distinguishes missing from null.",
            flat: true,
            uniform: false,
            rows: [20, 10_000, 1_000_000],
            generate: sparse_events,
        },
        DatasetSpec {
            name: "high_card_strings",
            description: "High-cardinality strings: UUID-shaped ids, 64-hex hashes, base62 tokens, random URL paths.",
            flat: true,
            uniform: true,
            rows: [20, 10_000, 500_000],
            generate: high_card_strings,
        },
        DatasetSpec {
            name: "repetitive_logs",
            description: "Low-cardinality log lines: skewed level, service, status, region, templated message, int latency.",
            flat: true,
            uniform: true,
            rows: [20, 10_000, 1_000_000],
            generate: repetitive_logs,
        },
        DatasetSpec {
            name: "unicode_strings",
            description: "Pathological strings: CJK, Cyrillic, Arabic, emoji, combining marks, quotes, delimiters, colons, newlines, tabs, backslashes, leading/trailing spaces, numeric-looking and keyword-looking strings, empty strings.",
            flat: true,
            uniform: true,
            rows: [20, 10_000, 200_000],
            generate: unicode_strings,
        },
        DatasetSpec {
            name: "numeric_mix",
            description: "Numbers across magnitudes: small/large/negative ints, i64 extremes, floats needing 17 digits, tiny (1e-9) and huge (1e22) magnitudes, .5 ties, money, booleans.",
            flat: true,
            uniform: true,
            rows: [20, 10_000, 1_000_000],
            generate: numeric_mix,
        },
        DatasetSpec {
            name: "mixed_types",
            description: "Heterogeneous records: a `value` field whose type varies per record (int, float, string, bool, null, array, object), optional tags of varying type, meta object with varying keys.",
            flat: false,
            uniform: false,
            rows: [20, 10_000, 500_000],
            generate: mixed_types,
        },
        DatasetSpec {
            name: "wide_table",
            description: "Wide flat table: id plus 40 typed columns (10 int, 10 float, 10 short string, 10 bool), for large tabular runs.",
            flat: true,
            uniform: true,
            rows: [20, 10_000, 1_000_000],
            generate: wide_table,
        },
        DatasetSpec {
            name: "deep_nesting",
            description: "Pathological depth: object chains 4-16 levels deep and arrays of arrays of arrays.",
            flat: false,
            uniform: true,
            rows: [10, 2_000, 50_000],
            generate: deep_nesting,
        },
        DatasetSpec {
            name: "empty_values",
            description: "Pathological emptiness and keys: empty arrays, empty objects, empty strings, nulls, keys with spaces, dashes, dots, colons, unicode and an empty key.",
            flat: false,
            uniform: true,
            rows: [20, 10_000, 200_000],
            generate: empty_values,
        },
        DatasetSpec {
            name: "long_text",
            description: "Documents with a 200-2000 character body containing sentences, commas, quotes and paragraph breaks.",
            flat: true,
            uniform: true,
            rows: [10, 2_000, 100_000],
            generate: long_text,
        },
    ]
}

pub fn find(name: &str) -> Option<DatasetSpec> {
    catalog().into_iter().find(|d| d.name == name)
}

// ---- vocabularies -----------------------------------------------------------

const FIRST: &[&str] = &[
    "Ada", "Bob", "Cy", "Dee", "Eve", "Finn", "Gus", "Hana", "Ivo", "Jun", "Kai", "Lea", "Max",
    "Nia", "Oli", "Pia", "Quin", "Rui", "Sol", "Tea", "Uma", "Val", "Wes", "Xia", "Yan", "Zoe",
    "Ana", "Ben", "Cleo", "Dan", "Elsa", "Fay", "Gil", "Hal", "Iris", "Jo", "Kim", "Lou", "Mia",
    "Ned",
];
const LAST: &[&str] = &[
    "Lovelace",
    "Hopper",
    "Turing",
    "Knuth",
    "Dijkstra",
    "Liskov",
    "Ritchie",
    "Thompson",
    "Kay",
    "Wirth",
    "Backus",
    "McCarthy",
    "Hamilton",
    "Lamport",
    "Torvalds",
    "Rossum",
    "Matsumoto",
    "Gosling",
    "Stroustrup",
    "Pike",
    "Kernighan",
    "Shannon",
    "Boole",
    "Babbage",
    "Noether",
    "Curie",
    "Franklin",
    "Goodall",
    "Carson",
    "Ride",
    "Johnson",
    "Vaughan",
    "Jackson",
    "Easley",
    "Clarke",
    "Perlman",
    "Allen",
    "Bartik",
    "Holberton",
    "Meltzer",
];
const COUNTRIES: &[&str] = &[
    "DK", "UK", "US", "DE", "FR", "ES", "IT", "PT", "BR", "AR", "MX", "JP", "KR", "CN", "IN", "AU",
    "NZ", "CA", "SE", "NO",
];
const CITIES: &[&str] = &[
    "Copenhagen",
    "London",
    "Austin",
    "Berlin",
    "Paris",
    "Madrid",
    "Rome",
    "Lisbon",
    "São Paulo",
    "Buenos Aires",
    "Mexico City",
    "Tokyo",
    "Seoul",
    "Shanghai",
    "Mumbai",
    "Sydney",
    "Auckland",
    "Toronto",
    "Stockholm",
    "Oslo",
];
const DOMAINS: &[&str] = &[
    "example.com",
    "example.org",
    "mail.test",
    "corp.example",
    "uni.edu",
];
const STATUS: &[&str] = &[
    "pending",
    "paid",
    "shipped",
    "delivered",
    "cancelled",
    "refunded",
];
const SKUS: &[&str] = &[
    "A1", "B2", "C3", "D4", "E5", "F6", "G7", "H8", "J9", "K10", "L11", "M12",
];
const LEVELS: &[&str] = &["DEBUG", "INFO", "WARN", "ERROR", "FATAL"];
const LEVEL_W: &[u32] = &[30, 55, 10, 4, 1];
const SERVICES: &[&str] = &[
    "api",
    "auth",
    "billing",
    "search",
    "ingest",
    "mailer",
    "scheduler",
    "gateway",
];
const REGIONS: &[&str] = &["eu-west", "eu-north", "us-east", "ap-south"];
const HTTP: &[i64] = &[200, 201, 204, 301, 400, 401, 403, 404, 429, 500, 502, 503];
const HTTP_W: &[u32] = &[60, 8, 5, 2, 6, 3, 2, 8, 1, 3, 1, 1];
const TEMPLATES: &[&str] = &[
    "request completed",
    "cache miss for key",
    "retrying upstream call",
    "user session expired",
    "payment captured",
    "rate limit applied",
    "index rebuilt",
    "connection reset by peer",
    "job scheduled",
    "slow query detected",
];
const WORDS: &[&str] = &[
    "the",
    "data",
    "format",
    "token",
    "record",
    "field",
    "value",
    "schema",
    "stream",
    "model",
    "table",
    "row",
    "column",
    "nested",
    "array",
    "object",
    "string",
    "number",
    "null",
    "true",
    "false",
    "quote",
    "comma",
    "colon",
    "line",
    "byte",
    "memory",
    "latency",
    "accuracy",
    "benchmark",
    "measure",
    "report",
    "result",
    "signal",
    "noise",
    "sample",
    "context",
    "window",
    "prompt",
    "answer",
    "question",
    "lookup",
    "filter",
    "aggregate",
    "compare",
    "extract",
    "reason",
];
const UNICODE_NAMES: &[&str] = &[
    "李雷",
    "Zoë Müller",
    "Иван Петров",
    "محمد علي",
    "Ana María",
    "Sørën",
    "Ōkami 🐺",
    "Ngọc Ánh",
    "Ελένη",
    "חנה",
    "José Ñandú",
    "Åsa",
    "🙂 smiley",
    "e\u{301}toile",
    "Ærø",
    "İstanbul",
    "北京市",
    "Dzintars",
    "Kōji",
    "Ævar",
];
const TRICKY: &[&str] = &[
    "a,b",
    "x:y",
    "with \"quotes\"",
    "back\\slash",
    "line1\nline2",
    "tab\there",
    " leading",
    "trailing ",
    "pipe|char",
    "[bracket]",
    "{brace}",
    "-dash",
    "#hash",
    "007",
    "1e5",
    "+1",
    "true",
    "null",
    "false",
    "",
    "NaN",
    "Infinity",
    "0x10",
    "1_000",
    ".5",
    "1.",
    "- item",
    "key: value",
    "\u{feff}bom",
    "zero\u{0}byte",
];

fn full_name(r: &mut Rng) -> String {
    format!("{} {}", r.pick(FIRST), r.pick(LAST))
}

fn date(r: &mut Rng) -> String {
    format!(
        "{:04}-{:02}-{:02}",
        2015 + r.below(11),
        1 + r.below(12),
        1 + r.below(28)
    )
}

fn timestamp(r: &mut Rng, index: u64) -> String {
    let secs = 1_700_000_000u64 + index * 7 + r.below(7);
    let (d, s) = (secs / 86_400, secs % 86_400);
    // Days since 1970-01-01 to civil date (Howard Hinnant's algorithm).
    let z = d as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if month <= 2 { y + 1 } else { y };
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        s / 3600,
        (s % 3600) / 60,
        s % 60
    )
}

// ---- generators -------------------------------------------------------------

fn flat_users(r: &mut Rng, i: u64) -> Value {
    let name = full_name(r);
    let email = format!(
        "{}.{}@{}",
        name.split(' ').next().unwrap().to_lowercase(),
        i,
        r.pick(DOMAINS)
    );
    obj! {
        "id" => i as i64 + 1,
        "name" => name,
        "email" => email,
        "age" => r.range_i64(18, 90),
        "active" => r.chance(0.7),
        "score" => r.decimal(0.0, 100.0, 2),
        "country" => *r.pick(COUNTRIES),
        "signup_date" => date(r),
    }
}

fn nested_orders(r: &mut Rng, i: u64) -> Value {
    let n_items = 1 + r.below(4) as usize;
    let mut total = 0.0;
    let items: Vec<Value> = (0..n_items)
        .map(|_| {
            let qty = r.range_i64(1, 5);
            let price = r.decimal(1.0, 500.0, 2);
            total += qty as f64 * price;
            obj! {"sku" => *r.pick(SKUS), "qty" => qty, "price" => price}
        })
        .collect();
    let total = (total * 100.0).round() / 100.0;
    let note: Value = if r.chance(0.3) {
        Value::Null
    } else {
        Value::from(format!("{} {}", r.pick(WORDS), r.pick(WORDS)))
    };
    obj! {
        "order_id" => format!("ORD-{:06}", i + 1),
        "customer" => obj! {
            "id" => r.range_i64(1, 5000),
            "name" => full_name(r),
            "address" => obj! {
                "city" => *r.pick(CITIES),
                "country" => *r.pick(COUNTRIES),
                "zip" => format!("{:05}", r.below(100_000)),
            },
        },
        "items" => items,
        "total" => total,
        "status" => *r.pick(STATUS),
        "shipped" => r.chance(0.5),
        "note" => note,
    }
}

fn sparse_events(r: &mut Rng, i: u64) -> Value {
    let mut fields: Vec<(String, Value)> = vec![
        ("event_id".into(), Value::from(i as i64 + 1)),
        (
            "type".into(),
            Value::from(*r.pick(&["click", "view", "purchase", "signup", "error"])),
        ),
        ("ts".into(), Value::from(timestamp(r, i))),
    ];
    type Optional = (&'static str, f64, fn(&mut Rng) -> Value);
    let optional: [Optional; 12] = [
        ("user_id", 0.6, |r| Value::from(r.range_i64(1, 100_000))),
        ("session", 0.5, |r| Value::from(r.alnum(12))),
        ("amount", 0.2, |r| Value::from(r.decimal(0.5, 999.0, 2))),
        ("currency", 0.2, |r| {
            Value::from(*r.pick(&["EUR", "USD", "GBP", "JPY"]))
        }),
        ("referrer", 0.3, |r| {
            Value::from(format!("https://{}/{}", r.pick(DOMAINS), r.pick(WORDS)))
        }),
        ("device", 0.4, |r| {
            Value::from(*r.pick(&["ios", "android", "web", "tv"]))
        }),
        ("country", 0.4, |r| Value::from(*r.pick(COUNTRIES))),
        ("error_code", 0.1, |r| Value::from(r.range_i64(1000, 9999))),
        ("retries", 0.15, |r| Value::from(r.range_i64(0, 5))),
        ("latency_ms", 0.5, |r| Value::from(r.range_i64(1, 5000))),
        ("flag", 0.3, |r| Value::from(r.chance(0.5))),
        ("label", 0.25, |r| Value::from(*r.pick(WORDS))),
    ];
    for (name, p, make) in optional {
        let roll = r.unit();
        if roll < p {
            fields.push((name.into(), make(r)));
        } else if roll < p + 0.1 {
            // Explicitly null, distinct from absent.
            fields.push((name.into(), Value::Null));
        }
    }
    Value::Object(fields)
}

fn high_card_strings(r: &mut Rng, i: u64) -> Value {
    let uuid = format!(
        "{}-{}-4{}-{}{}-{}",
        r.hex(8),
        r.hex(4),
        r.hex(3),
        r.pick(&["8", "9", "a", "b"]),
        r.hex(3),
        r.hex(12)
    );
    obj! {
        "id" => i as i64 + 1,
        "uuid" => uuid,
        "hash" => r.hex(64),
        "token" => r.alnum(24),
        "url" => format!("https://{}/{}/{}/{}", r.pick(DOMAINS), r.pick(WORDS), r.alnum(8), r.below(10_000)),
        "label" => format!("{}-{}", r.pick(WORDS), r.alnum(5)),
    }
}

fn repetitive_logs(r: &mut Rng, i: u64) -> Value {
    obj! {
        "ts" => timestamp(r, i),
        "level" => *r.pick_weighted(LEVELS, LEVEL_W),
        "service" => *r.pick(SERVICES),
        "status" => *r.pick_weighted(HTTP, HTTP_W),
        "region" => *r.pick(REGIONS),
        "message" => *r.pick(TEMPLATES),
        "latency_ms" => r.range_i64(1, 2000),
    }
}

fn unicode_strings(r: &mut Rng, i: u64) -> Value {
    obj! {
        "id" => i as i64 + 1,
        "name" => *r.pick(UNICODE_NAMES),
        "tricky" => TRICKY[(i as usize) % TRICKY.len()],
        "tricky2" => *r.pick(TRICKY),
        "emoji" => *r.pick(&["🙂", "🚀🚀", "👩‍💻", "🇧🇷", "✓", "→", "—", "…"]),
        "mixed" => format!("{} {} {}", r.pick(UNICODE_NAMES), r.pick(TRICKY), r.pick(WORDS)),
        "empty" => "",
    }
}

fn numeric_mix(r: &mut Rng, i: u64) -> Value {
    let magnitude = r.range_i64(-12, 12) as i32;
    let float = r.unit() * 10f64.powi(magnitude);
    obj! {
        "id" => i as i64 + 1,
        "small_int" => r.range_i64(-100, 100),
        "big_int" => r.range_i64(i64::MAX / 2, i64::MAX),
        "neg" => -(r.below(1_000_000) as i64),
        "float" => float,
        "tiny" => r.unit() * 1e-9,
        "huge" => r.unit() * 1e22,
        "half" => r.below(1000) as f64 + 0.5,
        "money" => r.decimal(0.0, 10_000.0, 2),
        "ratio" => r.below(10) as f64 * 0.1 + r.below(10) as f64 * 0.1,
        "zero" => if r.chance(0.5) { Value::from(0) } else { Value::from(0.0) },
        "flag" => r.chance(0.5),
        "flag2" => r.chance(0.1),
    }
}

fn mixed_types(r: &mut Rng, i: u64) -> Value {
    let value: Value = match r.below(7) {
        0 => Value::from(r.range_i64(-1000, 1000)),
        1 => Value::from(r.decimal(-100.0, 100.0, 3)),
        2 => Value::from(*r.pick(WORDS)),
        3 => Value::from(r.chance(0.5)),
        4 => Value::Null,
        5 => Value::Array(
            (0..r.below(4))
                .map(|_| Value::from(r.range_i64(0, 9)))
                .collect(),
        ),
        _ => obj! {"k" => *r.pick(WORDS), "n" => r.range_i64(0, 9)},
    };
    let mut fields: Vec<(String, Value)> = vec![
        ("id".into(), Value::from(i as i64 + 1)),
        ("value".into(), value),
    ];
    match r.below(4) {
        0 => fields.push((
            "tags".into(),
            Value::Array(
                (0..1 + r.below(3))
                    .map(|_| Value::from(*r.pick(WORDS)))
                    .collect(),
            ),
        )),
        1 => fields.push(("tags".into(), Value::from(*r.pick(WORDS)))),
        2 => fields.push(("tags".into(), Value::Null)),
        _ => {}
    }
    let mut meta: Vec<(String, Value)> = Vec::new();
    if r.chance(0.5) {
        meta.push(("source".into(), Value::from(*r.pick(SERVICES))));
    }
    if r.chance(0.5) {
        meta.push(("weight".into(), Value::from(r.decimal(0.0, 1.0, 4))));
    }
    if r.chance(0.3) {
        meta.push((
            "nested".into(),
            obj! {"deep" => obj! {"x" => r.range_i64(0, 9)}},
        ));
    }
    fields.push(("meta".into(), Value::Object(meta)));
    Value::Object(fields)
}

fn wide_table(r: &mut Rng, i: u64) -> Value {
    let mut fields: Vec<(String, Value)> = Vec::with_capacity(41);
    fields.push(("id".into(), Value::from(i as i64 + 1)));
    for c in 1..=10 {
        fields.push((format!("i{c:02}"), Value::from(r.range_i64(0, 1_000_000))));
    }
    for c in 1..=10 {
        fields.push((format!("f{c:02}"), Value::from(r.decimal(0.0, 1000.0, 3))));
    }
    for c in 1..=10 {
        fields.push((format!("s{c:02}"), Value::from(*r.pick(WORDS))));
    }
    for c in 1..=10 {
        fields.push((format!("b{c:02}"), Value::from(r.chance(0.5))));
    }
    Value::Object(fields)
}

fn deep_nesting(r: &mut Rng, i: u64) -> Value {
    let depth = r.range_i64(4, 16) as usize;
    let mut node = obj! {"leaf" => *r.pick(WORDS), "n" => r.range_i64(0, 99)};
    for level in (0..depth).rev() {
        node = obj! {"level" => level as i64, "child" => node};
    }
    let matrix: Vec<Value> = (0..3)
        .map(|_| {
            Value::Array(
                (0..3)
                    .map(|_| Value::Array((0..2).map(|_| Value::from(r.range_i64(0, 9))).collect()))
                    .collect(),
            )
        })
        .collect();
    obj! {"id" => i as i64 + 1, "depth" => depth as i64, "tree" => node, "matrix" => matrix}
}

fn empty_values(r: &mut Rng, i: u64) -> Value {
    let k = i % 4;
    let empties: [Value; 4] = [
        Value::Array(vec![]),
        Value::Object(vec![]),
        Value::from(""),
        Value::Null,
    ];
    Value::Object(vec![
        ("id".into(), Value::from(i as i64 + 1)),
        ("empty_a".into(), empties[k as usize].clone()),
        ("empty_b".into(), empties[((k + 1) % 4) as usize].clone()),
        ("empty_c".into(), empties[((k + 2) % 4) as usize].clone()),
        ("with space".into(), Value::from(*r.pick(WORDS))),
        ("with-dash".into(), Value::from(r.range_i64(0, 9))),
        ("dotted.key".into(), Value::from(r.chance(0.5))),
        ("colon:key".into(), Value::from(*r.pick(WORDS))),
        ("ключ".into(), Value::from(*r.pick(UNICODE_NAMES))),
        ("".into(), Value::from("empty key")),
        (
            "nested_empty".into(),
            obj! {"a" => Value::Array(vec![]), "b" => Value::Object(vec![])},
        ),
    ])
}

fn long_text(r: &mut Rng, i: u64) -> Value {
    let target = r.range_i64(200, 2000) as usize;
    let mut body = String::new();
    while body.len() < target {
        let n = r.range_i64(5, 18);
        let mut sentence = String::new();
        for w in 0..n {
            if w > 0 {
                sentence.push(if r.chance(0.1) { ',' } else { ' ' });
                if sentence.ends_with(',') {
                    sentence.push(' ');
                }
            }
            sentence.push_str(r.pick(WORDS));
        }
        let mut chars = sentence.chars();
        let first = chars.next().unwrap().to_uppercase().to_string();
        sentence = first + chars.as_str();
        if r.chance(0.1) {
            sentence = format!("\"{sentence}\"");
        }
        sentence.push(*r.pick(&['.', '.', '.', '!', '?']));
        body.push_str(&sentence);
        body.push_str(if r.chance(0.15) { "\n\n" } else { " " });
    }
    obj! {
        "id" => i as i64 + 1,
        "title" => format!("{} {} {}", r.pick(WORDS), r.pick(WORDS), r.pick(WORDS)),
        "author" => full_name(r),
        "body" => body.trim_end().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ldf_data::FlatSchema;

    #[test]
    fn catalog_flags_match_reality() {
        for spec in catalog() {
            let recs: Vec<Value> = spec.records(7, 200).collect();
            let flat = FlatSchema::infer(&recs);
            assert_eq!(flat.is_ok(), spec.flat, "{} flat flag", spec.name);
            if let Ok(s) = flat {
                assert_eq!(
                    s.uniform && s.same_order,
                    spec.uniform,
                    "{} uniform flag",
                    spec.name
                );
            }
        }
    }

    #[test]
    fn records_are_pure_functions_of_seed_and_index() {
        for spec in catalog() {
            let a = spec.record(42, 17);
            let b = spec.record(42, 17);
            let c = spec.record(43, 17);
            assert_eq!(a, b, "{}", spec.name);
            assert_ne!(a, c, "{}", spec.name);
        }
    }
}
