# ADR 0004: Motion-graphics architecture

- Status: Accepted
- Date: 2026-09-01
- Issue: [#10](https://github.com/matiHirCab/AI-Open-Cut/issues/10)
- Related roadmap: [motion-graphics implementation plan](../motion-graphics-implementation-plan.md)
- Extends: [ADR 0003](0003-editor-core-module-boundaries.md)

## Context

OpenCut persists a flat root timeline in `Project.tracks` and already separates deterministic render planning from FFmpeg execution. The motion-graphics roadmap introduces reusable compositions, richer graphics and shaped text, inherited transforms, masks and effects, preset-authored animation, and more complex audio timing. Those milestones need one architectural contract before they add public or persisted fields; otherwise hierarchy, ordering, and renderer assumptions could diverge across preview, export, desktop, headless, and MCP paths.

This ADR locks six cross-milestone decisions. It does not implement motion-graphics types or behavior, bump project schema version 6, or change current render output.

## Decision

### 1. Additive root-track model

`Project.tracks` remains the root composition timeline. A future persisted milestone will add reusable component definitions beside root tracks, and root or component tracks may contain typed composition-instance items. Existing simple track and item operations retain their meaning.

A group is a transform/effect node. Children refer to it through a scoped `parent_id`; the group does not keep a second child list. Parent and component references must be typed, scoped to one composition, acyclic, and explicitly depth-bounded by `editor-core`. Slot bindings target typed stable properties rather than arbitrary JSON paths.

Root time and component-local time use integer milliseconds. Component intervals are relative to their component; an instance maps them into its parent scope through typed start, trim, duration, and a finite positive time scale.

We rejected replacing `Project.tracks` with a universal graph because that would require a breaking root-project rewrite. We rejected duplicating full nested timelines per instance because that loses reuse, provenance, and bounded updates.

### 2. Canonical EvaluatedScene seam

`crates/editor-core` owns canonical scene evaluation behind the render-planning seam established by ADR 0003. Given a validated immutable project revision plus a frame or range request, it resolves component instances, parent transforms, time expressions, compiled preset primitives, repeaters, masks, effects, graphics, audio events, resources, and stable ordering into one immutable, renderer-neutral `EvaluatedScene`.

Frame preview, audiovisual range preview, draft preview, and final export must consume the same evaluator and scene semantics. Range evaluation may retain bounded lazy instructions, but it must be semantically equivalent to evaluation of its constituent frames and audio intervals. A renderer backend must not inspect persisted tracks or items to reconstruct hierarchy, timing, preset expansion, inputs, resources, layer order, or audio order.

Evaluation rejects missing or cyclic references, invalid timing, non-finite values, and work exceeding explicit canonical complexity limits with stable typed errors before rasterization, FFmpeg execution, or artifact publication. Atomic mutation failures preserve the prior revision and history.

Issue #12 establishes the first concrete, crate-private form of this seam in `crates/editor-core/src/evaluated_scene.rs`. It owns canvas metadata, integer-millisecond half-open spans, stable track/item order keys, logical media resources, closed flat visual instructions for media, text, solid color, and rectangles, separate audio instructions with resolved mute, fade, automation, role, and ducking settings, and one scene-level merged voiceover-activity table shared by every ducked layer. The records own their values and never retain `Project`, `Track`, timeline-item, or `Asset` references. Explicit inclusive limits are 4,096 visual layers, 4,096 logical media resources, 4,096 audio layers, 4,096 emitted transition endpoint facts, 10,000 keyframes per property channel, and 10,000 positive voiceover activity ranges before merging. Evaluation checks referenced assets first, then validates checked video/audio source ends against overflow and known asset duration while preserving the renderer's image-source exception, then validates all named limits before allocating output collections or the scene-level interval table. The activity-range preflight uses only per-item temporary results already bounded by the keyframe limit. Transition facts are indexed once in declaration order; a transition whose source and target are the same item emits both `Out` and `In` facts.

This initial model resolves media by logical asset ID and represents an explicitly configured text font with a generated logical font-resource ID; it contains no project-relative or absolute path, URL, renderer expression, prepared file, backend command, or artifact destination. Evaluation returns the scene in a private process-local envelope beside `SceneResourceBindings`: a separate deterministic sidecar that binds logical media IDs to project-relative requests and logical font-resource IDs to the original requested path/family selection. Requested paths exist only in that sidecar; canonicalized filesystem paths remain owned by the existing root-constrained resource-preparation layer and never enter `EvaluatedScene` or a backend unchecked. Text with neither configured path nor family emits no font binding, leaving default-font selection to renderer configuration.

The scene and binding envelope are derived process-local state, so schema version 6, retained undo/redo history, public transports, capability reporting, and the fixture-only status of `contracts/motion-graphics-v1.json` remain unchanged. During the issue #12 compatibility stage the production planner performs this bounded evaluation and then continues through the existing borrowed planning representation. Issue #13 owns making every render entry point consume the owned scene and sidecar directly and removing that temporary duplicate traversal after preview/export parity is proven.

We rejected independent flattening in each render entry point because it permits preview/export drift. We rejected producing FFmpeg expressions in `editor-core` because that couples canonical semantics to one backend and creates an unsafe expression surface.

### 3. Hybrid renderer boundary

FFmpeg remains responsible for media probing and decode, audio processing, final composition, and encoding. A deterministic Rust graphics backend rasterizes complex vectors and shaped text into bounded intermediate layers. Graphics rasterization, process execution, and artifact I/O remain behind narrow replaceable interfaces; backend-specific types never enter project files or public contracts.

All inputs are canonical typed data. The owning layer must reject raw FFmpeg expressions, executable SVG content, arbitrary paths, network resources, non-finite values, and content exceeding explicit complexity limits before a renderer backend receives them. SVG, fonts, and media resolve only through sanitized managed or content-addressed inputs, preserving deterministic reopen behavior.

#### Fallback policy

A future implementation may fail over only among locally configured graphics backends, selected by a deterministic local priority shared by frame preview, audiovisual range preview, draft preview, and final export. A substitute is conforming only when it supports the complete `EvaluatedScene` and preserves the same scene semantics and documented visual/audio output tolerance. Backend identity remains an implementation detail and does not change project files or public contracts.

A backend that cannot execute every evaluated instruction must not omit, approximate, downgrade, reorder, or remotely acquire resources for unsupported work. If no conforming local backend is ready for the complete scene, readiness or rendering fails with `DEPENDENCY_UNAVAILABLE` before graphics rasterization, FFmpeg execution, or artifact publication. No partial or degraded artifact is published, and preview and export never report degraded rendering as success.

We rejected an FFmpeg-only implementation because it cannot safely provide the required deterministic vector and shaped-text semantics. A full GPU compositor is deferred until the model is stable; backend replacement must not change project files, public operations, ordering, or preview/export tolerances. External pre-rendered overlays remain assets, not a substitute for editable canonical semantics.

### 4. Normative ordering and compositing

The root and every component use a top-left origin, with positive X rightward and positive Y downward. Pixel coordinates are the compatibility default. A future explicit normalized unit resolves against its containing composition dimensions. Time uses integer milliseconds and half-open intervals `[start_ms, end_ms)`.

Within a composition, tracks render from lowest array index to highest. Within a track, visual items sort by ascending explicit `z_index`, then stable item array order, then stable item ID only as a final tie-break for synthesized or otherwise equivalent order inputs. Later entries composite above earlier entries. Duplicate IDs remain invalid. Evaluated audio order must be explicit and must not depend on map iteration.

The normative visual pipeline is:

1. source decode or deterministic rasterization;
2. crop and local clip;
3. item masks in declared order;
4. item effects in declared order;
5. local anchor translation, scale, skew, rotation, and position;
6. ancestor transforms from nearest parent outward;
7. track matte;
8. inherited opacity;
9. blend into the destination in stable layer order.

Ancestor opacity multiplies. Compositing uses premultiplied alpha in linear light, followed by conversion to the configured output color space at the output boundary. These rules are observable and require deterministic golden tolerance coverage when the future evaluated graphics path ships.

We rejected renderer-defined order because iteration and filter construction could change output. We rejected array order alone because explicit local layering is needed without destructive array surgery. We rejected a group-owned child order because it would create a second source of truth.

### 5. Presets compile to persisted primitives

A preset is a pure, bounded `editor-core` compiler from a versioned preset ID and typed finite parameters into the same canonical keyframes, effects, masks, and audio events accepted by low-level operations. A successful preset mutation persists the resolved primitives plus optional provenance containing preset ID, version, and parameters. Provenance is descriptive, not executable; evaluation never needs to run a preset.

Preset application follows optimistic revision checks, commits atomically, and is undoable and redoable. When exposed to agents, it must work both as a standalone operation and inside `timeline_batch_edit` with the existing creation-alias rules. Compilation or batch failure publishes nothing.

We rejected persisting only preset names because output would drift as preset libraries evolve. We rejected MCP-only expansion because desktop and headless callers would not share canonical behavior. We rejected baking presets to pixels because it loses editability.

### 6. Additive schema-version policy

This ADR does not change schema version 6. Each independently shippable future persisted-model milestone performs exactly one additive project-schema bump. Under the project lock, migration must deterministically upgrade current state and every retained undo and redo snapshot, validate the complete result, and publish it as one recoverable atomic generation. Compatible defaults preserve existing simple operations and rendering semantics. Unknown future versions fail closed without downgrade or rewrite.

Public request and response changes remain typed and additive unless a separately approved breaking contract provides an explicit migration path. When clients need to distinguish support, the canonical catalog, every governed consumer, capability/version reporting, and parity evidence change together. Stable error codes are added or changed only through the same governed contract workflow.

We rejected one schema bump for the entire initiative because it couples independently shippable milestones. We rejected a bump for every field because it creates migration churn. We rejected migrating only current state because undo or redo could restore an obsolete schema.

## Ownership, compatibility, and failure rules

`crates/editor-core` remains the canonical owner of models, finite-value and complexity validation, hierarchy and reference integrity, preset compilation, migrations, scene evaluation, revisions, atomic batch rollback, undo/redo, and deterministic reopen behavior. Headless and MCP layers submit typed inputs and translate core results; they do not accept renderer expressions or duplicate validation. Renderers consume evaluated instructions and cannot redefine persisted semantics.

Every later milestone must define exact finite limits, stable missing-reference and invalid-input errors, revision-conflict behavior, standalone and batch alias behavior where applicable, migration fixtures, undo/redo and reopen tests, and shared preview/export tolerance evidence in an approved OpenSpec change.

## Contract fixture impact

Issue #10 is a decision-only documentation change. It adds no runtime or persisted field, headless request or response, MCP tool or resource, capability identifier, provider contract, error code, dependency, migration, or rendering behavior. Therefore `contracts/contract-ownership-v1.json` and all versioned public fixtures remain unchanged.

The first later milestone that introduces any such surface must follow ADR 0002: update the canonical fixture or catalog, every governed Rust/TypeScript/Python consumer, capability/version reporting when clients need it, and parity evidence in the same change.

## Consequences

Motion-graphics features can ship incrementally without replacing the root timeline. Persisted projects remain renderer-independent, preset evolution cannot change reopened output, and preview/export paths share one scene meaning. The cost is a formal evaluation layer, bounded intermediate graphics resources, larger projects when presets expand to primitives, and mandatory atomic migrations for all retained history.

The hybrid CPU path may initially be slower than a GPU compositor. Bounded lazy evaluation, caching, and replaceable interfaces contain that trade-off. Exact graphics backend selection and numeric complexity limits remain decisions for the milestones that implement them; they cannot override the six decisions above.

## Rollout and rollback

This documentation-only change rolls back by reverting the ADR and its living requirements. Later persisted milestones roll forward with another migration or require a compatible reader; an older binary must never silently downgrade a future schema. Each implementing milestone must cite this ADR and provide its own rollout, failure, fixture, and rollback evidence.
