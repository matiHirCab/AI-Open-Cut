## Why

Review of issue #42 reproduced missing staggered repeater copies and edits that publish unsafe derived clocks or inherited raster bounds before rendering rejects them. The archived change's native evidence also omits required nested render scenarios, and existing documentation contradicts the new behavior.

## What Changes

- Compose component-child stagger into a repeater controller's complete generated source branch, including signed offsets, nested clocks, intervals and animation stages.
- Reuse canonical scene projection and bounds preflight before publishing affected edits, batches, drafts and migrated retained snapshots; stop validation before generated-copy materialization.
- Add the reviewed store-to-evaluated_scene preflight dependency to ADR 0003 and its architecture test.
- Add independent nested visual, shifted-media and audio expectations across frame, audiovisual range, draft and export, and replace unsupported verification claims with explicit evidence.
- Synchronize existing animation-channel and repeater documentation with schema-26 timing semantics.

**Non-goals:** New wire fields, capabilities, operations, schema versions, provider behavior, raw expressions, audio stagger/repetition, unrelated renderer redesign, commits, pushes, or modifying the original archived change.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `inherited-animation-timing`: Explicit controller-branch delay composition, pre-publication derived bounds, independent render conformance and accurate current documentation.
- `project-persistence`: Atomic validation of affected final candidates and retained snapshots before any state, draft or staged-resource publication.
- `motion-graphics-architecture`: One renderer-neutral validation-only preflight port consumed by store orchestration and render evaluation.

## Impact

Editor-core scene evaluation, store orchestration, focused core/native tests, ADR 0003 and the architecture test, documentation, and OpenSpec evidence are affected. Headless and MCP remain thin typed adapters and receive public-boundary regression tests. Public request/response shapes, schema 26, capabilities, errors and retryability remain unchanged. Rejection of unsafe inherited states enforces already-approved requirements; missing/zero timing preserves valid older behavior. There is no new persisted migration or public-contract version. The original issue-42 archive remains historical evidence and this follow-up records its corrections.

## Approval

Status: approved. The user explicitly approved the proposal, all three delta specifications, design and tasks with “Aprove” in this conversation on 2026-09-29.
