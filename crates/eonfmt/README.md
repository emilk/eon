# Formatter for Eon config files
[![Latest version](https://img.shields.io/crates/v/eonfmt.svg)](https://crates.io/crates/eonfmt)
[![unsafe forbidden](https://img.shields.io/badge/unsafe-forbidden-success.svg)](https://github.com/rust-secure-code/safety-dance/)
![MIT](https://img.shields.io/badge/license-MIT-blue.svg)
![Apache](https://img.shields.io/badge/license-Apache-blue.svg)

See <https://github.com/emilk/eon> for info about Eon.

## Installation
```
cargo install --locked eonfmt
```

## Usage
You can format individual files, or a whole folder recursively.
When given a folder, only `.eon` files will be formatted,
and `.gitignore` will be respected.

```
eonfmt file.rs
eonfmt folder/
eonfmt .
```

You can also check whether or not files are formatted:

```
# Error if there is an unformatted .eon file that is not in `.gitignore`.
eonfmt --check .
```
