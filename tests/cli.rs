//! End-to-end tests: the real binary on the fixture and on small temp files.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};

const FIXTURE: &str = "tests/fixtures/sample.csv";

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_azure-cost-top"))
}

fn run(args: &[&str]) -> Output {
    bin().args(args).output().unwrap()
}

fn out(args: &[&str]) -> String {
    let o = run(args);
    assert!(
        o.status.success(),
        "args {args:?} failed: {}",
        String::from_utf8_lossy(&o.stderr)
    );
    String::from_utf8(o.stdout).unwrap()
}

fn fail(args: &[&str]) -> (i32, String) {
    let o = run(args);
    assert!(!o.status.success(), "args {args:?} should fail");
    (
        o.status.code().unwrap(),
        String::from_utf8(o.stderr).unwrap(),
    )
}

fn temp(name: &str, content: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    let path = dir.join(name);
    std::fs::write(&path, content).unwrap();
    path
}

fn p(path: &Path) -> &str {
    path.to_str().unwrap()
}

#[test]
fn default_run_shows_three_sections() {
    let o = out(&[FIXTURE]);
    assert!(o.contains("Top meter costs"));
    assert!(o.contains("Top resource group costs"));
    assert!(o.contains("Top tag costs"));
}

#[test]
fn default_run_total_is_the_whole_file() {
    let o = out(&[FIXTURE]);
    assert!(o.contains("Period: all months   Rows: 12"));
    assert!(o.contains("Total: 264.00 EUR"));
}

#[test]
fn month_filter_limits_total() {
    let o = out(&[FIXTURE, "--month", "2026-09"]);
    assert!(o.contains("Period: 2026-09   Rows: 5"));
    assert!(o.contains("Total: 112.00 EUR"));
}

#[test]
fn meter_ranking_is_by_cost() {
    let o = out(&[FIXTURE, "--by", "meter", "-m", "2026-10"]);
    let order: Vec<usize> = [
        "Gen5 vCore",
        "D2s v3",
        "Data Ingestion",
        "Hot LRS Data Stored",
        "B2s",
        "Data Transfer Out",
    ]
    .iter()
    .map(|n| o.find(n).unwrap_or_else(|| panic!("{n} missing")))
    .collect();
    assert!(order.windows(2).all(|w| w[0] < w[1]), "{o}");
}

#[test]
fn resource_groups_merge_by_case() {
    let o = out(&[FIXTURE, "--by", "group", "-m", "2026-10"]);
    assert!(o.contains("62.50"), "rg-web should merge RG-WEB: {o}");
    assert!(!o.contains("RG-WEB"));
    assert!(o.contains("showing 3 of 3"));
}

#[test]
fn delta_defaults_to_latest_month() {
    let o = out(&[FIXTURE, "--delta", "--by", "meter"]);
    assert!(o.contains("Period: 2026-10 vs 2026-09"));
    assert!(o.contains("Total: 152.00 EUR (was 112.00, +40.00, +35.7%)"));
}

#[test]
fn delta_marks_new_meters() {
    let o = out(&[FIXTURE, "-d", "--by", "meter"]);
    let line = o.lines().find(|l| l.contains("Data Ingestion")).unwrap();
    assert!(line.ends_with("new"), "{line}");
}

#[test]
fn delta_for_group_values() {
    let o = out(&[FIXTURE, "-d", "--by", "group"]);
    let line = o.lines().find(|l| l.contains("rg-data")).unwrap();
    assert!(
        line.contains("64.50")
            && line.contains("59.50")
            && line.contains("+5.00")
            && line.contains("+8.4%"),
        "{line}"
    );
}

#[test]
fn explicit_month_pair_with_vs() {
    let o = out(&[FIXTURE, "-m", "2026-09", "--vs", "2026-10", "--by", "group"]);
    assert!(o.contains("Period: 2026-09 vs 2026-10"));
    let line = o.lines().find(|l| l.contains("rg-ops")).unwrap();
    assert!(line.contains("-100.0%"), "{line}");
}

