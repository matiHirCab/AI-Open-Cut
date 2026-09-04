# Motion-Graphics Architecture Specification

## Purpose

Define the cross-milestone architecture constraints that all motion-graphics model, evaluation, rendering, preset, and schema work must preserve.

## Requirements

### Requirement: Additive root composition model
Future persisted motion-graphics semantics MUST preserve root `Project.tracks` as the project composition root, SHALL add reusable component definitions beside root tracks, and MUST represent hierarchy through scoped, typed, acyclic, explicitly depth-bounded references owned and validated by editor-core.

#### Scenario: Extend an existing project
- **WHEN** a future milestone adds reusable motion-graphics compositions to a schema-v6 project
- **THEN** the migrated project retains its root tracks and existing simple operations while new component definitions and instances are added through compatible typed fields

#### Scenario: Reject an invalid hierarchy
- **WHEN** a proposed parent or component reference is missing, crosses its allowed scope, forms a direct or indirect cycle, or exceeds the configured depth limit
- **THEN** editor-core rejects the entire mutation with a stable typed error and preserves the prior revision and undo/redo history

### Requirement: Canonical evaluated scene
Editor-core MUST evaluate every validated immutable project revision into one renderer-neutral scene semantics that resolves hierarchy, timing, presets, ordering, masks, effects, graphics, and audio before backend execution, and frame preview, audiovisual range preview, draft preview, and final export MUST consume that same semantics.

#### Scenario: Compare preview and export
- **WHEN** preview and final export evaluate the same project revision, dimensions, time, and output settings
- **THEN** their visual and audio results follow identical resolved scene instructions within the documented deterministic tolerance

#### Scenario: Reject evaluation complexity
- **WHEN** scene expansion contains a non-finite value, invalid timing or reference, cycle, or work exceeding an explicit canonical complexity limit
- **THEN** evaluation fails with a stable typed error before graphics rasterization, FFmpeg execution, or artifact publication

### Requirement: Hybrid renderer boundary
The render architecture SHALL retain FFmpeg for media decode, audio processing, final composition, and encoding, SHALL place deterministic complex-vector and shaped-text rasterization behind a replaceable graphics interface, and MUST keep backend-specific expressions and types out of persisted and public contracts. Backend selection MUST follow a deterministic local priority shared by frame preview, audiovisual range preview, draft preview, and final export, and MUST limit failover to a locally available substitute that supports the complete evaluated scene and preserves the same `EvaluatedScene` semantics and documented output tolerance. A backend MUST NOT omit, approximate, downgrade, reorder, or remotely acquire resources for unsupported instructions. When no conforming backend is ready for the complete scene, readiness or rendering MUST fail with `DEPENDENCY_UNAVAILABLE` before graphics rasterization, FFmpeg execution, or artifact publication, and MUST NOT publish a partial or degraded artifact.

#### Scenario: Replace a graphics backend
- **WHEN** a conforming graphics implementation replaces the initial deterministic Rust backend
- **THEN** project files, public operations, evaluated-scene semantics, ordering, and preview/export tolerance contracts remain unchanged

#### Scenario: Reject unsafe renderer input
- **WHEN** input attempts to supply a raw FFmpeg expression, executable SVG content, arbitrary path, network resource, non-finite value, or content exceeding an explicit complexity limit
- **THEN** the canonical owning layer rejects it before it reaches a renderer backend

#### Scenario: Fail over to a conforming local backend
- **WHEN** the preferred graphics backend is unavailable and the next locally configured backend supports every instruction in the complete evaluated scene
- **THEN** the renderer selects that backend by deterministic priority for preview and export while preserving the same scene semantics and documented output tolerance

#### Scenario: Reject degraded fallback
- **WHEN** no locally available backend can execute every instruction in the complete evaluated scene without omission, approximation, downgrade, reordering, or remote resource acquisition
- **THEN** readiness or rendering fails with `DEPENDENCY_UNAVAILABLE` before graphics rasterization, FFmpeg execution, or artifact publication and no partial or degraded artifact is published

### Requirement: Normative coordinate and compositing semantics
Motion-graphics evaluation MUST use a top-left coordinate origin with positive X rightward and positive Y downward, integer-millisecond half-open time intervals, explicit coordinate units, deterministic bottom-to-top layer ordering, the documented transform/mask/effect pipeline, premultiplied alpha, and linear-light compositing before output-color conversion.

#### Scenario: Resolve equal z-index layers
- **WHEN** multiple visual items in one track have the same explicit z-index
- **THEN** evaluation orders them by stable item array order and uses stable item ID only as a final deterministic tie-break for synthesized or otherwise equivalent order inputs

