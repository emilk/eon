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

## Library usage

`eonfmt` can also be used as a small formatting library. To avoid pulling in
the CLI dependencies (`clap` and `ignore`), disable default features:

```toml
[dependencies]
eonfmt = { version = "*", default-features = false }
```

Then use the formatting API directly:

```rust
let formatted = eonfmt::format_str("key:true\n")?;
assert_eq!(formatted, "key: true\n");
```

## Usage
You can format individual files, or a whole folder recursively.
When given a folder, only `.eon` files will be formatted,
and `.gitignore` will be respected.

That recursive directory-walking behavior is why the CLI enables the `ignore`
crate. Library and wasm users do not need that dependency.

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
