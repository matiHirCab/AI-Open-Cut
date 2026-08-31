## Why

Review of PR #94 found that the editor-core architecture test still misses valid Rust dependency forms such as crate-root-relative `super` imports, top-level grouped imports, and production items declared after a `#[cfg(test)]` module. The enforcement must match the living requirement that every undocumented internal edge fails regardless of supported import spelling or item order.

## What Changes

- Replace the test-local textual source scanner with a Rust AST visitor that resolves crate-root owner dependencies from complete use trees and qualified paths.
- Analyze every production item while excluding only items whose `cfg` is test-only, rather than truncating the file at the first test module.
- Add deterministic regressions for relative, grouped, nested, aliased, and qualified dependency forms, production items after tests, test-only exclusions, exact owner matching, and parse diagnostics.
- Add `syn` only as an editor-core development dependency; production code and dependencies remain unchanged.

### Non-goals

- No change to the allowed dependency matrix, ADR ownership graph, public Rust API, persisted formats, runtime behavior, or application contracts.
- No macro expansion or general-purpose Rust semantic analysis beyond the owner paths represented in parsed source.
- No changes outside editor-core architecture enforcement and its OpenSpec documentation.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `editor-core-architecture`: Require enforcement to recognize crate-root-relative and top-level grouped owner paths and to inspect production items regardless of their position around test-only items.

## Impact

- Primary code: `crates/editor-core/tests/architecture.rs`.
- Development dependencies: `syn 2.0.119` with `full` and `visit`; the package is already present in the workspace lockfile transitively.
- Specification: `editor-core-architecture` dependency-enforcement scenario.
- Compatibility: no public, persisted, runtime, matrix, or ADR changes.
- Delivery: one additional commit on PR #94; the unrelated untracked motion-graphics plan remains excluded.