#### Scenario: Evaluate an inherited visual
- **WHEN** a visual has local crop, clip, masks, effects, transform, matte, opacity, blend mode, and ancestor transforms
- **THEN** evaluation applies source rasterization, crop and clip, declared masks, declared effects, local anchor/scale/skew/rotation/position, nearest-to-outer ancestor transforms, matte, inherited opacity, and destination blend in that order

### Requirement: Presets compile to canonical primitives
Every motion-graphics preset MUST be a pure bounded editor-core compilation from a versioned preset identifier and typed finite parameters to canonical primitives, and a successful preset mutation MUST persist the resolved primitives with optional non-authoritative provenance through the same atomic revision and history behavior as low-level edits.

#### Scenario: Reopen after a preset evolves
- **WHEN** a project containing compiled preset primitives is reopened after the matching preset implementation changes or is unavailable
- **THEN** evaluation uses the persisted primitives and produces the same scene without re-running or depending on the preset implementation

#### Scenario: Apply a preset atomically
- **WHEN** an agent-addressable preset is applied standalone or inside `timeline_batch_edit` using valid creation aliases
- **THEN** its primitive expansion commits as one revision and is undoable/redoable, while any compilation or batch failure rolls back the entire mutation

### Requirement: Additive schema milestone policy
Each independently shippable persisted motion-graphics milestone MUST perform one additive project-schema bump, MUST migrate current state and every retained undo/redo snapshot deterministically and atomically under the project lock, and MUST reject unknown future versions without rewriting them.

#### Scenario: Migrate a retained generation
- **WHEN** a supported older project with non-empty undo and redo history is opened by a motion-graphics milestone
- **THEN** current state and every retained snapshot migrate, validate, and publish as one recoverable generation before any migrated state is returned

#### Scenario: Distinguish additive client support
- **WHEN** a milestone adds a client-addressable motion-graphics field, operation, or behavior that clients need to detect
- **THEN** its typed contract, canonical fixture, governed consumers, capability/version report, and parity evidence are updated together without changing existing simple-operation meaning

#### Scenario: Reject a future project
- **WHEN** a project declares a schema version newer than the running editor supports
- **THEN** open fails closed with the stable compatibility behavior and does not downgrade, partially migrate, or rewrite current state or retained history

### Requirement: Concrete flat EvaluatedScene representation
Editor-core MUST provide an owned renderer-neutral `EvaluatedScene` whose header contains explicit canvas dimensions, frame rate, and integer-millisecond project duration; whose logical media resources contain no filesystem path or network locator; and whose closed typed instructions represent the current flat media, text, solid-color, rectangle, and associated audio behavior without containing persisted-record references, raw FFmpeg expressions, executable SVG, backend types, prepared files, or artifact destinations.

#### Scenario: Evaluate current flat items
- **WHEN** editor-core evaluates a validated project containing visible media, text, solid-color, and rectangle items
- **THEN** it produces owned typed resource and layer instructions containing the timing, transform, animation, styling, media, and audio facts needed to reproduce the existing output semantics

#### Scenario: Keep evaluation renderer-neutral
- **WHEN** a caller inspects any evaluated resource or instruction
- **THEN** it contains logical identifiers and typed finite values and contains no arbitrary path, network resource, renderer expression, executable content, prepared temporary file, backend command, or artifact destination

#### Scenario: Preserve the project snapshot
- **WHEN** evaluation succeeds or fails for an immutable project revision
- **THEN** the source project, revision, current state, retained undo/redo history, and persisted schema remain unchanged and repeated evaluation of the same inputs produces an equal scene or equal typed error

### Requirement: Deterministic flat scene timing and ordering
Flat scene evaluation MUST use integer-millisecond half-open intervals, MUST preserve the current top-left pixel coordinate behavior, MUST omit hidden tracks and hidden items, and MUST order current visual instructions bottom-to-top by ascending track array index and then ascending item array index. Logical resource requests and audio instructions MUST use deterministic first-use order. Transition endpoint facts MUST preserve transition declaration order, MUST emit an `Out` fact for a source endpoint and an `In` fact for a target endpoint including both facts when the endpoints are the same item, and transition, mute, fade, automation, and ducking facts MUST resolve without consulting renderer-specific behavior. Merged voiceover activity intervals MUST be stored once at scene scope in deterministic order, and each ducked audio layer MUST retain only its gain, attack, and release settings while consuming that shared table.

#### Scenario: Order equal flat layers
- **WHEN** visible flat items occur across multiple tracks and item positions without a future explicit z-index
- **THEN** evaluated visual instructions are ordered by track array index and item array index and retain a stable item identifier as identity without reordering by hash-map or filesystem iteration

