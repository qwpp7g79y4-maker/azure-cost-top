# Quality

Tested commit: 5962d17 (the last commit that is not a pure document).

Test count: 197

This is the number of `#[test]` functions in `src/` and `tests/`, which is also
what `cargo test` reports as passed. `tests/docs.rs` fails when this number,
the one in `README.md` or the suite changes without the others.

The Dutch version is `QUALITY.nl.md`.

Checks before a commit: `cargo fmt --check`, `cargo clippy --all-targets -- -D
warnings`, `cargo test`, `cargo publish --dry-run`.

## README claims and their tests

| Claim | Tests |
|---|---|
| Reads the usual export columns, any header case | `finds_columns_case_and_space_insensitively`, `uses_alias_priority_for_cost`, `accepts_legacy_column_names`, `legacy_column_names_work` |
| Date formats | `parses_us_dates`, `parses_iso_dates`, `parses_iso_with_time`, `parses_slash_year_first_and_compact`, `rejects_impossible_dates`, `leap_years` |
| BOM and CRLF | `strips_bom`, `crlf_line_endings`, `handles_bom_and_crlf` |
| Ranking: highest first, ties by name, `-n` | `meters_are_sorted_by_cost_descending`, `ties_break_by_name`, `top_cuts_the_list_but_keeps_distinct_count`, `top_limits_lines`, `meter_ranking_is_by_cost` |
| Resource groups compared case-insensitively | `groups_merge_case_insensitively`, `resource_groups_merge_by_case` |
| Credits are summed in | `negative_credits_reduce_totals_and_rank_last`, `credits_show_as_negative_costs` |
| Tags as JSON pairs, with or without braces | `without_braces`, `with_braces`, `bare_values_and_null`, `escapes`, `parses_tags_from_a_quoted_cell` |
| Tag pairs count under each pair | `tag_pairs_count_a_row_under_each_pair`, `tag_pairs_without_key` |
| `--tag-key` groups by value, `(untagged)`, key case-insensitive | `tag_key_groups_by_value_and_ignores_key_case`, `tag_key_groups_by_value`, `tag_key_and_currency` |
| Month filter | `month_filters_rows_and_total`, `month_filter_limits_total`, `month_without_rows_is_an_error`, `month_without_rows_exits_1` |
| Delta: default latest month vs the month before | `delta_defaults_to_latest_month_vs_the_month_before`, `delta_defaults_to_latest_month`, `prev_wraps_the_year` |
| `--vs` picks the other month and implies delta | `vs_sets_the_comparison_month_and_implies_delta`, `explicit_month_pair_with_vs`, `vs_equal_to_month_is_a_usage_error`, `month_vs_and_delta` |
| Delta columns, `new` instead of a percentage | `table_delta_adds_columns`, `delta_marks_new_meters`, `delta_includes_new_and_vanished_names`, `delta_for_group_values`, `unchanged_entry_shows_zero_percent` |
| Table, CSV and JSON output | `table_has_heading_and_totals`, `csv_header_and_rows`, `csv_delta_has_extra_columns_and_blank_pct_for_new`, `json_has_expected_fields`, `json_without_delta_uses_null`, `json_braces_balance`, `csv_output_is_machine_readable`, `json_output_has_totals`, `render_dispatches_on_format`, `format_forms` |
| CSV and JSON keep full precision, table has two decimals | `exact_keeps_precision_and_two_decimals`, `fixed2_rounds_half_away_from_zero`, `e_notation_costs_are_summed_exactly` |
| Exact amounts, `1.2E-5` included | `parses_exponent`, `sums_are_exact`, `parses_decimal`, `rounds_beyond_nine_decimals_half_up`, `rejects_out_of_range` |
| One currency only, `--currency` | `mixed_currencies_are_refused`, `currency_filter_selects_one_currency`, `currency_filter_without_match_is_an_error`, `mixed_currency_needs_a_choice` |
| Reads standard input | `reads_standard_input`, `stdin_dash_is_an_input` |
| Input is never changed or deleted | `does_not_modify_or_delete_the_input` |
| Exit codes 0, 1, 2 and the line of the error | `usage_exit_code_is_2`, `input_exit_code_is_1`, `usage_errors_exit_2`, `missing_file_exits_1`, `bad_row_names_its_line`, `bad_cost_names_the_line`, `short_and_long_rows_error_with_line_number`, `help_and_version_exit_zero` |
| A date cell is cut on character boundaries; an unreadable date exits 2 | `multibyte_cells_are_errors_not_panics`, `multibyte_date_cell_exits_2_without_panic`, `unreadable_date_exits_2`, `date_exit_code_is_2` |
| Control characters never reach the table or the CSV | `control_characters_never_reach_the_table`, `control_characters_never_reach_the_csv`, `control_characters_in_the_input_name_and_tag_key_are_replaced` |
| Error messages on stderr never carry escapes or fake lines | `display_cleans_escapes_and_newlines`, `stderr_date_error_has_no_escape_bytes`, `stderr_date_with_newline_makes_no_fake_line`, `stderr_amount_error_has_no_escape_bytes`, `stderr_currency_error_has_no_escape_bytes`, `stderr_tag_key_error_has_no_escape_bytes`, `stderr_usage_error_keeps_the_hint_and_is_clean` |
| CSV formula cells get a quote, amounts stay numbers | `csv_formula_cells_get_a_quote_but_amounts_stay_numbers` |
| Tag `\uXXXX` surrogate pairs are one character | `surrogate_pair_is_one_character`, `lone_surrogate_is_one_replacement_and_keeps_its_neighbour`, `bad_unicode_escapes_error`, `two_emoji_in_tags_stay_two_names` |
| A file name that is not UTF-8 can be read (Unix) | `a_file_name_that_is_not_utf8_can_be_read` |
| A full stderr does not panic and keeps the exit code (Unix) | `a_full_stderr_does_not_panic_and_keeps_the_exit_code` |
| `README.nl.md` makes the safe-text promise too | `readme_nl_makes_the_safe_text_promise_too` |
| No dependencies, no network | `the_tool_makes_no_network_dependencies` |
| `--help` lists every option | `help_mentions_every_option` |
| Licenses, changelog, version | `licenses_are_present_and_match_cargo_toml`, `changelog_has_the_current_version` |

