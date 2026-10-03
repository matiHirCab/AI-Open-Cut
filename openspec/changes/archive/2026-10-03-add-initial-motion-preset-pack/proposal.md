## Why

Issue46 requests five reusable motion presets after issue45 establishes the versioned compiler. The roadmap's creative gate calls for slam overshoot/shake/flash/motion blur and seamless looping scan/radar behavior; the existing scalar seed leaves agents assembling these primitives manually.

## What Changes

- Add impact_slam@1, slide_left@1, scan@1, pulse@1 and radar_expand@1 as pure bounded core expansion, applied to existing compatible visual items.
- Define exact explicit endpoint fields and integer phase times; impact combines five channels plus enabled existing MotionBlur, scan repeats a sweep/return, pulse can repeat, and radar expands/fades/resets invisibly before repeating. The explicitly approved flash is an opacity dip-and-return.
- Extend the existing preset operation's strict parameter union, atomic multi-channel reject/replace handling and optional generated blur assignment. Report initial_motion_preset_pack_v1. Persist authoritative channels/blur and descriptive complete per-property sources.
- **BREAKING persisted compatibility:** schema30 for the new tagged source shapes. Atomically migrate supported current/component/retained undo/redo generations without changing old scalar channels/sources. Old schema29 builds reject30; rollback requires complete backup. Existing scalar requests and output meaning remain unchanged.

## Capabilities

### New Capabilities

- `initial-motion-preset-pack`: exact creative pack expansion, phase/loop/blur/parameter/collision semantics.

### Modified Capabilities

- `animation-presets`: additive catalog/strict parameter union, multi-channel output, descriptive pack provenance/lifecycle and bounded budgets.
- `project-persistence`: schema30 migration, complete retained generation compatibility and recovery.
- `agent-bridge`: typed pack request/status/MCP parity and capability reporting.

## Impact

Core model/compiler/validation/migration/lifecycle tests; canonical preset, channel/schema, headless and MCP fixtures and all governed consumers; bridge typed union/Zod/tool docs and source/packaged workflows; headless protocol/capabilities; docs and CODEOWNER review by @matiHirCab. Existing MotionBlur field and channel/loop evaluator remain authoritative. No new renderer, provider, dependency edge or generated geometry.

## Non-goals

New shapes/effects or rings created by the preset; bright tint flash; new interpolation, loops or shutter sampling algorithms; audio presets; draft preset intents; direct component-definition authoring; automatic split/trim retiming or changing existing accepted/failing split behavior; unrelated CI/tolerance changes. This issue covers the pack's creative behavior, not every remaining milestone/initiative gate.

## Approval and prerequisite

Design explicitly approved by user message Sentinel_1762337a6698819190f92bfa17a3420a at2026-10-02 21:19:22UTC; attributable evidence is in approval.md. Implementation is authorized from corrected prerequisite0dd191d2240718e9550712f412fa03812f3f342c. Required prerequisite checks, independent review, archive and protected postarchive gates passed. Its origin is unmerged draft PR134 head67d05dbfde533521c98ae163b83b6e7bcb3ac426, main51cd085253402bd0a2f1798f7a4bba418f83bb5d. Issue45 remains open/unmerged. Contract CODEOWNER acceptance is required before completion; parent scope confirmation is required before publication. Do not merge the prerequisite or claim the dependency closed.

## Approved implementation checkpoint

Explicit approval and its attributable transcript are in approval.md. The prerequisite is corrected0dd191d2; its required checks and postarchive gates passed. Issue45 remains unmerged, designated CODEOWNER acceptance and separate publication confirmation remain required. Historical planning-only/base67d/unresolved finding notes above describe the proposal origin, not current status.
