# Eon for Zed: Implementation Plan

Last updated: 2026-02-14

## 1. Goal

Build a fully fledged Zed IDE extension for Eon that includes:

- Language detection for `.eon` files
- Syntax highlighting and editor behavior (brackets, indentation, outline, text objects)
- Formatting support (using Eon formatter logic)
- LSP diagnostics and completion
- Cross-platform install experience for end users
- CI + release flow + publish-ready metadata

## 1.1 Current Implementation Status (2026-02-14)

Completed in this repo:

- Added `crates/eon_lsp` with:
  - diagnostics (`didOpen`/`didChange`/`didSave`)
  - formatting (`textDocument/formatting`)
  - keyword + map-key completion (including nested maps/lists/variants)
  - document symbols for map keys
  - unit tests for UTF-16 mapping/diagnostics/symbol extraction/completion/format idempotency
- Added `zed-extension/` scaffold with:
  - `extension.toml`
  - language metadata and query files
  - Rust extension entrypoint wired to `eon-lsp`
  - language server binary resolution: settings path -> PATH -> GitHub release download
  - unit tests for asset naming/version normalization/arch filtering
- Added `tree-sitter-eon/` scaffold with:
  - starter grammar with top-level map/single-value/stream parsing
  - special number handling (`+inf`, `-inf`, `+nan`) and multiline string coverage
  - happy-path and malformed-input corpus tests
  - npm scripts and generated parser artifacts
- Added CI workflow for grammar tests + wasm extension check:
  - `.github/workflows/eon_zed_extension.yml`
- Added release workflow scaffold for cross-platform `eon_lsp` artifacts:
  - `.github/workflows/eon_lsp_release.yml`

## 2. Reference Docs

- Zed: Developing Extensions  
  https://zed.dev/docs/extensions/developing-extensions
- Zed: Language Extensions  
  https://zed.dev/docs/extensions/languages
- Zed extension API README  
  https://github.com/zed-industries/zed/blob/main/crates/extension_api/README.md
- Example extension repos:
  - https://github.com/zed-extensions/toml
  - https://github.com/zed-extensions/terraform
  - https://github.com/narqo/zed-jsonnet

## 3. Proposed Architecture

Use 3 components:

1. `tree-sitter-eon` (new repo)
- Owns Tree-sitter grammar + corpus tests

2. `eon` (this repo)
- Add `crates/eon_lsp` (language server)
- Reuse `eon_syntax` and formatting logic

3. `zed-eon` extension (new repo or `zed-extension/` subdirectory)
- Zed manifest (`extension.toml`)
- Language metadata + query files (`languages/eon/*.scm`)
- Rust WASM entrypoint (`src/lib.rs`)
- Starts `eon-lsp` and optionally downloads binary

## 4. Definition of Done

The extension is done when all of this is true:

- Installing as a dev extension in Zed works
- Opening `.eon` auto-selects `Eon`
- Highlighting, brackets, and indentation work for real-world files
- `Format` and format-on-save produce stable output
- Invalid Eon shows diagnostics with useful locations/messages
- Basic completions appear (`null`, `true`, `false`, `+nan`, `+inf`, `-inf`)
- CI passes for grammar, LSP, and extension build
- Extension metadata is publish-ready and license-compliant

## 5. Execution Plan (Phases)

## Phase 0: Setup and Repo Layout

Status: `in_progress`  
Owner: You  
Estimate: 0.5-1 day

### Tickets

#### EON-ZED-0001 - Create tracking board and labels
Tasks:
- Create labels: `grammar`, `queries`, `lsp`, `extension`, `release`, `docs`, `bug`
- Create project board columns: `Backlog`, `Ready`, `In Progress`, `Review`, `Done`

Acceptance criteria:
- Labels exist
- Board exists with all phase tickets added

#### EON-ZED-0002 - Confirm component locations
Tasks:
- Decide if extension is separate repo or `zed-extension/` subdirectory
- Create placeholder README for each component

Acceptance criteria:
- Chosen structure documented
- Team can tell where each feature belongs