## What this page does not claim

This page records checks. It is not a guarantee and it is not a fuzz report.

On 2026-11-25, on this machine, Linux, in a clean worktree of commit 6461226, these four commands finished with exit 0: cargo fmt --check, cargo clippy --all-targets --locked -- -D warnings, cargo test --locked, and cargo test --release --locked. Debug and release passed the same tests. 0 failed. Earlier runs on other commits are listed below as history only.

The CI file lists deny and audit. Those jobs have not run on GitHub for this tree, because this tree is not pushed.

`README.nl.md` and `QUALITY.nl.md` are the Dutch versions. `task.md` and `blueprint.md` are local working notes: they are listed in `.gitignore` and are not committed.

## Measured run

Date: 2026-11-25.

Commit: 5962d17, clean worktree, Linux, cargo 1.99.0.

- cargo fmt --check: exit 0.
- cargo clippy --all-targets --locked -- -D warnings: exit 0.
- cargo test --locked: exit 0, 197 passed, 0 failed (141 in the library, 0 in the binary, 47 in tests/cli.rs, 9 in tests/docs.rs, 0 doc-tests).
- cargo test --release --locked: exit 0, the same 197 passed, 0 failed.

## Red before, green after (B1)

The six `stderr_*` tests in `tests/cli.rs` were run against the `src/` of b30d9c4 (before the fix). Command: cargo test --locked --test cli stderr_. Result, red:

`test result: FAILED. 0 passed; 6 failed; 0 ignored; 0 measured; 37 filtered out; finished in 0.00s`

On 5446a27 the same tests pass.

## Red before, green after (B-1, B-2, B-3 of azure-eind-98eba9b)

