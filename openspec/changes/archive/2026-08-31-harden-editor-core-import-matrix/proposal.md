## Why

Review of PR #94 found that the editor-core dependency-matrix test recognizes qualified module paths such as `persistence::...` but can miss a forbidden internal dependency imported under an alias. The enforcement must cover every supported Rust import form so the living architecture requirement is true in practice.

## What Changes

- Extract internal owner dependencies from direct, grouped, nested, and aliased `use crate::...` declarations instead of inferring them only from qualified uses in the source body.
- Add deterministic regression tests proving forbidden direct, grouped, nested, and aliased imports are detected while allowed imports remain accepted.
- Keep the existing owner matrix, production modules, public APIs, persisted formats, runtime behavior, and PR scope unchanged.

### Non-goals

- No change to the allowed dependency graph or ADR ownership decisions.
- No Rust parser dependency unless the focused extractor cannot remain small and deterministic.
- No runtime, public contract, persistence, rendering, or asset behavior change.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `editor-core-architecture`: Clarify that dependency enforcement covers aliased and grouped Rust imports, not only qualified module references.

## Impact

- Primary code: `crates/editor-core/tests/architecture.rs` only.
- Specification: `editor-core-architecture` import-enforcement scenario.
- Compatibility: no public, persisted, runtime, or dependency-graph changes.
- Delivery: one follow-up commit on PR #94 after focused and repository verification.
