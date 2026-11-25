# Security

## Reporting a problem

Send security reports by e-mail to pep_ai@icloud.com. Please do not open a
public issue for a security problem. Say which version you used
(`azure-cost-top --version`), what you did and what happened. Remove real
billing data from anything you attach.

## What to expect

Reports are read by one person, on a best-effort basis. There is no fixed
response time and no bug bounty.

## Scope

azure-cost-top reads one local CSV file or standard input and prints to
standard output. It uses no network and has no dependencies besides the Rust
standard library. It never changes or deletes the input file. Reports about
crashes, wrong sums or misleading output on crafted input are in scope.

## Supported versions

Only the latest released version.
