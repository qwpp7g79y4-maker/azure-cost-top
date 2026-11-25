//! Error type shared by all modules.

use std::fmt;

/// Failure of a run. `Usage` and `Date` map to exit code 2, `Input` to exit code 1.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// The command line is wrong.
    Usage(String),
    /// The export or the data in it is wrong, or it could not be read.
    Input(String),
    /// A date cell that cannot be read. Exit code 2, without the usage hint.
    Date(String),
}

impl Error {
    /// Process exit code for this error.
    pub fn exit_code(&self) -> u8 {
        match self {
            Error::Usage(_) | Error::Date(_) => 2,
            Error::Input(_) => 1,
        }
    }

    pub(crate) fn usage(msg: impl Into<String>) -> Error {
        Error::Usage(msg.into())
    }

    pub(crate) fn date(msg: impl Into<String>) -> Error {
        Error::Date(msg.into())
    }

    pub(crate) fn input(msg: impl Into<String>) -> Error {
        Error::Input(msg.into())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // The message can hold text from the export (a date, an amount, a
            // currency, a tag key) or the file name; clean it so it cannot
            // start a fake line or a terminal sequence on stderr.
            Error::Usage(m) | Error::Input(m) | Error::Date(m) => {
                f.write_str(&crate::render::clean(m))
            }
        }
    }
}

impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usage_exit_code_is_2() {
        assert_eq!(Error::usage("x").exit_code(), 2);
    }

    #[test]
    fn date_exit_code_is_2() {
        assert_eq!(Error::date("x").exit_code(), 2);
    }

    #[test]
    fn input_exit_code_is_1() {
        assert_eq!(Error::input("x").exit_code(), 1);
    }

    #[test]
    fn display_cleans_escapes_and_newlines() {
        for e in [
            Error::usage("a\u{1b}[31m\u{7}b\nc"),
            Error::input("a\u{1b}]0;x\u{7}b\nc"),
            Error::date("a\u{1b}[2Jb\nc"),
        ] {
            let s = e.to_string();
            assert!(!s.contains(['\u{1b}', '\u{7}', '\n']), "{s:?}");
            assert!(s.starts_with('a') && s.contains('c'), "{s:?}");
        }
    }

    #[test]
    fn display_is_the_message() {
        assert_eq!(Error::input("boom").to_string(), "boom");
    }
}
