## Context

Issue #38 delivered schema-22 typed channels, `set_animation_channels`, and shared evaluated-scene sampling for a small active property subset. The runtime curve is currently a closed `hold`/`linear` string enum; the fixture-only motion-graphics vocabulary already names `cubic_bezier` and `spring`. Issue #39 activates those curves. Editor-core owns validation, persistence, and interpolation; headless and the bridge translate typed requests. ADR 0002 governs the canonical fixture and every consumer.

## Goals / Non-Goals

**Goals:** Parameter validation with explicit bounds; deterministic interpolation; additive typed wire shape; atomic migration of current state and history; identical evaluated semantics across render intents; cross-language contract and media evidence.

**Non-Goals:** Activating deferred channels, marker-relative time, loops, legacy `set_keyframes` curve changes, raw expressions, remote resources, or new transport operations.

## Decisions

1. **Extend the existing curve union in place.** Keep string `"hold"` and `"linear"` unchanged. Add closed objects `{type:"cubic_bezier",x1,y1,x2,y2}` and `{type:"spring",mass,stiffness,damping,initialVelocity}`. A parameterized curve is used only on a keyframe with a following segment; the terminal keyframe remains `hold` or `linear`. Extend the canonical channel catalog, Rust model, Zod schemas, headless type, MCP surface fixture, and capability level together. Alternative: a second operation or all-object curve shape; that would add unnecessary workflow and break old payloads.
2. **Validate once in editor-core.** Enforce finite named bounds and a monotone Bézier X control order before persistence, reopening, or render preflight. Zod rejects malformed wire shape but core decides semantic validity. Existing item/channel/keyframe count and property bounds remain. Invalid curves return `INVALID_ARGUMENT` with no revision or history change. Alternative: transport or renderer-specific validation; that risks inconsistent acceptance.
3. **Use fixed numeric rules.** Normalize segment time to [0,1]. For Bézier, run exactly 40 midpoint bisection steps on monotone X, then evaluate Y. For spring, use the analytic damped oscillator solution in normalized time with explicit underdamped, critical, and overdamped branches. Exact endpoints select stored values. Preserve intermediate overshoot for position and gain, then constrain each resulting scalar to the channel's canonical bound, using `0.000001` as the positive scale floor. Reject non-finite results in core. Golden scalar samples use `1e-9` absolute tolerance across supported platforms; the existing render tolerance applies to decoded media. Alternative: timestep integration or platform-native easing calls; both can vary with frame rate or backend.
4. **Carry canonical curve semantics through the evaluated scene.** The current render plan emits time expressions from `EvaluatedKeyframe`, including for range preview and export; it does not rasterize a scene independently at each requested frame. Extend the evaluated curve enum with validated parameters. Editor-core owns both numeric sampling and a compiler from those evaluated curve semantics to generated FFmpeg expressions. Compile Bézier's 40 bisection steps using FFmpeg's `st`/`ld` expression registers, and compile each analytic spring regime from the same equations used by the numeric sampler. The generated expressions contain only core-produced numeric constants and operators; no caller-controlled expression is accepted. Existing channel and keyframe limits bound expression work, and preflight checks finite coefficients before artifact work. Existing legacy keyframe easing stays on its current path. Alternative: independently implemented renderer curves would permit preview/export drift; per-millisecond lookup tables would scale with duration and create large filter graphs.
5. **Advance to project schema 23.** Source schema 22 and below reject parameterized objects, then migrate all supported current and retained undo/redo snapshots under the project lock. The 22-to-23 step only changes the version for old valid records. Revalidate the complete candidate and publish one recoverable generation. Retained drafts follow existing snapshot materialization rules. Existing schema-22 `hold`/`linear` bytes and results stay compatible. Older binaries reject schema 23; rollback is recovery of the prior generation, not downgrade. Alternative: silently accept new objects in schema 22; this would make versioned reopen semantics ambiguous.
6. **Verify contract and render boundaries.** Add boundary/negative fixtures to checked-in catalogs and Rust/TypeScript parity tests. Exercise standalone and batch alias edits, revision conflicts, missing items, rollback, undo/redo, migration, reopen, and preflight failure. Compare fixed scalar samples and output-level frame/range/draft/export samples, including audio gain, with the existing SSIM/RMS/timing tolerances. No provider or desktop-specific interpolation is introduced.

## Risks / Trade-offs

- [Analytic spring may differ by floating-point library] → Pin branch equations and scalar tolerance with fixed fixtures; use exact stored endpoints and one core evaluator.
- [Spring overshoot may exceed property bounds] → Constrain final sampled scalars in core before scene construction; test each active property family.
- [Numeric sampler and generated expressions may diverge] → Compare fixed core scalar samples with FFmpeg expression output at boundaries and intermediate times, then compare decoded render intents. Keep both algorithms in editor-core and use the same validated evaluated-curve records.
- [New object curve shape may drift across languages] → Update canonical fixture, every governed consumer, and the MCP schema digest together; run contract parity.
- [Schema migration may expose malformed retained history] → Validate source and destination snapshots before atomic publication; fault-inject every transaction phase.
- [Rendering may differ by output intent] → Compare semantic plans and decoded media across all four intents on the same immutable scene.

## Migration Plan

Implement fixture and source-version guards first, then model/validation/sampler, then migration and transports, then output tests and docs. Existing schema-22 projects migrate to 23 at open under lock, preserving all old curve results. A failed migration leaves the prior authoritative generation; recovery selects one complete generation. No automatic downgrade is provided.

## Open Questions For Approval

Approval confirms the proposed curve ranges, monotone Bézier restriction, terminal-keyframe rule, fixed 40-step inversion, analytic spring time normalization, property-bound handling of overshoot, `1e-9` scalar tolerance, and schema-23 activation. These choices are observable and will not be silently changed in code.
