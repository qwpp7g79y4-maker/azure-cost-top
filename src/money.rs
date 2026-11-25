//! Exact decimal money in units of 1e-9, so sums never drift.

use std::fmt;
use std::iter::Sum;
use std::ops::{Add, Neg, Sub};

const SCALE_DIGITS: i64 = 9;
/// Absolute values at or above 1e30 nano-units are refused.
const LIMIT: i128 = 1_000_000_000_000_000_000_000_000_000_000;

/// An amount with nine decimals. The currency is tracked elsewhere.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Money(i128);

impl Money {
    pub const ZERO: Money = Money(0);

    pub fn from_nanos(n: i128) -> Money {
        Money(n)
    }

    pub fn nanos(self) -> i128 {
        self.0
    }

    pub fn is_zero(self) -> bool {
        self.0 == 0
    }

    /// Parse `12`, `-3.5`, `+0.25`, `.5`, `7.`, `1.2E-5`. No thousands separators.
    pub fn parse(input: &str) -> Result<Money, String> {
        let s = input.trim();
        if s.is_empty() {
            return Err("empty amount".to_string());
        }
        let (neg, rest) = match s.as_bytes()[0] {
            b'-' => (true, &s[1..]),
            b'+' => (false, &s[1..]),
            _ => (false, s),
        };
        let (mant, exp_part) = match rest.find(['e', 'E']) {
            Some(i) => (&rest[..i], Some(&rest[i + 1..])),
            None => (rest, None),
        };
        let (int_part, frac_part) = mant.split_once('.').unwrap_or((mant, ""));
        let bad = || format!("invalid amount '{}'", input.trim());
        if int_part.is_empty() && frac_part.is_empty() {
            return Err(bad());
        }
        if !int_part.bytes().all(|b| b.is_ascii_digit())
            || !frac_part.bytes().all(|b| b.is_ascii_digit())
        {
            return Err(bad());
        }
        let exp: i64 = match exp_part {
            None => 0,
            Some(e) => {
                let (eneg, digits) = match e.as_bytes().first() {
                    Some(b'-') => (true, &e[1..]),
                    Some(b'+') => (false, &e[1..]),
                    _ => (false, e),
                };
                if digits.is_empty()
                    || digits.len() > 4
                    || !digits.bytes().all(|b| b.is_ascii_digit())
                {
                    return Err(bad());
                }
                let v: i64 = digits.parse().map_err(|_| bad())?;
                if eneg {
                    -v
                } else {
                    v
                }
            }
        };
        let all_digits = format!("{int_part}{frac_part}");
        let trimmed = all_digits.trim_start_matches('0');
        if trimmed.len() > 30 {
            return Err(format!("amount '{}' has too many digits", input.trim()));
        }
        let mantissa: i128 = if trimmed.is_empty() {
            0
        } else {
            trimmed.parse().map_err(|_| bad())?
        };
        if mantissa == 0 {
            return Ok(Money::ZERO);
        }
        let shift = exp - frac_part.len() as i64 + SCALE_DIGITS;
        let value = if shift >= 0 {
            let p = 10i128
                .checked_pow(u32::try_from(shift).map_err(|_| out_of_range(input))?)
                .ok_or_else(|| out_of_range(input))?;
            mantissa.checked_mul(p).ok_or_else(|| out_of_range(input))?
        } else {
            let n = -shift;
            if n > 31 {
                0
            } else {
                let p = 10i128.pow(n as u32);
                let q = mantissa / p;
                let r = mantissa % p;
                if r * 2 >= p {
                    q + 1
                } else {
                    q
                }
            }
        };
        if value >= LIMIT {
            return Err(out_of_range(input));
        }
        Ok(Money(if neg { -value } else { value }))
    }

    /// Two decimals, rounded half away from zero. Never prints `-0.00`.
    pub fn fixed2(self) -> String {
        let abs = self.0.unsigned_abs();
        let cents = (abs + 5_000_000) / 10_000_000;
        let sign = if self.0 < 0 && cents != 0 { "-" } else { "" };
        format!("{sign}{}.{:02}", cents / 100, cents % 100)
    }

    /// Full precision, at least two decimals, trailing zeros trimmed beyond that.
    pub fn exact(self) -> String {
        let abs = self.0.unsigned_abs();
        let int = abs / 1_000_000_000;
        let frac = abs % 1_000_000_000;
        let mut f = format!("{frac:09}");
        while f.len() > 2 && f.ends_with('0') {
            f.pop();
        }
        let sign = if self.0 < 0 { "-" } else { "" };
        format!("{sign}{int}.{f}")
    }