---

## Phase 1: Tree-sitter Grammar for Eon

Status: `in_progress`  
Owner: You  
Estimate: 4-7 days  
Depends on: Phase 0

### Tickets

#### EON-ZED-1001 - Scaffold `tree-sitter-eon`
Tasks:
- Initialize grammar project
- Add CI for `tree-sitter test`
- Add README with local test command

Acceptance criteria:
- `tree-sitter test` runs locally
- CI runs grammar tests on pull requests

#### EON-ZED-1002 - Implement core grammar rules
Tasks:
- Add grammar rules for:
  - comments (`//`)
  - identifiers
  - numbers (including `+nan`, `+inf`, `-inf`)
  - strings: basic, multiline basic, literal, multiline literal
  - list/map/variant structures

Acceptance criteria:
- Parser builds
- Example docs parse without syntax errors

#### EON-ZED-1003 - Add corpus tests
Tasks:
- Add happy-path corpus files from `example.eon` and README examples
- Add negative/error corpus cases:
  - malformed strings
  - malformed numbers
  - unbalanced braces/brackets/parens

Acceptance criteria:
- Corpus includes both valid and invalid coverage
- Tests are deterministic and green

#### EON-ZED-1004 - Pin stable grammar commit
Tasks:
- Create first stable tag or commit reference
- Document compatibility expectations

Acceptance criteria:
- Extension can reference an immutable grammar revision

---

## Phase 2: Extension Skeleton in Zed

Status: `in_progress`  
Owner: You  
Estimate: 1-2 days  
Depends on: Phase 1

### Tickets

#### EON-ZED-2001 - Create extension manifest and crate
Tasks:
- Add `extension.toml` with required fields:
  - `id`, `name`, `version`, `schema_version`, `authors`, `description`, `repository`
- Add `Cargo.toml` with `cdylib` and `zed_extension_api`
- Add `src/lib.rs` with `register_extension!`

Acceptance criteria:
- Extension compiles to WASM
- Zed can install it as a dev extension

#### EON-ZED-2002 - Add language metadata
Tasks:
- Add `languages/eon/config.toml`
- Configure:
  - `name = "Eon"`
  - `grammar = "eon"`
  - `path_suffixes = ["eon"]`
  - comment style

Acceptance criteria:
- `.eon` files resolve to Eon language mode in Zed

#### EON-ZED-2003 - Register grammar in `extension.toml`
Tasks:
- Add `[grammars.eon]` with grammar repository and pinned revision

Acceptance criteria:
- Parsing/highlighting pipeline initializes in Zed with no load errors

---

## Phase 3: Tree-sitter Query Coverage

Status: `in_progress`  
Owner: You  
Estimate: 3-5 days  
Depends on: Phase 2

### Tickets

#### EON-ZED-3001 - Add `highlights.scm`
Tasks:
- Capture key tokens:
  - comments, strings, numbers, booleans, null-like identifiers, punctuation, keys
- Validate in multiple Zed themes

Acceptance criteria:
- Highlighting is readable and consistent for sample files

#### EON-ZED-3002 - Add `brackets.scm` and `indents.scm`
Tasks:
- Define bracket pairs for `{}`, `[]`, `()`, and optional quote handling
- Define indentation regions for map/list/variant constructs

Acceptance criteria:
- Auto-indent behaves as expected for nested structures
- Bracket matching and rainbow brackets work correctly

#### EON-ZED-3003 - Add `outline.scm` and `textobjects.scm`
Tasks:
- Outline:
  - map key/value entries as items
  - include contextual captures where useful
- Text objects:
  - function/class analogs are limited in config languages, focus on practical chunks

Acceptance criteria:
- Outline panel shows meaningful entries
- Vim text objects do not break and are minimally useful

#### EON-ZED-3004 - Add optional query files
Tasks:
- `overrides.scm` for scope-specific behavior
- `redactions.scm` for sensitive values
- `injections.scm` only if needed

