# Eon Core / Minimal Dependency Production Plan

Last updated: 2026-03-30
Branch: `add-lsp-and-zed-extension`
Baseline commit: `ea2f5c1`

## Goal

Ship a production-grade `eon_core` that is:

- zero-dependency
- `no_std`
- zero-copy where possible
- deterministic and well-specified
- suitable as the foundation for embedded use, wasm/browser use, and high-performance server use

In parallel, build a minimal formatting path so `eonfmt` can be split into:

- a small reusable formatting library with minimal dependencies
- a thin CLI wrapper for filesystem walking and argument parsing

## Current Scope

In scope for this plan:

- `crates/eon_core`
- the core-backed parse/write paths in `crates/eon`
- the minimal formatting path that can replace the current `eonfmt` dependency chain
- performance, binary size, fuzzing, and hardening needed for production quality

Out of scope for this phase:

- `tree-sitter-eon/`
- `zed-extension/`
- editor integration work not required for the core/minimal formatter path

## Success Criteria

This effort is done when all of the following are true:

- `eon_core` remains zero-dependency
- `eon_core` supports `no_std`
- `eon_core` uses `alloc` only where strictly necessary
- parser and writer semantics are documented and covered by regression tests
- supported documents roundtrip through the core parser/writer without semantic drift
- the minimal formatter builds for `wasm32-unknown-unknown`
- `eonfmt` depends on the minimal formatter library, not on the rich syntax stack
- fuzzing and regression tests cover known parser, formatter, and serialization edge cases
- performance and size budgets are tracked in CI

## Status Legend

- `[ ]` not started
- `[-]` in progress
- `[x]` complete
- `[!]` blocked

## Dependency Policy

### `eon_core`

- Allowed: `core`
- Allowed when necessary: `alloc`
- Disallowed: external crates

### Minimal formatter library

- Allowed: `core`
- Allowed when necessary: `alloc`
- Disallowed on the reusable path: `clap`, `ignore`, `logos`, `ariadne`

### Thin wrappers

- `eonfmt` may keep CLI-only dependencies
- richer syntax/debugging crates may keep editor/diagnostic dependencies
- those dependencies must not leak into the minimal reusable path

## Workstreams

## WS1 - Semantics and Contract

Status: `[-]`
Goal: freeze what `eon_core` guarantees before optimizing further.

Deliverables:

- written parser/writer contract
- explicit list of supported root document kinds
- explicit list of known syntax/value-model boundaries
- compatibility policy between legacy and core paths

Tasks:

- [ ] Write a short spec for `eon_core` parse semantics
- [ ] Write a short spec for `eon_core` write semantics
- [ ] Define exact roundtrip guarantees
- [ ] Define which legacy behaviors are compatibility-only and not core guarantees
- [ ] Document unsupported or intentionally ambiguous cases

Exit criteria:

- every fuzz/property invariant matches a documented contract
- unsupported cases are identified instead of silently drifting

## WS2 - Zero-Copy Parsing

Status: `[-]`
Goal: keep the hot parse path borrowed from source as long as possible.

Deliverables:

- borrowed token/scalar representation
- deferred string unescape model
- owned conversion only at adapter boundaries

Tasks:

- [ ] Audit every allocation in `eon_core`
- [ ] Keep identifiers, numbers, comments, and unescaped strings borrowed
- [ ] Represent escaped strings as raw slice plus "needs decode" metadata
- [ ] Avoid building owned `Value` trees on the critical path unless requested
- [ ] Add tests that prove borrowed paths work across representative inputs

Exit criteria:

- the parser can process common documents without allocating for simple scalars
- borrowed and owned adapter paths are clearly separated

## WS3 - Parser / Writer Parity

Status: `[-]`
Goal: make supported core syntax stable and internally self-consistent.

Deliverables:

- parity tests for root implicit map/list/value forms
- container depth rules that are consistent across root and nested cases
- write -> parse -> write stability for supported syntax

Tasks:

