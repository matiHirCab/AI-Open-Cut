## ADDED Requirements

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
The render architecture SHALL retain FFmpeg for media decode, audio processing, final composition, and encoding, SHALL place deterministic complex-vector and shaped-text rasterization behind a replaceable graphics interface, and MUST keep backend-specific expressions and types out of persisted and public contracts.

#### Scenario: Replace a graphics backend
- **WHEN** a conforming graphics implementation replaces the initial deterministic Rust backend
- **THEN** project files, public operations, evaluated-scene semantics, ordering, and preview/export tolerance contracts remain unchanged

#### Scenario: Reject unsafe renderer input
- **WHEN** input attempts to supply a raw FFmpeg expression, executable SVG content, arbitrary path, network resource, non-finite value, or content exceeding an explicit complexity limit
- **THEN** the canonical owning layer rejects it before it reaches a renderer backend

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
