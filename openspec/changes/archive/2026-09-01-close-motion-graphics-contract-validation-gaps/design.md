## Context

Review mutations demonstrated three accepted-invalid catalogs: two project component definitions with `maxComponentDefinitions` set to one, duplicate fixture IDs in the TypeScript suite, and a project-scoped component entered as a managed resource. Static comparison also found Rust semantic checks missing constraints already enforced by Zod. The catalog remains inactive fixture evidence, so the correction can preserve version 1 without migration or compatibility work.

## Goals / Non-Goals

**Goals:**

- Make aggregate named limits operate on payload-derived objects grouped by their semantic owner.
- Reject duplicate fixture IDs before normalization or result-map insertion.
- Treat managed resources as exactly project-scoped assets.
- Make Rust and Zod reject the same malformed concept fields.
- Add focused evidence that would fail under the reviewed implementation.

**Non-Goals:**

- Activate motion graphics in production types, persistence, headless, MCP, providers, evaluation, preview, export, or packaging.
- Change stable errors, migrations, catalog version/status, dependencies, ownership, or generated artifacts.
- Modify an archived change.

## Decisions

### Count limits after payload derivation

Both validators will accumulate validated payload-derived definitions before global reference closure. Component definitions count project-wide. Layers and markers count independently for each `root` or `component:<id>` composition. Slots count for each component scope. Audio events count for each root or component composition. Limits are inclusive. Per-channel keyframes, per-layer masks/effects, per-component slot membership, and hierarchy depth remain enforced at their existing payload or graph boundary.

Tests may lower a copied catalog's limit to the number of constructed records, allowing exact-boundary and overflow evidence without generating production-sized arrays. Counting metadata instead of payloads is rejected because metadata is itself untrusted evidence and must not control validation.

### Reserve fixture identities before validation

A single ID set covers `validFixtures` and `invalidFixtures`. Each ID is reserved before parsing its payload or inserting an observed failure, so duplicates within either array or across arrays fail deterministically and cannot overwrite map entries. Rust's existing behavior will be extended with the same cross-array regressions; TypeScript will adopt it.

### Use a dedicated managed-asset type

The catalog wrapper will parse each managed resource with a closed `{ kind: "asset", scope: "project", id }` schema/type rather than the generic reference schema. Duplicate tuple detection remains before set normalization, and payload references must still resolve to the resulting managed-asset definitions.

### Mirror semantic constraints explicitly

Rust retains closed Serde declarations and adds semantic checks equivalent to the current strict Zod schemas: non-empty animation-channel strings; valid IDs for component tracks and slot-value keys; valid marker ID/name and safe-integer timestamp; and a non-empty curve collection. The implementation audit will compare every adjacent field of the affected Rust structs with its Zod schema and add missing identifier, required-value, collection, numeric, or length checks rather than weakening Zod.

## Risks / Trade-offs

- [Aggregate counts can be assigned to the wrong owner] → Derive the owner from the validated tuple scope and test root plus multiple component scopes independently.
- [Duplicate checks can change which malformed error appears first] → Reserve fixture IDs before payload validation and document identity failure as a catalog-wrapper invariant, not a fixture reason key.
- [Manual schemas can drift again] → Add mirrored table-driven mutations for every corrected field and retain independent identifier catalogs.

## Migration Plan

No runtime or data migration applies. After approval, update both test helpers and focused tests together, update living documentation, run every repository gate, sync the modified specification, and archive only this corrective change. Rollback removes these corrective implementation/spec edits while leaving prior archives untouched.

## Open Questions

None.
