# Eon vNext Plan

This document tracks the next major direction for Eon. The goal is to keep the
core format small and obvious while adding the tooling needed for serious
human-authored configuration.

## Goals

- Keep ordinary `.eon` files easy to read and write.
- Make one canonical syntax surface for parsers, formatters, docs, and LSPs.
- Add Rust-first schema/introspection so code-defined config structures can
  drive autocomplete, validation, generated examples, and migrations.
- Add a small optional composition layer for imports and immutable references.
- Keep tool-specific behavior out of the base data language whenever possible.

## Canonical Syntax

vNext should promote the core-backed syntax as the canonical syntax:

```eon
mode: Release
color: Rgb({ r: 255, g: 0, b: 0 })
mapping: { Release(): Debug }
label: "Release"
```

The legacy quoted-variant syntax should remain parse-compatible for a migration
window, but default formatters and serializers should emit the canonical form.
The language should avoid multiple equal-status spellings for the same semantic
shape.

Keyword map keys such as `null`, `true`, and `false` need a stricter rule. They
should either roundtrip consistently across all paths or be rejected/quoted in
positions where the current behavior is surprising.

## Schema And Tooling

The most important new capability is a first-class schema model.

Recommended shape:

- Add an optional Rust-first schema API, likely as a separate `eon_schema`
  crate plus an optional derive crate.
- Derive schemas from the same Rust structs/enums that are used for typed
  deserialization.
- Honor relevant `serde` attributes such as `rename`, `default`, `flatten`, and
  skipped fields.
- Preserve Rust doc comments as schema docs for hover text and generated
  commented examples.
- Support field docs, defaults, examples, enum variants, deprecations, numeric
  and string constraints, open/closed object policy, map key/value types, and
  schema-local extension metadata.
- Keep extension metadata namespaced so tools such as `vsr` can add custom
  behavior without changing the Eon syntax.

`eon_lsp` should consume this schema model to provide:

- Context-aware key completion.
- Enum and variant completion.
- Snippets for object and variant payloads.
- Hover documentation.
- Unknown-key diagnostics with did-you-mean suggestions.
- Missing required field diagnostics.
- Default insertion and generated starter config actions.
- Go-to-definition and validation support for imported references.

Schema discovery should be explicit. A good first design is a generated schema
artifact or a configured `schema_command`, rather than the editor compiling and
executing arbitrary Rust crates automatically.

## Composition Layer

Imports and variables should not become a general expression language. vNext
should use a small optional composition layer with immutable references.

Recommended starting syntax:

```eon
use: {
    common: "../common.eon"
    app: "../app/config.eon"
}

database: $common.database
port: $app.server.port
```

Selected imports can avoid destructuring syntax:

```eon
use: {
    app_key: {
        from: "../app/config.eon"
        select: ".app_config.key"
    }
}

active_key: $app_key
```

Initial resolver rules:

- Imports are local files, relative to the importing file.
- Network imports are not enabled by default.
- Environment access is not part of the base resolver.
- Import cycles are errors with a clear cycle trace.
- `$alias.path` references only declared imports or local aliases.
- References are immutable value references, not text substitution visible to
  users.
- Resolution happens before schema validation and typed deserialization.
- Diagnostics should preserve source chains across imported files.

Avoid for the first implementation:

- `import("../file.eon")` function syntax.
- Destructuring assignments such as `{ app_config: { key } } = use("file.eon")`.
- String interpolation.
- Arithmetic, conditionals, filters, wildcard paths, or globs.
- Implicit merge behavior.

## Implementation Phases

1. Done: Document the vNext direction and add an experimental `eon_compose`
   crate.
2. Done: Implement whole-file `use` imports and `$alias.path` references.
3. Done: Add selected imports with `{ from, select }`.
4. Done: Preserve source chains in composition errors.
5. Done: Add initial schema model and Rust derive support.
6. Done: Wire initial schema and composition plumbing into `eon_lsp`.
7. Done: Add schema discovery/configuration for `eon_lsp`.
8. Done: Add formatter migration support for canonical variant heads.
9. Next: Add schema validation diagnostics and hover support in `eon_lsp`.
