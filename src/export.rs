//! Reads an Azure Cost Management CSV export into typed rows.

use crate::csv;
use crate::date::Month;
use crate::error::Error;
use crate::money::Money;
use crate::tags;

const COST: &[&str] = &["CostInBillingCurrency", "PreTaxCost", "Cost", "CostUSD"];
const METER: &[&str] = &["MeterName", "Meter"];
const GROUP: &[&str] = &["ResourceGroup", "ResourceGroupName"];
const DATE: &[&str] = &[
    "Date",
    "UsageDate",
    "UsageDateTime",
    "BillingPeriodStartDate",
];
const TAGS: &[&str] = &["Tags", "Tag"];
const CURRENCY: &[&str] = &[
    "BillingCurrencyCode",
    "BillingCurrency",
    "Currency",
    "CurrencyCode",
];

/// One usage line of the export.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Row {
    pub line: usize,
    pub month: Option<Month>,
    pub meter: Option<String>,
    pub group: Option<String>,
    pub tags: Vec<(String, String)>,
    pub cost: Money,
    pub currency: Option<String>,
}

/// Which optional columns must be read.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Needs {
    pub meter: bool,
    pub group: bool,
    pub tags: bool,
    pub month: bool,
}

/// A parsed CSV with its header resolved.
#[derive(Debug, Clone)]
pub struct Table {
    records: Vec<csv::Record>,
    width: usize,
    cost: usize,
    meter: Option<usize>,
    group: Option<usize>,
    date: Option<usize>,
    tags: Option<usize>,
    currency: Option<usize>,
}

