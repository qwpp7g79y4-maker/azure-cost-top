//! Billing months and the date formats Azure exports use.

use std::fmt;

/// A calendar month, ordered by year then month.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Month {
    year: i32,
    month: u8,
}

impl Month {
    pub fn new(year: i32, month: u8) -> Option<Month> {
        if (1..=9999).contains(&year) && (1..=12).contains(&month) {
            Some(Month { year, month })
        } else {
            None
        }
    }

    /// Parse `YYYY-MM`.
    pub fn parse(s: &str) -> Result<Month, String> {
        let bad = || format!("invalid month '{s}', expected YYYY-MM");
        let (y, m) = s.trim().split_once('-').ok_or_else(bad)?;
        if y.len() != 4 || !(1..=2).contains(&m.len()) {
            return Err(bad());
        }
        let year = digits(y).ok_or_else(bad)?;
        let month = digits(m).ok_or_else(bad)?;
        Month::new(year as i32, month as u8).ok_or_else(bad)
    }

    /// The month of a date cell such as `10/31/2026`, `2026-10-31`,
    /// `2026-10-31T00:00:00Z`, `2026/10/31` or `20261031`.
    pub fn from_date(s: &str) -> Result<Month, String> {
        let bad = || format!("invalid date '{}'", s.trim());
        let s = s.trim();
        let date = s.split(['T', ' ']).next().unwrap_or("");
        let (y, m, d): (u32, u32, u32) = if date.contains('-') || date.contains('/') {
            let sep = if date.contains('-') { '-' } else { '/' };
            let parts: Vec<&str> = date.split(sep).collect();
            if parts.len() != 3 {
                return Err(bad());
            }
            let (y, m, d) = if parts[0].len() == 4 {
                (parts[0], parts[1], parts[2])
            } else if parts[2].len() == 4 && sep == '/' {
                (parts[2], parts[0], parts[1])
            } else {
                return Err(bad());
            };
            if !(1..=2).contains(&m.len()) || !(1..=2).contains(&d.len()) {
                return Err(bad());
            }
            (
                digits(y).ok_or_else(bad)?,
                digits(m).ok_or_else(bad)?,
                digits(d).ok_or_else(bad)?,
            )
        } else if date.len() == 8 && date.is_ascii() {
            // Only ASCII is cut by byte; any other cell is not a date.
            (
                digits(&date[0..4]).ok_or_else(bad)?,
                digits(&date[4..6]).ok_or_else(bad)?,
                digits(&date[6..8]).ok_or_else(bad)?,
            )
        } else {
            return Err(bad());
        };
        let month = Month::new(y as i32, m as u8).ok_or_else(bad)?;
        if d == 0 || d > days_in_month(month.year, month.month) {
            return Err(bad());
        }
        Ok(month)
    }

    /// The month before this one; `None` before year 1.
    pub fn prev(self) -> Option<Month> {
        if self.month == 1 {
            Month::new(self.year - 1, 12)
        } else {
            Month::new(self.year, self.month - 1)
        }
    }
}

fn digits(s: &str) -> Option<u32> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        None
    } else {
        s.parse().ok()
    }
}

fn days_in_month(year: i32, month: u8) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        _ => {
            if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 {
                29
            } else {
                28
            }
        }
    }
}

impl fmt::Display for Month {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}", self.year, self.month)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m(s: &str) -> Month {
        Month::from_date(s).unwrap()
    }

    #[test]
    fn parses_us_dates() {
        assert_eq!(m("10/31/2026").to_string(), "2026-10");
        assert_eq!(m("1/5/2026").to_string(), "2026-01");
    }

    #[test]
    fn parses_iso_dates() {
        assert_eq!(m("2026-10-31").to_string(), "2026-10");
    }

    #[test]
    fn parses_iso_with_time() {
        assert_eq!(m("2026-10-31T23:59:59Z").to_string(), "2026-10");
        assert_eq!(m("2026-10-31 00:00:00").to_string(), "2026-10");
    }

    #[test]
    fn parses_slash_year_first_and_compact() {
        assert_eq!(m("2026/03/01").to_string(), "2026-03");
        assert_eq!(m("20260301").to_string(), "2026-03");
    }

    #[test]
    fn rejects_impossible_dates() {
        assert!(Month::from_date("02/30/2026").is_err());
        assert!(Month::from_date("13/01/2026").is_err());
        assert!(Month::from_date("2026-00-10").is_err());
        assert!(Month::from_date("2026-04-31").is_err());
        assert!(Month::from_date("00/10/2026").is_err());
    }

    #[test]
    fn leap_years() {
        assert!(Month::from_date("02/29/2028").is_ok());
        assert!(Month::from_date("02/29/2026").is_err());
        assert!(Month::from_date("02/29/2100").is_err());
        assert!(Month::from_date("02/29/2000").is_ok());
    }

    #[test]
    fn multibyte_cells_are_errors_not_panics() {
        // 8 bytes, no separator, a character boundary inside every cut.
        assert_eq!(
            "a\u{e9}\u{e9}\u{e9}b".len(),
            8,
            "the cell must be 8 bytes long"
        );
        assert!(Month::from_date("a\u{e9}\u{e9}\u{e9}b").is_err());
        assert!(Month::from_date("\u{e9}\u{e9}\u{e9}\u{e9}").is_err());
        assert!(Month::from_date("2026\u{e9}\u{e9}01").is_err());
        assert!(Month::from_date("\u{1F600}\u{1F600}").is_err());
        assert!(Month::from_date("2026-1\u{e9}-01").is_err());
    }

    #[test]
    fn rejects_garbage() {
        assert!(Month::from_date("").is_err());
        assert!(Month::from_date("yesterday").is_err());
        assert!(Month::from_date("10-31-2026").is_err());
        assert!(Month::from_date("2026-10").is_err());
    }

    #[test]
    fn parse_month_ok() {
        assert_eq!(Month::parse("2026-09").unwrap().to_string(), "2026-09");
        assert_eq!(Month::parse("2026-9").unwrap().to_string(), "2026-09");
    }

    #[test]
    fn parse_month_rejects() {
        assert!(Month::parse("2026").is_err());
        assert!(Month::parse("2026-13").is_err());
        assert!(Month::parse("26-01").is_err());
        assert!(Month::parse("2026-00").is_err());
        assert!(Month::parse("abcd-01").is_err());
    }

    #[test]
    fn prev_wraps_the_year() {
        assert_eq!(
            Month::parse("2026-01").unwrap().prev().unwrap().to_string(),
            "2025-12"
        );
        assert_eq!(
            Month::parse("2026-05").unwrap().prev().unwrap().to_string(),
            "2026-04"
        );
    }

    #[test]
    fn prev_before_year_one_is_none() {
        assert!(Month::new(1, 1).unwrap().prev().is_none());
    }

    #[test]
    fn months_order_chronologically() {
        assert!(Month::parse("2025-12").unwrap() < Month::parse("2026-01").unwrap());
    }
}
