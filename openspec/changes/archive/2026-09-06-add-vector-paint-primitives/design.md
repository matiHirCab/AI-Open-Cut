## Context

Issue #27 precedes #28, which explicitly owns ShapeItem and timeline_add_shape. Existing living requirements cover legacy editing, schema migration, private EvaluatedScene, and fixture-governed adoption. ADRs 0002, 0003, and 0004 require core ownership, synchronized catalogs, and renderer-independent semantics. The working tree had no active change authorizing #27.

## Goals / Non-Goals

Goals: reusable production Rust primitives, pure bounded validation, exact wire vocabulary, parity evidence, and documented geometry/color semantics for later consumers.

Non-goals: new timeline items, mutations, renderer backend, public transport operation, persisted field, SVG, tessellation, or animation. No rendering support is advertised merely because the library types exist.

## Decisions

1. Add a focused vector module under editor-core, exported through lib.rs. Use strict Serde DTOs and pure validators returning CoreError/INVALID_ARGUMENT; validate directly constructed Rust values as well as decoded data. Check collection lengths before iteration and nested values before derived arithmetic. This keeps domain rules out of transports and avoids adding backend dependencies. A renderer-owned model was rejected because it violates ADR 0003.
2. Use explicit RGBA components and tagged paints rather than overloaded CSS/hex/SVG strings. Keep current string colors unchanged. Gradients use local pixel space, pad extension, strict endpoint-inclusive stops, and linear-light premultiplied interpolation semantics. Implement data validation here; gradient raster sampling remains #28. Hard-stop duplicates, radial focal points, spread modes, and alternate units can be additive future contracts.
3. Require explicit stroke options, even dash arrays, and circular per-corner radii. Do not silently duplicate odd dash arrays. Provide a pure radius-resolution helper implementing the spec's common factor; test it with independently calculated asymmetric examples. Stroke geometric semantics are documented now; rasterization remains deferred.
4. Use absolute point-bearing commands and an iterative path state machine. A move-only path is legal empty geometry. After close, require a new moveTo. No path strings, external references, or executable inputs enter the model. Raw SVG parsing and renderer expressions were rejected for ambiguity and safety.
5. Add contracts/vector-primitives-v1.json with unique fixture IDs, exact tags/fields, explicit limits, and core_primitives_only activation status. Register canonical ownership and actual Rust/TypeScript consumers in contract-ownership-v1.json. Reuse the repository's fixture validation/parity harness and strict Zod convention. Zod mirrors the core contract for transport readiness and evidence; it is not a new application validation service. Python providers are not consumers and acquire no vector semantics.
6. Keep headless and MCP catalogs unchanged because no operation consumes these values yet. Add reusable bridge schema exports and typed declarations as needed for parity, without dead feature flags or registration. Require CODEOWNER review from @matiHirCab for catalog and consumer changes.

## Risks / Trade-offs

- Generic issue acceptance text mentions migrations and batch operations, while #28 owns their activation: approval must explicitly accept this primitive-only scope. Expanding activation requires revised artifacts before implementation.
- Early numeric limits constrain future shapes: document exact bounds and test limit and limit-plus-one cases before downstream adoption.
- Serialization alone can admit invalid constructed floats: every public validator checks finiteness; Rust-specific NaN/infinity tests supplement JSON fixtures.
- Paint semantics can drift at raster activation: #28 must implement the same color/geometry semantics through EvaluatedScene with independent pixel evidence. This change does not claim vector render parity it cannot exercise.
- An active proposal blocks protected Moon policy by design: use focused pinned OpenSpec validation while authoring and report the gate until verified archival.

## Migration Plan

No persisted field changes; schema remains 13. No migration is necessary and retained state/history remain byte-compatible. Existing migration, future-version rejection, revision, rollback, undo/redo, reopen, and render tests provide regression evidence. Missing-reference and new creation-alias tests are not applicable because primitives contain no references or mutation entry points. Rollback removes the additive library/schema/catalog work together; no persisted data transformation is required.

## Verification Plan

Each requirement and scenario in vector-primitives/spec.md maps to focused Rust tests and shared fixture parity tests. Test every tag, strict field shape, finite boundary, collection boundary/overflow, path transition, radius scaling example, and immutable round trip. Run contract parity, workspace Rust fmt/strict Clippy/tests, bridge typecheck/lint/unit and integration/packaged smoke, relevant hermetic provider suites, and existing render regression tests. No test is waived merely because it is inconvenient; record environment failures explicitly.

After implementation run openspec-verify-change, resolve all mismatches, obtain contract-owner review, synchronize/archive with openspec-archive-change, and run moon run openspec-validate (or the configured root-qualified task) and required repository checks. Keep a scenario-to-test evidence table with command results in verification.md.

## Open Questions

The user explicitly approved the scope and numeric/serialization semantics on 2026-09-06. Implementation may proceed through tasks.md.
