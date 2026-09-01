## Context

OpenCut currently persists a flat `Project.tracks` timeline and already separates deterministic render planning from FFmpeg execution. The motion-graphics roadmap adds reusable compositions, richer graphics and text, inherited transforms, masks and effects, preset-authored animation, and more complex audio timing. Without a prior decision, each vertical slice could encode a different hierarchy, ordering rule, or renderer assumption into public and persisted contracts.

This change records the governing architecture only. `crates/editor-core` remains the canonical owner of persisted semantics, validation, migration, evaluation, revisions, and history. Headless and MCP layers remain typed transports. The renderer remains downstream of core evaluation. Existing projects, operations, and render output do not change in this issue.

## Goals / Non-Goals

**Goals:**

- Lock the additive root-track model, evaluated-scene seam, hybrid renderer boundary, observable ordering rules, preset compilation policy, and schema-version policy.
- State enough coordinate, timing, compositing, security, compatibility, and failure semantics that later milestones cannot make contradictory choices.
- Keep future preview and export paths deterministic and structurally unable to diverge.
- Give every later public or persisted addition a clear owning layer and migration obligation.

**Non-Goals:**

- Add motion-graphics model types, mutations, migrations, renderer dependencies, or public operations.
- Freeze every future type name, graphics backend crate, effect parameter, complexity constant, or client workflow.
- Promise that every planned primitive ships in one schema version.
- Change existing rendering output or reinterpret current track/item array order.

## Decisions

### 1. Root tracks remain the additive composition root

`Project.tracks` remains the root timeline. Reusable component definitions will be added beside root tracks in a future persisted milestone, and root/component tracks may contain typed composition-instance items. A group is a transform/effect node whose children refer to it by `parent_id`; it does not own a duplicate child list. Parent and component references are scoped, acyclic, and depth-bounded in editor-core.

Root time is expressed in integer milliseconds. Component item times are relative to their component and an instance maps them into its parent scope through a typed start, trim, duration, and finite positive time scale. Slot bindings target typed stable properties, never arbitrary JSON paths.

This preserves simple-operation compatibility and lets nesting ship additively. Replacing `Project.tracks` with a universal composition graph was rejected because it would force a breaking root-project rewrite before any independently useful motion-graphics capability could ship. Duplicating complete nested timelines per instance was rejected because it loses reuse, stable provenance, and bounded updates.

### 2. Editor-core produces one immutable `EvaluatedScene`

Canonical evaluation lives in `crates/editor-core` behind the existing render-planning seam. For a validated immutable project revision and a frame or range request, it resolves component instances, parent transforms, time expressions, preset primitives, repeaters, masks, effects, audio events, and stable ordering into renderer-neutral instructions. Evaluation rejects missing or cyclic references, non-finite values, invalid timing, and exceeded explicit complexity limits with stable typed errors before renderer execution.

Frame preview, audiovisual range preview, draft preview, and final export must all consume this same representation and evaluator. Range evaluation may retain lazy/bounded instructions, but it must be semantically equivalent to evaluating its frames and audio intervals individually. Render backends may not inspect persisted tracks/items to reconstruct scene semantics.

Letting each entry point flatten the project independently was rejected because it permits preview/export drift. Emitting FFmpeg filter expressions from editor-core was rejected because it couples persisted semantics to one renderer and would expose an unsafe expression surface.

### 3. Rendering is hybrid and backend-neutral above compositing

FFmpeg remains responsible for media probing/decoding, audio processing, final composition, and encoding. Complex vectors and shaped text will be rasterized by a deterministic Rust graphics backend into bounded intermediate layers supplied to the existing composition plan. Graphics rasterization, process execution, and artifact I/O stay behind narrow replaceable interfaces; no graphics backend type enters persisted or public contracts.

Inputs are canonical typed data. Raw FFmpeg expressions, executable SVG content, arbitrary filesystem paths, and network resources are never accepted. Inline or managed SVG is sanitized and complexity-bounded before evaluation. Fonts and assets resolve through managed, content-addressed provenance so reopen behavior cannot silently change.

An FFmpeg-only renderer was rejected because its text/vector semantics and filter-expression surface cannot provide the required deterministic richness safely. A full GPU renderer was deferred because it adds deployment and parity risk before the model is stable. Pre-rendered external overlays were rejected because they disconnect editable semantics from project state.

### 4. Ordering and compositing rules are normative

The root and each component use a top-left origin with positive X rightward and positive Y downward. Pixel coordinates are the compatibility default; normalized coordinates, when explicitly selected by a future typed field, resolve against the containing composition dimensions. Times use integer milliseconds and half-open intervals `[start_ms, end_ms)`.

Within one composition scope, tracks render from lowest array index to highest. Within a track, visual items sort by ascending explicit `z_index`, then by their stable array order, then by stable item ID only as a final deterministic tie-break for synthesized/equivalent inputs. Later entries composite above earlier entries. Audio ordering follows the evaluated audio plan and must not depend on map iteration.

