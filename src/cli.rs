//! Command line parsing. No dependencies, so the build stays small and offline.

use crate::date::Month;
use crate::error::Error;
use crate::render::Format;
use crate::report::{Dimension, Options};
use std::ffi::OsString;
use std::path::PathBuf;

pub const HELP: &str = "\
azure-cost-top - top costs per meter, resource group and tag from an Azure Cost Management CSV export

USAGE:
    azure-cost-top [OPTIONS] <FILE>
    azure-cost-top [OPTIONS] -          read the export from standard input

OPTIONS:
    -b, --by <LIST>        sections to show: meter, group, tag (comma separated).
                           Default: every one the export has a column for.
    -t, --tag-key <NAME>   group the tag section by the value of this tag name
                           (default: one line per name=value pair)
    -n, --top <N>          lines per section (default 10)
    -m, --month <YYYY-MM>  only this month (default: all rows)
    -d, --delta            compare the month with the month before it
                           (default month: the latest one in the file)
        --vs <YYYY-MM>     compare with this month instead; implies --delta
        --currency <CODE>  keep only this billing currency
    -f, --format <FMT>     table (default), csv or json
    -h, --help             show this help
    -V, --version          show the version

The program reads one local CSV file. It never uses the network.
";

/// What the command line asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Help,
    Version,
    Run(Run),
}

/// A normal run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Run {
    /// File path, or `-` for standard input.
    pub input: PathBuf,
    pub options: Options,
    pub format: Format,
}

fn parse_dimension(s: &str) -> Result<Dimension, Error> {
    match s.trim().to_ascii_lowercase().as_str() {
        "meter" | "meters" => Ok(Dimension::Meter),
        "group" | "groups" | "resourcegroup" | "resource-group" | "resource_group" | "rg" => {
            Ok(Dimension::Group)
        }
        "tag" | "tags" => Ok(Dimension::Tag),
        other => Err(Error::usage(format!(
            "unknown section '{other}' in --by (use meter, group, tag)"
        ))),
    }
}

fn parse_by(list: &str) -> Result<Vec<Dimension>, Error> {
    let mut dims: Vec<Dimension> = Vec::new();
    for part in list.split(',') {
        let d = parse_dimension(part)?;
        if !dims.contains(&d) {
            dims.push(d);
        }
    }
    Ok(dims)
}

fn parse_format(s: &str) -> Result<Format, Error> {
    match s.trim().to_ascii_lowercase().as_str() {
        "table" => Ok(Format::Table),
        "csv" => Ok(Format::Csv),
        "json" => Ok(Format::Json),
        other => Err(Error::usage(format!(
            "unknown format '{other}' (use table, csv or json)"
        ))),
    }
}

fn parse_month(name: &str, s: &str) -> Result<Month, Error> {
    Month::parse(s).map_err(|e| Error::usage(format!("{name}: {e}")))
}

