## Why

Caption provenance persists its source asset identifier, but asset deletion currently guards only current media timeline items. Deletion or garbage collection can therefore leave captions or retained drafts with dangling source references, undermining inspection, regeneration, undo/redo, and draft reopen behavior.

## What Changes

- Centralize discovery and classification of every persisted asset reference in `editor-core`, covering current project state, caption provenance, generated-asset metadata, retained undo/redo snapshots, and durable drafts.
- Define a blocking deletion policy: a current caption provenance reference or a retained draft reference prevents logical asset deletion with the existing stable `ASSET_IN_USE` error; no reference is silently detached.
- Use the same centralized reachability graph for managed-file garbage collection, retaining content reachable from current state, undo/redo history, or any durable draft.
- Validate project and retained-state references deterministically so legacy dangling asset identifiers fail with `ASSET_INTEGRITY_FAILED` and an actionable message.
- Add reopen, undo/redo, draft, deletion, and garbage-collection coverage for caption-source assets and synchronize media-assets and transcription-captions requirements.

### Non-goals

- This change does not add a force-delete or explicit provenance-detachment API.
- This change does not introduce durable asset tombstones or change generated speech provenance shape.
- This change does not add new asset-bearing timeline entities or provider workflows.
- This change does not retain discarded drafts or history snapshots after their existing retention lifecycle ends.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `media-assets`: Expand deletion guards, integrity validation, and managed-file reachability from media timeline items and retained snapshots to every persisted asset reference, including caption provenance and durable drafts.
- `transcription-captions`: Define durable caption-source referential integrity, deletion behavior, retained-state validation, and reopen/undo expectations.

## Impact

- Primary implementation: `crates/editor-core` reference discovery, validation, deletion, persistence, draft enumeration, and garbage collection.
- Tests and fixtures: editor-core project/history/draft migration and persistence tests, plus affected headless and bridge contract or smoke assertions if observable messages are asserted there.
- Compatibility surfaces: persisted project/history/draft JSON and the public `ASSET_IN_USE` and `ASSET_INTEGRITY_FAILED` error contracts. The persisted schema remains additive-compatible and requires no field-shape change.
- Breaking changes: none for valid projects. Legacy persisted state containing dangling asset references will now fail closed deterministically instead of opening with corrupted provenance.
- Dependencies: no new runtime dependencies.