#[test]
fn tag_key_groups_by_value() {
    let o = out(&[FIXTURE, "-t", "team", "-m", "2026-10", "--by", "tag"]);
    assert!(o.contains("Top tag team costs (showing 4 of 4)"));
    assert!(o.contains("(untagged)"));
    assert!(o.lines().any(|l| l.contains("data") && l.contains("64.50")));
}

#[test]
fn tag_pairs_without_key() {
    let o = out(&[FIXTURE, "-m", "2026-10", "--by", "tag"]);
    assert!(o.contains("env=prod"));
    assert!(o.contains("team=web"));
}

#[test]
fn top_limits_lines() {
    let o = out(&[FIXTURE, "-n", "2", "--by", "meter"]);
    assert!(o.contains("showing 2 of 6"));
}

#[test]
fn csv_output_is_machine_readable() {
    let o = out(&[
        FIXTURE, "-f", "csv", "-m", "2026-10", "--by", "meter", "-n", "2",
    ]);
    let lines: Vec<&str> = o.lines().collect();
    assert_eq!(lines[0], "dimension,rank,name,cost,share_pct");
    assert_eq!(lines[1], "meter,1,Gen5 vCore,60.75,40.0");
    assert_eq!(lines.len(), 3);
}

#[test]
fn csv_delta_has_delta_columns() {
    let o = out(&[FIXTURE, "-f", "csv", "-d", "--by", "group", "-n", "1"]);
    assert!(o.starts_with("dimension,rank,name,cost,share_pct,previous,delta,delta_pct\n"));
    assert!(o.contains("resource_group,1,rg-data,64.50,42.4,59.50,5.00,8.4"));
}

#[test]
fn json_output_has_totals() {
    let o = out(&[
        FIXTURE, "--format", "json", "-d", "--by", "meter", "-n", "1",
    ]);
    assert!(o.contains("\"total\": 152.00"));
    assert!(o.contains("\"previous_total\": 112.00"));
    assert!(o.contains("\"name\": \"Gen5 vCore\""));
    assert!(o.trim_end().ends_with('}'));
}

