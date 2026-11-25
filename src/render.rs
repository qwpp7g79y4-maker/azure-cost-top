//! Table, CSV and JSON output.

use crate::money::{percent, Money};
use crate::report::{Dimension, Entry, Report, Section};

/// Output format.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Format {
    Table,
    Csv,
    Json,
}

pub fn render(report: &Report, format: Format) -> String {
    match format {
        Format::Table => table(report),
        Format::Csv => csv(report),
        Format::Json => json(report),
    }
}

/// Text that goes into the table or the CSV: control characters (newline,
/// tab, NUL, ESC, the C1 range) and the Unicode line and paragraph separators
/// become U+FFFD, so a name cannot start a fake line or a terminal sequence.
pub(crate) fn clean(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_control() || c == '\u{2028}' || c == '\u{2029}' {
                '\u{fffd}'
            } else {
                c
            }
        })
        .collect()
}

fn share(entry: &Entry, total: Money) -> Option<f64> {
    percent(entry.cost, total)
}

/// Delta percentage against the previous cost; `None` when there is no base.
fn delta_pct(entry: &Entry) -> Option<f64> {
    let prev = entry.previous?;
    percent(entry.cost - prev, prev.abs())
}

fn pct_text(p: Option<f64>) -> String {
    match p {
        Some(v) => format!("{v:.1}%"),
        None => "-".to_string(),
    }
}

fn signed_pct_text(entry: &Entry) -> String {
    match (entry.previous, delta_pct(entry)) {
        (Some(_), Some(v)) => format!("{v:+.1}%"),
        (Some(p), None) if entry.cost == p => "0.0%".to_string(),
        (Some(_), None) => "new".to_string(),
        (None, _) => String::new(),
    }
}

fn signed(m: Money) -> String {
    if m.nanos() > 0 && m.fixed2() != "0.00" {
        format!("+{}", m.fixed2())
    } else {
        m.fixed2()
    }
}

fn title(s: &Section) -> String {
    let what = match (&s.dimension, &s.tag_key) {
        (Dimension::Tag, Some(k)) => format!("tag {}", clean(k)),
        (d, _) => d.label().to_string(),
    };
    format!(
        "Top {what} costs (showing {} of {})",
        s.entries.len(),
        s.distinct
    )
}

fn pad_left(s: &str, w: usize) -> String {
    let n = s.chars().count();
    format!("{}{}", s, " ".repeat(w.saturating_sub(n)))
}

fn pad_right(s: &str, w: usize) -> String {
    let n = s.chars().count();
    format!("{}{}", " ".repeat(w.saturating_sub(n)), s)
}

fn heading(r: &Report) -> String {
    let mut s = format!(
        "azure-cost-top {} - {}\n",
        env!("CARGO_PKG_VERSION"),
        clean(&r.input)
    );
    let cur = r
        .currency
        .as_deref()
        .map(|c| format!(" {}", clean(c)))
        .unwrap_or_default();
    let period = match (r.month, r.previous_month) {
        (Some(m), Some(p)) => format!("{m} vs {p}"),
        (Some(m), None) => m.to_string(),
        _ => "all months".to_string(),
    };
    s.push_str(&format!("Period: {period}   Rows: {}\n", r.rows));
    s.push_str(&format!("Total: {}{cur}", r.total));
    if let Some(p) = r.previous_total {
        let d = r.total - p;
        let pct = match percent(d, p.abs()) {
            Some(v) => format!(", {v:+.1}%"),
            None => String::new(),
        };
        s.push_str(&format!(" (was {p}, {}{pct})", signed(d)));
    }
    s.push('\n');
    s
}

