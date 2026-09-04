## Context

Schema version 6 stores `transform` independently on media, text, solid-color, and rectangle items and stores `hidden` independently on every timeline variant. Caption and transition items have no transform value. This duplication blocks later milestones from evolving one visual contract, while issue #17 must not absorb the Transform2D, stacking, hierarchy, component, mask, effect, or animation work assigned to later issues.

Project documents, headless project responses, MCP schemas, and TypeScript declarations are compatibility surfaces. Migration already runs under the project lock and commits current state and retained history through one recoverable transaction. Production frame, range, draft, and export paths already consume one editor-core `EvaluatedScene`.

## Goals / Non-Goals

**Goals:**

- Establish one editor-core `VisualProperties` owner for the current transform and visibility state on every timeline item.
- Advance schema v6 to v7 by materializing deterministic defaults across current state and retained undo/redo snapshots without altering evaluated pixels, audio, timing, ordering, or revisions.
- Preserve existing serialized field names and edit-operation request shapes.
- Keep contract fixtures and Rust/TypeScript consumers structurally exact.

**Non-Goals:**

- Add Transform2D fields or semantics, z-index, parents/groups, components, slots, masks, effects, new animation channels, new operations, new capabilities, or UI.
- Change stable errors, retryability, batch alias behavior, revision semantics, render tolerance, path/resource handling, or provider contracts.

## Decisions

### Use one flattened common value on every item variant

Each timeline item owns `visual_properties: VisualProperties`. `VisualProperties` contains the existing `Transform` and `hidden` values and is flattened by Serde into the item object. Media, text, solid-color, and rectangle preserve their exact `transform` and `hidden` JSON keys. Caption and transition gain a default identity transform in schema v7, while their renderer behavior continues to ignore it until a separately approved milestone defines such behavior.

This choice creates one Rust ownership seam without introducing a nested public object or changing existing edit payloads. A nested `visualProperties` JSON object was rejected because it would needlessly break every current project consumer. Leaving caption and transition outside the common value was rejected because it would preserve the duplication this milestone exists to remove.

### Keep compatibility sugar at the operation boundary

Existing `add_*`, `update_item.transform`, and `set_item_visibility` request fields remain unchanged. Editor-core constructs or updates `VisualProperties`; transports continue to submit typed requests without duplicating defaults or validation. No operation returns a newly created reference, so no new batch alias case is needed.

A new `set_visual_properties` operation was rejected because it expands the public surface without adding user-visible behavior. Transport-side translation into a second visual model was rejected because editor-core owns persisted semantics.

### Perform a model-level v7 migration across the complete retained generation

The migration owner upgrades supported schema versions through v7 and applies identity `Transform::default()` plus `hidden: false` when an older serialized item lacks the newly common field. Existing transform and hidden values are preserved exactly. Current state and all undo/redo snapshots are migrated and validated before the existing transaction publishes any part of the generation.

Migration is idempotent for v7, rejects version 0 and versions above 7, and leaves the on-disk generation untouched on deserialization, migration, or validation failure. Migrating only current state was rejected because undo or redo could restore schema-v6 semantics. Publishing files independently was rejected because it violates crash-consistent generation ownership.

Compatibility defaults apply during deserialization regardless of whether the input already declares schema v7. A read-only reopen of such a v7 document does not force a rewrite solely to materialize omitted fields; returned project state and any later committed serialization contain the explicit canonical keys. Requiring explicit keys on v7 input was rejected because the milestone intentionally introduces additive defaults and existing editor-core persisted models use compatibility defaults at deserialization boundaries.

After model migration, validation of common visual properties and retained asset references runs across current state and all history before legacy asset normalization. This order is required because asset normalization may publish a content-addressed file even before the project/history transaction begins. Validating after that step was rejected because malformed retained state could leave a newly published managed asset despite the open failing.

### Keep evaluation and rendering behavior unchanged

`EvaluatedScene` reads transform and visibility through common accessors but emits the same flat instructions in the same deterministic order. Caption and transition identity transforms are non-operative in this milestone. Golden semantic/filter-graph and decoded visual/audio fixtures remain the pixel-equivalence authority for frame, range, and export; draft preview receives a focused equivalence assertion through its shared evaluator path.

Direct renderer support for the richer motion-graphics fixture transform was rejected because it belongs to issue #18. Activating additional motion-graphics fixture concepts or capability identifiers was rejected because this change exposes no new client-addressable behavior.

### Update only governed project-shape evidence

The schema version and additive default fields are updated in the canonical protocol/project examples and every listed Rust and TypeScript/Zod consumer. The version-1 operation and MCP catalogs retain their existing meanings. `contracts/motion-graphics-v1.json` remains `fixture_only`; documentation records that common-property ownership is active while Transform2D and layer semantics remain inactive.

Changing a stable error or adding a capability was rejected because clients cannot exercise any new behavior in this milestone.

## Risks / Trade-offs

- [Flattening can hide ownership in JSON] → Keep the Rust field named `visual_properties`, document the flattened compatibility form, and assert exact serialization in canonical parity tests.
- [Caption/transition transforms could accidentally affect pixels] → Do not route those identity transforms into evaluated output and verify all render intents against existing golden references.
- [Legacy defaults could be applied inconsistently across history] → Migrate the current project and every undo/redo snapshot with the same function, validate the whole envelope, and publish once under the lock.
- [A malformed or future document could be partially rewritten] → Complete conversion and validation in memory first and preserve fail-closed transaction behavior.
- [Legacy asset normalization can publish before a later validation failure] → Validate common visuals and retained references before invoking the side-effectful asset migration path, and assert the managed asset store remains unchanged on rejection.
- [Large fixture churn] → Restrict updates to the canonical project shape and consumers named by contract ownership; do not activate unrelated motion-graphics concepts.

## Migration Plan

1. Add common model/accessors and schema-v7 compatibility deserialization while the schema constant remains controlled by focused tests.
2. Update the migration to transform current state and every retained history snapshot, then validate and atomically publish the complete generation.
3. Update timeline mutations and scene evaluation to use the common value without changing request or evaluated-scene shapes.
4. Advance canonical fixtures and all governed consumers together, then run contract parity.
5. Run migration fixtures for v1, v6, schema-zero rejection, schema-v7 compatibility defaults, every injected persistence phase, invalid visual/reference pre-publication failures, revision/batch/undo/redo/reopen tests, golden render parity, and the full required validation suite.
6. Roll forward from a defective v7 with a separately approved additive migration. Do not downgrade schema-v7 documents; older binaries fail closed as designed.

## Open Questions

None. Transform2D fields and stacking are explicitly deferred to issues #18 and #19.