The visual pipeline is: source decode/rasterization; crop and local clip; declared masks in order; declared effects in order; local anchor translation, scale, skew, rotation, and position; ancestor transforms from nearest parent outward; track matte; inherited opacity; blend into the destination. Ancestor opacity multiplies. Compositing uses premultiplied alpha in linear light and converts to the configured output color space at the output boundary.

Leaving ordering renderer-defined was rejected because map iteration, filter construction, or backend replacement could alter output. Using only array order was rejected because reusable/groups layers need explicit local ordering without destructive array surgery. A second group-owned child order was rejected because two sources of truth cannot be migrated or edited atomically without ambiguity.

### 5. Presets compile to persisted primitives

A preset is a pure, bounded editor-core compiler from a versioned preset ID plus typed finite parameters into the same primitive keyframes, effects, masks, and audio events accepted by low-level operations. The mutation persists the resolved primitives and optional provenance containing the preset ID/version and parameters. Evaluation and rendering use only resolved primitives; changing or removing a preset implementation cannot alter an existing project on reopen.

Preset application is a normal atomic revisioned mutation, is undoable/redoable, and when exposed to agents must work standalone and inside `timeline_batch_edit` with the existing alias rules. Compilation failure publishes nothing. Preset provenance is descriptive and never an executable fallback.

Persisting only preset names was rejected because output would drift as libraries evolve. Expanding presets only in MCP was rejected because desktop/headless callers and reopened projects would not share canonical behavior. Baking presets to pixels was rejected because it loses editability.

### 6. Persisted milestones version additively and migrate atomically

This ADR itself does not bump schema version 6. Each independently shippable future persisted-model milestone performs exactly one project-schema bump, adds deterministic migration for the current project plus every retained undo and redo snapshot under the project lock, and publishes the migrated generation atomically. Compatible defaults preserve existing simple operations and rendering semantics. Unknown future versions fail closed without rewriting data.

Public request/response changes remain typed and additive unless a separately approved breaking contract and explicit migration path is introduced. A new uniquely named operation or optional input is not sufficient by itself: the canonical contract catalog, all governed consumers, capability/version reporting, and parity evidence must change together whenever clients need to distinguish support.

One schema bump for the entire initiative was rejected because it would couple independently shippable milestones and leave partially implemented states ambiguous. Bumping for every field was rejected because it creates needless migration churn. Lazy migration of only current state was rejected because undo/redo could reintroduce an old schema.

## Risks / Trade-offs

- **[Risk]** The ADR constrains implementation before exact public types are designed. **Mitigation:** lock only cross-milestone invariants; require each later OpenSpec change to define exact types, limits, stable errors, fixtures, and tests.
- **[Risk]** Hybrid CPU rasterization can be slower than a GPU path. **Mitigation:** keep evaluation and compositor interfaces backend-neutral, use bounded lazy expansion/caching, and define performance limits in the implementing milestone.
- **[Risk]** Stable tie-breaking can hide duplicate logical IDs in malformed internal data. **Mitigation:** validation still rejects duplicate IDs; the final ID tie-break only guarantees deterministic handling of synthesized/equivalent inputs.
- **[Risk]** Persisting resolved primitives increases project size. **Mitigation:** bound preset expansion and prefer compact canonical primitives; provenance stays optional and non-authoritative.
- **[Risk]** Linear-light premultiplied compositing may differ from current FFmpeg shortcuts. **Mitigation:** it applies only when the future evaluated graphics pipeline ships, with golden tolerance fixtures shared by preview and export.
- **[Risk]** A milestone could expose a public operation before capability reporting is updated. **Mitigation:** fixture-governed contract parity and the later change's acceptance tests must cover the catalog and every consumer together.

## Migration Plan

1. Merge this documentation-only ADR and living requirements without changing schema version 6 or runtime output.
2. Require each later motion-graphics milestone to cite ADR 0004 and define its exact additive contracts, validation limits, errors, migration, deterministic fixtures, and rollback behavior in a separate approved OpenSpec change.
3. For a persisted milestone, migrate current state and retained history on a clone under lock, validate the complete migrated generation, then publish atomically through the existing recoverable transaction boundary.
4. Rollback of this decision-only change is a documentation revert. Rollback after a persisted milestone must use a forward migration or a compatible reader; binaries must never silently downgrade an unknown future schema.

## Open Questions

- Which deterministic Rust shaping/rasterization backend best satisfies packaging, font, color, and performance constraints?
- What exact maximum nesting depth, repeater count, SVG/path complexity, effect count, and raster resource budget should each implementing milestone expose?
- Which stable error codes and capability identifiers should accompany the first public motion-graphics model additions?

These questions are intentionally deferred because their answers do not alter the six locked architecture decisions.
