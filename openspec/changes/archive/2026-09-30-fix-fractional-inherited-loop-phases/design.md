## Context

The independent review reproduced a 0.995-rate component containing a looped parent with position keys (0,0), (99,100), (100,0). At root 100 ms, mapped time 99.5 ms requires translation 50, but nearest-millisecond loop mapping wraps early to zero in all four intents. The existing integer sampler and native 100-ms samples miss that discrepancy. The older loop requirement explicitly says integer local time, whereas inherited timing requires unrounded affine clocks. Repeaters also retain an obsolete zero-offset scenario that excludes ranked controller delay.

## Goals / Non-Goals

**Goals:** Exact fractional mapped visual phase before interpolation; independent seam/turn/exhaustion evidence; coherent loop/repeater requirements; integer-phase compatibility.

**Non-Goals:** Audio sampling/tempo changes, media output-grid resampling changes, wire/schema/capability changes, new dependency edges, resource rules, golden baseline or tolerance changes, commits or pushes.

## Decisions

1. **Distinguish stored timestamps and evaluated time.** Persisted keyframe/start/stagger/offset timestamps remain bounded integers. Mapped visual occurrence and parent-stage times remain finite f64 values. Resolve cycle/turn/exhaustion using that exact mapped time before curve interpolation. Keep the existing integer public/core sampling entry point and its checked arithmetic for integer clients. Correct affected fractional visual results rather than redefining stored timestamps. Alternative: round only after affine composition, rejected because it still wraps or completes a loop early.

2. **Keep precision selection explicit and local.** Animation owns canonical time/curve semantics; render planning compiles evaluated clocks through its existing bounded expression path. Use an explicit private precision mode or dedicated fractional loop helper for visual occurrence/parent-stage expression calls; preserve the existing unretimed integer sampling path and independent audio. Do not detect semantics by parsing expression strings or introduce transport-side samplers. Share curve compilation, clamping and endpoint rules. No new dependency edge is required. Alternative: globally remove rounding from every scalar expression, rejected because it would alter unrelated legacy/audio behavior.

3. **Test independent expected values.** Add scalar/backend cases at fractional interiors, before/at/after repeat seams and ping-pong turns, nonzero first-key offsets and finite exhaustion, including nested component/stagger/signed-copy clocks. Calculate expected phases and simple linear or analytic curve values independently of production helpers. Add a small native four-intent reproduction at output-frame-aligned root times with 0.995 rate; expected child pixels are 60..69 at root 100 ms, not 10..19. Include materialized draft output, and representative nested signed/stagger cases. Retain existing integer, zero-offset, audio, curve and loop regressions. Compatible FFmpeg/FFprobe 7.1.1 and pinned font are mandatory in required native mode. Existing SSIM/pixel/PCM tolerances and canonical references remain unchanged. Alternative: output-to-output equality only, rejected because all four already share the defect.

4. **Synchronize authoritative wording.** Amend the complete loop-evaluation requirement to distinguish integer stored offsets and fractional derived visual phase. Amend the complete repeater-copy requirement and its interval scenario to include controller ranked delay plus signed offsets. Clarify current timing/loop guides and the persisted-versus-derived clock sentence in ADR 0004; ownership and dependency maps remain unchanged. Preserve both previous archives and add this change's own requirement-to-test report. Alternative: rewriting historical archives, rejected because it would erase reviewed evidence.

## Risks / Trade-offs

- Floating arithmetic at exact seams: use checked finite affine facts and test exact representable boundaries as well as nearby fractional values; do not add coarse millisecond snapping.
- Legacy output drift: explicitly preserve integer-time sampler behavior and unrelated unretimed/audio paths; run existing golden/semantic-plan and loop conformance checks.
- Native cost: use a small independent fixture plus focused scalar cases; reuse unchanged valid check evidence only when permitted, and run all mandatory final gates.
- Specification conflict: copy complete modified requirement blocks, preserve existing scenarios and add precise compatibility scenarios before synchronization.

## Migration Plan

No persisted migration, schema bump or public surface change. Correct fractional visual sampling for existing schema-26 compositions. Integer-phase output, revisions, history, resource safety, stable errors and independent audio remain intact. Regression rollback means reverting only this follow-up's code/spec changes, never discarding unrelated worktree changes.

## Open Questions

The user explicitly approved proposal, all three delta specs, design and tasks with “Approve” on 2026-09-29, before implementation.