Acceptance criteria:
- No parser/query errors
- Optional behavior is documented

---

## Phase 4: Build `eon-lsp` in This Workspace

Status: `in_progress`  
Owner: You  
Estimate: 4-8 days  
Depends on: Phase 0

### Tickets

#### EON-ZED-4001 - Add `crates/eon_lsp`
Tasks:
- Create new crate in workspace
- Choose LSP implementation library (recommended: `tower-lsp`)
- Add entrypoint with stdio transport

Acceptance criteria:
- `cargo run -p eon_lsp` starts and waits for LSP messages

#### EON-ZED-4002 - Implement diagnostics
Tasks:
- Parse incoming document text via `eon_syntax`
- Convert `Span` byte offsets to LSP line/character positions
- Return `PublishDiagnostics` on open/change/save

Acceptance criteria:
- Invalid files show diagnostics in editor
- Diagnostic ranges point to meaningful locations

#### EON-ZED-4003 - Implement formatting
Tasks:
- Implement `textDocument/formatting`
- Reuse `eon_syntax::reformat` and return full-document text edit
- Keep behavior aligned with `eonfmt`

Acceptance criteria:
- Format command rewrites doc correctly
- Re-format on already formatted doc is no-op

#### EON-ZED-4004 - Implement baseline completion
Tasks:
- Return keyword completions:
  - `null`, `true`, `false`, `+nan`, `+inf`, `-inf`
- Optionally include context-aware key suggestions from nearest map

Acceptance criteria:
- Completion list appears in common edit positions
- No crashes on malformed documents

#### EON-ZED-4005 - Implement document symbols
Tasks:
- Extract top-level map keys (and nested where useful)
- Return symbol hierarchy for outline and navigation

Acceptance criteria:
- Symbol view is stable and useful on real config files

#### EON-ZED-4006 - Add tests
Tasks:
- Unit tests for span mapping
- Golden tests for diagnostics
- Formatting roundtrip and idempotency tests

Acceptance criteria:
- Tests cover core protocol behavior and edge cases

---

## Phase 5: Integrate LSP with Zed Extension

Status: `in_progress`  
Owner: You  
Estimate: 2-3 days  
Depends on: Phase 3, Phase 4

### Tickets

#### EON-ZED-5001 - Add language server manifest entries
Tasks:
- In `extension.toml`, add:
  - `[language_servers.eon-lsp]`
  - `name`
  - `languages = ["Eon"]`

Acceptance criteria:
- Zed discovers the language server for Eon files

#### EON-ZED-5002 - Implement `language_server_command`
Tasks:
- In extension Rust code, implement command resolution order:
  1. User-configured binary path (if supplied by settings)
  2. `worktree.which("eon-lsp")`
  3. Download managed binary release fallback
- Surface install statuses (`CheckingForUpdate`, `Downloading`)

Acceptance criteria:
- Server launches on all supported platforms
- Fallback logic works when local binary is missing

#### EON-ZED-5003 - Wire LSP settings pass-through
Tasks:
- Implement workspace configuration forwarding if needed
- Document settings keys for users

Acceptance criteria:
- User settings are honored and testable

---

## Phase 6: Packaging, CI, and Release

Status: `in_progress`  
Owner: You  
Estimate: 2-4 days  
Depends on: Phase 5

### Tickets

#### EON-ZED-6001 - CI for all components
Tasks:
- Grammar CI: `tree-sitter test`
- Rust CI: `cargo test --workspace`
- Extension CI: wasm target build + manifest checks

Acceptance criteria:
- Green CI is required for merge

#### EON-ZED-6002 - Multi-platform binary releases for `eon-lsp`
Tasks:
- Build and upload assets for:
  - macOS (x86_64, aarch64)
  - Linux (x86_64, aarch64 if feasible)
  - Windows (x86_64)
- Use predictable asset naming for extension downloader

Acceptance criteria:
- Extension download logic finds matching assets for each platform

#### EON-ZED-6003 - Versioning and changelog
Tasks:
- Define semver policy for grammar/LSP/extension
- Add changelog generation/update workflow

