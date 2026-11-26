# Kwaliteit

pep: 7

## Open punten die meetellen

| Punt | Klasse | Aftrek |
|---|---|---|
| Geen THIRD-PARTY-LICENSES.html-pagina en geen link ernaar | S | 3 |

Cijfer: 10 − 3 = 7. Andere open punten in dit bestand hebben geen klasse en tellen niet mee. Ze zijn nog niet hersteld.

Gemeten commit: 5962d17 (de laatste commit die geen puur document is).

Aantal tests: 197

Dit is het aantal `#[test]`-functies in `src/` en `tests/`, en ook wat `cargo test`
als geslaagd meldt. `tests/docs.rs` controleert de Engelse `QUALITY.md` en
`README.md`; dit Nederlandse bestand volgt `QUALITY.md`.

Controles vóór een commit: `cargo fmt --check`, `cargo clippy --all-targets -- -D
warnings`, `cargo test`, `cargo publish --dry-run`.

## Beweringen in de README en hun tests

| Bewering | Tests |
|---|---|
| Leest de gebruikelijke exportkolommen, elke hoofdletterstand | `finds_columns_case_and_space_insensitively`, `uses_alias_priority_for_cost`, `accepts_legacy_column_names`, `legacy_column_names_work` |
| Datumformaten | `parses_us_dates`, `parses_iso_dates`, `parses_iso_with_time`, `parses_slash_year_first_and_compact`, `rejects_impossible_dates`, `leap_years` |
| BOM en CRLF | `strips_bom`, `crlf_line_endings`, `handles_bom_and_crlf` |
| Rangschikking: hoogste eerst, gelijke bedragen op naam, `-n` | `meters_are_sorted_by_cost_descending`, `ties_break_by_name`, `top_cuts_the_list_but_keeps_distinct_count`, `top_limits_lines`, `meter_ranking_is_by_cost` |
| Resourcegroepen hoofdletterongevoelig vergeleken | `groups_merge_case_insensitively`, `resource_groups_merge_by_case` |
| Tegoeden worden meegeteld | `negative_credits_reduce_totals_and_rank_last`, `credits_show_as_negative_costs` |
| Tags als JSON-paren, met of zonder accolades | `without_braces`, `with_braces`, `bare_values_and_null`, `escapes`, `parses_tags_from_a_quoted_cell` |
| Tagparen tellen onder elk paar | `tag_pairs_count_a_row_under_each_pair`, `tag_pairs_without_key` |
| `--tag-key` groepeert op waarde, `(untagged)`, sleutel hoofdletterongevoelig | `tag_key_groups_by_value_and_ignores_key_case`, `tag_key_groups_by_value`, `tag_key_and_currency` |
| Maandfilter | `month_filters_rows_and_total`, `month_filter_limits_total`, `month_without_rows_is_an_error`, `month_without_rows_exits_1` |
| Delta: standaard laatste maand tegen de maand ervoor | `delta_defaults_to_latest_month_vs_the_month_before`, `delta_defaults_to_latest_month`, `prev_wraps_the_year` |
| `--vs` kiest de andere maand en impliceert delta | `vs_sets_the_comparison_month_and_implies_delta`, `explicit_month_pair_with_vs`, `vs_equal_to_month_is_a_usage_error`, `month_vs_and_delta` |
| Deltakolommen, `new` in plaats van een percentage | `table_delta_adds_columns`, `delta_marks_new_meters`, `delta_includes_new_and_vanished_names`, `delta_for_group_values`, `unchanged_entry_shows_zero_percent` |
| Uitvoer als tabel, CSV en JSON | `table_has_heading_and_totals`, `csv_header_and_rows`, `csv_delta_has_extra_columns_and_blank_pct_for_new`, `json_has_expected_fields`, `json_without_delta_uses_null`, `json_braces_balance`, `csv_output_is_machine_readable`, `json_output_has_totals`, `render_dispatches_on_format`, `format_forms` |
| CSV en JSON houden volle precisie, de tabel heeft twee decimalen | `exact_keeps_precision_and_two_decimals`, `fixed2_rounds_half_away_from_zero`, `e_notation_costs_are_summed_exactly` |
| Exacte bedragen, `1.2E-5` inbegrepen | `parses_exponent`, `sums_are_exact`, `parses_decimal`, `rounds_beyond_nine_decimals_half_up`, `rejects_out_of_range` |
| Eén valuta, `--currency` | `mixed_currencies_are_refused`, `currency_filter_selects_one_currency`, `currency_filter_without_match_is_an_error`, `mixed_currency_needs_a_choice` |
| Leest standaardinvoer | `reads_standard_input`, `stdin_dash_is_an_input` |
| Invoer wordt nooit gewijzigd of verwijderd | `does_not_modify_or_delete_the_input` |
| Afsluitcodes 0, 1, 2 en de regel van de fout | `usage_exit_code_is_2`, `input_exit_code_is_1`, `usage_errors_exit_2`, `missing_file_exits_1`, `bad_row_names_its_line`, `bad_cost_names_the_line`, `short_and_long_rows_error_with_line_number`, `help_and_version_exit_zero` |
| Een datumcel wordt op tekengrenzen gesneden; een onleesbare datum geeft exit 2 | `multibyte_cells_are_errors_not_panics`, `multibyte_date_cell_exits_2_without_panic`, `unreadable_date_exits_2`, `date_exit_code_is_2` |
| Besturingstekens komen niet in de tabel of de CSV | `control_characters_never_reach_the_table`, `control_characters_never_reach_the_csv`, `control_characters_in_the_input_name_and_tag_key_are_replaced` |
| Foutmeldingen op stderr bevatten geen escapes of nepregels | `display_cleans_escapes_and_newlines`, `stderr_date_error_has_no_escape_bytes`, `stderr_date_with_newline_makes_no_fake_line`, `stderr_amount_error_has_no_escape_bytes`, `stderr_currency_error_has_no_escape_bytes`, `stderr_tag_key_error_has_no_escape_bytes`, `stderr_usage_error_keeps_the_hint_and_is_clean` |
| CSV-formulecellen krijgen een aanhalingsteken, bedragen blijven getallen | `csv_formula_cells_get_a_quote_but_amounts_stay_numbers` |
| Tag-surrogaatparen `\uXXXX` zijn één teken | `surrogate_pair_is_one_character`, `lone_surrogate_is_one_replacement_and_keeps_its_neighbour`, `bad_unicode_escapes_error`, `two_emoji_in_tags_stay_two_names` |
| Geen dependencies, geen netwerk | `the_tool_makes_no_network_dependencies` |
| `--help` toont elke optie | `help_mentions_every_option` |
| Licenties, changelog, versie | `licenses_are_present_and_match_cargo_toml`, `changelog_has_the_current_version` |

