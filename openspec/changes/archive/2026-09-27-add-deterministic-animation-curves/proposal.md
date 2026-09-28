## Why

Issue [#39](https://github.com/matiHirCab/AI-Open-Cut/issues/39) requires parameter-validated cubic Bézier and spring interpolation for typed animation channels. The current runtime accepts only `hold` and `linear`, although the motion-graphics fixture catalog reserves both later curves. Issue #38, which established typed channels, is closed.

## What Changes

- Add bounded, typed cubic Bézier and spring curves to the existing `set_animation_channels` operation and its batch/MCP forms. Existing `hold` and `linear` payloads remain valid.
- Define exact parameter, boundary, overshoot, numeric tolerance, and failure behavior. Sample both curves once in editor-core so frame preview, audiovisual range preview, draft preview, and export share the result.
- Version and migrate persisted current state and retained undo/redo history, including deterministic reopen and future-version rejection.
- Update canonical contracts, capability reporting, cross-language parity fixtures, documentation, and automated success/failure evidence.

**Non-goals:** Activating deferred channel properties, marker timing, looping, changing legacy `set_keyframes` easings, or adding renderer expressions or resource inputs. This is additive for existing request and error contracts; the new persisted project version requires migration and remains unreadable by older binaries.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `animation-channels`: Activate typed cubic Bézier and spring curves with deterministic sampling and validation.
- `motion-graphics-contracts`: Govern the additive curve wire variants, limits, capability, and Rust/TypeScript parity.
- `project-persistence`: Migrate current and retained history to the new schema while preserving transactional recovery.
- `rendering-export`: Require equivalent curve samples across every evaluated-scene output intent.

## Impact

Editor-core channel model, validation, sampling, migrations, evaluated scene, and tests; headless and MCP typed input/capability surfaces; `contracts/animation-channels-v1.json`, relevant motion-graphics fixture and MCP catalogs, and governed Rust/TypeScript consumers; animation and rendering documentation. Review of changed canonical contracts and consumers belongs to `@matiHirCab` under ADR 0002 and `contracts/contract-ownership-v1.json`.