pub fn table(r: &Report) -> String {
    let mut out = heading(r);
    for sec in &r.sections {
        out.push('\n');
        out.push_str(&title(sec));
        out.push('\n');
        let delta_mode = r.previous_total.is_some();
        let mut header = vec![
            "#".to_string(),
            "Name".to_string(),
            "Cost".to_string(),
            "Share".to_string(),
        ];
        if delta_mode {
            header.extend([
                "Previous".to_string(),
                "Delta".to_string(),
                "Delta%".to_string(),
            ]);
        }
        let mut body: Vec<Vec<String>> = Vec::new();
        for (i, e) in sec.entries.iter().enumerate() {
            let mut row = vec![
                (i + 1).to_string(),
                clean(&e.name),
                e.cost.fixed2(),
                pct_text(share(e, r.total)),
            ];
            if delta_mode {
                row.push(e.previous.map(|p| p.fixed2()).unwrap_or_default());
                row.push(e.delta().map(signed).unwrap_or_default());
                row.push(signed_pct_text(e));
            }
            body.push(row);
        }
        let cols = header.len();
        let widths: Vec<usize> = (0..cols)
            .map(|c| {
                std::iter::once(&header[c])
                    .chain(body.iter().map(|r| &r[c]))
                    .map(|s| s.chars().count())
                    .max()
                    .unwrap_or(0)
            })
            .collect();
        let line = |cells: &[String]| {
            let parts: Vec<String> = cells
                .iter()
                .enumerate()
                .map(|(c, s)| {
                    if c == 1 {
                        pad_left(s, widths[c])
                    } else {
                        pad_right(s, widths[c])
                    }
                })
                .collect();
            format!("  {}\n", parts.join("  ").trim_end())
        };
        out.push_str(&line(&header));
        for row in &body {
            out.push_str(&line(row));
        }
    }
    out
}

/// A text cell for the CSV. Control characters are replaced, and a cell that a
/// spreadsheet would read as a formula (it starts with =, +, - or @) gets a
/// single quote in front. Amounts do not go through here.
fn csv_cell(s: &str) -> String {
    let s = clean(s);
    let s = if s.starts_with(['=', '+', '-', '@']) {
        format!("'{s}")
    } else {
        s
    };
    if s.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", s.replace('"', "\"\""))
    } else {
        s.to_string()
    }
}

pub fn csv(r: &Report) -> String {
    let delta_mode = r.previous_total.is_some();
    let mut out = String::from("dimension,rank,name,cost,share_pct");
    if delta_mode {
        out.push_str(",previous,delta,delta_pct");
    }
    out.push('\n');
    for sec in &r.sections {
        for (i, e) in sec.entries.iter().enumerate() {
            let dim = match (&sec.dimension, &sec.tag_key) {
                (Dimension::Tag, Some(k)) => format!("tag:{}", clean(k)),
                (d, _) => d.key().to_string(),
            };
            out.push_str(&format!(
                "{},{},{},{},{}",
                csv_cell(&dim),
                i + 1,
                csv_cell(&e.name),
                e.cost.exact(),
                share(e, r.total)
                    .map(|v| format!("{v:.1}"))
                    .unwrap_or_default()
            ));
            if delta_mode {
                out.push_str(&format!(
                    ",{},{},{}",
                    e.previous.map(|p| p.exact()).unwrap_or_default(),
                    e.delta().map(|d| d.exact()).unwrap_or_default(),
                    delta_pct(e).map(|v| format!("{v:.1}")).unwrap_or_default()
                ));
            }
            out.push('\n');
        }
    }
    out
}

