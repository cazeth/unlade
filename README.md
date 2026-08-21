# unlade

[![CI](https://github.com/cazeth/unlade/actions/workflows/ci.yml/badge.svg)](https://github.com/cazeth/unlade/actions/workflows/ci.yml)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](#license)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](#minimum-supported-rust-version)

Filters the [crates.io database dump](https://crates.io/data-access) by name,
update date, downloads, and dependent count.

```console
$ unlade 2026-08-18-020030/data --updated-before 2020-01-01 --min-downloads 5000000
quickcheck  2019-06-01  41207714
term        2018-03-05  18922065
```

## Installation

```console
$ cargo install --locked unlade-cli
```

## Usage

Download a dump and point `unlade` at its `data` directory.

```console
$ curl -O https://static.crates.io/db-dump.tar.gz
$ tar -xzf db-dump.tar.gz
$ unlade */data --updated-before 2021-01-01
```

| Option | Effect |
| --- | --- |
| `--updated-before <YYYY-MM-DD>` | Keep crates last updated before that day |
| `--updated-after <YYYY-MM-DD>` | Keep crates last updated on or after that day |
| `--name-contains <TEXT>` | Keep crates whose name contains the text |
| `--min-downloads <COUNT>` | Keep crates downloaded at least that many times |
| `--min-dependents <COUNT>` | Keep crates that at least that many crates depend on |
| `--limit <COUNT>` | Stop after that many crates |

Dates are UTC; output follows dump order. Count filters add their counts as
columns. Dependent counts use each crate's greatest published semantic version
and include optional ordinary dependencies, but not build or development ones.
This requires the large `versions.csv` and `dependencies.csv` files, so it is
slower than other filters.

## Workspace

| Crate | Purpose |
| --- | --- |
| [`unlade-core`](crates/unlade-core) | Shared types |
| [`unlade-parser`](crates/unlade-parser) | Shared CSV parsing |
| [`unlade-crates-parser`](crates/unlade-crates-parser) | Crate and download parsing |
| [`unlade-dependencies-parser`](crates/unlade-dependencies-parser) | Dependent counting |
| [`unlade-cli`](crates/unlade-cli) | Command-line interface |

## Development

```console
$ cargo test --workspace --all-features --all-targets --locked
$ cargo clippy --workspace --all-features --all-targets --locked -- -D warnings
$ cargo fmt --all --check
```

## Minimum supported Rust version

Rust 1.85; CI also checks the pinned development toolchain and latest stable.

## License

[Apache-2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT), at your option.
