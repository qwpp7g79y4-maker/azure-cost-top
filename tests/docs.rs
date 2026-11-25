//! Keeps README.md, QUALITY.md, CHANGELOG.md and the licenses honest.

use std::fs;
use std::path::Path;

/// Reads a text file with `\r\n` turned into `\n`, so a CRLF checkout (Windows)
/// gives the same text as an LF checkout.
fn read(path: &str) -> String {
    fs::read_to_string(path)
        .unwrap_or_else(|e| panic!("cannot read {path}: {e}"))
        .replace("\r\n", "\n")
}

fn rust_files(dir: &str) -> Vec<String> {
    let mut v: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "rs"))
        .map(|p| p.to_string_lossy().into_owned())
        .collect();
    v.sort();
    v
}

fn all_sources() -> String {
    ["src", "tests"]
        .iter()
        .flat_map(|d| rust_files(d))
        .map(|f| read(&f))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Number of test functions in `src/` and `tests/`.
fn suite_size() -> usize {
    let attr = concat!("#[", "test]");
    all_sources().lines().filter(|l| l.trim() == attr).count()
}

fn stated_count(doc: &str) -> usize {
    read(doc)
        .lines()
        .find_map(|l| l.strip_prefix("Test count:"))
        .unwrap_or_else(|| panic!("{doc} has no 'Test count:' line"))
        .trim()
        .parse()
        .unwrap_or_else(|_| panic!("{doc}: 'Test count:' is not a number"))
}

#[test]
fn readme_test_count_equals_the_suite() {
    assert_eq!(stated_count("README.md"), suite_size());
}

#[test]
fn quality_test_count_equals_the_suite() {
    assert_eq!(stated_count("QUALITY.md"), suite_size());
}

#[test]
fn suite_has_at_least_sixty_tests() {
    assert!(suite_size() >= 60, "only {} tests", suite_size());
}

#[test]
fn every_test_named_in_quality_exists() {
    let src = all_sources();
    let quality = read("QUALITY.md");
    let mut checked = 0;
    for part in quality.split('`').skip(1).step_by(2) {
        let looks_like_test = !part.is_empty()
            && part
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
            && part.contains('_');
        if looks_like_test {
            assert!(
                src.contains(&format!("fn {part}()")),
                "QUALITY.md names a test that does not exist: {part}"
            );
            checked += 1;
        }
    }
    assert!(checked >= 60, "only {checked} tests are mapped");
}

#[test]
fn changelog_has_the_current_version() {
    let log = read("CHANGELOG.md");
    let heading = format!("## [{}] - ", env!("CARGO_PKG_VERSION"));
    assert!(log.contains(&heading), "CHANGELOG.md lacks '{heading}'");
    assert!(log.contains("## [0.1.0] - "));
}

#[test]
fn licenses_are_present_and_match_cargo_toml() {
    assert!(Path::new("LICENSE-MIT").exists());
    assert!(Path::new("LICENSE-APACHE").exists());
    assert!(read("LICENSE-MIT").starts_with("MIT License"));
    assert!(read("LICENSE-APACHE").contains("Apache License"));
    assert_eq!(env!("CARGO_PKG_LICENSE"), "MIT OR Apache-2.0");
    assert!(read("README.md").contains("MIT OR Apache-2.0"));
}

#[test]
fn readme_mentions_every_option_in_the_help() {
    let readme = read("README.md");
    for opt in [
        "--by",
        "--tag-key",
        "--top",
        "--month",
        "--delta",
        "--vs",
        "--currency",
        "--format",
    ] {
        assert!(readme.contains(opt), "README does not mention {opt}");
    }
}

#[test]
fn readme_states_the_limits() {
    let readme = read("README.md");
    assert!(readme.contains("no network"));
    assert!(readme.contains("no Azure\nSDK") || readme.contains("no Azure SDK"));
}

#[test]
fn readme_nl_makes_the_safe_text_promise_too() {
    let nl = read("README.nl.md");
    let safe = nl
        .lines()
        .find(|l| l.contains("Veilige tekst"))
        .expect("README.nl.md has no 'Veilige tekst' paragraph");
    for word in [
        "U+FFFD",
        "=",
        "+",
        "-",
        "@",
        "aanhalingsteken",
        "stderr",
        "JSON",
    ] {
        assert!(safe.contains(word), "Veilige tekst lacks {word:?}: {safe}");
    }
}