fn norm(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn find(header: &[String], aliases: &[&str]) -> Option<usize> {
    aliases
        .iter()
        .find_map(|a| header.iter().position(|h| *h == norm(a)))
}

impl Table {
    /// Parse the CSV text and locate the columns.
    pub fn parse(text: &str) -> Result<Table, Error> {
        let mut records = csv::parse(text).map_err(Error::input)?;
        if records.is_empty() {
            return Err(Error::input("the export is empty: no header line found"));
        }
        let header_rec = records.remove(0);
        let header: Vec<String> = header_rec.fields.iter().map(|h| norm(h)).collect();
        let cost = find(&header, COST).ok_or_else(|| {
            Error::input(format!(
                "no cost column found (looked for: {})",
                COST.join(", ")
            ))
        })?;
        Ok(Table {
            width: header.len(),
            cost,
            meter: find(&header, METER),
            group: find(&header, GROUP),
            date: find(&header, DATE),
            tags: find(&header, TAGS),
            currency: find(&header, CURRENCY),
            records,
        })
    }

    pub fn has_meter(&self) -> bool {
        self.meter.is_some()
    }

    pub fn has_group(&self) -> bool {
        self.group.is_some()
    }

    pub fn has_tags(&self) -> bool {
        self.tags.is_some()
    }

    pub fn has_date(&self) -> bool {
        self.date.is_some()
    }

    /// Number of data lines (header excluded).
    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Convert the data lines. Only the columns in `needs` are parsed strictly.
    pub fn rows(&self, needs: Needs) -> Result<Vec<Row>, Error> {
        let missing = |what: &str, aliases: &[&str]| {
            Error::input(format!(
                "the export has no {what} column (looked for: {})",
                aliases.join(", ")
            ))
        };
        if needs.meter && self.meter.is_none() {
            return Err(missing("meter", METER));
        }
        if needs.group && self.group.is_none() {
            return Err(missing("resource group", GROUP));
        }
        if needs.tags && self.tags.is_none() {
            return Err(missing("tags", TAGS));
        }
        if needs.month && self.date.is_none() {
            return Err(missing("date", DATE));
        }
        let mut rows = Vec::with_capacity(self.records.len());
        for rec in &self.records {
            if rec.fields.len() != self.width {
                return Err(Error::input(format!(
                    "line {}: {} fields, but the header has {}",
                    rec.line,
                    rec.fields.len(),
                    self.width
                )));
            }
            let cell = |i: usize| rec.fields[i].trim();
            let cost = Money::parse(cell(self.cost))
                .map_err(|e| Error::input(format!("line {}: {e}", rec.line)))?;
            let month = match self.date {
                Some(i) if needs.month => Some(
                    Month::from_date(cell(i))
                        .map_err(|e| Error::date(format!("line {}: {e}", rec.line)))?,
                ),
                _ => None,
            };
            let text =
                |idx: Option<usize>| idx.map(cell).filter(|s| !s.is_empty()).map(str::to_string);
            let tag_pairs = match self.tags {
                Some(i) if needs.tags => tags::parse(cell(i))
                    .map_err(|e| Error::input(format!("line {}: {e}", rec.line)))?,
                _ => Vec::new(),
            };
            rows.push(Row {
                line: rec.line,
                month,
                meter: text(self.meter),
                group: text(self.group),
                tags: tag_pairs,
                cost,
                currency: text(self.currency),
            });
        }
        Ok(rows)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: Needs = Needs {
        meter: true,
        group: true,
        tags: true,
        month: true,
    };

    fn table(s: &str) -> Table {
        Table::parse(s).unwrap()
    }

    #[test]
    fn finds_columns_case_and_space_insensitively() {
        let t = table("date,Meter Name,RESOURCE GROUP,tags,cost in billing currency\n");
        assert!(t.has_meter() && t.has_group() && t.has_tags() && t.has_date());
    }

    #[test]
    fn uses_alias_priority_for_cost() {
        let t = table("Cost,CostInBillingCurrency\n1,2\n");
        let rows = t.rows(Needs::default()).unwrap();
        assert_eq!(rows[0].cost, Money::parse("2").unwrap());
    }

    #[test]
    fn accepts_legacy_column_names() {
        let t = table(
            "UsageDateTime,Meter,ResourceGroupName,PreTaxCost,Currency\n2026-10-01,vm,rg,1.5,EUR\n",
        );
        let r = &t.rows(ALL_NO_TAGS).unwrap()[0];
        assert_eq!(r.meter.as_deref(), Some("vm"));
        assert_eq!(r.group.as_deref(), Some("rg"));
        assert_eq!(r.currency.as_deref(), Some("EUR"));
        assert_eq!(r.month.unwrap().to_string(), "2026-10");
    }

    const ALL_NO_TAGS: Needs = Needs {
        meter: true,
        group: true,
        tags: false,
        month: true,
    };

    #[test]
    fn missing_cost_column_is_an_error() {
        let e = Table::parse("Date,MeterName\n").unwrap_err();
        assert!(e.to_string().contains("no cost column"));
    }

    #[test]
    fn empty_input_is_an_error() {
        let e = Table::parse("").unwrap_err();
        assert!(e.to_string().contains("empty"));
    }

    #[test]
    fn header_only_has_no_rows() {
        let t = table("Cost\n");
        assert!(t.is_empty());
        assert_eq!(t.rows(Needs::default()).unwrap().len(), 0);
    }

    #[test]
    fn counts_data_lines() {
        assert_eq!(table("Cost\n1\n2\n").len(), 2);
    }

    #[test]
    fn requested_but_missing_columns_error() {
        let t = table("Cost\n1\n");
        for needs in [
            Needs {
                meter: true,
                ..Needs::default()
            },
            Needs {
                group: true,
                ..Needs::default()
            },
            Needs {
                tags: true,
                ..Needs::default()
            },
            Needs {
                month: true,
                ..Needs::default()
            },
        ] {
            assert!(t.rows(needs).is_err());
        }
    }

    #[test]
    fn short_and_long_rows_error_with_line_number() {
        let e = table("Cost,Date\n1\n").rows(Needs::default()).unwrap_err();
        assert!(e.to_string().starts_with("line 2:"), "{e}");
        let e = table("Cost\n1,2\n").rows(Needs::default()).unwrap_err();
        assert!(e.to_string().contains("header has 1"), "{e}");
    }

    #[test]
    fn bad_cost_names_the_line() {
        let e = table("Cost\n1\nabc\n").rows(Needs::default()).unwrap_err();
        assert!(e.to_string().starts_with("line 3:"), "{e}");
    }

    #[test]
    fn empty_cost_cell_is_an_error() {
        assert!(table("Cost,Date\n,01/01/2026\n")
            .rows(Needs::default())
            .is_err());
    }

    #[test]
    fn bad_date_only_matters_when_months_are_needed() {
        let t = table("Cost,Date\n1,nonsense\n");
        assert!(t.rows(Needs::default()).is_ok());
        let e = t
            .rows(Needs {
                month: true,
                ..Needs::default()
            })
            .unwrap_err();
        assert!(e.to_string().contains("invalid date"));
        assert_eq!(e.exit_code(), 2);
    }

    #[test]
    fn bad_tags_only_matter_when_tags_are_needed() {
        let t = table("Cost,Tags\n1,not json\n");
        assert!(t.rows(Needs::default()).is_ok());
        assert!(t
            .rows(Needs {
                tags: true,
                ..Needs::default()
            })
            .is_err());
    }

    #[test]
    fn parses_tags_from_a_quoted_cell() {
        let t = table("Cost,Tags\n1,\"\"\"env\"\": \"\"prod\"\"\"\n");
        let rows = t
            .rows(Needs {
                tags: true,
                ..Needs::default()
            })
            .unwrap();
        assert_eq!(rows[0].tags, vec![("env".to_string(), "prod".to_string())]);
    }

    #[test]
    fn blank_text_cells_become_none() {
        let t = table("Cost,MeterName,ResourceGroup\n1,, \n");
        let r = &t
            .rows(Needs {
                meter: true,
                group: true,
                ..Needs::default()
            })
            .unwrap()[0];
        assert_eq!(r.meter, None);
        assert_eq!(r.group, None);
    }

    #[test]
    fn full_row_roundtrip() {
        let t = table("Date,MeterName,ResourceGroup,Tags,Cost,Currency\n10/03/2026,D2s,rg-a,\"\"\"a\"\":\"\"b\"\"\",-2.5,EUR\n");
        let r = &t.rows(ALL).unwrap()[0];
        assert_eq!(r.line, 2);
        assert_eq!(r.cost, Money::parse("-2.5").unwrap());
        assert_eq!(r.tags.len(), 1);
    }
}
