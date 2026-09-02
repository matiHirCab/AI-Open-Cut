## Context

The private flat-scene evaluator currently constructs some collections before it has proved their persisted input sizes are bounded. Transition facts are found by scanning every transition for every visual layer, and configured text fonts are represented only by an opaque ID with no process-local binding back to the selection request. The existing production renderer still consumes its borrowed plan, so this corrective change can harden the new seam without changing output or public contracts.

## Goals / Non-Goals

**Goals:**

- Reject invalid references and excessive work before scene allocation, voiceover derivation, or downstream work.
- Preserve deterministic first-use and declaration ordering with linear transition indexing.
- Preserve enough process-local media and font selection information for issue #13 while keeping `EvaluatedScene` path-free and renderer-neutral.
- Maintain current typed-error precedence and all existing output behavior.

**Non-Goals:**

- Routing production preview/export entry points through `EvaluatedScene`.
- Resolving or canonicalizing filesystem paths during scene evaluation.
- Changing public/persisted schemas, contracts, capabilities, errors, migrations, revision semantics, or history.

## Decisions

### Preflight referenced assets before complexity

Evaluation first indexes the project assets and checks every visible media reference. This preserves `ASSET_NOT_FOUND` when the same input also exceeds a complexity limit. It then counts keyframes by property, emitted transition endpoint facts, visible layers, first-use resources, and audio layers before allocating output collections or deriving voiceover intervals.

Alternative: return the first complexity error encountered while walking. Rejected because it changes established missing-asset precedence and makes failures sensitive to allocation order.

### Bound emitted transition facts and index once

The inclusive transition-fact limit is 4,096 facts. Each source endpoint emits an `Out` fact and each target endpoint emits an `In` fact; a transition whose source equals its target therefore emits two facts. A single stable index keyed by item ID is built in transition declaration order, replacing per-layer full scans.

Alternative: limit transition records rather than emitted facts. Rejected because renderer work and scene size are proportional to endpoint facts, not records.

### Separate scene semantics from resource selection requests

The evaluator returns `EvaluatedSceneResult { scene, resource_bindings }`. `EvaluatedScene` retains only logical resource IDs. `SceneResourceBindings` stores project-relative media requests and requested font path/family selections for process-local preparation. A font resource ID exists whenever either a path or family is configured. Raw requested paths remain exclusively in this sidecar; resolved paths continue to be produced only by the existing root-constrained preparation layer.

Alternative: embed paths in text layers or logical resources. Rejected because arbitrary paths are expressly forbidden in the renderer-neutral scene. Alternative: discard configured font selection until routing. Rejected because the scene seam would be insufficient to reproduce current behavior without re-traversing persisted records.

### Keep the envelope private

The module and result remain private to editor-core and are evaluated but discarded by the current borrowed render-plan path. Issue #13 will consume the envelope. Architecture tests guard that the sidecar does not acquire renderer-process/backend types and that path fields do not enter `EvaluatedScene`.

Alternative: expose the result now. Rejected because it would expand public compatibility scope and prematurely couple this correction to routing.

## Risks / Trade-offs

- [Risk] Preflight walks input more than once. → The walks are linear, bounded before output allocation, and replace an existing potentially quadratic transition scan.
- [Risk] A sidecar containing requested paths could leak into renderer-neutral contracts. → Keep it private, structurally separate, and enforce source-level architecture assertions.
- [Risk] Error precedence could shift during refactoring. → Add combined missing-reference/complexity tests and retain missing-asset validation as the first preflight phase.
- [Risk] Transition indexing could reorder facts. → Append facts in transition declaration order and test order plus self-endpoint dual emission.

## Migration Plan

No data or API migration is required. The change is an internal derivation refactor. Rollback consists of reverting the corrective evaluator, tests, and documentation together; persisted projects remain untouched.

## Open Questions

None.
