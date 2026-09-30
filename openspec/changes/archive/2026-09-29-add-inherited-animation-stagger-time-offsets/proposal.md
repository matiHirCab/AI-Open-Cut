## Why

Issue #42 requires parent motion, child stagger, and repeater copy timing to compose in one deterministic scene. Today active channels reject group and component-instance targets, group descendants have no stagger, and repeater copies explicitly retain the source clock.

## What Changes

- Activate the existing typed visual transform channels on groups and component instances, with inherited transform and opacity evaluation in every render intent.
- Add bounded, optional stagger timing to groups and component instances, and bounded, optional per-copy time offset to repeater descriptors. Define ordering, clock composition, clipping, and held or looped animation behavior for nested occurrences.
- Extend existing typed standalone and batch edits so agents can create and update these settings with aliases, revisions, rollback, undo/redo, and stable failures.
- Add a new persisted schema version and atomic migration of current project state and retained undo/redo history; keep missing fields compatible with old projects.
- Synchronize canonical public fixtures, Rust/TypeScript/Python consumers where governed, capability reporting, documentation, and deterministic preview/export evidence.

No existing field or operation is removed or reinterpreted for old projects. The change is additive on current public requests and responses; the new persisted schema version is intentionally unreadable by older binaries under the existing future-schema rule.

**Non-goals:** activating deferred rotation, crop, path, gradient, or effect channels; marker-relative keyframes; audio inheritance or audio repeater copies; new renderer expressions; preset or motion-blur behavior; a new transport operation where existing typed edits suffice.

## Capabilities

### New Capabilities

- `inherited-animation-timing`: Parent channel inheritance and deterministic staggered occurrence clocks, including nested composition and bounds.

### Modified Capabilities

- `animation-channels`: Activate currently supported visual properties on group and component-instance parents.
- `component-evaluation`: Compose component child stagger with existing local clocks and nested instances.
- `repeaters`: Replace the no-time-offset rule with bounded per-copy timing and expanded preflight.
- `motion-graphics-architecture`: Apply animated ancestor transforms and child clocks through the canonical EvaluatedScene.
- `timeline-editing`: Make new settings available through typed atomic standalone and alias-aware batch edits.
- `project-persistence`: Migrate current state and retained history to the new optional timing fields.
- `motion-graphics-contracts`: Govern additive request/project fields, capability reporting, and cross-language parity.

## Impact

Expected owners are `crates/editor-core` for models, validation, migration, edit semantics, and scene evaluation; `contracts` for canonical fixtures/catalogs; `apps/headless` and `apps/agent-bridge` for typed transport and MCP parity; and documentation plus focused Rust, TypeScript, Python, integration, and packaged-render tests. Existing scene, resource, revision, error, and security limits remain authoritative.
