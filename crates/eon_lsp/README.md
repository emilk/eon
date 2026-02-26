# eon_lsp

Language Server Protocol implementation for Eon.

## Current features

- Parse + semantic diagnostics
- Document formatting via `eon_syntax::reformat`
- Baseline keyword completion (`null`, `true`, `false`, `+nan`, `+inf`, `-inf`)
- Document symbols for map keys

## Run locally

```sh
cargo run -p eon_lsp
```