#[test]
fn reads_standard_input() {
    let csv = std::fs::read(FIXTURE).unwrap();
    let mut child = bin()
        .args(["-", "--by", "meter", "-n", "1"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(&csv).unwrap();
    let o = child.wait_with_output().unwrap();
    assert!(o.status.success());
    let s = String::from_utf8(o.stdout).unwrap();
    assert!(s.contains("azure-cost-top 0.1.0 - stdin"));
    assert!(s.contains("Gen5 vCore"));
}

#[test]
fn help_and_version_exit_zero() {
    assert!(out(&["--help"]).contains("USAGE:"));
    assert_eq!(
        out(&["--version"]),
        format!("azure-cost-top {}\n", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn help_mentions_every_option() {
    let h = out(&["-h"]);
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
        assert!(h.contains(opt), "help lacks {opt}");
    }
}

#[test]
fn usage_errors_exit_2() {
    let (code, err) = fail(&[]);
    assert_eq!(code, 2);
    assert!(err.contains("--help"));
    assert_eq!(fail(&[FIXTURE, "--bogus"]).0, 2);
    assert_eq!(fail(&[FIXTURE, "--top", "0"]).0, 2);
    assert_eq!(fail(&[FIXTURE, "--month", "oct"]).0, 2);
}

#[test]
fn missing_file_exits_1() {
    let (code, err) = fail(&["/no/such/export.csv"]);
    assert_eq!(code, 1);
    assert!(err.contains("cannot read"));
}

#[test]
fn month_without_rows_exits_1() {
    let (code, err) = fail(&[FIXTURE, "-m", "2025-01"]);
    assert_eq!(code, 1);
    assert!(err.contains("no rows in month 2025-01"));
}

#[test]
fn invalid_utf8_is_reported() {
    let path = temp("bad-utf8.csv", "");
    std::fs::write(&path, b"Cost\n\xff\xfe\n").unwrap();
    let (code, err) = fail(&[p(&path)]);
    assert_eq!(code, 1);
    assert!(err.contains("UTF-8"));
}

#[test]
fn does_not_modify_or_delete_the_input() {
    let path = temp("keep.csv", "Cost,MeterName\n1,a\n");
    let before = std::fs::read(&path).unwrap();
    out(&[p(&path)]);
    assert_eq!(std::fs::read(&path).unwrap(), before);
}

#[test]
fn handles_bom_and_crlf() {
    let path = temp(
        "bom-crlf.csv",
        "\u{feff}Cost,MeterName\r\n1.5,a\r\n2.5,b\r\n",
    );
    let o = out(&[p(&path), "--by", "meter"]);
    assert!(o.contains("Total: 4.00"));
}

#[test]
fn mixed_currency_needs_a_choice() {
    let path = temp("mixed.csv", "Cost,MeterName,Currency\n1,a,EUR\n2,b,USD\n");
    let (code, err) = fail(&[p(&path)]);
    assert_eq!(code, 1);
    assert!(err.contains("--currency"));
    let o = out(&[p(&path), "--currency", "usd"]);
    assert!(o.contains("Total: 2.00 USD"));
}

#[test]
fn bad_row_names_its_line() {
    let path = temp("badrow.csv", "Cost,MeterName\n1,a\nxyz,b\n");
    let (_, err) = fail(&[p(&path)]);
    assert!(err.contains("line 3"), "{err}");
}

#[test]
fn e_notation_costs_are_summed_exactly() {
    let path = temp("enotation.csv", "Cost,MeterName\n1.5E-3,a\n2.5E-3,a\n");
    let o = out(&[p(&path), "-f", "csv"]);
    assert!(o.contains("meter,1,a,0.004,100.0"), "{o}");
}

#[test]
fn legacy_column_names_work() {
    let path = temp(
        "legacy.csv",
        "UsageDateTime,Meter,ResourceGroupName,PreTaxCost,Currency\n2026-10-01T00:00:00,vm,rg1,3,GBP\n2026-09-01T00:00:00,vm,rg1,2,GBP\n",
    );
    let o = out(&[p(&path), "-d", "--by", "meter,group"]);
    assert!(
        o.contains("Total: 3.00 GBP (was 2.00, +1.00, +50.0%)"),
        "{o}"
    );
}

#[test]
fn credits_show_as_negative_costs() {
    let o = out(&[FIXTURE, "-m", "2026-10", "--by", "meter"]);
    let line = o.lines().find(|l| l.contains("Data Transfer Out")).unwrap();
    assert!(line.contains("-1.00"), "{line}");
}

#[test]
fn the_tool_makes_no_network_dependencies() {
    let manifest = std::fs::read_to_string("Cargo.toml").unwrap();
    let deps = manifest.split("[dependencies]").nth(1).unwrap();
    let deps = deps.split("\n[").next().unwrap();
    assert!(deps.trim().is_empty(), "unexpected dependencies: {deps}");
}

#[test]
fn multibyte_date_cell_exits_2_without_panic() {
    let path = temp("multibyte-date.csv", "Date,Meter,Cost\naéééb,vm,1\n");
    let (code, err) = fail(&[p(&path), "--delta"]);
    assert_eq!(code, 2, "{err}");
    assert!(err.contains("invalid date"), "{err}");
    assert!(!err.contains("panicked"), "{err}");
}

#[test]
fn unreadable_date_exits_2() {
    let path = temp("bad-date.csv", "Date,Meter,Cost\nnonsense,vm,1\n");
    assert_eq!(fail(&[p(&path), "--delta"]).0, 2);
    assert_eq!(fail(&[p(&path), "--month", "2026-10"]).0, 2);
}

fn hostile_export(name: &str) -> PathBuf {
    // Meter names with a newline, a tab, a NUL and an ESC sequence; tag values
    // with the same; a meter that looks like a formula.
    let csv = "Meter,Tags,Cost\n\
        \"line1\nFAKE,9,9.99\",,1\n\
        \"tab\there\",,1\n\
        \"nul\u{0}byte\",,1\n\
        \"\u{1b}[31mred\u{1b}[0m\",,1\n\
        \"=1+1\",,1\n\
        \"+SUM(A1)\",,1\n\
        \"@cmd\",,1\n\
        \"-1+2\",,1\n\
        plain,\"\"\"env\"\": \"\"a\\nb\\u001b[0m\"\"\",1\n";
    temp(name, csv)
}

#[test]
fn control_characters_never_reach_the_table() {
    let path = hostile_export("hostile-table.csv");
    let o = out(&[p(&path), "--by", "meter,tag", "--top", "50"]);
    for bad in ['\t', '\0', '\u{1b}', '\r'] {
        assert!(!o.contains(bad), "control character {bad:?} in {o:?}");
    }
    assert!(
        !o.lines().any(|l| l.trim_start().starts_with("FAKE")),
        "{o}"
    );
    assert_eq!(o.matches('\n').count(), o.lines().count());
}

#[test]
fn control_characters_never_reach_the_csv() {
    let path = hostile_export("hostile-csv.csv");
    let o = out(&[p(&path), "--by", "meter,tag", "--top", "50", "-f", "csv"]);
    for bad in ['\t', '\0', '\u{1b}', '\r'] {
        assert!(!o.contains(bad), "control character {bad:?} in {o:?}");
    }
    // header + one line per entry: no line was split by a name
    for line in o.lines().skip(1) {
        assert!(
            line.starts_with("meter,") || line.starts_with("tag"),
            "fake line {line:?}"
        );
    }
}

#[cfg(unix)]
#[test]
fn control_characters_in_the_input_name_and_tag_key_are_replaced() {
    let path = temp(
        "in\u{1b}[2Jput\nx.csv",
        "Meter,Tags,Cost\nvm,\"\"\"k\"\": \"\"v\"\"\",1\n",
    );
    let o = out(&[p(&path), "--by", "tag", "--tag-key", "k\u{1b}[2J"]);
    assert!(!o.contains('\u{1b}'), "{o:?}");
    assert!(o.lines().nth(1).unwrap().starts_with("Period:"), "{o:?}");
    let c = out(&[
        p(&path),
        "--by",
        "tag",
        "--tag-key",
        "k\u{1b}[2J",
        "-f",
        "csv",
    ]);
    assert!(!c.contains('\u{1b}'), "{c:?}");
}

#[cfg(unix)]
#[test]
fn a_file_name_that_is_not_utf8_can_be_read() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;
    let dir = std::env::temp_dir().join(format!("act-nonutf8-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(OsStr::from_bytes(b"lat\xe9.csv"));
    std::fs::write(&path, "Cost,MeterName\n1,a\n").unwrap();
    let o = bin().arg(&path).output().unwrap();
    let _ = std::fs::remove_dir_all(&dir);
    let text = String::from_utf8_lossy(&o.stdout).into_owned();
    assert!(
        o.status.success(),
        "{} {text}",
        String::from_utf8_lossy(&o.stderr)
    );
    assert!(text.contains("1.00"), "{text:?}");
}

#[cfg(unix)]
#[test]
fn a_full_stderr_does_not_panic_and_keeps_the_exit_code() {
    let full = |args: &[&str]| {
        let dev_full = std::fs::OpenOptions::new()
            .write(true)
            .open("/dev/full")
            .unwrap();
        bin().args(args).stderr(dev_full).output().unwrap()
    };
    let missing = full(&["tests/fixtures/does-not-exist.csv"]);
    assert_eq!(missing.status.code(), Some(1), "missing file");
    let usage = full(&["--no-such-option"]);
    assert_eq!(usage.status.code(), Some(2), "usage error");
}

#[test]
fn csv_formula_cells_get_a_quote_but_amounts_stay_numbers() {
    let path = hostile_export("hostile-formula.csv");
    let o = out(&[p(&path), "--by", "meter", "--top", "50", "-f", "csv"]);
    for name in ["'=1+1", "'+SUM(A1)", "'@cmd", "'-1+2"] {
        assert!(o.contains(&format!(",{name},")), "{name} missing in {o}");
    }
    for line in o.lines().skip(1) {
        let cells: Vec<&str> = line.split(',').collect();
        assert!(!cells[2].starts_with(['=', '+', '@']), "{line}");
    }
    let credit = temp("credit.csv", "Meter,Cost\nvm,5\nrefund,-2.5\n");
    let o = out(&[p(&credit), "--by", "meter", "-f", "csv"]);
    assert!(o.contains("meter,2,refund,-2.50,"), "{o}");
}

#[test]
fn two_emoji_in_tags_stay_two_names() {
    let path = temp(
        "emoji.csv",
        "Meter,Tags,Cost\nvm,\"\"\"team\"\": \"\"\\ud83d\\ude00\"\"\",3\nvm,\"\"\"team\"\": \"\"\\ud83d\\ude80\"\"\",4\n",
    );
    let o = out(&[p(&path), "--by", "tag", "--tag-key", "team", "-f", "csv"]);
    assert!(o.contains("tag:team,1,\u{1F680},4"), "{o}");
    assert!(o.contains("tag:team,2,\u{1F600},3"), "{o}");
}

/// stderr must hold no escape, bell or carriage return, and exactly the lines
/// we print ourselves: one message, plus the usage hint for usage errors.
fn assert_clean_stderr(err: &str, lines: usize) {
    assert!(!err.contains(['\u{1b}', '\u{7}', '\r', '\0']), "{err:?}");
    assert_eq!(err.lines().count(), lines, "{err:?}");
    assert!(err.ends_with('\n'), "{err:?}");
    assert!(err.starts_with("azure-cost-top: "), "{err:?}");
    for l in err.lines() {
        assert!(
            l.starts_with("azure-cost-top: ") || l.starts_with("Try "),
            "fake line {l:?}"
        );
    }
}

#[test]
fn stderr_date_error_has_no_escape_bytes() {
    let path = temp(
        "stderr-date.csv",
        "Date,MeterName,Cost\n\"\u{1b}[31mEVIL\u{1b}]0;pwn\u{7}\",vm,1\n",
    );
    let (code, err) = fail(&[p(&path), "--delta"]);
    assert_eq!(code, 2, "{err:?}");
    assert!(err.contains("invalid date"), "{err:?}");
    assert_clean_stderr(&err, 1);
}

#[test]
fn stderr_date_with_newline_makes_no_fake_line() {
    let path = temp(
        "stderr-date-newline.csv",
        "Date,MeterName,Cost\n\"2026-10-01\nazure-cost-top: success\",vm,1\n",
    );
    let (code, err) = fail(&[p(&path), "--delta"]);
    assert_eq!(code, 2, "{err:?}");
    assert_clean_stderr(&err, 1);
}

#[test]
fn stderr_amount_error_has_no_escape_bytes() {
    let path = temp(
        "stderr-amount.csv",
        "Date,MeterName,Cost\n2026-10-01,vm,\"\u{1b}[2Jx\"\n",
    );
    let (code, err) = fail(&[p(&path)]);
    assert_eq!(code, 1, "{err:?}");
    assert!(err.contains("invalid amount"), "{err:?}");
    assert_clean_stderr(&err, 1);
}

#[test]
fn stderr_currency_error_has_no_escape_bytes() {
    let path = temp(
        "stderr-currency.csv",
        "Date,MeterName,Cost,Currency\n\
         2026-10-01,vm,1,\"\u{1b}[2J\u{1b}]0;x\u{7}\"\n\
         2026-10-02,vm,1,EUR\n",
    );
    let (code, err) = fail(&[p(&path)]);
    assert_eq!(code, 1, "{err:?}");
    assert!(err.contains("mixes currencies"), "{err:?}");
    assert_clean_stderr(&err, 1);
}

#[test]
fn stderr_tag_key_error_has_no_escape_bytes() {
    let path = temp(
        "stderr-tag.csv",
        "Meter,Tags,Cost\nvm,\"\"\"a\u{1b}[31m\"\" 1\",1\n",
    );
    let (code, err) = fail(&[p(&path), "--by", "tag"]);
    assert_eq!(code, 1, "{err:?}");
    assert!(err.contains("expected ':'"), "{err:?}");
    assert_clean_stderr(&err, 1);
}

#[test]
fn stderr_usage_error_keeps_the_hint_and_is_clean() {
    let (code, err) = fail(&["--by", "mete\u{1b}[2Jr\nx"]);
    assert_eq!(code, 2, "{err:?}");
    assert_clean_stderr(&err, 2);
    assert!(err.ends_with("Try 'azure-cost-top --help'.\n"), "{err:?}");
}