#### Scenario: Exclude hidden content
- **WHEN** a track or item is hidden, or an audio-bearing track or media item is muted
- **THEN** the hidden visual contributes no visual instruction and the muted source contributes no audio instruction while other visible instructions retain their relative order

#### Scenario: Resolve flat audio behavior
- **WHEN** visible media has audio, volume keyframes, fades, track roles, or ducking settings
- **THEN** evaluation produces deterministic audio instructions and a single scene-level table of resolved voiceover intervals using the same timing and gain semantics as the current renderer

#### Scenario: Share voiceover intervals across ducked layers
- **WHEN** multiple music layers enable ducking while voiceover activity exists
- **THEN** every ducked layer retains its own gain, attack, and release settings without owning a cloned interval vector, and all use the one scene-level merged interval table

#### Scenario: Index transition endpoint facts deterministically
- **WHEN** visible items reference transitions including a transition whose source and target are the same item
- **THEN** evaluation indexes the transition list once, preserves declaration order for each endpoint, and emits both the source `Out` and target `In` facts for that item

### Requirement: Bounded and fail-closed scene evaluation
Editor-core MUST reject evaluated values that are non-finite, intervals that are not valid non-empty half-open ranges, missing logical asset references, non-image media source intervals whose checked end overflows or exceeds a known asset duration, and work exceeding the named inclusive limits of 4,096 visual layers, 4,096 logical media resources, 4,096 audio layers, 4,096 emitted transition endpoint facts, 10,000 keyframes per property channel, or 10,000 positive voiceover activity ranges before merging. Evaluation MUST return `ASSET_NOT_FOUND` for a missing media asset and `INVALID_ARGUMENT` for invalid values, timing, or complexity. Referenced assets MUST be checked before source timing and complexity rejection; source timing and all named complexity limits MUST be checked before output collection allocation, scene-level voiceover interval allocation, path resolution, graphics rasterization, FFmpeg execution, or artifact publication.

#### Scenario: Reject a missing media reference
- **WHEN** a visible media item references an asset absent from the evaluated project revision, including when its source timing is invalid or other evaluated work exceeds a complexity limit
- **THEN** evaluation fails with `ASSET_NOT_FOUND`, produces no partial scene, performs no renderer or filesystem I/O, and leaves project state and history unchanged

#### Scenario: Reject a non-finite evaluated value
- **WHEN** any transform, opacity, audio gain, automation value, or derived evaluated numeric value is non-finite
- **THEN** evaluation fails with `INVALID_ARGUMENT` before producing a scene or invoking any downstream adapter

#### Scenario: Enforce each scene complexity boundary
- **WHEN** evaluation encounters exactly a named inclusive limit it succeeds, and when it encounters one additional visual layer, logical resource, audio layer, emitted transition endpoint fact, per-property-channel keyframe, or positive pre-merge voiceover activity range
- **THEN** the overflow fails with `INVALID_ARGUMENT` before allocating or publishing a partial downstream render result

#### Scenario: Preflight voiceover activity
- **WHEN** voiceover volume automation produces 10,000 positive activity ranges before merging
- **THEN** evaluation succeeds and creates one deterministically merged scene-level interval table
- **WHEN** it produces 10,001 positive activity ranges before merging
- **THEN** evaluation fails with `INVALID_ARGUMENT` before allocating the scene-level interval table

#### Scenario: Validate non-image source timing
- **WHEN** a video or audio source end is representable and equals its known asset duration, or equals `u64::MAX` when asset duration is unknown
- **THEN** evaluation accepts the source interval
- **WHEN** the checked source end overflows or exceeds a known asset duration by any amount
- **THEN** evaluation fails with `INVALID_ARGUMENT` before complexity checks or output allocation

#### Scenario: Preserve image source-offset behavior
- **WHEN** an image media item has a source offset that would overflow if added to its timeline duration
- **THEN** evaluation preserves current image behavior and does not reject it on source timing because image rendering ignores the offset

#### Scenario: Preflight voiceover keyframes
- **WHEN** a voiceover media item contains more than 10,000 keyframes in one property channel
- **THEN** evaluation fails with `INVALID_ARGUMENT` before deriving voiceover intervals or allocating an output keyframe collection

#### Scenario: Reject invalid evaluated timing
- **WHEN** an evaluated flat instruction would have an empty, reversed, or overflowing integer-millisecond interval
- **THEN** evaluation fails with `INVALID_ARGUMENT` instead of saturating or passing ambiguous timing to a renderer

