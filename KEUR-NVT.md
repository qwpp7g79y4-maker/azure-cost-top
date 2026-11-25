# Not applicable

One line per module the documentation standard lists but this tool does not have.

build.rs: the crate has no build script; it has no dependencies and generates no code at build time.
about.hbs: no third-party licence report is generated, because the crate has no dependencies to list.
about.toml: cargo-about is not used, since there are no dependencies whose licences need collecting.
man: no man page; the options are listed by `--help` and in README.md and README.nl.md.
docs: no separate documentation folder; README.md, README.nl.md and QUALITY.md cover the whole tool.
examples: no examples folder; the sample export lives in tests/fixtures/ and the README uses it.
samples: no samples folder; the one small sample export is tests/fixtures/sample.csv, used by the tests.
scripts: no helper scripts; build and test are plain cargo commands listed in the README.
.github/workflows/release.yml: no release workflow; nothing is released from CI and no tag is made.
