## Why

Issue #70 requires an inspectable narration-driven example that joins verified speech alignment, scoped markers, visual presets and semantic audio. The individual core capabilities exist on main after PR #164, but the desktop does not expose their combined authored state and no integrated narration fixture proves the workflow.

## What Changes

- Add bounded read-only desktop marker and audio inspection, including scoped timing bindings, saved alignment quality/producer metadata, semantic event provenance, bus routing, DSP, ducking and master normalization settings.
- Add a deterministic synthetic narration recipe binding visual presets and semantic events to EVERY, SINGLE, ONE, rules, Starting with and Venusaur cues through existing typed core operations.
- Prove saved alignment and the existing explicit-alignment marker API agree without inference, and cover atomic failures, aliases, revisions, undo/redo and reopen.
- Verify shared evaluated behavior and native preview/export evidence with independent timing, visual and audio references; document actual desktop workflow evidence.

## Capabilities

### New Capabilities

- `narration-driven-fixture`: Integrated bounded synthetic recipe, alignment compatibility, lifecycle and independent render evidence.
- `desktop-narration-inspection`: Bounded scoped presentation of authoritative markers, speech provenance and audio controls.

### Modified Capabilities

None. Existing domain and transport requirements remain authoritative.

## Impact

Desktop presentation helpers/panels and their tests; test-only core recipe/generator and native tests; versioned fixture recipe under contracts; bridge integration evidence and fixture/workflow documentation. Preserve schema44, protocol1, existing capability/error catalogs, speech-worker requests/responses and the explicit `speech_markers_generate.alignment` field. No breaking contract changes or migration are proposed.

## Non-goals

No new speech inference, timestamp estimation, provider protocol, public transport operation, persisted field, schema migration, audio DSP algorithm or render semantic. Inspection is read-only; existing edit APIs remain the route for authoring. No merge or deployment. Next-issue implementation needs its own approved specification.

## Approval

The user explicitly approved the proposed scope in this session on 2026-10-09: "Approve the proposed scope". This approval covers the proposal, design, both capability specs and tasks. No issue #70 implementation was recovered in the initial checkout, stashes, worktrees, unreachable objects or remote branch inventory.
