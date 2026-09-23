## Why

Issue [#38](https://github.com/matiHirCab/AI-Open-Cut/issues/38) asks for typed animation channels and values across motion-graphics properties. Today the editable keyframe model has only position, uniform scale, opacity, and volume, so richer properties cannot be represented safely or addressed consistently by agents.

## What Changes

- Add an additive, closed animation-channel vocabulary and channel-specific value types for transform, crop/source, graphic/path, effect, and audio properties. Define which channels are active for current item kinds and which require a later property/effect milestone.
- Add a typed, atomic channel-edit operation for currently supported targets, with bounded finite values, reference validation, deterministic ordering, and alias-aware batch use. Keep the existing `set_keyframes` operation and its serialized values compatible.
- Persist channel data in the next project schema, migrating current state and retained undo/redo snapshots atomically. Preserve legacy output for projects without new channels and fail closed before rendering for any persisted channel that the current evaluator cannot consume.
- Update canonical contract fixtures, Rust and TypeScript consumers, MCP discovery, capability reporting, tests, and client documentation together.
- Declare the value tag and prospective target kind for every inactive channel, with numeric bounds explicitly deferred until activation. Keep inactive writes rejected. Prove every active visual channel and audio gain through native render output, and keep the Windows worker-crash regression deterministic.

No breaking public contract change is intended. A project written at the new schema is not readable by an older build; the existing future-schema rejection remains the compatibility boundary.

## Capabilities

### New Capabilities

- `animation-channels`: Typed channel identity, target/value compatibility, bounded keyframe storage, and deterministic evaluation semantics.

### Modified Capabilities

- `timeline-editing`: Add atomic, alias-aware channel edits while preserving legacy keyframes.
- `project-persistence`: Add an atomic schema migration for channel storage and retained history.
- `motion-graphics-architecture`: Define activation and fail-closed evaluation boundaries for the typed foundation.
- `editor-core-architecture`: Keep channel records and semantic rules in their existing canonical owners and enforce a complete private-owner inventory.
- `motion-graphics-contracts`: Activate and version the governed channel vocabulary and cross-language evidence.
- `agent-bridge`: Expose typed channel editing and capability discovery consistently across headless and MCP.
- `rendering-export`: Preserve legacy output and deterministic preview/export behavior for supported channel edits.

## Impact

The owning model, validation, persistence, and evaluator are in `crates/editor-core`; serialized channel types belong to `model` and channel validation belongs to `validation` under ADR 0003. `apps/headless` and `apps/agent-bridge` translate the typed operation without duplicating validation. `contracts/` owns shared fixtures and operation/capability catalogs. Documentation and tests must cover standalone and batch operations, invalid and missing references, revision conflicts, rollback, history, migration, reopen, and render parity. The existing Windows worker-crash test needs a readable-PID synchronization fix; worker protocol and production behavior do not change. No new external dependency is expected.

## Non-goals

Cubic Bézier and spring curves (#39), marker timing (#40), loops (#41), inherited/staggered motion (#42), full rotation/crop/path/effect rendering (#43), motion blur (#44), presets (#45–46), and comprehensive split/trim behavior (#47) remain separate issues. This change must not silently accept a channel whose output semantics are unsupported by the current evaluator.
