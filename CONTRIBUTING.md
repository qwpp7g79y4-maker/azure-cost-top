# Contributing

Thanks for looking at azure-cost-top. It is a small, offline tool with no
dependencies, and it should stay that way.

## Issues first

Patches come in through Issues. Open an issue that describes the problem or
the change, and attach the patch there. Please do not send secrets or real
billing exports; trim an export to a few rows and remove names you do not want
to share.

## A patch needs a test

A patch should come with a test. A bug fix needs a test that fails before the
fix and passes after it. A new option or column needs a test and a line in
`README.md`. Tests live next to the code in `src/` and in `tests/`.

## Before you send it

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

`tests/docs.rs` checks that the test count in `README.md` and `QUALITY.md`
equals the number of tests, so update both when you add or remove a test.

## Rules

- No new dependencies and no network access.
- The input file is never changed.
- Keep `README.md` and `README.nl.md` in step.
- Licence: contributions are under `MIT OR Apache-2.0`, like the rest.

Security problems do not belong in a public issue, see `SECURITY.md`.
