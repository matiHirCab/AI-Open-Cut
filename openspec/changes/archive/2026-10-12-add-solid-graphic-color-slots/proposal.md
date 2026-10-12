## Why

Issue #183 needs independent palette inputs for native illustrated components. Existing typed color slots only target text, although native shapes and dot grids already support solid paint.

## What Changes

- Add exactly three color bindings: `shape.fill.color`, `shape.stroke.color`, and `grid.dot.paint.color`. Each requires an existing solid target paint; replace unassociated sRGB RGB channels from the existing six-digit hex slot value while preserving the paint's alpha and all other properties.
- Reuse existing definition/instance operations, aliases, locks, optimistic revisions, defaults/overrides, history, supported drafts and native evaluation. Reject unsupported targets, gradients, absent paint and wrong kinds before publication.
- Govern additive request/property/capability surfaces and schema45 adoption. Migrate existing current and undo/redo snapshots atomically without changing their content beyond schema version; reject new bindings in predecessor schemas and unknown future versions.
- Demonstrate canonical Rust/TypeScript parity, complete failure atomicity, independently colored occurrences and actual source/fresh packaged native frame/range/draft output.

## Capabilities

### New Capabilities
- `solid-graphic-color-slots`: closed graphic binding matrix, exact RGB/alpha semantics, instance isolation, schema compatibility and independent native conformance.

### Modified Capabilities
- `template-slots`: extend the closed local binding-property mapping with the three named graphic targets while preserving all existing kinds, constraints and failures.

## Impact

Core model/validation/effective evaluation and schema migration; existing headless capability metadata; bridge structural schemas; manually reviewed canonical slot/headless/MCP/ownership fixtures and all governed parity consumers. No new operations, dependencies, error codes or renderer foundation. Additive protocol-v1 extension: existing requests and slot values keep exact meaning. Persisted schema44 advances to45 so older builds reject unsupported new content rather than reinterpret it. CODEOWNER remains @matiHirCab; request review on the draft PR and record outstanding human review separately from delegated specification approval.

## Non-goals

No general property binding language, alpha slot/value extension, gradient slots, missing-paint creation, SVG editing, line-grid color binding, palette UI, animation/preset changes, broad motif pack, performance/concurrency guarantee or desktop work. #181 was merged in initial main50ec842c. #182/PR195 is not a dependency; it merged externally during verification and is preserved in reconciled main84da152f. No merge, auto-merge or deployment.

External main reconciliation: #182/PR195 merged externally during checks; current base84da152fac0378396099d8f01fc3b7339690361f is reconciled without conflict. Complete hero recipe/source/package/native assertions remain alongside the graphic assertions. All327 captured Rust/catalog inputs are byte-identical before/after this reconciliation, verified in the external reconciliation receipt; full workspace1526/12 existing ignored, fmt and strict Clippy passing evidence remains valid under AGENTS. All bridge/native/Python stages execute after reconciliation. Original captured catalog base50 provenance remains historical.

Exact-head CI correction: repair passive release process observation across same-PID exec; preserve native behavior, cache policy, every assertion/capture/report/budget/deadline, and all previously verified graphic behavior. Synchronize existing motion-release requirements with current schema45 adoption. No renderer/API/dependency change.
