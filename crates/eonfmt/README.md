# Formatter for Eon config files

See <https://github.com/emilk/eon> for info about Eon.

## Installation
```
cargo install eonfmt
```

## Usage
You can format indivudal files, or a whole folder recursively.
When given a folder, only `.eon` files will be formatted,
and `.gitignore` will be respected.

```
eonfmt file.rs
eonfmt folder/
eonfmt .
```

You can also check wether or not files are formatted:

```
# Error if there is an unformatted .eon file that is not in `.gitignore`.
eonfmt --check .
```
