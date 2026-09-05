## Why

Issue #19 (MG-M1-03) requires agents to control stacking without renderer workarounds. The current schema-v8 evaluator follows track/item arrays and exposes indexed track updates, but has no explicit z-index or item reorder operation.

## What Changes

- Persist signed `zIndex` and canonical `stackOrder` on timeline items in schema 9; migrate supported current state and all retained history atomically, preserving existing output.
- Add `item_set_z_index`, `item_reorder`, and `track_reorder` standalone and batch operations, including creation aliases and existing revision/history protections.
- Evaluate visuals bottom-to-top by track array index, z-index, stack order, and stable ID as the final tie-break. Keep item array order synchronized with stack order.
- Publish a governed runtime stacking catalog and additive `stacking` capability, with typed headless/MCP parity and render regression coverage.

## Capabilities

### New Capabilities

None; extend existing capabilities.

### Modified Capabilities

- `timeline-editing`: bounded persisted ordering and transactional reorder operations.
- `project-persistence`: schema-v9 current/history migration and fail-closed recovery.
- `motion-graphics-architecture`: canonical flat visual ordering with explicit stack values.
- `motion-graphics-contracts`: runtime stacking vocabulary, discovery, and parity.
- `rendering-export`: ordering conformance across every rendering intent.

## Impact

Core owns model, mutations, migration, validation, and evaluation. Headless and bridge adapt typed operations; canonical catalogs, ownership entries, fixtures, tests, and documentation change together. Public additions preserve protocol major version, existing operation payloads, errors, and retryability. Persisted schema advances from 8 to 9 and older binaries reject it through existing future-version handling. No new package dependencies or provider contracts are intended. The approved rendering correction adds the inward evaluated_scene -> validation edge in ADR 0003 and architecture checks so direct render inputs reuse canonical ordering validation. Contract review belongs to @matiHirCab.

## Non-goals

Groups, components, parenting, masks, blend modes, animation of z-index, audio routing, new desktop controls, and changes to timing or Transform2D are outside this issue. Roadmap concepts remain inactive.

## Approval

User explicitly approved these artifacts with "Approve" in this task on 2026-09-04. Implementation is authorized under AGENTS.md. Final contract-owner review remains tracked separately.

The user subsequently requested implementation of the rendering-boundary fix plan, explicitly approving its architecture amendment and regression coverage. This does not constitute final contract-owner review.