### Requirement: Process-local scene resource bindings
Editor-core MUST return evaluated scene semantics with a separate process-local resource-binding collection that maps logical media resources to project-relative media requests and configured logical font resources to their original path and family selection request. Raw requested paths MUST remain outside `EvaluatedScene`, resolved filesystem paths MUST remain owned by the existing path-safe preparation layer, and neither collection SHALL become a public or persisted contract in this milestone.

#### Scenario: Preserve configured font selection
- **WHEN** text configures a font path, a font family, or both
- **THEN** evaluation assigns a logical font resource identifier and records the original selection in the separate resource-binding collection without placing a path or URL in `EvaluatedScene`

#### Scenario: Keep default font renderer-configured
- **WHEN** text configures neither a font path nor a font family
- **THEN** evaluation emits no font resource binding and downstream preparation remains responsible for selecting its configured default font

#### Scenario: Preserve path-safe preparation
- **WHEN** a later render-routing milestone consumes a requested font path from the resource-binding collection
- **THEN** the request is canonicalized and checked against configured font roots before any resolved filesystem path reaches a backend

### Requirement: EvaluatedScene foundation remains non-public and non-persisted
`EvaluatedScene` MUST remain an editor-core process-local derivation and MUST NOT itself persist state or add a public request, response, operation, MCP, provider, or stable-error contract. Separately approved persisted-schema and render-routing milestones MAY migrate a project before evaluation or advertise their own capability without making `EvaluatedScene` a persisted or public model.

#### Scenario: Evaluation does not mutate persisted state
- **WHEN** an already opened project revision is evaluated
- **THEN** evaluation itself does not rewrite the project, change its revision, or modify retained undo/redo history; any supported older-schema migration occurs during project opening under the separately specified persistence workflow

#### Scenario: Keep the scene model private
- **WHEN** clients inspect headless operations, MCP tools, provider contracts, or stable errors
- **THEN** they observe no serialized or directly addressable `EvaluatedScene` model
### Requirement: Production EvaluatedScene consumption
The production renderer MUST consume the complete owned `EvaluatedScene` and its structurally separate process-local resource bindings, MUST keep raw requested and resolved filesystem paths outside `EvaluatedScene`, and MUST derive backend syntax, input indexes, prepared resources, output clipping, and artifact destinations only after canonical evaluation succeeds.

#### Scenario: Prepare a logical media resource
- **WHEN** an evaluated visual or audio layer references a logical asset identifier
- **THEN** path-safe preparation resolves that identifier through the separate binding collection without consulting persisted asset or timeline records and without adding a path to `EvaluatedScene`

#### Scenario: Prepare an evaluated text layer
- **WHEN** an evaluated text instruction references a logical font resource identifier
- **THEN** path-safe preparation resolves its requested path or family through the separate font binding and preserves the evaluated text semantics without consulting the persisted text item

#### Scenario: Reject an inconsistent binding envelope
- **WHEN** an evaluated instruction references a logical media or font resource absent from its binding collection
- **THEN** rendering fails deterministically before backend execution or artifact publication and does not attempt to reconstruct the reference from project records

### Requirement: Intent-independent scene semantics
Frame, range, draft, and export output intents MUST clip and encode one common evaluated scene without changing its resolved coordinate system, half-open timing, layer order, transforms, animation, transition, text, media, or audio facts.

#### Scenario: Select a frame and a range
- **WHEN** frame and range intents select the same timestamp from an equivalent evaluated scene
- **THEN** both consume the same ordered active instructions and differ only in intent-required seeking, duration, audio inclusion, and encoding behavior

#### Scenario: Preserve repeated logical assets
- **WHEN** several evaluated layers reference one logical media asset with different source intervals or audio settings
- **THEN** backend preparation creates deterministic input instances that preserve each layer's evaluated timing and settings without duplicating or reordering canonical scene semantics

### Requirement: Common visual properties activate as an isolated persisted milestone
Schema version 7 MUST establish common visual-property ownership for current transform and visibility state without activating the fixture-only Transform2D, layer, component, slot, marker, curve, mask, effect, or audio-event concepts. `contracts/motion-graphics-v1.json` MUST remain fixture-only, and no new public operation, capability identifier, provider surface, renderer expression, resource locator, or stable error SHALL be introduced by this milestone.

#### Scenario: Inspect milestone boundaries
- **WHEN** a schema-v7 project, public operation catalog, capability report, and motion-graphics fixture catalog are inspected
- **THEN** common transform and visibility ownership is present, existing operations retain their meanings, the motion-graphics catalog remains fixture-only, and all later concepts remain inactive

#### Scenario: Defer Transform2D behavior
- **WHEN** a common visual property is evaluated in this milestone
- **THEN** it uses the schema-v6 position, uniform scale, and opacity semantics with no units, anchor transform, independent scale, rotation, skew, or new transform ordering
