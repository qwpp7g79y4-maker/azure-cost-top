//! Azure writes the Tags column as JSON pairs, with or without outer braces:
//! `"env": "prod", "team": "web"` or `{"env":"prod"}`.

/// Parse the Tags cell into `(name, value)` pairs, in order.
pub fn parse(cell: &str) -> Result<Vec<(String, String)>, String> {
    let mut s = cell.trim();
    if let Some(inner) = s.strip_prefix('{') {
        s = inner
            .trim_end()
            .strip_suffix('}')
            .ok_or("unclosed '{' in tags")?;
    }
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    let mut out = Vec::new();
    loop {
        skip_ws(&chars, &mut i);
        if i >= chars.len() {
            break;
        }
        let key = quoted(&chars, &mut i)?;
        skip_ws(&chars, &mut i);
        if chars.get(i) != Some(&':') {
            return Err(format!("expected ':' after tag name '{key}'"));
        }
        i += 1;
        skip_ws(&chars, &mut i);
        let value = if chars.get(i) == Some(&'"') {
            quoted(&chars, &mut i)?
        } else {
            let start = i;
            while i < chars.len() && chars[i] != ',' {
                i += 1;
            }
            let raw: String = chars[start..i].iter().collect();
            let raw = raw.trim();
            if raw.is_empty() {
                return Err(format!("missing value for tag '{key}'"));
            }
            if raw == "null" {
                String::new()
            } else {
                raw.to_string()
            }
        };
        out.push((key, value));
        skip_ws(&chars, &mut i);
        match chars.get(i) {
            None => break,
            Some(',') => i += 1,
            Some(c) => return Err(format!("unexpected '{c}' in tags")),
        }
    }
    Ok(out)
}

fn skip_ws(chars: &[char], i: &mut usize) {
    while *i < chars.len() && chars[*i].is_whitespace() {
        *i += 1;
    }
}

fn quoted(chars: &[char], i: &mut usize) -> Result<String, String> {
    if chars.get(*i) != Some(&'"') {
        return Err("expected a quoted tag name".to_string());
    }
    *i += 1;
    let mut s = String::new();
    while *i < chars.len() {
        let c = chars[*i];
        *i += 1;
        match c {
            '"' => return Ok(s),
            '\\' => {
                let e = *chars.get(*i).ok_or("dangling backslash in tags")?;
                *i += 1;
                match e {
                    '"' | '\\' | '/' => s.push(e),
                    'n' => s.push('\n'),
                    't' => s.push('\t'),
                    'u' => {
                        let code = hex4(chars, *i)?;
                        *i += 4;
                        let c = match code {
                            // A high surrogate and a following low surrogate
                            // escape are one character.
                            0xD800..=0xDBFF => match low_surrogate(chars, *i) {
                                Some(low) => {
                                    *i += 6;
                                    let both = 0x10000 + ((code - 0xD800) << 10) + (low - 0xDC00);
                                    char::from_u32(both).unwrap_or('\u{fffd}')
                                }
                                // Alone: one replacement character, and the
                                // next escape is left for its own turn.
                                None => '\u{fffd}',
                            },
                            0xDC00..=0xDFFF => '\u{fffd}',
                            _ => char::from_u32(code).unwrap_or('\u{fffd}'),
                        };
                        s.push(c);
                    }
                    other => return Err(format!("unknown escape '\\{other}' in tags")),
                }
            }
            _ => s.push(c),
        }
    }
    Err("unterminated string in tags".to_string())
}

/// Four hex digits at `at`.
fn hex4(chars: &[char], at: usize) -> Result<u32, &'static str> {
    let bad = "bad \\u escape in tags";
    let h = chars.get(at..at + 4).ok_or(bad)?;
    if !h.iter().all(|c| c.is_ascii_hexdigit()) {
        return Err(bad);
    }
    let hex: String = h.iter().collect();
    u32::from_str_radix(&hex, 16).map_err(|_| bad)
}

