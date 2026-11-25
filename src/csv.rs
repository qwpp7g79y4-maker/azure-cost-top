//! Small RFC 4180 reader: quotes, doubled quotes, embedded newlines, CRLF, BOM.

/// One CSV record and the line it started on (1-based).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    pub line: usize,
    pub fields: Vec<String>,
}

/// Parse CSV text. Blank lines are skipped. A leading BOM is removed.
pub fn parse(input: &str) -> Result<Vec<Record>, String> {
    let input = input.strip_prefix('\u{feff}').unwrap_or(input);
    let mut records = Vec::new();
    let mut fields: Vec<String> = Vec::new();
    let mut field = String::new();
    let mut in_quotes = false;
    let mut quoted_field = false;
    let mut line = 1usize;
    let mut record_line = 1usize;
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if in_quotes {
            match c {
                '"' => {
                    if chars.peek() == Some(&'"') {
                        chars.next();
                        field.push('"');
                    } else {
                        in_quotes = false;
                    }
                }
                '\n' => {
                    line += 1;
                    field.push('\n');
                }
                _ => field.push(c),
            }
            continue;
        }
        match c {
            '"' if field.is_empty() && !quoted_field => {
                in_quotes = true;
                quoted_field = true;
            }
            ',' => {
                fields.push(std::mem::take(&mut field));
                quoted_field = false;
            }
            '\r' => {
                if chars.peek() != Some(&'\n') {
                    // lone CR: treat as a line break too
                    finish_record(
                        &mut records,
                        &mut fields,
                        &mut field,
                        quoted_field,
                        record_line,
                    );
                    quoted_field = false;
                    line += 1;
                    record_line = line;
                }
            }
            '\n' => {
                finish_record(
                    &mut records,
                    &mut fields,
                    &mut field,
                    quoted_field,
                    record_line,
                );
                quoted_field = false;
                line += 1;
                record_line = line;
            }
            _ => field.push(c),
        }
    }
    if in_quotes {
        return Err(format!("line {record_line}: quoted field is never closed"));
    }
    finish_record(
        &mut records,
        &mut fields,
        &mut field,
        quoted_field,
        record_line,
    );
    Ok(records)
}

fn finish_record(
    records: &mut Vec<Record>,
    fields: &mut Vec<String>,
    field: &mut String,
    quoted: bool,
    line: usize,
) {
    if fields.is_empty() && field.is_empty() && !quoted {
        return; // blank line
    }
    fields.push(std::mem::take(field));
    records.push(Record {
        line,
        fields: std::mem::take(fields),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows(s: &str) -> Vec<Vec<String>> {
        parse(s).unwrap().into_iter().map(|r| r.fields).collect()
    }

    #[test]
    fn simple_rows() {
        assert_eq!(rows("a,b\n1,2\n"), vec![vec!["a", "b"], vec!["1", "2"]]);
    }

    #[test]
    fn no_trailing_newline() {
        assert_eq!(rows("a,b\n1,2"), vec![vec!["a", "b"], vec!["1", "2"]]);
    }

    #[test]
    fn crlf_line_endings() {
        assert_eq!(rows("a,b\r\n1,2\r\n"), vec![vec!["a", "b"], vec!["1", "2"]]);
    }

    #[test]
    fn quoted_comma() {
        assert_eq!(rows("\"a,b\",c\n"), vec![vec!["a,b", "c"]]);
    }

    #[test]
    fn doubled_quotes() {
        assert_eq!(
            rows("\"say \"\"hi\"\"\",x\n"),
            vec![vec!["say \"hi\"", "x"]]
        );
    }

    #[test]
    fn embedded_newline_in_quotes() {
        assert_eq!(rows("\"a\nb\",c\n"), vec![vec!["a\nb", "c"]]);
    }

    #[test]
    fn empty_fields_are_kept() {
        assert_eq!(rows("a,,c\n"), vec![vec!["a", "", "c"]]);
        assert_eq!(rows(",\n"), vec![vec!["", ""]]);
    }

    #[test]
    fn blank_lines_are_skipped() {
        assert_eq!(rows("a\n\n\nb\n"), vec![vec!["a"], vec!["b"]]);
    }

    #[test]
    fn quoted_empty_field_is_a_record() {
        assert_eq!(rows("\"\"\n"), vec![vec![""]]);
    }

    #[test]
    fn strips_bom() {
        assert_eq!(rows("\u{feff}a,b\n"), vec![vec!["a", "b"]]);
    }

    #[test]
    fn unterminated_quote_is_an_error() {
        let e = parse("a,\"b\n").unwrap_err();
        assert!(e.contains("never closed"), "{e}");
    }

    #[test]
    fn lines_are_numbered_across_embedded_newlines() {
        let r = parse("a\n\"x\ny\"\nb\n").unwrap();
        assert_eq!(r[0].line, 1);
        assert_eq!(r[1].line, 2);
        assert_eq!(r[2].line, 4);
    }

    #[test]
    fn quote_inside_unquoted_field_is_literal() {
        assert_eq!(rows("ab\"c,d\n"), vec![vec!["ab\"c", "d"]]);
    }

    #[test]
    fn empty_input_has_no_records() {
        assert!(parse("").unwrap().is_empty());
        assert!(parse("\n\n").unwrap().is_empty());
    }

    #[test]
    fn lone_cr_breaks_lines() {
        assert_eq!(rows("a\rb\r"), vec![vec!["a"], vec!["b"]]);
    }
}