## Wat deze pagina niet beweert

Deze pagina legt controles vast. Ze is geen garantie en geen fuzzrapport.

Op 2026-11-25, op deze machine, Linux, in een schone worktree van commit 6461226, eindigden deze vier commando's met exit 0: cargo fmt --check, cargo clippy --all-targets --locked -- -D warnings, cargo test --locked en cargo test --release --locked. Debug en release haalden dezelfde tests. 0 mislukt. Eerdere runs op andere commits staan hieronder alleen als geschiedenis.

Het CI-bestand noemt deny en audit. Die taken hebben voor deze boom niet op GitHub gedraaid, omdat deze boom niet is gepusht.

`README.nl.md` en `QUALITY.nl.md` zijn de Nederlandse versies. `task.md` en `blueprint.md` zijn lokale werknotities: ze staan in `.gitignore` en worden niet gecommit.

## Gemeten run

Datum: 2026-11-25.

Commit: 5962d17, schone worktree, Linux, cargo 1.99.0.

- cargo fmt --check: exit 0.
- cargo clippy --all-targets --locked -- -D warnings: exit 0.
- cargo test --locked: exit 0, 197 geslaagd, 0 mislukt (141 in de bibliotheek, 0 in het programma, 47 in tests/cli.rs, 9 in tests/docs.rs, 0 doc-tests).
- cargo test --release --locked: exit 0, dezelfde 197 geslaagd, 0 mislukt.

Toolchain 1.85 (cargo 1.85.1, rustc 1.85.1), gedraaid op 2026-11-25 op dezelfde boom en dezelfde code als 5962d17 (HEAD ab8a774 verschilt alleen in documenten), Linux:

- cargo +1.85 fmt --check: exit 0.
- cargo +1.85 clippy --all-targets --locked -- -D warnings: exit 0.
- cargo +1.85 test --locked: exit 0, 197 geslaagd, 0 mislukt (141 in de bibliotheek, 0 in het programma, 47 in tests/cli.rs, 9 in tests/docs.rs, 0 doc-tests).
- cargo +1.85 test --release --locked: exit 0, 197 geslaagd, 0 mislukt, 0 genegeerd, geteld op 2026-11-25 uit de regels "test result" (141 in de bibliotheek, 0 in het programma, 47 in tests/cli.rs, 9 in tests/docs.rs, 0 doc-tests).

## Rood vooraf, groen erna (B1)

De zes `stderr_*`-tests in `tests/cli.rs` zijn gedraaid tegen de `src/` van b30d9c4 (vóór de fix). Commando: cargo test --locked --test cli stderr_. Uitslag, rood:

`test result: FAILED. 0 passed; 6 failed; 0 ignored; 0 measured; 37 filtered out; finished in 0.00s`

Op 5446a27 slagen dezelfde tests.

## Rood vooraf, groen erna (B-1, B-2, B-3 van azure-eind-98eba9b)

Nieuwe tests zijn gedraaid tegen de onherstelde `src/main.rs` en `README.nl.md` van 98eba9b.