/// A `\uDC00`..`\uDFFF` escape starting at `at`, if there is one.
fn low_surrogate(chars: &[char], at: usize) -> Option<u32> {
    if chars.get(at) != Some(&'\\') || chars.get(at + 1) != Some(&'u') {
        return None;
    }
    hex4(chars, at + 2)
        .ok()
        .filter(|c| (0xDC00..=0xDFFF).contains(c))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pairs(s: &str) -> Vec<(String, String)> {
        parse(s).unwrap()
    }

    fn kv(k: &str, v: &str) -> (String, String) {
        (k.to_string(), v.to_string())
    }

    #[test]
    fn without_braces() {
        assert_eq!(
            pairs("\"env\": \"prod\",\"team\": \"web\""),
            vec![kv("env", "prod"), kv("team", "web")]
        );
    }

    #[test]
    fn with_braces() {
        assert_eq!(pairs("{\"env\":\"prod\"}"), vec![kv("env", "prod")]);
    }

    #[test]
    fn empty_cell_has_no_tags() {
        assert!(pairs("").is_empty());
        assert!(pairs("   ").is_empty());
        assert!(pairs("{}").is_empty());
    }

    #[test]
    fn bare_values_and_null() {
        assert_eq!(
            pairs("\"n\": 5, \"x\": null, \"b\": true"),
            vec![kv("n", "5"), kv("x", ""), kv("b", "true")]
        );
    }

    #[test]
    fn escapes() {
        assert_eq!(pairs(r#""a": "x\"y\\zA""#), vec![kv("a", "x\"y\\zA")]);
    }

    #[test]
    fn surrogate_pair_is_one_character() {
        assert_eq!(pairs(r#""a": "\ud83d\ude00""#), vec![kv("a", "\u{1F600}")]);
        assert_eq!(
            pairs(r#""a": "x\uD83D\uDE80y""#),
            vec![kv("a", "x\u{1F680}y")]
        );
        assert_ne!(
            pairs(r#""a": "\ud83d\ude00""#),
            pairs(r#""a": "\ud83d\ude80""#)
        );
    }

    #[test]
    fn lone_surrogate_is_one_replacement_and_keeps_its_neighbour() {
        assert_eq!(pairs(r#""a": "\ud83d""#), vec![kv("a", "\u{fffd}")]);
        assert_eq!(pairs(r#""a": "\ude00""#), vec![kv("a", "\u{fffd}")]);
        // high then a normal escape: the normal one survives
        assert_eq!(pairs(r#""a": "\ud83d\u0041""#), vec![kv("a", "\u{fffd}A")]);
        // high then high: two replacements, not a pair
        assert_eq!(
            pairs(r#""a": "\ud83d\ud83d""#),
            vec![kv("a", "\u{fffd}\u{fffd}")]
        );
        // low then high: two replacements
        assert_eq!(
            pairs(r#""a": "\ude00\ud83d""#),
            vec![kv("a", "\u{fffd}\u{fffd}")]
        );
        // high followed by a literal, not an escape
        assert_eq!(
            pairs(r#""a": "\ud83dude00""#),
            vec![kv("a", "\u{fffd}ude00")]
        );
    }

    #[test]
    fn bad_unicode_escapes_error() {
        assert!(parse(r#""a": "\u12""#).is_err());
        assert!(parse(r#""a": "\u+123""#).is_err());
        assert!(parse(r#""a": "\uzzzz""#).is_err());
    }

    #[test]
    fn commas_inside_values() {
        assert_eq!(
            pairs("\"a\": \"x,y\", \"b\": \"z\""),
            vec![kv("a", "x,y"), kv("b", "z")]
        );
    }

    #[test]
    fn trailing_comma_is_tolerated() {
        assert_eq!(pairs("\"a\": \"b\","), vec![kv("a", "b")]);
    }

    #[test]
    fn malformed_input_errors() {
        assert!(parse("env=prod").is_err());
        assert!(parse("\"a\" \"b\"").is_err());
        assert!(parse("\"a\": \"b").is_err());
        assert!(parse("{\"a\": \"b\"").is_err());
        assert!(parse("\"a\":").is_err());
        assert!(parse("\"a\": \"b\" \"c\": \"d\"").is_err());
        assert!(parse(r#""a": "\q""#).is_err());
    }
}
