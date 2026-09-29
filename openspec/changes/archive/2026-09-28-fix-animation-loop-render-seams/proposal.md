## Why

Issue #41's loop implementation maps phase with checked integer milliseconds in editor-core, but the FFmpeg plan uses floating-point seconds and `mod`. A repeat seam at 13 ms for a channel whose first keyframe is at 3 ms can render the preceding held value instead of the first value; a finite loop beginning at 100 ms can also miss its 700 ms exhaustion boundary. The archived change's native seam test uses a zero start and a binary-friendly period, so it does not detect these supported failures.

## What Changes

- Make backend loop phase selection agree with the canonical item-local integer-millisecond mapping at exact repeat seams, ping-pong turns and round-trip seams, and finite exhaustion boundaries, including nonzero item and first-keyframe starts.
- Add failing native FFmpeg 6 and 8 regressions before the fix, covering held and parameterized reverse segments, finite and infinite counts, and equivalent absolute timestamps across frame, audiovisual range, materialized draft, and export.
- Fill the issue #41 evidence gaps with a durable schema-24 project and nonempty undo/redo channel history migration test and checked-in canonical boundary/malformed loop fixtures consumed by Rust and TypeScript parity tests.
- Preserve existing unlooped output, project schema 25, protocol major 1, stable errors, all request/response shapes, and existing revision, history, and batch semantics. There are no breaking changes.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `rendering-export`: Make exact-seam and finite-exhaustion parity explicit for nonzero starts and held segments across all render intents.

## Impact

`crates/editor-core/src/render_plan.rs` owns the FFmpeg translation; core loop validation and integer sampling remain canonical. Native render tests, the schema-25 migration tests, and canonical loop fixture parity tests are affected. No public contract, schema, migration rule, operation, capability identifier, transport adapter, or dependency edge changes. The migration and parity additions verify existing requirements in `project-persistence` and `motion-graphics-contracts`. No golden baseline update is expected; any fixture change requires separate provenance and review.