B-1, commando cargo test --locked --test cli a_full, rood:

`test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 43 filtered out; finished in 0.00s`

B-3, commando cargo test --locked --test docs readme_nl, rood:

`test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.00s`

B-2, commando cargo test --locked --test docs, in een kopie van 98eba9b (`git archive`) waarvan de tekstbestanden met `sed -i 's/$/\r/'` CRLF kregen, rood:

`test result: FAILED. 7 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s` (`readme_states_the_limits`, zoals de review vond).

Op d83b8fd slagen alle drie: 194 geslaagd. Voor B-2 is er geen aparte nieuwe test: `read()` in `tests/docs.rs` maakt nu van `\r\n` een `\n`, en de CRLF-run hierboven is het bewijs. `.gitattributes` zet `* text=auto eol=lf`. Windows zelf is niet uitgevoerd.

## Rood vooraf, groen erna (B-1 van azure-eind-1f954c4)

De invoer wordt als `PathBuf` bewaard; to_string_lossy staat alleen nog in de tekst van de uitvoer en van fouten. De nieuwe test `a_file_name_that_is_not_utf8_can_be_read` (`#[cfg(unix)]`) maakt met `OsStrExt::from_bytes` een bestand `lat` + byte e9 + `.csv` en eist een bon van 1.00. Optiewaarden worden nog als tekst gelezen.

Gedraaid tegen een `git archive` van 1f954c4 met de nieuwe `tests/cli.rs`, commando cargo test --locked --test cli a_file_name, rood (de melding was: cannot read ...latU+FFFD.csv: No such file or directory):

`test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 44 filtered out; finished in 0.00s`

Op 13a7c65 slaagt hij: 195 geslaagd, in debug en in release. De GitHub-run is Not run. Windows is niet uitgevoerd.

## Rood vooraf, groen erna (gesloten standaarduitvoer)

Met standaarduitvoer gesloten bij de start (`azure-cost-top x.csv >&-`) gaf het programma exit 0: Rust vervangt een gesloten descriptor 1 vóór `main` door /dev/null, dus het schrijven slaagde en de uitvoer ging stil verloren. `src/stdout_probe.rs` (overgenomen uit cwlogs-retentie-bug) legt in `.init_array` vast of descriptor 1 gesloten was; `main` meldt dan `standard output is closed` en stopt met exit 2. `1>/dev/null` blijft exit 0. De probe is de enige `unsafe`-code, alleen in het programma (Linux; elders meldt hij niet gesloten).

Nieuwe tests (`#[cfg(target_os = "linux")]`): `a_closed_stdout_at_start_is_exit_2_with_a_message` en `stdout_on_dev_null_is_still_exit_0`. Rood op c8adc52 met de nieuwe test, commando cargo test --locked --test cli closed_stdout:

`test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 46 filtered out; finished in 0.01s` (left: Some(0), right: Some(2))

Op 5962d17 slagen beide: 197 geslaagd in debug en in release. Gecontroleerd en geen gat gevonden: er is geen optie `--output` en geen `build.rs`, dus er is geen symlink of speciaal bestand om naar te schrijven en geen `.git/HEAD` om te lezen.

## Windows-bestandsnaam (B1 van azure-eind-522d116)

`control_characters_in_the_input_name_and_tag_key_are_replaced` maakt een bestand met ESC en een regeleinde in de naam. Windows weigert zulke namen, dus deze test heeft `#[cfg(unix)]` en draait alleen op Unix. Geen andere test bouwt zo'n naam. Er is geen rode cargo-run hiervoor: de fout zit op Windows en is daar niet uitgevoerd. Op Linux slaagde de test ervoor en erna (192 geslaagd).

## Nog open

Hier niet gerepareerd: bevindingen a tot en met l van azure-eind-1f954c4 (die blijven open), bevindingen 1 tot en met 7 van de review azure-eind-98eba9b (dubbele kolomkoppen, dubbele tagsleutels, een lege valutacel, JSON houdt C1 en bidi, geheugengebruik, het bedragbereik, MSRV 1.75 niet gedraaid), bidi- en zero-width-tekens in tabel en CSV, geheugengebruik en CI die nog niet gedraaid heeft. Een volle stderr is hersteld (B-1).

## Niet uitgevoerd

Gedraaid op 5962d17 op 2026-11-25: cargo fmt --check, clippy -D warnings, cargo test --locked en cargo test --release --locked, 197 geslaagd (zie Gemeten run). Eerdere runs: 6461226, 195 geslaagd; 13a7c65, 195 geslaagd; d83b8fd, 194 geslaagd; 2026-11-24, 192 geslaagd (vóór het herstel van B-1 tot en met B-3).

Niet uitgevoerd (not run) in deze meting: Windows, cargo deny, cargo audit, cargo publish --dry-run, een GitHub Actions-run, fuzzing en mutation testing.
