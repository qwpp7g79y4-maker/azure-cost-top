//! Builds the ranked sections of a report from an export.

use std::collections::{BTreeMap, BTreeSet};

use crate::date::Month;
use crate::error::Error;
use crate::export::{Needs, Row, Table};
use crate::money::Money;

pub const NO_METER: &str = "(none)";
pub const NO_GROUP: &str = "(none)";
pub const UNTAGGED: &str = "(untagged)";

/// What a section groups by.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dimension {
    Meter,
    Group,
    Tag,
}

impl Dimension {
    /// Stable name used in csv and json output.
    pub fn key(self) -> &'static str {
        match self {
            Dimension::Meter => "meter",
            Dimension::Group => "resource_group",
            Dimension::Tag => "tag",
        }
    }

    /// Singular label used in the table heading.
    pub fn label(self) -> &'static str {
        match self {
            Dimension::Meter => "meter",
            Dimension::Group => "resource group",
            Dimension::Tag => "tag",
        }
    }
}

/// Settings of one report.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Options {
    /// `None` means every dimension the export has a column for.
    pub dimensions: Option<Vec<Dimension>>,
    /// Group the tag section by the value of this tag name.
    pub tag_key: Option<String>,
    pub top: usize,
    pub month: Option<Month>,
    pub delta: bool,
    pub vs: Option<Month>,
    pub currency: Option<String>,
}

impl Default for Options {
    fn default() -> Options {
        Options {
            dimensions: None,
            tag_key: None,
            top: 10,
            month: None,
            delta: false,
            vs: None,
            currency: None,
        }
    }
}

/// One ranked line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Entry {
    pub name: String,
    pub cost: Money,
    /// Cost in the comparison month, present in delta mode.
    pub previous: Option<Money>,
}

impl Entry {
    pub fn delta(&self) -> Option<Money> {
        self.previous.map(|p| self.cost - p)
    }
}

/// The top entries of one dimension.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Section {
    pub dimension: Dimension,
    pub tag_key: Option<String>,
    pub entries: Vec<Entry>,
    /// How many distinct names exist before cutting to the top.
    pub distinct: usize,
}

/// Everything the renderers need.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    pub input: String,
    pub rows: usize,
    pub currency: Option<String>,
    pub month: Option<Month>,
    pub previous_month: Option<Month>,
    pub total: Money,
    pub previous_total: Option<Money>,
    pub sections: Vec<Section>,
}

/// Build a report for `table`. `input` is only a label for the output.
pub fn build(input: &str, table: &Table, opts: &Options) -> Result<Report, Error> {
    if opts.top == 0 {
        return Err(Error::usage("--top must be at least 1"));
    }
    if let Some(k) = &opts.tag_key {
        if k.trim().is_empty() {
            return Err(Error::usage("--tag-key must not be empty"));
        }
        if let Some(d) = &opts.dimensions {
            if !d.contains(&Dimension::Tag) {
                return Err(Error::usage("--tag-key needs the tag dimension in --by"));
            }
        }
    }
    let dims = resolve_dimensions(table, opts)?;
    let want_months = opts.month.is_some() || opts.delta || opts.vs.is_some();
    let needs = Needs {
        meter: dims.contains(&Dimension::Meter),
        group: dims.contains(&Dimension::Group),
        tags: dims.contains(&Dimension::Tag),
        month: want_months,
    };
    let mut rows = table.rows(needs)?;
    if table.is_empty() {
        return Err(Error::input("the export has a header but no data lines"));
    }

    // Currency: never add amounts of different currencies.
    if let Some(want) = &opts.currency {
        rows.retain(|r| {
            r.currency
                .as_deref()
                .is_some_and(|c| c.eq_ignore_ascii_case(want))
        });
        if rows.is_empty() {
            return Err(Error::input(format!("no rows with currency {want}")));
        }
    }
    let currencies: BTreeSet<String> = rows
        .iter()
        .filter_map(|r| r.currency.as_ref().map(|c| c.to_uppercase()))
        .collect();
    if currencies.len() > 1 {
        return Err(Error::input(format!(
            "the export mixes currencies ({}); pick one with --currency",
            currencies.into_iter().collect::<Vec<_>>().join(", ")
        )));
    }
    let currency = currencies.into_iter().next();

    let delta = opts.delta || opts.vs.is_some();
    let (month, previous_month) = if want_months {
        let current = match opts.month {
            Some(m) => m,
            None => rows
                .iter()
                .filter_map(|r| r.month)
                .max()
                .ok_or_else(|| Error::input("no rows to pick a month from"))?,
        };
        let previous = if delta {
            let p = match opts.vs {
                Some(p) => p,
                None => current
                    .prev()
                    .ok_or_else(|| Error::usage("there is no month before the chosen month"))?,
            };
            if p == current {
                return Err(Error::usage("--vs must differ from the chosen month"));
            }
            Some(p)
        } else {
            None
        };
        (Some(current), previous)
    } else {
        (None, None)
    };

    let current_rows: Vec<&Row> = rows
        .iter()
        .filter(|r| month.is_none() || r.month == month)
        .collect();
    if current_rows.is_empty() {
        return Err(Error::input(match month {
            Some(m) => format!("no rows in month {m}"),
            None => "no rows".to_string(),
        }));
    }
    let previous_rows: Option<Vec<&Row>> =
        previous_month.map(|pm| rows.iter().filter(|r| r.month == Some(pm)).collect());

    let total: Money = current_rows.iter().map(|r| r.cost).sum();
    let previous_total = previous_rows
        .as_ref()
        .map(|p| p.iter().map(|r| r.cost).sum::<Money>());

    let sections = dims
        .iter()
        .map(|&dim| {
            section(
                dim,
                opts.tag_key.as_deref(),
                &current_rows,
                previous_rows.as_deref(),
                opts.top,
            )
        })
        .collect();

    Ok(Report {
        input: input.to_string(),
        rows: current_rows.len(),
        currency,
        month,
        previous_month,
        total,
        previous_total,
        sections,
    })
}