/// Parse the arguments after the program name.
///
/// Arguments are `OsString`: a file name that is not valid UTF-8 stays exact.
pub fn parse_args<I: IntoIterator<Item = OsString>>(args: I) -> Result<Command, Error> {
    let mut it = args.into_iter();
    let mut options = Options::default();
    let mut format = Format::Table;
    let mut input: Option<PathBuf> = None;
    let mut positional_only = false;

    while let Some(raw) = it.next() {
        let arg = match raw.into_string() {
            Ok(s) => s,
            Err(raw) => {
                // Not UTF-8: only a file name can be that. Keep it byte for byte.
                if positional_only || !raw.to_string_lossy().starts_with('-') {
                    set_input(&mut input, PathBuf::from(raw))?;
                    continue;
                }
                return Err(Error::usage(format!(
                    "unknown option '{}'",
                    raw.to_string_lossy()
                )));
            }
        };
        if positional_only || arg == "-" || !arg.starts_with('-') {
            set_input(&mut input, PathBuf::from(arg))?;
            continue;
        }
        if arg == "--" {
            positional_only = true;
            continue;
        }
        // Split "--name=value" and "-nVALUE".
        let (name, inline): (String, Option<String>) = if let Some(rest) = arg.strip_prefix("--") {
            match rest.split_once('=') {
                Some((n, v)) => (format!("--{n}"), Some(v.to_string())),
                None => (arg.clone(), None),
            }
        } else {
            let mut chars = arg.chars();
            chars.next();
            let c = chars.next().unwrap_or('-');
            let rest = chars.as_str();
            (
                format!("-{c}"),
                if rest.is_empty() {
                    None
                } else {
                    Some(rest.to_string())
                },
            )
        };

        let mut value = |what: &str| -> Result<String, Error> {
            match inline.clone() {
                Some(v) => Ok(v),
                None => it
                    .next()
                    .map(|v| v.to_string_lossy().into_owned())
                    .ok_or_else(|| Error::usage(format!("{what} needs a value"))),
            }
        };
        let no_value = |inline: &Option<String>| -> Result<(), Error> {
            if inline.is_some() {
                Err(Error::usage(format!("{name} does not take a value")))
            } else {
                Ok(())
            }
        };

        match name.as_str() {
            "--help" | "-h" => {
                no_value(&inline)?;
                return Ok(Command::Help);
            }
            "--version" | "-V" => {
                no_value(&inline)?;
                return Ok(Command::Version);
            }
            "--delta" | "-d" => {
                no_value(&inline)?;
                options.delta = true;
            }
            "--by" | "-b" => options.dimensions = Some(parse_by(&value("--by")?)?),
            "--tag-key" | "-t" => options.tag_key = Some(value("--tag-key")?),
            "--top" | "-n" => {
                let v = value("--top")?;
                let n: usize = v
                    .trim()
                    .parse()
                    .map_err(|_| Error::usage(format!("--top: '{v}' is not a number")))?;
                if n == 0 {
                    return Err(Error::usage("--top must be at least 1"));
                }
                options.top = n;
            }
            "--month" | "-m" => options.month = Some(parse_month("--month", &value("--month")?)?),
            "--vs" => {
                options.vs = Some(parse_month("--vs", &value("--vs")?)?);
                options.delta = true;
            }
            "--currency" => options.currency = Some(value("--currency")?),
            "--format" | "-f" => format = parse_format(&value("--format")?)?,
            _ => return Err(Error::usage(format!("unknown option '{arg}'"))),
        }
    }

    match input {
        Some(input) => Ok(Command::Run(Run {
            input,
            options,
            format,
        })),
        None => Err(Error::usage(
            "no input file given (use '-' for standard input)",
        )),
    }
}

