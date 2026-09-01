## Context

PR #94 uses a hand-written scanner in `crates/editor-core/tests/architecture.rs` to enforce ADR 0003. The scanner recognizes `use crate::...` and qualified `crate::...` paths, but valid Rust syntax can express the same dependency through `super`, a top-level use group, or production items placed after a test module. The current `production_source` helper also discards the entire suffix after the first `#[cfg(test)] mod tests`.

The correction is confined to an integration test and its development dependency. The allowed matrix, production modules, public facade, persisted formats, and runtime behavior must remain unchanged.

## Goals / Non-Goals

**Goals:**

- Derive internal owner dependencies from parsed Rust syntax rather than token spellings.
- Resolve complete use trees and qualified paths rooted at `crate`, at a root-reaching sequence of `super`, or at an alias for the crate root.
- Exclude test-only items without hiding later production items.
- Preserve exact-owner matching and the existing owner/dependency/allowed-set diagnostic.

**Non-Goals:**

- Expanding macros or implementing compiler name resolution.
- Changing ADR 0003, the allowed matrix, production dependencies, runtime behavior, public APIs, or persisted contracts.
- Adding `syn` to production dependencies or introducing a reusable production parser.

## Decisions

### 1. Parse owner files with `syn`

Add `syn = { version = "2.0.119", features = ["full", "visit"] }` under editor-core dev-dependencies. Parse each owner with `syn::parse_file`; a parse failure fails the architecture test with the owner, file, and parser diagnostic.

An AST visitor will recursively process `ItemUse` trees and expression/type `Path` nodes. Use-tree groups are traversed at any level, so both `use crate::{persistence as storage}` and `use {crate::persistence as storage}` resolve to the same owner. Qualified paths continue to count without requiring an import.

Alternative considered: extend the textual scanner with more prefixes and delimiters. Rejected because two rounds of review have exposed independent legal Rust forms that its grammar omitted, and further string matching would remain open-ended.

### 2. Resolve paths relative to module depth

The visitor starts at depth one because each scanned file is a crate-root owner module and increments depth for inline modules. `crate::<owner>` always resolves from the root; a sequence of `super` resolves from the root only when it climbs the current module depth. Exact identifiers are compared with the centralized owner set, so similarly prefixed names remain ignored.

Aliases introduced for the crate root by use-tree renames are tracked in their lexical/module scope and resolved when they prefix later use trees or qualified paths. Ordinary aliases of an owner are already captured from the original use tree.

Alternative considered: forbid relative paths or crate-root aliases in owner modules. Rejected because the living requirement promises enforcement across supported import forms rather than imposing a new undocumented source convention.

### 3. Skip test-only AST subtrees instead of truncating source

The visitor evaluates `cfg` predicates with `test=false` sufficiently to identify items that cannot exist in a non-test build. It skips only those subtrees and continues with every sibling item before and after them. Predicates that may remain active in a production configuration are scanned conservatively.

Alternative considered: remove the textual range occupied by `mod tests`. Rejected because it would still be brittle around attributes, nested test modules, and other test-only item kinds.

### 4. Keep policy and diagnostics separate from extraction

AST extraction returns a deterministic `BTreeSet` of referenced owners. The existing matrix validator remains responsible for ignoring self references, checking the allowed set, and formatting violations. Focused data tests cover extraction independently from the real-source matrix integration test.

Alternative considered: assert policy directly while visiting. Rejected because it would couple syntax coverage to the matrix and make regression cases harder to isolate.

## Risks / Trade-offs

- [The visitor could approximate conditional compilation incorrectly] → Treat uncertain `cfg` expressions as production and skip only predicates proven false for `test=false`.
- [Relative paths depend on inline-module depth] → Track depth explicitly and test root and nested-module cases.
- [Crate-root aliases introduce lexical scope] → Track aliases per scope and add an alias regression rather than relying on global string replacement.
- [Macros can emit unseen imports] → Keep macro expansion outside scope; direct parsed imports and paths remain the enforced contract.
- [A new dev-dependency can affect the lockfile] → Pin the already-locked `syn 2.0.119` and verify that no production dependency graph changes.

## Migration Plan

Create and approve this OpenSpec change, add failing syntax regressions, replace the scanner, run the full repository gates, verify and archive the change, then publish one follow-up commit to PR #94. Rollback is the single test-only commit and requires no data or API migration.

## Open Questions

None.
