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

- [ ] Differential tests for legacy vs core on supported overlapping syntax
- [ ] Roundtrip tests for root value, implicit map, and implicit list documents
- [ ] Roundtrip tests for variants, lists, maps, strings, and comments where applicable
- [ ] Document and test map-key ambiguity boundaries
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
- [-] Add formatter idempotency tests
- [x] Add browser/wasm build validation for the minimal formatter

Decision to make:

- Can formatting be built from a core token/trivia layer without carrying the full current syntax tree?

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

Status: `[ ]`
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
- [ ] Measure `wasm32-unknown-unknown` output size
- [ ] Track release binary size for the minimal formatter path
- [ ] Define acceptable regression thresholds

Suggested initial budgets:

- no new external dependencies in `eon_core`
- no parse throughput regressions on current core benchmarks
- no unexplained wasm size jumps on the minimal formatter path

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

Status: `[ ]`
Goal: make the minimal path continuously validated.

Deliverables:

- CI matrix for no-std, wasm, tests, and benchmarks
- release checklist for promoting core-backed paths

Tasks:

- [ ] Add `cargo check -p eon_core --no-default-features`
- [ ] Add `cargo check -p eon_core --target wasm32-unknown-unknown`
- [ ] Add minimal formatter wasm check once that crate exists
- [ ] Run core and adapter tests in CI
- [ ] Add a fuzz smoke lane
- [ ] Add benchmark reporting or stored comparison data

Exit criteria:

- every merge validates the minimal path
- release readiness is objective rather than manual guesswork

## Immediate Next Steps

These are the next concrete tasks after the current formatter-core landing:

1. [ ] Add differential tests against `eon_syntax::reformat` for supported syntax
2. [ ] Add explicit formatter idempotency tests on real fixtures
3. [ ] Add CLI-focused tests for `eonfmt` stdin, `--check`, and directory walking
4. [ ] Measure wasm artifact size for `eon_formatter_core`
5. [ ] Decide which remaining legacy formatting behaviors are compatibility-only

## Current Implementation Batch

This is the batch I am implementing next.

- [x] `codex1` lane: bootstrap formatter-core parsing/reformatting and switch `eonfmt` to the minimal reusable path
- [x] `codex2` lane: add formatter-facing token views and trivia classification on top of the flat item stream
- [x] `codex2` lane: add root document analysis for implicit map/list/value shapes on top of the formatter-core stream
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