    pub fn to_f64(self) -> f64 {
        self.0 as f64 / 1e9
    }

    pub fn abs(self) -> Money {
        Money(self.0.abs())
    }
}

fn out_of_range(input: &str) -> String {
    format!("amount '{}' is out of range", input.trim())
}

/// `num / den * 100`, or `None` when `den` is zero.
pub fn percent(num: Money, den: Money) -> Option<f64> {
    if den.is_zero() {
        None
    } else {
        Some(num.0 as f64 / den.0 as f64 * 100.0)
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.fixed2())
    }
}

impl Add for Money {
    type Output = Money;
    fn add(self, o: Money) -> Money {
        Money(self.0 + o.0)
    }
}

impl Sub for Money {
    type Output = Money;
    fn sub(self, o: Money) -> Money {
        Money(self.0 - o.0)
    }
}

impl Neg for Money {
    type Output = Money;
    fn neg(self) -> Money {
        Money(-self.0)
    }
}

impl Sum for Money {
    fn sum<I: Iterator<Item = Money>>(iter: I) -> Money {
        iter.fold(Money::ZERO, |a, b| a + b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn p(s: &str) -> Money {
        Money::parse(s).unwrap()
    }

    #[test]
    fn parses_integer() {
        assert_eq!(p("12").nanos(), 12_000_000_000);
    }

    #[test]
    fn parses_decimal() {
        assert_eq!(p("0.000123456").nanos(), 123_456);
    }

    #[test]
    fn parses_negative_and_plus() {
        assert_eq!(p("-3.5").nanos(), -3_500_000_000);
        assert_eq!(p("+3.5").nanos(), 3_500_000_000);
    }

    #[test]
    fn parses_leading_and_trailing_dot() {
        assert_eq!(p(".5").nanos(), 500_000_000);
        assert_eq!(p("7.").nanos(), 7_000_000_000);
    }

    #[test]
    fn parses_exponent() {
        assert_eq!(p("1.2E-5").nanos(), 12_000);
        assert_eq!(p("3e2").nanos(), 300_000_000_000);
    }

    #[test]
    fn rounds_beyond_nine_decimals_half_up() {
        assert_eq!(p("0.0000000005").nanos(), 1);
        assert_eq!(p("0.0000000004").nanos(), 0);
    }

    #[test]
    fn trims_whitespace() {
        assert_eq!(p("  1.5 ").nanos(), 1_500_000_000);
    }

    #[test]
    fn rejects_empty_and_garbage() {
        assert!(Money::parse("").is_err());
        assert!(Money::parse("  ").is_err());
        assert!(Money::parse("abc").is_err());
        assert!(Money::parse("1,5").is_err());
        assert!(Money::parse("1.2.3").is_err());
        assert!(Money::parse(".").is_err());
        assert!(Money::parse("-").is_err());
        assert!(Money::parse("1e").is_err());
        assert!(Money::parse("1e99999").is_err());
    }

    #[test]
    fn rejects_out_of_range() {
        assert!(Money::parse("1e40").is_err());
        assert!(Money::parse(&"9".repeat(31)).is_err());
    }

    #[test]
    fn zero_with_huge_exponent_is_zero() {
        assert_eq!(p("0e9999"), Money::ZERO);
    }

    #[test]
    fn fixed2_rounds_half_away_from_zero() {
        assert_eq!(p("1.005").fixed2(), "1.01");
        assert_eq!(p("-1.005").fixed2(), "-1.01");
        assert_eq!(p("1.004").fixed2(), "1.00");
    }

    #[test]
    fn fixed2_never_prints_negative_zero() {
        assert_eq!(p("-0.001").fixed2(), "0.00");
    }

    #[test]
    fn exact_keeps_precision_and_two_decimals() {
        assert_eq!(p("12").exact(), "12.00");
        assert_eq!(p("0.000123456").exact(), "0.000123456");
        assert_eq!(p("-2.5").exact(), "-2.50");
    }

    #[test]
    fn sums_are_exact() {
        let total: Money = (0..10).map(|_| p("0.1")).sum();
        assert_eq!(total, p("1"));
    }

    #[test]
    fn percent_handles_zero_denominator() {
        assert_eq!(percent(p("1"), Money::ZERO), None);
        assert_eq!(percent(p("1"), p("4")), Some(25.0));
    }

    #[test]
    fn display_uses_two_decimals() {
        assert_eq!(p("3.14159").to_string(), "3.14");
    }
}