fn resolve_dimensions(table: &Table, opts: &Options) -> Result<Vec<Dimension>, Error> {
    match &opts.dimensions {
        Some(d) => Ok(d.clone()),
        None => {
            let mut dims = Vec::new();
            if table.has_meter() {
                dims.push(Dimension::Meter);
            }
            if table.has_group() {
                dims.push(Dimension::Group);
            }
            if table.has_tags() {
                dims.push(Dimension::Tag);
            } else if opts.tag_key.is_some() {
                return Err(Error::input(
                    "--tag-key needs a Tags column, the export has none",
                ));
            }
            if dims.is_empty() {
                return Err(Error::input(
                    "the export has no meter, resource group or tags column to group by",
                ));
            }
            Ok(dims)
        }
    }
}

/// Names a row falls under for one dimension.
fn names(row: &Row, dim: Dimension, tag_key: Option<&str>) -> Vec<String> {
    match dim {
        Dimension::Meter => vec![row.meter.clone().unwrap_or_else(|| NO_METER.to_string())],
        // Azure treats resource group names case-insensitively.
        Dimension::Group => vec![row
            .group
            .as_ref()
            .map(|g| g.to_lowercase())
            .unwrap_or_else(|| NO_GROUP.to_string())],
        Dimension::Tag => match tag_key {
            Some(key) => {
                let found = row
                    .tags
                    .iter()
                    .find(|(k, _)| k.eq_ignore_ascii_case(key.trim()))
                    .map(|(_, v)| v.clone());
                vec![found.unwrap_or_else(|| UNTAGGED.to_string())]
            }
            None => {
                if row.tags.is_empty() {
                    vec![UNTAGGED.to_string()]
                } else {
                    let mut seen = BTreeSet::new();
                    row.tags
                        .iter()
                        .map(|(k, v)| format!("{}={v}", k.to_lowercase()))
                        .filter(|n| seen.insert(n.clone()))
                        .collect()
                }
            }
        },
    }
}

fn sums(rows: &[&Row], dim: Dimension, tag_key: Option<&str>) -> BTreeMap<String, Money> {
    let mut map: BTreeMap<String, Money> = BTreeMap::new();
    for row in rows {
        for name in names(row, dim, tag_key) {
            let slot = map.entry(name).or_insert(Money::ZERO);
            *slot = *slot + row.cost;
        }
    }
    map
}

