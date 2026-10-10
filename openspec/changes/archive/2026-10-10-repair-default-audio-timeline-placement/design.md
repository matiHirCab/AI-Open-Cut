## Context

Base main16b41e07476b18cb15f4631be6229b2a4fafcfb8 includes the reviewed cleanup repair. Ordinary append_audio_layer currently shifts PTS before amix; retained/component/active-bus branches physically delay samples. Fresh exact-main exports independently show absent-DSP audio at the wrong time. Core owns the repair under ADR0003/0004; outer transports remain unchanged.

## Goals / Non-Goals

Honor authored sample clocks identically with absent/explicit-identity/nonidentity DSP and neutral routing across preview/export. Preserve local source trimming, gains, automation, fades, component clocks, ducking, immutable state and publication lifecycle. Exclude new authoring features, unrelated refactors, public/persisted contract changes, provider changes and merge/deployment.

## Decisions

1. Place every ordinary layer through physical silence before sequential mixing, retaining canonical evaluated span and existing component mapping. Timestamp-only placement is rejected by the actual reproduction; forcing nonidentity DSP is an invalid workaround that changes gain/readiness. Preserve source-local volume/fade evaluation before placement and global role-ducking evaluation after placement. Avoid introducing a second evaluator or enabling DSP merely for placement.
2. Existing retained/instance placement stays canonical and must be covered against double delay. Maintain source-in selection in existing prepared input binding and existing requested-range cropping. A zero-start ordinary layer may retain its old equivalent graph where useful to minimize unrelated reference changes, provided native sample clocks remain correct.
3. Inspect base readiness for physical-delay filter support. If required coverage is absent, admit only the necessary existing typed dependency readiness check before side effects; no degraded timestamp fallback or public-contract change. Add a missing-filter negative if readiness changes.
4. Native tests must render actual H264/AAC and decode48k float PCM. Use independently specified frequencies/envelopes and silence windows derived from literal authored times, not renderer graphs/output-derived oracles. Compare exact semantic placement with conservative AAC onset/window bounds (one output frame maximum timing displacement); preview/export shared-interval RMS uses the established0.0001 bound with appropriate fixed aligned fixture. Gain changes are amplitude-normalized only for timing comparisons, never asserted sample-equivalent. Exercise zero start, delayed clip, consecutive/gapped/overlapping clips, source trim, preview starting inside a clip/gap, roles/explicit bus routes, neutral and active DSP, local fade/automation and existing component/retained clocks.
5. Existing historical witnesses remain intact. First run verify-only golden/native suites and identify actual affected references. If a selected reference is impacted, present a separate concrete old/new PCM/graph diff, exact manifests/pointers/hashes and independent timing oracle for reviewer approval before using its governed atomic update path. No arbitrary tolerance relaxation, graph-only proof or coordinated oracle rewrite is permitted.
6. Scoped lifecycle tests verify immutable rendering, edit undo/redo/reopen timing, stale/missing failures without publication, failed/cancelled output cleanup and successful later retry. Reuse existing canonical tests where they actually cover these boundaries; add only missing changed-scenario evidence.

## Risks / Trade-offs

- Correcting an inherited bug can invalidate legacy graph/PCM witnesses → explicitly reviewed narrow exception in delta specs, preserve historical evidence, separate concrete golden approval before installation.
- Physical silence can change role-ducking clock, source-local automation or duplicate existing delay → independent PCM windows/fade/trim and retained/component regressions.
- AAC boundary smearing can obscure exact sample onset → validate conservative interior silence/activity and frequency/source identity as well as timing; preserve existing parity tolerances.
- Native cross-platform results differ numerically → existing portable tolerance policy remains unchanged; require genuine platform CI and fail on absent prerequisites.
- Exact Windows Brock project unavailable → report limitation; shared canonical timing is independently tested without claiming that project was exercised.

## Migration Plan

No schema/protocol migration or catalog update. Normal re-rendering uses corrected sample placement; old exported files remain historical. Verify all gates, synchronize/archive approved deltas, publish a separate draft PR from latest main and await exact-final-head required CI. Rollback is a scoped code/reference/spec revert, with no persisted-data downgrade. No merge/deploy.

## Acceptance for reviewer approval

1. The independent500ms/2000ms ordinary clip repro contains silence at0.1–0.3s and the expected tone at2.1–2.3s under absent, explicit identity and gainDb=-1 DSP, with documented actual exports/PCM.
2. Multiple nonoverlapping/gapped/overlapping clips, nonzero source trim and local fades/automation preserve literal timeline/source expectations and routing semantics without concatenation or double delay.
3. Audiovisual previews cropped inside clips/gaps and exports agree on selected authored content within existing parity/timing bounds; retained/component clocks remain correct.
4. Default/explicit stem/master routes and absent/identity/active DSP differ only by intended processing, not clip placement; existing role ducking stays globally clocked.
5. Stale/missing/dependency failures, render failure/cancellation, retry and undo/redo/reopen retain typed errors, immutable state and publication/cleanup guarantees.
6. Public/persisted contracts and previous cleanup fixes are preserved; changed golden references require separate exact-evidence approval and historical witnesses are retained.
7. Mandatory local checks, conformance verification, synchronized archived specs, protected validation and all exact-final-head required CI succeed before claiming completion; limitations and any failures are reported.

## Open Questions

Explicit reviewer approval was received before implementation and is recorded in approval.md. Any concrete impacted golden generation requires subsequent reviewer approval before installation. Any public-contract necessity or unrelated failure needing scope expansion returns for approval.