fn set_input(slot: &mut Option<PathBuf>, arg: PathBuf) -> Result<(), Error> {
    if slot.is_some() {
        return Err(Error::usage(format!(
            "unexpected extra argument '{}'",
            arg.to_string_lossy()
        )));
    }
    *slot = Some(arg);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn parse(args: &[&str]) -> Result<Command, Error> {
        parse_args(args.iter().map(OsString::from))
    }

    fn run(args: &[&str]) -> Run {
        match parse(args).unwrap() {
            Command::Run(r) => r,
            other => panic!("expected Run, got {other:?}"),
        }
    }

    #[test]
    fn defaults() {
        let r = run(&["f.csv"]);
        assert_eq!(r.input, Path::new("f.csv"));
        assert_eq!(r.format, Format::Table);
        assert_eq!(r.options, Options::default());
        assert_eq!(r.options.top, 10);
    }

    #[test]
    fn help_and_version() {
        assert_eq!(parse(&["--help"]).unwrap(), Command::Help);
        assert_eq!(parse(&["-h"]).unwrap(), Command::Help);
        assert_eq!(parse(&["x.csv", "-V"]).unwrap(), Command::Version);
        assert_eq!(parse(&["--version"]).unwrap(), Command::Version);
    }

    #[test]
    fn stdin_dash_is_an_input() {
        assert_eq!(run(&["-"]).input, Path::new("-"));
    }

    #[test]
    fn missing_input_is_a_usage_error() {
        assert!(matches!(parse(&[]), Err(Error::Usage(_))));
        assert!(matches!(parse(&["--top", "3"]), Err(Error::Usage(_))));
    }

    #[test]
    fn two_inputs_are_refused() {
        assert!(matches!(parse(&["a.csv", "b.csv"]), Err(Error::Usage(_))));
    }

    #[test]
    fn top_forms() {
        assert_eq!(run(&["f", "--top", "3"]).options.top, 3);
        assert_eq!(run(&["f", "--top=4"]).options.top, 4);
        assert_eq!(run(&["f", "-n", "5"]).options.top, 5);
        assert_eq!(run(&["f", "-n6"]).options.top, 6);
    }

    #[test]
    fn top_rejects_zero_and_text() {
        assert!(parse(&["f", "--top", "0"]).is_err());
        assert!(parse(&["f", "--top", "x"]).is_err());
        assert!(parse(&["f", "--top", "-1"]).is_err());
    }

    #[test]
    fn by_list_with_aliases_and_dedup() {
        let r = run(&["f", "--by", "tags,rg,Meter,tag"]);
        assert_eq!(
            r.options.dimensions,
            Some(vec![Dimension::Tag, Dimension::Group, Dimension::Meter])
        );
    }

    #[test]
    fn by_rejects_unknown_section() {
        assert!(parse(&["f", "--by", "meter,region"]).is_err());
    }

    #[test]
    fn month_vs_and_delta() {
        let r = run(&["f", "--month", "2026-10", "--vs", "2026-08"]);
        assert_eq!(r.options.month.unwrap().to_string(), "2026-10");
        assert_eq!(r.options.vs.unwrap().to_string(), "2026-08");
        assert!(r.options.delta);
        assert!(run(&["f", "-d"]).options.delta);
        assert!(run(&["f", "--delta"]).options.delta);
        assert!(!run(&["f"]).options.delta);
    }

    #[test]
    fn bad_month_is_a_usage_error() {
        assert!(matches!(
            parse(&["f", "--month", "oct"]),
            Err(Error::Usage(_))
        ));
        assert!(matches!(
            parse(&["f", "--vs", "2026-13"]),
            Err(Error::Usage(_))
        ));
    }

    #[test]
    fn format_forms() {
        assert_eq!(run(&["f", "--format", "csv"]).format, Format::Csv);
        assert_eq!(run(&["f", "-f", "JSON"]).format, Format::Json);
        assert_eq!(run(&["f", "--format=table"]).format, Format::Table);
        assert!(parse(&["f", "--format", "xml"]).is_err());
    }

    #[test]
    fn tag_key_and_currency() {
        let r = run(&["f", "-t", "team", "--currency", "EUR"]);
        assert_eq!(r.options.tag_key.as_deref(), Some("team"));
        assert_eq!(r.options.currency.as_deref(), Some("EUR"));
    }

    #[test]
    fn option_without_value_errors() {
        assert!(parse(&["f", "--top"]).is_err());
        assert!(parse(&["f", "--by"]).is_err());
    }

    #[test]
    fn flag_with_value_errors() {
        assert!(parse(&["f", "--delta=yes"]).is_err());
        assert!(parse(&["--help=x"]).is_err());
    }

    #[test]
    fn unknown_option_errors() {
        let e = parse(&["f", "--wat"]).unwrap_err();
        assert!(e.to_string().contains("unknown option"));
    }

    #[test]
    fn double_dash_ends_options() {
        assert_eq!(run(&["--", "-weird.csv"]).input, Path::new("-weird.csv"));
    }

    #[test]
    fn options_may_come_before_the_file() {
        let r = run(&["-n", "2", "f.csv"]);
        assert_eq!(r.input, Path::new("f.csv"));
        assert_eq!(r.options.top, 2);
    }
}
