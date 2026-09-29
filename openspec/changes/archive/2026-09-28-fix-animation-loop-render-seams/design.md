## Context

The archived issue #41 change defines loop phase in integer item-local milliseconds. `animation::map_loop_time` implements that rule, while `render_plan::looped_time_expression` turns evaluated keyframes into FFmpeg expressions using decimal seconds and floating-point `mod`. At a 3-to-13 ms repeat interval, floating-point subtraction leaves the mapped time just before the exact seam, so a held segment produces the old value; a 100-to-300 ms interval with three finite repeats also misses the exact 700 ms exhaustion boundary. The current native seam test uses zero first time and 500 ms periods, which are exactly representable in binary. FFmpeg 6 and 8 both reproduce these mismatches. ADR 0003 puts backend translation in `render_plan`, and ADR 0004 requires one evaluated scene across render intents.

## Goals / Non-Goals

**Goals:** Match canonical loop phase at exact seams, turns, finite exhaustion, and adjacent representable render samples for nonzero starts; keep visual/audio frame, range, draft, and export parity; add native and durable migration/contract evidence missing from issue #41.

**Non-Goals:** Change loop wire shape, curve equations, item-local time semantics, schema 25, transport operations, capability reporting, user UI, media playback loops, or existing unlooped rendering. No new dependency edge or golden baseline change is planned.

## Decisions

1. **Quantize backend timestamps to integer milliseconds before phase arithmetic.** Translate the evaluated scene's closed loop record in `render_plan` using a shared millisecond phase expression for repeat and ping-pong. Use bounded arithmetic and compare finite exhaustion in the same unit. This follows core's integer phase rule and prevents decimal-second modulo from picking the prior cycle at a seam. Keep the curve sampler and exact endpoint handling after phase mapping. Alternative: add a floating epsilon around each seam; rejected because the needed tolerance varies with timestamp and could move adjacent valid samples to the wrong cycle.
2. **Test the translated expression and decoded outputs.** First add a failing native scalar comparison on FFmpeg 6 and 8 with keyframes at 3, 8, and 13 ms, a held final segment, and samples at 12/13/14 ms; include finite exhaustion at 700 ms for a 100-to-300 ms loop, ping-pong turns, round-trip seams, and reflected Bézier/spring. Then compare the same absolute timestamps in frame, later-start range, draft, and export, including audio gain. Alternative: rely only on core phase tests; rejected because they cannot detect FFmpeg expression rounding.
3. **Close evidence gaps without changing contracts.** Add a serialized schema-24 project with channels and nonempty undo/redo snapshots to the existing recoverable migration tests; assert schema-25 reopen and output equivalence. Add canonical maximum-count and malformed loop examples to the channel fixture, and make Rust and TypeScript tests consume them. Existing request/response, persisted layout, and stable error behavior remain unchanged. Alternative: introduce a new contract version or migration; rejected because this is a backend correction and added evidence for existing rules.
4. **Keep the reviewed unlooped semantic plan stable.** The golden harness formats evaluated scenes with `Debug`, so the new optional loop field would otherwise add `loop: None` to every legacy keyframe. Omit that field from the debug representation only when absent, and retain it for looped keyframes. This preserves the reviewed unlooped golden without changing the evaluated scene or regenerating references.

## Risks / Trade-offs

- **Timestamp quantization near a noninteger frame time** → Compare actual frame/range/export timestamps and preserve the existing timing tolerance; use the same quantization for every render intent.
- **Very large composition times exceed exact FFmpeg floating integer precision** → Use item-relative arithmetic where possible and retain preflight failure for any representability limit rather than silently render a wrong phase.
- **Native tests depend on FFmpeg version and environment** → Run targeted regression with supported FFmpeg 6 and 8 binaries, then the required repository gates; keep full logs and report unavailable platforms separately.
- **Existing user changes are uncommitted** → Edit only scoped files and new OpenSpec artifacts, preserving the issue #41 working tree.

## Migration Plan

No deployed data migration or public API rollout is needed. After approval, add failing regression tests and fixture cases, implement the `render_plan` correction, run affected tests and the required repository gates, verify conformance, sync the delta, and archive the change. Reverting the backend correction restores the prior renderer but leaves schema 25 and all persisted data readable.

## Open Questions

Approval of this design authorizes the millisecond-quantized backend phase and the listed regression scope. No contract or schema decision remains open.
