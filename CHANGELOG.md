# Changelog

## Unreleased

### Breaking

- `eon` no longer enables `serde` by default. Typed `Serialize` / `Deserialize` support now requires `features = ["serde"]`.
- The default `eon::Value` parse path is now core-backed, so bare identifiers in value position parse as unit variants instead of legacy “unknown keyword” errors.
- The default typed `eon::from_str` path is now core-backed when the optional `serde` feature is enabled.

### Added

- Added the experimental `eon_core` crate: a zero-dependency, `no_std`, borrowed event parser and compact event writer for Eon.
- Added the experimental `eon_formatter_core` crate: a zero-dependency, `no_std`, reusable formatting core used by `eonfmt`.
- Added `eon::experimental::{from_str_with_core,to_string_with_core,value_from_str_with_core,value_to_string_with_core}` for core-backed typed and dynamic parse/write paths.
- Added benchmark baselines, wasm size measurement, and CI checks for the minimal dependency path.
- Added large nested end-to-end fixture coverage for legacy parse, core parse, legacy formatting, formatter-core formatting, and compact core roundtrips.
- Added fuzz targets and broader regression coverage for parser, formatter, security, and roundtrip edge cases.

### Changed

- `eonfmt` now formats through `eon_formatter_core` instead of the older rich syntax stack.
- The experimental compact core syntax accepts bare identifier variants in value position, e.g. `EnumValue` and `EnumValue(...)`, while keeping quoted strings distinct.
- Unit variants are preserved in the owned `Value` model on the core-backed path instead of being collapsed to strings.
- Root-map and formatter behavior is now better specified and covered, including empty root maps, brace-less non-empty root maps, and composite root keys.
- Parser depth accounting was fixed across the legacy and formatter-core parsers so deeply nested but valid documents do not fail too early.

### Security

- Literal invisible Unicode format/control characters are now rejected in comments and quoted text; explicit `\u{...}` escapes remain allowed.
- Typed deserialization now rejects duplicate map keys across both the legacy and core-backed paths, including alias-normalized duplicates.
- Variant payload parsing is stricter about inline `(` attachment so hidden/comment-separated payload tricks are rejected.

### Performance

- Added direct typed core serialization and typed core deserialization so the experimental path no longer has to route through the legacy syntax stack for its hot path.
- Added benchmark coverage comparing the legacy serde path, legacy owned `Value` path, and the experimental core-backed parse/write paths.
