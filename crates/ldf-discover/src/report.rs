//! Markdown rendering of `stage-a.json`.

use std::fmt::Write as _;

use serde_json::Value as J;

fn f(v: &J) -> f64 {
    v.as_f64().unwrap_or(f64::NAN)
}

fn u(v: &J) -> u64 {
    v.as_u64().unwrap_or(0)
}

fn s(v: &J) -> &str {
    v.as_str().unwrap_or("")
}

pub fn render(doc: &J) -> String {
    let mut o = String::new();
    let ds = doc["datasets"].as_array().cloned().unwrap_or_default();
    let _ = writeln!(o, "# Stage A: token search over the knob space\n");
    let _ = writeln!(
        o,
        "Fixtures `{}`, first {} records per dataset, tokenizer `{}` (exact, tiktoken-rs). \
         {} configurations, encoded once each; distinct outputs round-trip checked and counted. \
         A candidate passes when it is lossless (byte, semantic or semantic-unordered) and at \
         most {:.2} of compact JSON tokens. Run took {:.0} s on {} threads. Token counts do \
         not depend on the machine.\n",
        s(&doc["scale"]),
        u(&doc["sample"]),
        s(&doc["tokenizer"]),
        u(&doc["space"]),
        f(&doc["max_ratio"]),
        f(&doc["seconds"]),
        u(&doc["machine_threads"]),
    );

    let _ = writeln!(o, "## Room beyond LDF v0.1\n");
    let _ = writeln!(
        o,
        "| dataset | json | ldf v0.1 | v0.1 / json | best | best / json | best / v0.1 | distinct | lossless | passing | engine v0.1 = crates/ldf |"
    );
    let _ = writeln!(o, "|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---|");
    for d in &ds {
        let best = d["top"].get(0);
        let _ = writeln!(
            o,
            "| {} | {} | {} | {:.3} | {} | {} | {} | {} | {} | {} | {} |",
            s(&d["name"]),
            u(&d["json_tokens"]),
            u(&d["ldf_tokens"]),
            u(&d["ldf_tokens"]) as f64 / u(&d["json_tokens"]) as f64,
            best.map_or("-".into(), |b| u(&b["tokens"]).to_string()),
            best.map_or("-".into(), |b| format!("{:.3}", f(&b["ratio_json"]))),
            best.map_or("-".into(), |b| format!("{:.3}", f(&b["ratio_ldf"]))),
            u(&d["distinct"]),
            u(&d["lossless"]),
            u(&d["passing"]),
            if d["v01_identical"].as_bool() == Some(true) {
                "yes"
            } else {
                "no"
            },
        );
    }

    let _ = writeln!(o, "\n## Best candidate per dataset\n");
    for d in &ds {
        let _ = writeln!(o, "**{}**\n", s(&d["name"]));
        let top = d["top"].as_array().cloned().unwrap_or_default();
        if top.is_empty() {
            let _ = writeln!(o, "No lossless candidate within the bar.\n");
            continue;
        }
        let _ = writeln!(
            o,
            "| rank | tokens | / json | / v0.1 | level | configs | id |"
        );
        let _ = writeln!(o, "|---:|---:|---:|---:|---|---:|---|");
        for (i, t) in top.iter().take(5).enumerate() {
            let _ = writeln!(
                o,
                "| {} | {} | {:.3} | {:.3} | {} | {} | `{}` |",
                i + 1,
                u(&t["tokens"]),
                f(&t["ratio_json"]),
                f(&t["ratio_ldf"]),
                s(&t["level"]),
                u(&t["configs"]),
                s(&t["id"]),
            );
        }
        let _ = writeln!(o);
    }

    let _ = writeln!(o, "## Reference formats (tokens / compact JSON)\n");
    let refs: Vec<String> = ds
        .first()
        .and_then(|d| d["references"].as_array())
        .map(|a| a.iter().map(|r| s(&r["format"]).to_string()).collect())
        .unwrap_or_default();
    let _ = writeln!(o, "| dataset | {} |", refs.join(" | "));
    let _ = writeln!(o, "|---|{}", "---:|".repeat(refs.len()));
    for d in &ds {
        let cells: Vec<String> = d["references"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|r| {
                let lvl = s(&r["level"]);
                match r["ratio_json"].as_f64() {
                    None => "unsupported".into(),
                    Some(x) if matches!(lvl, "lossy" | "decode-error") => format!("{x:.3} ({lvl})"),
                    Some(x) => format!("{x:.3}"),
                }
            })
            .collect();
        let _ = writeln!(o, "| {} | {} |", s(&d["name"]), cells.join(" | "));
    }

    let _ = writeln!(o, "\n## One configuration for all datasets\n");
    let _ = writeln!(
        o,
        "Configurations lossless on every dataset, ranked by mean tokens / JSON.\n"
    );
    let _ = writeln!(
        o,
        "| rank | mean / json | mean / v0.1 | worst / json | id |"
    );
    let _ = writeln!(o, "|---:|---:|---:|---:|---|");
    for (i, g) in doc["global"].as_array().into_iter().flatten().enumerate() {
        let _ = writeln!(
            o,
            "| {} | {:.3} | {:.3} | {:.3} | `{}` |",
            i + 1,
            f(&g["mean_ratio_json"]),
            f(&g["mean_ratio_ldf"]),
            f(&g["max_ratio_json"]),
            s(&g["id"]),
        );
    }
    let v = &doc["v01_global"];
    if v.is_object() {
        let _ = writeln!(
            o,
            "| v0.1 | {:.3} | {:.3} | {:.3} | `{}` |",
            f(&v["mean_ratio_json"]),
            f(&v["mean_ratio_ldf"]),
            f(&v["max_ratio_json"]),
            s(&v["id"]),
        );
    } else {
        let _ = writeln!(o, "\nLDF v0.1 is not lossless on every dataset.");
    }

    let _ = writeln!(o, "\n## Knob effects\n");
    let _ = writeln!(
        o,
        "Per dataset: `-` when no change of the knob alone changes the output; otherwise the \
         value with the lowest lossless tokens and, in brackets, how much more the best \
         configuration with the worst value costs.\n"
    );
    let names: Vec<&str> = ds.iter().map(|d| s(&d["name"])).collect();
    let _ = writeln!(o, "| knob | {} |", names.join(" | "));
    let _ = writeln!(o, "|---|{}", "---|".repeat(names.len()));
    let nknobs = ds
        .first()
        .and_then(|d| d["knobs"].as_array())
        .map_or(0, Vec::len);
    for k in 0..nknobs {
        let mut row = vec![s(&ds[0]["knobs"][k]["knob"]).to_string()];
        for d in &ds {
            let kn = &d["knobs"][k];
            if kn["matters"].as_bool() != Some(true) {
                row.push("-".into());
                continue;
            }
            let vals: Vec<(String, u64)> = kn["best_by_value"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|p| Some((s(&p[0]).to_string(), p[1].as_u64()?)))
                .collect();
            let best = vals.iter().min_by_key(|v| v.1);
            let worst = vals.iter().max_by_key(|v| v.1);
            row.push(match (best, worst) {
                (Some(b), Some(w)) => {
                    format!("{} (+{:.1}%)", b.0, 100.0 * (w.1 as f64 / b.1 as f64 - 1.0))
                }
                _ => "lossy".into(),
            });
        }
        let _ = writeln!(o, "| {} |", row.join(" | "));
    }

    let rc = doc["rechecks"].as_array().cloned().unwrap_or_default();
    if !rc.is_empty() {
        let _ = writeln!(
            o,
            "\n## Recheck at {} records\n\nThe top candidates of each dataset re-encoded on the larger sample, sorted by tokens; `rank` is the rank they had at {} records.\n",
            u(&doc["recheck"]),
            u(&doc["sample"]),
        );
        for r in &rc {
            let _ = writeln!(
                o,
                "**{}** ({} records, json {}, v0.1 {} = {:.3})\n",
                s(&r["dataset"]),
                u(&r["records"]),
                u(&r["json_tokens"]),
                u(&r["ldf_tokens"]),
                u(&r["ldf_tokens"]) as f64 / u(&r["json_tokens"]) as f64,
            );
            let _ = writeln!(o, "| rank | tokens | / json | / v0.1 | level |");
            let _ = writeln!(o, "|---:|---:|---:|---:|---|");
            for row in r["rows"].as_array().into_iter().flatten() {
                let _ = writeln!(
                    o,
                    "| {} | {} | {:.3} | {:.3} | {} |",
                    u(&row["rank_at_sample"]),
                    u(&row["tokens"]),
                    f(&row["ratio_json"]),
                    f(&row["ratio_ldf"]),
                    s(&row["level"]),
                );
            }
            let _ = writeln!(o);
        }
    }

    let _ = writeln!(o, "## Dropped outputs\n");
    let _ = writeln!(o, "| dataset | lossy | decode-error | first failure |");
    let _ = writeln!(o, "|---|---:|---:|---|");
    for d in &ds {
        let first = d["failures"]
            .get(0)
            .map(|x| {
                format!(
                    "{}: {} `{}`",
                    s(&x["level"]),
                    s(&x["detail"]).replace('|', "\\|"),
                    s(&x["id"])
                )
            })
            .unwrap_or_default();
        let _ = writeln!(
            o,
            "| {} | {} | {} | {} |",
            s(&d["name"]),
            u(&d["lossy"]),
            u(&d["decode_errors"]),
            first
        );
    }
    o
}
