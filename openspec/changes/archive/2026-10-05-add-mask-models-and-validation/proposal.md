## Why

Issue #50 requires validated authored alpha/luma mask models with ordered add/subtract/intersect/exclude, inversion, transform, feather and signed expansion. Current runtime projects have none. The fixture-only motion-graphics catalog is deliberately not an active authoring contract. Introduce an independently bounded model so #51 can subsequently activate mask sampling/rasterization without accepting raw SVG, paths or backend expressions.

## What Changes

- Add optional ordered `masks` to existing visual-property `update_item` edits, alias-aware batches, persisted eligible visual leaves, project responses and draft operations. Omission preserves edits; empty arrays clear; present null rejects.
- Define a closed path source containing existing `VectorPath` and `Paint`, alpha/luma channel, four operations, inversion, existing Transform2D, feather and signed expansion. Painted paths provide meaningful luma without source-layer references.
- **BREAKING (persisted-version compatibility):** Advance persisted project schema 31 to 32; atomically migrate current state, retained undo/redo and applicable drafts under the project lock, defaulting old documents to no masks and rejecting premature fields.
- Add additive editor authoring capability `mask_models_v1` and canonical `contracts/mask-models-v1.json`, synchronizing all governed Rust/Serde, TypeScript/Zod and MCP consumers and deliberately reviewing the expanded MCP digest transition.
- Preserve renderer semantics, plans, output, audio and existing publication guarantees with authored masks present: this is metadata-only. #51 is the rendering activation milestone.

## Capabilities

### New Capabilities

- `mask-models`: Closed painted path-mask representation, limits, ownership, ordered editing and observable metadata semantics.

### Modified Capabilities

- `project-persistence`: Schema32 current/history/draft atomic migration and fail-closed source-version guards.
- `rendering-export`: Metadata-only output preservation and explicit absence of mask-rendering support.
- `contract-governance`: Active mask catalog, additive capability, synchronized schemas and deliberate expanded digest transition.
- `linear-light-compositing`: Reconcile Explicit current layer pipeline so authored schema 32 metadata is accepted while rendered masks stay identity; retain existing math/limits and matte/blend/effect scope.
- `motion-graphics-architecture`: Reconcile Normative coordinate and compositing semantics with model activation versus future raster activation.

## Non-goals

No rasterization, animated mask properties, source-layer masks, track mattes/DAG, managed-resource sources, stroke masks, raw SVG/string paths, new edit operations, new stable errors, desktop inspector, non-normal blends or new effects. No activation or reinterpretation of fixture-only motion-graphics records. No change to layer timing/order, core geometry certification, or existing renderer budgets.

## Impact

Request/response changes are additive on existing protocol major 1 and existing operations; schema 32 is a persisted version transition that older binaries reject. Old accepted requests and field aliases remain valid; strict response validators pinned to schema 31 require upgrade for schema 32 status/project responses, consistent with the existing persisted-version boundary. Canonical mask identity is item-local, unlike fixture-only scoped placeholder definitions. Root update_item targets retain current addressing; component definitions carry masks through existing component_create/component_update payloads. All validation, migration and publication stay in editor-core. Designated CODEOWNER @matiHirCab must review synchronized contracts; parent approval of this proposal is a separate implementation gate. The prerequisite #49 is verified/archived in baseline b0a9075f, including the reviewed readiness-conformance correction. Schema31 and the expanded MCP predecessor digest remain the independently recomputed approved production baseline. Explicit parent and independent specification approval are recorded in approval.md; implementation and conformance evidence are tracked in tasks.md and verification.md.

The schema32 transition also synchronizes the current projectSchemaVersion marker of six existing active animation catalogs (animation-channels, animation-presets, extended-visual-animation, inherited-animation-timing, motion-blur-sampling and initial-motion-preset-pack) and their governed expectations. Only that metadata literal changes; historical source-version guards, feature identifiers, bounds, fixtures and MCP predecessor projection remain unchanged.
