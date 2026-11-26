# azure-cost-top

[![crates.io](https://img.shields.io/crates/v/azure-cost-top.svg)](https://crates.io/crates/azure-cost-top) [![downloads](https://img.shields.io/crates/d/azure-cost-top.svg)](https://crates.io/crates/azure-cost-top) [![CI](https://github.com/qwpp7g79y4-maker/azure-cost-top/actions/workflows/ci.yml/badge.svg)](https://github.com/qwpp7g79y4-maker/azure-cost-top/actions/workflows/ci.yml) [![PEPAI checked](https://img.shields.io/endpoint?url=https%3A%2F%2Fraw.githubusercontent.com%2Fqwpp7g79y4-maker%2Fazure-cost-top%2Fmain%2F.github%2Fbadge.json)](QUALITY.md) [![MSRV 1.85](https://img.shields.io/badge/MSRV-1.85-blue.svg)](https://github.com/qwpp7g79y4-maker/azure-cost-top/blob/main/Cargo.toml) [![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](https://github.com/qwpp7g79y4-maker/azure-cost-top#license)

Offline CLI that reads an **Azure Cost Management CSV export** and shows the
top costs per **meter**, **resource group** and **tag**, with an optional
**month-over-month delta**.

The tests and the measured numbers are in `QUALITY.md`.

It reads one local file (or standard input). It uses no network and no Azure
SDK, and it has no dependencies besides the Rust standard library.

Test count: 197

## Installation

Version 0.1.0. From crates.io:

```sh
cargo install azure-cost-top
```

To build from source, run `cargo build --release`; the binary is `target/release/azure-cost-top`.

## Quick start

From a checkout of this repository, with a Rust toolchain installed:

```sh
cargo build --release
./target/release/azure-cost-top tests/fixtures/sample.csv --by meter -n 3
```

This reads the small sample export in `tests/fixtures/` and prints the three
most expensive meters. Nothing is written to disk.

## Quickstart in 5 minutes

1. Install a Rust toolchain (1.85 or newer).
2. Run `cargo build --release` in this directory.
3. Export your costs from Azure Cost Management as CSV.
4. Run `./target/release/azure-cost-top export.csv`.
5. Add `--delta` to see the latest month against the month before it.

## Usage

```sh
azure-cost-top export.csv                        # all months, all sections
azure-cost-top export.csv --month 2026-10        # one month
azure-cost-top export.csv --delta                # latest month vs the month before
azure-cost-top export.csv -m 2026-10 --vs 2026-07
azure-cost-top export.csv --by meter,group -n 5  # only two sections, 5 lines each
azure-cost-top export.csv --by tag --tag-key team
azure-cost-top export.csv --format csv           # or json
cat export.csv | azure-cost-top -
```

`azure-cost-top --help` lists every option.

## Example

```sh
azure-cost-top tests/fixtures/sample.csv --delta --by group -n 3
```

```
azure-cost-top 0.1.0 - tests/fixtures/sample.csv
Period: 2026-10 vs 2026-09   Rows: 7
Total: 152.00 EUR (was 112.00, +40.00, +35.7%)

Top resource group costs (showing 3 of 3)
  #  Name      Cost  Share  Previous   Delta  Delta%
  1  rg-data  64.50  42.4%     59.50   +5.00   +8.4%
  2  rg-web   62.50  41.1%     52.50  +10.00  +19.0%
  3  rg-ops   25.00  16.4%      0.00  +25.00     new
```

## What it does

- **Reads the usual export columns.** The cost column is `CostInBillingCurrency`,
  `PreTaxCost`, `Cost` or `CostUSD`. Other columns: `MeterName`/`Meter`,
  `ResourceGroup`/`ResourceGroupName`, `Date`/`UsageDate`/`UsageDateTime`,
  `Tags`, `BillingCurrencyCode`/`Currency`. Header matching ignores case,
  spaces and punctuation. Dates may be `MM/DD/YYYY`, `YYYY-MM-DD` (with or
  without time), `YYYY/MM/DD` or `YYYYMMDD`. A BOM and CRLF line endings are fine.
- **Ranks per meter, resource group and tag.** Highest cost first, ties by name.
  `-n` (`--top`) sets the number of lines per section (default 10). Resource group names
  are compared case-insensitively and shown in lower case. Credits (negative
  costs) are summed in.
- **Tags.** The `Tags` cell is read as JSON pairs, with or without braces. By
  default one line per `name=value` pair; a row with two tags counts under both,
  so tag lines can add up to more than the total. With `--tag-key team` the
  section groups by the value of that tag, and rows without it appear as
  `(untagged)`. Tag names are matched case-insensitively.
- **Month delta.** `--delta` compares a month (default: the latest in the file)
  with the month before it; `--vs YYYY-MM` picks another month to compare with.
  Each line shows previous cost, delta and delta percentage. A name that did not
  exist in the other month is `new` (no percentage).
- **Safe text.** In the table and in CSV, control characters in names (newline, tab, NUL, ESC) become U+FFFD, and a CSV text cell that starts with `=`, `+`, `-` or `@` gets a single quote in front so a spreadsheet does not read it as a formula. Amounts stay numbers. Error messages on stderr are cleaned the same way. JSON keeps the original text, escaped.
- **Output formats.** `table` (default), `csv` and `json`. CSV and JSON keep
  full precision; the table shows two decimals.
- **Exact amounts.** Costs are parsed as decimals (nine places, `1.2E-5`
  included) and added as integers, so sums do not drift like floats.
- **One currency only.** If the export mixes currencies the run stops and asks
  for `--currency CODE`. The amounts are never added across currencies.
- **Read only.** The input file is never changed or deleted.

## Exit codes

| Code | Meaning |
|---|---|
| 0 | Success (also for `--help` and `--version`). |
| 1 | Problem with the input. The error names the line of the export. |
| 2 | Wrong command line, or a date cell that cannot be read. |

## Limitations

Only the CSV export. No live Azure access, no authentication, no currency
conversion, no reservation or amortisation logic: it ranks the numbers in the
file.

## Maintenance

This is a small tool kept by one person. A patch should come with a test and
arrives through Issues, see `CONTRIBUTING.md`. Security reports go to the
address in `SECURITY.md`. Changes are listed in `CHANGELOG.md`. The tool has no
price list and no dependencies, so there is nothing to refresh besides the
Rust toolchain. A Dutch version of this page is `README.nl.md`.

After a release the tool is dormant: a new version appears only for a reported bug, a request, or a security problem. Security always comes first.

## Development

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo publish --dry-run
```

`QUALITY.md` maps each claim above to the tests that cover it.

## License

MIT OR Apache-2.0, see `LICENSE-MIT` and `LICENSE-APACHE`.

The binary includes no third-party crates; [THIRD-PARTY-LICENSES.html](THIRD-PARTY-LICENSES.html) says so.