fn json_str(s: &str) -> String {
    let mut o = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\n' => o.push_str("\\n"),
            '\r' => o.push_str("\\r"),
            '\t' => o.push_str("\\t"),
            c if (c as u32) < 0x20 => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

fn json_opt(s: Option<String>) -> String {
    s.unwrap_or_else(|| "null".to_string())
}

pub fn json(r: &Report) -> String {
    let mut o = String::from("{\n");
    o.push_str(&format!(
        "  \"tool\": \"azure-cost-top\",\n  \"version\": {},\n",
        json_str(env!("CARGO_PKG_VERSION"))
    ));
    o.push_str(&format!("  \"input\": {},\n", json_str(&r.input)));
    o.push_str(&format!("  \"rows\": {},\n", r.rows));
    o.push_str(&format!(
        "  \"currency\": {},\n",
        json_opt(r.currency.as_deref().map(json_str))
    ));
    o.push_str(&format!(
        "  \"month\": {},\n",
        json_opt(r.month.map(|m| json_str(&m.to_string())))
    ));
    o.push_str(&format!(
        "  \"previous_month\": {},\n",
        json_opt(r.previous_month.map(|m| json_str(&m.to_string())))
    ));
    o.push_str(&format!("  \"total\": {},\n", r.total.exact()));
    o.push_str(&format!(
        "  \"previous_total\": {},\n",
        json_opt(r.previous_total.map(|p| p.exact()))
    ));
    o.push_str("  \"sections\": [\n");
    for (si, sec) in r.sections.iter().enumerate() {
        o.push_str("    {\n");
        o.push_str(&format!(
            "      \"dimension\": {},\n",
            json_str(sec.dimension.key())
        ));
        o.push_str(&format!(
            "      \"tag_key\": {},\n",
            json_opt(sec.tag_key.as_deref().map(json_str))
        ));
        o.push_str(&format!("      \"distinct\": {},\n", sec.distinct));
        o.push_str("      \"entries\": [\n");
        for (i, e) in sec.entries.iter().enumerate() {
            o.push_str(&format!(
                "        {{\"rank\": {}, \"name\": {}, \"cost\": {}, \"share_pct\": {}, \"previous\": {}, \"delta\": {}, \"delta_pct\": {}}}",
                i + 1,
                json_str(&e.name),
                e.cost.exact(),
                json_opt(share(e, r.total).map(|v| format!("{v:.1}"))),
                json_opt(e.previous.map(|p| p.exact())),
                json_opt(e.delta().map(|d| d.exact())),
                json_opt(delta_pct(e).map(|v| format!("{v:.1}"))),
            ));
            o.push_str(if i + 1 < sec.entries.len() {
                ",\n"
            } else {
                "\n"
            });
        }
        o.push_str("      ]\n    }");
        o.push_str(if si + 1 < r.sections.len() {
            ",\n"
        } else {
            "\n"
        });
    }
    o.push_str("  ]\n}\n");
    o
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::export::Table;
    use crate::report::{build, Options};

    const CSV: &str = "Date,MeterName,Cost,Currency\n09/01/2026,vm,10,EUR\n10/01/2026,vm,15,EUR\n10/02/2026,\"a,\"\"b\",5,EUR\n";

    fn rep(opts: Options) -> Report {
        build("in.csv", &Table::parse(CSV).unwrap(), &opts).unwrap()
    }

    fn plain() -> Report {
        rep(Options::default())
    }

    fn delta() -> Report {
        rep(Options {
            delta: true,
            ..Options::default()
        })
    }

    #[test]
    fn table_has_heading_and_totals() {
        let t = table(&plain());
        assert!(t.starts_with("azure-cost-top 0.1.0 - in.csv\n"), "{t}");
        assert!(t.contains("Period: all months   Rows: 3"));
        assert!(t.contains("Total: 30.00 EUR"));
    }

    #[test]
    fn table_lists_ranked_names_with_share() {
        let t = table(&plain());
        assert!(t.contains("Top meter costs (showing 2 of 2)"));
        assert!(t.contains("  1  vm"));
        assert!(t.contains("83.3%"));
    }

    #[test]
    fn table_delta_adds_columns() {
        let t = table(&delta());
        assert!(t.contains("Period: 2026-10 vs 2026-09"));
        assert!(t.contains("Previous"));
        assert!(t.contains("Delta%"));
        assert!(t.contains("+5.00"));
        assert!(t.contains("+50.0%"));
        assert!(t.contains("new"));
        assert!(t.contains("(was 10.00, +10.00, +100.0%)"));
    }

    #[test]
    fn table_without_delta_has_no_delta_columns() {
        let t = table(&plain());
        assert!(!t.contains("Delta"));
        assert!(!t.contains("Previous"));
    }

    #[test]
    fn table_lines_have_no_trailing_spaces() {
        for l in table(&delta()).lines() {
            assert_eq!(l, l.trim_end());
        }
    }

    #[test]
    fn csv_header_and_rows() {
        let c = csv(&plain());
        let mut lines = c.lines();
        assert_eq!(lines.next().unwrap(), "dimension,rank,name,cost,share_pct");
        assert_eq!(lines.next().unwrap(), "meter,1,vm,25.00,83.3");
    }

    #[test]
    fn csv_quotes_names() {
        assert!(csv(&plain()).contains("\"a,\"\"b\""));
    }

    #[test]
    fn csv_delta_has_extra_columns_and_blank_pct_for_new() {
        let c = csv(&delta());
        assert!(c.starts_with("dimension,rank,name,cost,share_pct,previous,delta,delta_pct\n"));
        assert!(c.contains("meter,1,vm,15.00,75.0,10.00,5.00,50.0\n"));
        assert!(c.contains(",5.00,25.0,0.00,5.00,\n"));
    }

    #[test]
    fn json_escapes_strings() {
        assert_eq!(json_str("a\"b\\c\n\u{1}"), "\"a\\\"b\\\\c\\n\\u0001\"");
    }

    #[test]
    fn json_has_expected_fields() {
        let j = json(&delta());
        for needle in [
            "\"tool\": \"azure-cost-top\"",
            "\"month\": \"2026-10\"",
            "\"previous_month\": \"2026-09\"",
            "\"currency\": \"EUR\"",
            "\"total\": 20.00",
            "\"previous_total\": 10.00",
            "\"dimension\": \"meter\"",
            "\"rank\": 1",
        ] {
            assert!(j.contains(needle), "missing {needle} in {j}");
        }
    }

    #[test]
    fn json_without_delta_uses_null() {
        let j = json(&plain());
        assert!(j.contains("\"month\": null"));
        assert!(j.contains("\"previous_total\": null"));
        assert!(j.contains("\"delta_pct\": null"));
    }

    #[test]
    fn json_braces_balance() {
        let j = json(&delta());
        assert_eq!(j.matches('{').count(), j.matches('}').count());
        assert_eq!(j.matches('[').count(), j.matches(']').count());
    }

    #[test]
    fn zero_total_has_no_share() {
        let t = Table::parse("Cost,MeterName\n0,a\n").unwrap();
        let r = build("t", &t, &Options::default()).unwrap();
        assert!(table(&r).contains("  -"));
        assert!(csv(&r).contains("meter,1,a,0.00,\n"));
    }

    #[test]
    fn tag_key_shows_in_title_and_csv_dimension() {
        let t = Table::parse("Cost,Tags\n1,\"\"\"env\"\": \"\"p\"\"\"\n").unwrap();
        let r = build(
            "t",
            &t,
            &Options {
                tag_key: Some("env".into()),
                ..Options::default()
            },
        )
        .unwrap();
        assert!(table(&r).contains("Top tag env costs"));
        assert!(csv(&r).contains("tag:env,1,p,"));
    }

    #[test]
    fn signed_formats() {
        assert_eq!(signed(Money::parse("1.5").unwrap()), "+1.50");
        assert_eq!(signed(Money::parse("-1.5").unwrap()), "-1.50");
        assert_eq!(signed(Money::parse("0.001").unwrap()), "0.00");
    }

    #[test]
    fn render_dispatches_on_format() {
        let r = plain();
        assert_eq!(render(&r, Format::Table), table(&r));
        assert_eq!(render(&r, Format::Csv), csv(&r));
        assert_eq!(render(&r, Format::Json), json(&r));
    }

    #[test]
    fn unchanged_entry_shows_zero_percent() {
        let t = Table::parse("Date,MeterName,Cost\n09/01/2026,a,4\n10/01/2026,a,4\n").unwrap();
        let r = build(
            "t",
            &t,
            &Options {
                delta: true,
                ..Options::default()
            },
        )
        .unwrap();
        assert!(table(&r).contains("0.0%"));
    }
}
