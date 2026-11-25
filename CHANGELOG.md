# Changelog

## [Unreleased]

- Fix: a date cell of 8 bytes with a multibyte character no longer panics; an unreadable date exits 2.
- Fix: control characters in names, the input name and tag values are replaced in the table and in CSV.
- Fix: a CSV text cell starting with `=`, `+`, `-` or `@` gets a single quote in front.
- Fix: a tag `\uXXXX` surrogate pair is read as one character, so two emoji no longer share a name.
- Fix: error messages on stderr no longer carry terminal escapes or fake lines from cell text.
- Fix: a full or closed stderr no longer panics; the exit code is kept.
- Fix: a file name that is not valid UTF-8 can be read (Unix).
- Fix: the docs tests read CRLF files; `.gitattributes` sets LF.

- Documentation: README.nl.md now describes safe text. Dutch README and quality page, CONTRIBUTING, SECURITY, Quick start, Exit codes, Limitations and Maintenance sections.

## [0.1.0] - 2026-11-09

First release.

- Reads an Azure Cost Management CSV export from a file or standard input.
- Top costs per meter, resource group and tag (`--by`, `--tag-key`, `--top`).
- Month filter and month-over-month delta (`--month`, `--delta`, `--vs`).
- Table, CSV and JSON output.
- Exact decimal sums; refuses to mix currencies (`--currency`).
- No dependencies, no network.