- [x] Differential tests for legacy vs core on supported overlapping syntax
- [x] Roundtrip tests for root value, implicit map, and implicit list documents
- [-] Roundtrip tests for variants, lists, maps, strings, and comments where applicable
- [-] Document and test map-key ambiguity boundaries
- [ ] Keep writer behavior deterministic for all supported cases

Exit criteria:

- no known semantic mismatches remain on the supported syntax subset
- roundtrip and parity regressions are covered by tests

## WS4 - Minimal Formatting Path

Status: `[-]`
Goal: replace the current formatter dependency chain on the reusable path.

Deliverables:

- a minimal formatter library, likely separate from `eon_syntax`
- formatting engine that does not depend on `logos` or `ariadne`
- wasm-safe reusable API

Tasks:

- [x] Decide the formatter input model
- [x] Introduce a lightweight trivia-preserving token stream if needed
- [x] Port core formatting logic out of `eon_syntax`
- [-] Preserve or intentionally redefine formatting behavior case by case
- [x] Add formatter idempotency tests
- [x] Add browser/wasm build validation for the minimal formatter

Decision to make:

- Can formatting be built from a core token/trivia layer without carrying the full current syntax tree?

Current formatter compatibility contract:

- Required compatibility on the tested overlapping syntax subset:
  - `eon_formatter_core::reformat` should match `eon_syntax::reformat` output for root maps, root scalars, root lists, quoted variants, and comment-placement cases covered by differential tests.
  - Root maps are canonicalized without outer braces by default, matching the legacy formatter.
  - Single root values with trailing line comments remain single values rather than being rewritten as implicit lists.
  - Empty root maps remain explicit because brace-less output cannot represent them.
  - Non-empty root maps may remain brace-less even when the first key is composite, such as a list, map, or quoted string; callers that want explicit outer braces can opt in with `always_include_outer_braces`.
  - Plain whitespace and newlines between a map key and its `:` are accepted and canonicalized away, but comments in that position are rejected.
  - Comments between `:` and the value are accepted, then normalized as prefix comments on the whole key-value entry.
  - Prefix comments stay on preceding lines and suffix comments stay inline with the preceding value.
  - Simple single-line lists and variants use comma-separated formatting, while multiline containers omit commas in canonical output.
- Intentional extensions beyond the current legacy formatter/parser contract:
  - Bare identifier variant heads in value position are accepted and formatted as variants, e.g. `EnumValue` and `EnumValue(...)`.
  - Quoted variant heads remain supported for compatibility.
  - Variant payload parentheses must remain inline with the variant head; comments or newlines between the head and `(` are rejected.
- Explicitly not guaranteed today:
  - Byte-for-byte preservation of original commas, braces, or whitespace/trivia.
  - Matching legacy error messages.
  - Full legacy parity outside the documented and tested overlapping syntax subset.

Exit criteria:

- the reusable formatter library has no CLI-only dependencies
- the formatter builds for wasm
- `eonfmt` can be reduced to a thin wrapper

## WS5 - `eonfmt` Split

Status: `[-]`
Goal: keep the CLI small and isolate dependencies.

Deliverables:

- reusable formatting library
- thin CLI wrapper crate
- explicit dependency separation

Tasks:

- [x] Move formatting logic out of the current `eonfmt` binary path
- [x] Keep `clap` and `ignore` in CLI-only code
- [x] Expose a small formatting API for library and wasm use
- [x] Add tests for CLI behavior separately from formatter behavior

Exit criteria:

- the formatter library can be consumed without pulling CLI dependencies
- `eonfmt` becomes a transport layer, not the formatting engine

## WS6 - Serde / Value Adapters

Status: `[-]`
Goal: keep compatibility without forcing the core to depend on serde or owned trees.

Deliverables:

- clearly layered adapters
- typed and owned paths on top of `eon_core`
- documented behavior differences where they remain

Tasks:

- [ ] Keep serde support out of `eon_core`
- [ ] Keep owned `Value` construction optional and layered
- [ ] Reduce duplication between legacy and core-backed adapters where safe
- [ ] Add tests for typed parse/write parity across supported syntax

Exit criteria:

- `eon_core` remains independent
- adapter behavior is predictable and tested

## WS7 - Performance and Size

Status: `[-]`
Goal: make performance and binary size an enforced part of the release bar.

Deliverables:

- benchmarks
- allocation profiling
- wasm and release-size measurements
- explicit budgets

Tasks:

- [ ] Keep and extend `bench_core_vs_serde`
- [ ] Add representative small, medium, and large fixtures
- [ ] Measure allocation counts on hot paths
- [x] Measure `wasm32-unknown-unknown` output size
- [ ] Track release binary size for the minimal formatter path
- [-] Define acceptable regression thresholds

Suggested initial budgets:

- no new external dependencies in `eon_core`
- no parse throughput regressions on current core benchmarks
- no unexplained wasm size jumps on the minimal formatter path
- `eon_formatter_core` wasm size budget: raw `<= 40000` bytes, gzip `<= 18000` bytes

Exit criteria:

- performance and size budgets are visible in CI
- regressions are caught before merge

## WS8 - Hardening and Fuzzing

Status: `[-]`
Goal: keep the parser and formatter robust under hostile or malformed input.

Deliverables:

- stable fuzz harnesses
- regression tests for all found issues
- clear policy for suspicious Unicode and duplicate-key behavior

Tasks:

- [ ] Keep byte-level parser fuzzing in place
- [ ] Keep hidden-Unicode rejection fuzzing in place
- [ ] Keep typed-path fuzzing in place
- [ ] Stabilize value-roundtrip fuzzing around documented guarantees
- [ ] Add every real fuzz-found bug as a normal regression test
- [ ] Add longer-running fuzz jobs outside the fast CI lane

Known bugs already found and fixed by this effort:

- numeric infinity treated as integer-like in hashing/accessors
- legacy `\0` formatting/parsing mismatch
- hidden Unicode emitted by legacy formatter
- multiline variant formatter dropping payloads
- cached `Map` hash corruption on replacement

Exit criteria:

- fuzz harnesses are reliable
- security-sensitive cases have permanent regression tests

## WS9 - CI and Release Gates

Status: `[x]`
Goal: make the minimal path continuously validated.

Deliverables:

- CI matrix for no-std, wasm, tests, and benchmarks
- release checklist for promoting core-backed paths

Tasks:

- [x] Add `cargo check -p eon_core --no-default-features`
- [x] Add `cargo check -p eon_core --target wasm32-unknown-unknown`
- [x] Add minimal formatter wasm check once that crate exists
- [x] Run core and adapter tests in CI
- [x] Add a fuzz smoke lane
- [x] Add benchmark reporting or stored comparison data

Exit criteria:

- every merge validates the minimal path
- release readiness is objective rather than manual guesswork

## Immediate Next Steps

These are the next concrete tasks after the current formatter-core landing:

1. [x] Add differential tests against `eon_syntax::reformat` for supported syntax
2. [x] Add explicit formatter idempotency tests on real fixtures
3. [x] Add CLI-focused tests for `eonfmt` stdin, `--check`, and directory walking
4. [x] Measure wasm artifact size for `eon_formatter_core`
5. [x] Decide which remaining legacy formatting behaviors are compatibility-only

## Current Implementation Batch

This is the batch I am implementing next.