Acceptance criteria:
- Every release has clear user-facing change notes

---

## Phase 7: Publish to Zed Registry

Status: `todo`  
Owner: You  
Estimate: 1 day + PR review  
Depends on: Phase 6

### Tickets

#### EON-ZED-7001 - Prepare publish PR
Tasks:
- Ensure extension repo has accepted license
- Open PR to `zed-industries/extensions`:
  - add extension as submodule
  - update `extensions.toml` with version
  - run required sorting checks

Acceptance criteria:
- PR merges
- Extension appears in registry

Important date note:
- Zed requires accepted extension licenses since 2025-10-01

---

## Phase 8: Hardening and Follow-ups

Status: `todo`  
Owner: You  
Estimate: ongoing

### Tickets

#### EON-ZED-8001 - Improve diagnostics quality
Tasks:
- Better fix hints for common mistakes:
  - `nan` -> `+nan`
  - `inf` -> `+inf` or `-inf`
- Tune ranges/messages from parser errors

Acceptance criteria:
- Lower false-positive rate
- User-reported errors are actionable

#### EON-ZED-8002 - Add advanced editor features
Tasks:
- Explore code actions and additional completion contexts
- Consider semantic token integration when server matures

Acceptance criteria:
- Features are useful and do not regress baseline behavior

## 6. Suggested Backlog Order

If working solo, execute in this exact order:

1. `EON-ZED-0001`
2. `EON-ZED-0002`
3. `EON-ZED-1001`
4. `EON-ZED-1002`
5. `EON-ZED-1003`
6. `EON-ZED-1004`
7. `EON-ZED-2001`
8. `EON-ZED-2002`
9. `EON-ZED-2003`
10. `EON-ZED-3001`
11. `EON-ZED-3002`
12. `EON-ZED-3003`
13. `EON-ZED-3004`
14. `EON-ZED-4001`
15. `EON-ZED-4002`
16. `EON-ZED-4003`
17. `EON-ZED-4004`
18. `EON-ZED-4005`
19. `EON-ZED-4006`
20. `EON-ZED-5001`
21. `EON-ZED-5002`
22. `EON-ZED-5003`
23. `EON-ZED-6001`
24. `EON-ZED-6002`
25. `EON-ZED-6003`
26. `EON-ZED-7001`
27. `EON-ZED-8001`
28. `EON-ZED-8002`

## 7. Command Cheat Sheet (Junior-Friendly)

Use these while building:

```sh
# workspace tests
cargo test --workspace

# formatter checks
cargo run -p eonfmt -- --check .

# build eon_lsp once added
cargo build -p eon_lsp

# run eon_lsp manually (for local protocol testing)
cargo run -p eon_lsp

# add Zed wasm target (required by extension code)
rustup target add wasm32-wasip2
```

For extension local install in Zed:

1. Open command palette
2. Run `zed: Install Dev Extension`
3. Select extension directory
4. Open an `.eon` file and validate behavior

## 8. Risks and Mitigations

Risk: grammar ambiguity causes broken editor behavior  
Mitigation: add focused corpus tests before query tuning

Risk: byte-offset to LSP position conversion bugs  
Mitigation: explicit unit tests for multibyte UTF-8 and multiline spans

Risk: release asset naming mismatch breaks downloads  
Mitigation: assert asset name pattern in CI and in extension tests

Risk: extension API compatibility drift  
Mitigation: pin `zed_extension_api` and document tested Zed range

## 9. Week-by-Week Delivery Target

Week 1:
- Phase 0 + Phase 1 complete

Week 2:
- Phase 2 + Phase 3 complete

Week 3:
- Phase 4 complete

Week 4:
- Phase 5 + Phase 6 + Phase 7 (or ready for publish PR)

## 10. Optional Nice-to-Haves

- Add benchmark for very large `.eon` files in LSP parse loop
- Add telemetry hooks (local logs only) for diagnostics frequency during development
- Publish sample configs and "known patterns" pack in extension README
