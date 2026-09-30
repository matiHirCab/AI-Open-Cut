## Context

The original issue-42 change is already archived in this uncommitted worktree. Independent review reproduced a rank-one component-local repeater clipped after its short source has ended, a stagger overflow that commits, and excessive inherited scale that commits. Only one simple native root-group fixture has independent pixels; current guides still deny parent channels and copy offsets. The follow-up must repair these four findings without changing public shapes or schema 26.

## Goals / Non-Goals

**Goals:** Correct complete controller-branch timing; one canonical preflight before affected publication; transactional resource safety; independent nested render/audio evidence; accurate documentation and traceability.

**Non-Goals:** New fields, capabilities or schema migrations; audio stagger or repetition; provider work; unrelated renderer redesign; commits, pushes, or rewriting the original archive.

## Decisions

### 1. Apply controller delay once at the copied source boundary

Read the repeater's ranked delay from the same ScopeTiming data used for its interval. In containing-scope milliseconds, add that delay to copyIndex * timeOffsetMs, then convert once to root time through the scope rate. Apply the combined shift to the complete source-subtree clock, intervals and inherited animation stages at the existing source boundary. Keep the already-shifted controller clip and external ancestors outside that source shift. Ordinary sources, source-local rank delays, transform powers, order and identities remain intact. Use checked integer multiplication/addition before finite affine conversion. Hidden controllers retain rank. Alternative: only shift the controller interval, rejected because it loses short-lived copies; rewriting persisted source starts is rejected because it breaks shared source and history semantics.

### 2. Extract a validation-only mode from canonical scene projection

Refactor the evaluator so a crate-private validation-only entry point shares graph-safe traversal, retained scope expansion, clock/interval projection, transform stages and existing budget checks, and returns before the generated-copy clone/publication loop. Render evaluation continues through materialization after the same preflight succeeds. Do not call the full renderer or duplicate formulas in validation/store. Resolve primitive intrinsic sizes from evaluated facts before projection validation, including rectangle dimensions and composition-sized solids; include recorded media probe dimensions when available. Reuse canonical shape bounds and inherited-scale calculations. Font preparation remains staged and read-only until candidate preflight succeeds. Resource-dependent render measurement retains its existing owner and final checks; the preflight must never claim a missing measurement was verified. Alternative: independent bounds formulas in store, rejected for drift; full render evaluation before every edit, rejected for unnecessary generated-copy allocation and backend/resource work.

### 3. Store coordinates preflight before durable or resource publication

Add only the store-to-evaluated_scene inward dependency and update ADR 0003 plus the architecture test in the same review. Existing typed per-operation validation remains in timeline/validation. After operations and in-memory font staging, validate the final affected edit/batch candidate before publishing fonts, adding history, bumping revision or persisting. Draft create/update/rebase/commit validate the final materialized candidate before any draft or staged-font publication. Affected content means nonzero group/instance stagger, nonzero copy timing, or active group/instance visual channels in any retained scope; ordinary zero-timing scenes keep their existing behavior. Current and retained undo/redo candidates are checked before migration resource/transaction publication. Preserve revision/reference/lock error precedence and per-operation closed-field validation; final derived preflight runs once per operation list, allowing legal intermediate construction. Alternative: preflight after persistence, rejected for observed revision/history corruption of the contract; adding timeline/validation-to-evaluator edges is rejected as unnecessary coupling.

### 4. Verify with independent observations

Add pure clock/interval tests for controller stagger plus signed and nested offsets, and facade tests proving no publication for excessive inherited raster bounds or stagger overflow at all edit/draft/history boundaries. Include final-candidate success and missing/stale/locked/trailing-failure regressions. Use existing injected storage/fault ports and generated-materialization counters.

Extend native inherited fixtures with fractional components, staggered/animated groups, animated children, both parameterized curves, both loop modes and signed repeaters. Expected matrices, scalar phases and selected media samples must be calculated independently of production sampler/evaluator. Decode source-marker frames and a known tone/pulse reference to check visual source sampling and audio trim/retiming/onsets. Compare frame, range, draft and export using existing SSIM >= 0.99, established pixel tolerance and aligned PCM RMS <= 0.0001, with explicit independent oracle checks as well as cross-intent comparisons. Use compatible FFmpeg/FFprobe 7.1.1 and the pinned DejaVu Sans font; required native mode fails if tools are missing. Alternative: output-to-output equality only, rejected because shared defects pass it.

### 5. Preserve contract and historical evidence

Update current guides and cross-links for parent targets, signed offsets, ranked controller delay, half-open clipping and atomic failures. No public catalog/schema drift is intended; run all parity gates and add transport regression evidence without new domain rules. Keep original archived artifacts unchanged. Follow-up verification records which original claims were insufficient and maps each new scenario to inspected tests and fresh check logs. Alternative: retroactively mark the original archive as fully verified, rejected because it obscures the reproduced failures.

## Risks / Trade-offs

- Additional candidate projection cost: run once for final affected candidates, stop before generated-copy materialization and retain existing bounds.
- Double-applied delay across nested scopes: test independent source/controller/ancestor clocks and fractional mappings with signed offsets.
- Staged font or draft writes preceding rejection: split preparation from publication and assert byte/resource inventories around injected failures.
- Incomplete measurement availability: validate recorded and intrinsic facts without backend execution and preserve final render measurement checks; explicitly record evidence limitations.
- Stricter rejection of previously accepted unsafe inherited snapshots: return the required typed failure without rewriting or silently repairing persisted data.

## Migration Plan

No new persisted migration or version is introduced. Existing schema-26 source guards and recoverable migration transactions remain authoritative. Add retained preflight before migrated resources or state are published, and rerun source-version, current/history, fault-phase and no-rewrite tests. Preserve IDs, revisions, media provenance and valid zero-default output. Rollback preserves the previous authoritative generation; no schema downgrade or repair is inferred.

## Open Questions

No implementation decision remains open. Explicit artifact approval was received from the user (“Aprove”) on 2026-09-29.

## Verification refinement

The independent source-marker oracle accounts for the existing backend's final `fps` presentation-timestamp rounding (`near`), after the exact affine clock conversion. It selects the latest source timestamp mapped onto the output grid, rather than incorrectly flooring the continuous source clock. This preserves existing media behavior and does not introduce intermediate clock rounding. The combined fixture retains a cubic Bezier descendant opacity curve, spring parent motion, linear animated parent opacity, and both loop modes; redundant parent Bezier work is removed to bound native verification cost without removing curve coverage.
