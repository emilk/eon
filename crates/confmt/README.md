# Formatter for Con config files

See <https://github.com/emilk/con> for info about Con.

## Installation
```
cargo install confmt
```

## Usage
You can format indivudal files, or a whole folder recursively.
When given a folder, only `.con` files will be formatted,
and `.gitignore` will be respected.

```
confmt file.rs
confmt folder/
confmt .
```

You can also check wether or not files are formatted:

```
# Error if there is an unformatted .con file that is not in `.gitignore`.
confmt --check .
```