- [x] `codex1` lane: bootstrap formatter-core parsing/reformatting and switch `eonfmt` to the minimal reusable path
- [x] `codex2` lane: add formatter-facing token views and trivia classification on top of the flat item stream
- [x] `codex2` lane: add root document analysis for implicit map/list/value shapes on top of the formatter-core stream
- [x] `codex2` lane: add a reproducible wasm artifact size measurement path for `eon_formatter_core`
- [x] `codex2` lane: add an explicit wasm size budget and CI gate for `eon_formatter_core`
- [x] `codex2` lane: add `eon_core` minimal-path CI checks for `--no-default-features` and `wasm32-unknown-unknown`
- [x] `codex2` lane: broaden CI from library-only tests to workspace core/adapter coverage
- [x] `codex2` lane: add a stable fuzz smoke lane that replays corpora and runs deterministic one-step fuzz harness checks in CI
- [x] `codex2` lane: add a reproducible benchmark runner and stored comparison baseline for the core-backed path
- [x] `formatter-core`: create a zero-dependency crate for formatting-oriented lexing and token/trivia preservation
- [x] `formatter-core` input model: borrowed tokens, punctuation, strings, comments, and line-breaking trivia sufficient for reformatting
- [x] initial test coverage: lexer tests and formatter-input model tests for comments, strings, maps, lists, and variants
- [x] `eonfmt` integration: switch the CLI to the new formatter core after the first formatting slice is working
- [x] wasm validation: add `wasm32-unknown-unknown` checks once the formatter core API exists

## Parallelization Notes

These lanes can proceed mostly independently:

- Lane A: `eon_core` parser/writer semantics and zero-copy work
- Lane B: formatter-core extraction and `eonfmt` split
- Lane C: performance, size, and CI automation
- Lane D: fuzzing and hardening

Shared coordination points:

- syntax/format guarantees must be agreed before heavy formatter work
- adapter expectations must follow the core contract, not vice versa
- fuzz invariants must be kept aligned with documented guarantees

## Progress Log

Use this section to append short progress notes while multiple people work in parallel.

Template:

`YYYY-MM-DD | owner | workstream | change | blockers/notes`

Entries:

- `2026-03-30 | codex1 | WS3/WS8 | Pulled latest branch fixes for root implicit list parity and writer hardening | HEAD now ea2f5c1`
- `2026-03-30 | codex1 | WS8 | Added fuzz harnesses and fixed multiple parser/formatter/hash bugs found by fuzzing | value-roundtrip harness still intentionally scoped to documented guarantees`
- `2026-03-30 | codex1 | WS4/WS5 | Marked the first implementation batch around formatter-core extraction and minimal `eonfmt` path | next code work starts with borrowed formatter input model`
- `2026-03-30 | codex1 | WS4/WS5 | Pulled latest plan update and claimed the formatter-core bootstrap lane | immediate slice: zero-dependency crate + borrowed token/trivia lexer + tests`
- `2026-03-30 | codex2 | WS4/WS5 | Re-claimed the formatter-core bootstrap lane under explicit owner name | no evidence of a separate codex1 on branch; immediate scope stays lexer/model bootstrap`
- `2026-03-30 | codex2 | WS4/WS5 | Landed the formatter-core bootstrap crate with a borrowed token/trivia stream, manual lexer, tests, and a wasm check | next slice can build formatter behavior on top of the flat Item stream`
- `2026-03-30 | codex2 | WS4/WS5 | Started the token-view/trivia-classification slice on top of the flat formatter-core stream | target API: leading-gap classification, suffix-comment detection, token iteration`
- `2026-03-30 | codex2 | WS4/WS5 | Landed token views, trivia iterators, leading-gap classification, and suffix-comment detection on top of formatter-core | next slice should build the first formatter walker on top of TokenRef APIs`
- `2026-03-30 | codex2 | WS4/WS5 | Landed root document analysis for implicit map/list/value forms using token spans on top of formatter-core | next slice can build the first root formatter walker against Document/ValueSpan APIs`
- `2026-03-30 | codex1 | WS4/WS5 | Added formatter-core parsing/reformatting, switched `eonfmt` to depend on `eon_formatter_core`, and verified wasm buildability | follow-up is legacy parity, idempotency, and CLI coverage`
- `2026-03-30 | codex1 | WS5 | Added `eonfmt` CLI tests for stdin, --check, and directory walking | protects the new formatter boundary and the directory traversal fix`
- `2026-03-30 | codex1 | WS3/WS4 | Added differential reformat parity tests against \`eon_syntax\`, explicit idempotency tests, and canonical root-map formatting parity on the overlapping syntax subset | remaining work is broader legacy behavior review and size/perf tracking`
- `2026-03-30 | codex2 | WS7/WS9 | Synced to upstream formatter parity landing at 355efc9 and claimed the wasm-size baseline lane for the minimal formatter path | next slice is a reproducible artifact-size measurement for eon_formatter_core`
- `2026-03-30 | codex2 | WS7/WS9 | Added a wasm-size measurement script and example harness for eon_formatter_core; current baseline is 39618 raw bytes / 17430 gzip bytes | next slice should turn that baseline into an explicit budget or CI gate`
- `2026-03-30 | codex2 | WS7/WS9 | Added a committed wasm budget file, a --check mode for the formatter-core size script, and a rust.yml CI gate enforcing raw <= 40000 / gzip <= 18000 bytes | next non-overlapping CI slice can add eon_core no-default-features and wasm checks`
- `2026-03-30 | codex1 | WS4 | Documented the formatter compatibility contract and locked intentional extensions/non-goals with targeted tests | broader legacy behavior review still remains open where not yet covered by parity tests`
- `2026-03-30 | codex2 | WS9 | Added rust.yml CI coverage for eon_core on --no-default-features and wasm32-unknown-unknown after verifying both commands locally | next CI gap is broader core/adapter test coverage and fuzz smoke`
- `2026-03-30 | codex2 | WS9 | Switched rust.yml test coverage from cargo test --lib to cargo test --workspace so formatter-core, adapter, and CLI integration tests run in CI | next CI gap is a fuzz smoke lane or benchmark reporting`
- `2026-03-30 | codex2 | WS8/WS9 | Added scripts/run_fuzz_smoke.sh and a rust.yml fuzz smoke job that builds all fuzz targets, replays seeded corpora, and runs deterministic one-step checks for corpus-less harnesses | next CI/release-gap is benchmark reporting or stored comparison data`
- `2026-03-30 | codex1 | WS3/WS4 | Added root/container roundtrip coverage and fixed formatter-core root trailing-comment handling | single root values no longer get wrapped as implicit lists, explicit empty root maps stay explicit, and root-map trailing comments are now canonical and idempotent`
- `2026-03-30 | codex1 | WS3/WS4 | Added broader formatter-core roundtrip coverage for explicit/implicit root lists and fixed the single-map/list variant shortcut to preserve payload comments by falling back to the generic multiline path when payload values carry trivia | deterministic roundtrip coverage is broader, but nested strings/map-key boundaries still need more work`
- `2026-03-30 | codex1 | WS3 | Added roundtrip coverage for composite root map keys and escaped quoted-string keys | map-key boundary testing is started, but the remaining work is documenting which ambiguous forms are guaranteed versus merely accepted today`
- `2026-03-31 | codex1 | WS1/WS3/WS4 | Documented that non-empty root maps may remain brace-less even with composite first keys and added contract/roundtrip coverage for composite-key root maps with nested strings/comments | next boundary work is deciding which remaining ambiguous map-key forms are guaranteed versus merely tolerated`
- `2026-03-31 | codex1 | WS1/WS3/WS4 | Documented the key/colon boundary: whitespace and newlines before ':' are canonicalized, comments there are rejected, and comments after ':' normalize to entry-prefix comments | this narrows the remaining ambiguity work to other still-undocumented map-key forms rather than generic separator trivia`
- `2026-03-31 | codex1 | WS3 | Added formatter-core roundtrip coverage for literal keys plus multiline basic/literal string tokens in both key and value position | string-family coverage is broader, but full writer determinism across all nested shapes is still not declared complete`
- `2026-03-31 | codex2 | WS7/WS9 | Added scripts/run_benchmark_baseline.sh and benchmark-data/README.md with a reproducible local baseline for bench_parse and bench_core_vs_serde at 477118a | next performance slice is release-size tracking or stronger benchmark automation`
