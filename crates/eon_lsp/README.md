# eon_lsp

Language Server Protocol implementation for Eon.

## Current features

- Parse + semantic diagnostics
- Experimental composition diagnostics for `$alias.path` references and
  root-level `use` imports
- Document formatting via `eon_syntax::reformat`
- Baseline keyword completion (`null`, `true`, `false`, `+nan`, `+inf`, `-inf`)
- Schema-aware completion for fields and enum variants
- Document symbols for map keys

## Schema Discovery

`eon_lsp` loads an Eon schema artifact from the first available source:

- `initializationOptions.schemaPath`
- `initializationOptions.eonSchemaPath`
- `initializationOptions.eon.schemaPath`
- `.eon-schema.eon` in the workspace root

Example schema artifact:

```eon
kind: "object"
name: "Config"
fields: [
    { name: "port", type: "integer", docs: "Server port." }
]
```

## Run locally

```sh
cargo run -p eon_lsp
```