New tests were run against the unfixed `src/main.rs` and `README.nl.md` of 98eba9b.

B-1, command cargo test --locked --test cli a_full, red:

`test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 43 filtered out; finished in 0.00s`

B-3, command cargo test --locked --test docs readme_nl, red:

`test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.00s`

B-2, command cargo test --locked --test docs, in a copy of 98eba9b (`git archive`) whose text files were turned into CRLF with `sed -i 's/$/\r/'`, red:

`test result: FAILED. 7 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s` (`readme_states_the_limits`, as the review found).

On d83b8fd all three pass: 194 passed. The B-2 fix has no separate new test: `read()` in `tests/docs.rs` now turns `\r\n` into `\n`, and the CRLF run above is the proof. `.gitattributes` sets `* text=auto eol=lf`. Windows itself was not run.

## Red before, green after (B-1 of azure-eind-1f954c4)

The input is kept as a `PathBuf`; to_string_lossy is used only for the text in the output and in errors. The new test `a_file_name_that_is_not_utf8_can_be_read` (`#[cfg(unix)]`) makes a file named `lat` + byte e9 + `.csv` with `OsStrExt::from_bytes` and demands a bill of 1.00. Option values are still read as text.

Run against a `git archive` of 1f954c4 with the new `tests/cli.rs`, command cargo test --locked --test cli a_file_name, red (the message was: cannot read ...lat�.csv: No such file or directory):

`test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 44 filtered out; finished in 0.00s`

On 13a7c65 it passes: 195 passed in debug and in release. The GitHub run is Not run. Windows was not run.

## Red before, green after (closed standard output)

With standard output closed at start (`azure-cost-top x.csv >&-`) the binary exited 0: Rust replaces a closed descriptor 1 with /dev/null before `main`, so the write succeeded and the output was lost silently. `src/stdout_probe.rs` (ported from cwlogs-retentie-bug) records in `.init_array` whether descriptor 1 was closed; `main` then prints `standard output is closed` and exits 2. `1>/dev/null` stays exit 0. The probe is the only `unsafe` code, in the binary only (Linux; elsewhere it reports not closed).

New tests (`#[cfg(target_os = "linux")]`): `a_closed_stdout_at_start_is_exit_2_with_a_message` and `stdout_on_dev_null_is_still_exit_0`. Red on c8adc52 with the new test, command cargo test --locked --test cli closed_stdout:

`test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 46 filtered out; finished in 0.01s` (left: Some(0), right: Some(2))

On 5962d17 both pass: 197 passed in debug and in release. Checked and found no gap: there is no `--output` option and no `build.rs`, so there is no symlink or special-file write and no `.git/HEAD` read to test.

## Windows file name (B1 of azure-eind-522d116)

`control_characters_in_the_input_name_and_tag_key_are_replaced` makes a file whose name holds ESC and a newline. Windows refuses such names, so this test has `#[cfg(unix)]` and runs only on Unix. No other test builds such a name. There is no red cargo run for this: the failure is on Windows, and it was not run there. On Linux the test passed before and after (192 passed).

## Still open

Not fixed here: N-a to N-l of azure-eind-1f954c4 (they stay open), N1 to N7 of the review azure-eind-98eba9b (duplicate column headers, duplicate tag keys, an empty currency cell, JSON keeps C1 and bidi, memory use, the amount range, MSRV 1.75 not run), bidi and zero-width characters in table and CSV, memory use, and CI that has not run. A full stderr is fixed (B-1).

## Not run

Run on 5962d17 on 2026-11-25: cargo fmt --check, clippy -D warnings, cargo test --locked and cargo test --release --locked, 197 passed (see Measured run). Earlier runs: 6461226, 195 passed; 13a7c65, 195 passed; d83b8fd, 194 passed; 2026-11-24, 192 passed (before the B-1 to B-3 fix).

Not run in this measurement: Windows, cargo deny, cargo audit, cargo publish --dry-run, a GitHub Actions run, fuzzing, and mutation testing.