fn section(
    dimension: Dimension,
    tag_key: Option<&str>,
    current: &[&Row],
    previous: Option<&[&Row]>,
    top: usize,
) -> Section {
    let cur = sums(current, dimension, tag_key);
    let prev = previous.map(|p| sums(p, dimension, tag_key));
    let mut keys: BTreeSet<&String> = cur.keys().collect();
    if let Some(p) = &prev {
        keys.extend(p.keys());
    }
    let mut entries: Vec<Entry> = keys
        .into_iter()
        .map(|k| Entry {
            name: k.clone(),
            cost: cur.get(k).copied().unwrap_or(Money::ZERO),
            previous: prev
                .as_ref()
                .map(|p| p.get(k).copied().unwrap_or(Money::ZERO)),
        })
        .collect();
    let distinct = entries.len();
    entries.sort_by(|a, b| b.cost.cmp(&a.cost).then_with(|| a.name.cmp(&b.name)));
    entries.truncate(top);
    Section {
        dimension,
        tag_key: if dimension == Dimension::Tag {
            tag_key.map(str::to_string)
        } else {
            None
        },
        entries,
        distinct,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CSV: &str = "Date,MeterName,ResourceGroup,Tags,Cost,Currency\n\
09/05/2026,vm,RG-A,\"\"\"env\"\": \"\"prod\"\",\"\"team\"\": \"\"x\"\"\",10,EUR\n\
09/06/2026,disk,rg-b,,5,EUR\n\
10/05/2026,vm,rg-a,\"\"\"env\"\": \"\"prod\"\"\",30,EUR\n\
10/06/2026,disk,rg-b,\"\"\"Env\"\": \"\"dev\"\"\",2.5,EUR\n\
10/07/2026,net,rg-c,,1,EUR\n";

    fn run(opts: Options) -> Result<Report, Error> {
        build("t.csv", &Table::parse(CSV).unwrap(), &opts)
    }

    fn names_of(s: &Section) -> Vec<&str> {
        s.entries.iter().map(|e| e.name.as_str()).collect()
    }

    #[test]
    fn default_covers_every_available_dimension() {
        let r = run(Options::default()).unwrap();
        assert_eq!(r.sections.len(), 3);
        assert_eq!(r.sections[0].dimension, Dimension::Meter);
        assert_eq!(r.sections[1].dimension, Dimension::Group);
        assert_eq!(r.sections[2].dimension, Dimension::Tag);
    }

    #[test]
    fn total_and_rows_cover_the_whole_file_without_month() {
        let r = run(Options::default()).unwrap();
        assert_eq!(r.rows, 5);
        assert_eq!(r.total, Money::parse("48.5").unwrap());
        assert_eq!(r.month, None);
        assert_eq!(r.previous_total, None);
    }

    #[test]
    fn meters_are_sorted_by_cost_descending() {
        let r = run(Options::default()).unwrap();
        assert_eq!(names_of(&r.sections[0]), vec!["vm", "disk", "net"]);
        assert_eq!(r.sections[0].entries[0].cost, Money::parse("40").unwrap());
    }

    #[test]
    fn groups_merge_case_insensitively() {
        let r = run(Options::default()).unwrap();
        assert_eq!(names_of(&r.sections[1]), vec!["rg-a", "rg-b", "rg-c"]);
        assert_eq!(r.sections[1].entries[0].cost, Money::parse("40").unwrap());
    }

    #[test]
    fn top_cuts_the_list_but_keeps_distinct_count() {
        let r = run(Options {
            top: 2,
            ..Options::default()
        })
        .unwrap();
        assert_eq!(r.sections[0].entries.len(), 2);
        assert_eq!(r.sections[0].distinct, 3);
    }

    #[test]
    fn top_zero_is_a_usage_error() {
        let e = run(Options {
            top: 0,
            ..Options::default()
        })
        .unwrap_err();
        assert!(matches!(e, Error::Usage(_)));
    }

    #[test]
    fn ties_break_by_name() {
        let t = Table::parse("Cost,MeterName\n1,b\n1,a\n").unwrap();
        let r = build("t", &t, &Options::default()).unwrap();
        assert_eq!(names_of(&r.sections[0]), vec!["a", "b"]);
    }

    #[test]
    fn tag_pairs_count_a_row_under_each_pair() {
        let r = run(Options::default()).unwrap();
        let tag = &r.sections[2];
        assert_eq!(
            names_of(tag),
            vec!["env=prod", "team=x", "(untagged)", "env=dev"]
        );
        assert_eq!(tag.entries[0].cost, Money::parse("40").unwrap());
        let team = tag.entries.iter().find(|e| e.name == "team=x").unwrap();
        assert_eq!(team.cost, Money::parse("10").unwrap());
    }

    #[test]
    fn tag_key_groups_by_value_and_ignores_key_case() {
        let r = run(Options {
            tag_key: Some("ENV".into()),
            ..Options::default()
        })
        .unwrap();
        let tag = &r.sections[2];
        assert_eq!(names_of(tag), vec!["prod", "(untagged)", "dev"]);
        assert_eq!(tag.tag_key.as_deref(), Some("ENV"));
        assert_eq!(tag.entries[0].cost, Money::parse("40").unwrap());
        assert_eq!(tag.entries[2].cost, Money::parse("2.5").unwrap());
    }

    #[test]
    fn explicit_dimensions_are_kept_in_order() {
        let r = run(Options {
            dimensions: Some(vec![Dimension::Group, Dimension::Meter]),
            ..Options::default()
        })
        .unwrap();
        assert_eq!(r.sections[0].dimension, Dimension::Group);
        assert_eq!(r.sections[1].dimension, Dimension::Meter);
    }

    #[test]
    fn tag_key_without_tag_dimension_is_a_usage_error() {
        let e = run(Options {
            dimensions: Some(vec![Dimension::Meter]),
            tag_key: Some("env".into()),
            ..Options::default()
        })
        .unwrap_err();
        assert!(matches!(e, Error::Usage(_)));
    }

    #[test]
    fn month_filters_rows_and_total() {
        let r = run(Options {
            month: Some(Month::parse("2026-09").unwrap()),
            ..Options::default()
        })
        .unwrap();
        assert_eq!(r.rows, 2);
        assert_eq!(r.total, Money::parse("15").unwrap());
        assert_eq!(r.previous_month, None);
    }

    #[test]
    fn month_without_rows_is_an_error() {
        let e = run(Options {
            month: Some(Month::parse("2025-01").unwrap()),
            ..Options::default()
        })
        .unwrap_err();
        assert!(e.to_string().contains("no rows in month 2025-01"));
    }

    #[test]
    fn delta_defaults_to_latest_month_vs_the_month_before() {
        let r = run(Options {
            delta: true,
            ..Options::default()
        })
        .unwrap();
        assert_eq!(r.month.unwrap().to_string(), "2026-10");
        assert_eq!(r.previous_month.unwrap().to_string(), "2026-09");
        assert_eq!(r.total, Money::parse("33.5").unwrap());
        assert_eq!(r.previous_total, Some(Money::parse("15").unwrap()));
    }

    #[test]
    fn delta_entries_carry_previous_and_delta() {
        let r = run(Options {
            delta: true,
            ..Options::default()
        })
        .unwrap();
        let vm = &r.sections[0].entries[0];
        assert_eq!(vm.name, "vm");
        assert_eq!(vm.previous, Some(Money::parse("10").unwrap()));
        assert_eq!(vm.delta(), Some(Money::parse("20").unwrap()));
    }

    #[test]
    fn delta_includes_new_and_vanished_names() {
        let t =
            Table::parse("Date,MeterName,Cost\n09/01/2026,gone,4\n10/01/2026,fresh,3\n").unwrap();
        let r = build(
            "t",
            &t,
            &Options {
                delta: true,
                ..Options::default()
            },
        )
        .unwrap();
        let s = &r.sections[0];
        assert_eq!(names_of(s), vec!["fresh", "gone"]);
        assert_eq!(s.entries[0].previous, Some(Money::ZERO));
        assert_eq!(s.entries[1].cost, Money::ZERO);
        assert_eq!(s.entries[1].delta(), Some(Money::parse("-4").unwrap()));
    }

    #[test]
    fn vs_sets_the_comparison_month_and_implies_delta() {
        let r = run(Options {
            month: Some(Month::parse("2026-10").unwrap()),
            vs: Some(Month::parse("2026-09").unwrap()),
            ..Options::default()
        })
        .unwrap();
        assert_eq!(r.previous_month.unwrap().to_string(), "2026-09");
    }

    #[test]
    fn vs_equal_to_month_is_a_usage_error() {
        let m = Month::parse("2026-10").unwrap();
        let e = run(Options {
            month: Some(m),
            vs: Some(m),
            ..Options::default()
        })
        .unwrap_err();
        assert!(matches!(e, Error::Usage(_)));
    }

    #[test]
    fn delta_with_empty_previous_month_is_all_new() {
        let r = run(Options {
            month: Some(Month::parse("2026-10").unwrap()),
            vs: Some(Month::parse("2020-01").unwrap()),
            ..Options::default()
        })
        .unwrap();
        assert_eq!(r.previous_total, Some(Money::ZERO));
    }

    #[test]
    fn delta_needs_a_date_column() {
        let t = Table::parse("Cost,MeterName\n1,a\n").unwrap();
        let e = build(
            "t",
            &t,
            &Options {
                delta: true,
                ..Options::default()
            },
        )
        .unwrap_err();
        assert!(e.to_string().contains("no date column"));
    }

    #[test]
    fn mixed_currencies_are_refused() {
        let t = Table::parse("Cost,MeterName,Currency\n1,a,EUR\n2,a,USD\n").unwrap();
        let e = build("t", &t, &Options::default()).unwrap_err();
        assert!(e.to_string().contains("EUR, USD"));
    }

    #[test]
    fn currency_filter_selects_one_currency() {
        let t = Table::parse("Cost,MeterName,Currency\n1,a,EUR\n2,a,usd\n").unwrap();
        let r = build(
            "t",
            &t,
            &Options {
                currency: Some("USD".into()),
                ..Options::default()
            },
        )
        .unwrap();
        assert_eq!(r.total, Money::parse("2").unwrap());
        assert_eq!(r.currency.as_deref(), Some("USD"));
    }

    #[test]
    fn currency_filter_without_match_is_an_error() {
        let t = Table::parse("Cost,MeterName,Currency\n1,a,EUR\n").unwrap();
        assert!(build(
            "t",
            &t,
            &Options {
                currency: Some("GBP".into()),
                ..Options::default()
            }
        )
        .is_err());
    }

    #[test]
    fn no_currency_column_means_no_currency() {
        let t = Table::parse("Cost,MeterName\n1,a\n").unwrap();
        assert_eq!(build("t", &t, &Options::default()).unwrap().currency, None);
    }

    #[test]
    fn missing_meter_and_group_get_placeholders() {
        let t = Table::parse("Cost,MeterName,ResourceGroup\n1,,\n").unwrap();
        let r = build("t", &t, &Options::default()).unwrap();
        assert_eq!(r.sections[0].entries[0].name, NO_METER);
        assert_eq!(r.sections[1].entries[0].name, NO_GROUP);
    }

    #[test]
    fn negative_credits_reduce_totals_and_rank_last() {
        let t = Table::parse("Cost,MeterName\n5,a\n-2,credit\n").unwrap();
        let r = build("t", &t, &Options::default()).unwrap();
        assert_eq!(r.total, Money::parse("3").unwrap());
        assert_eq!(names_of(&r.sections[0]), vec!["a", "credit"]);
    }

    #[test]
    fn header_only_export_is_an_error() {
        let t = Table::parse("Cost,MeterName\n").unwrap();
        assert!(build("t", &t, &Options::default()).is_err());
    }

    #[test]
    fn no_groupable_column_is_an_error() {
        let t = Table::parse("Cost\n1\n").unwrap();
        let e = build("t", &t, &Options::default()).unwrap_err();
        assert!(e.to_string().contains("to group by"));
    }

    #[test]
    fn explicit_dimension_without_column_is_an_error() {
        let t = Table::parse("Cost,MeterName\n1,a\n").unwrap();
        let o = Options {
            dimensions: Some(vec![Dimension::Group]),
            ..Options::default()
        };
        assert!(build("t", &t, &o).is_err());
    }

    #[test]
    fn tag_key_without_tags_column_is_an_error_in_auto_mode() {
        let t = Table::parse("Cost,MeterName\n1,a\n").unwrap();
        let o = Options {
            tag_key: Some("env".into()),
            ..Options::default()
        };
        assert!(build("t", &t, &o).is_err());
    }
}
