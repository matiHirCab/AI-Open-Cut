## Context

Actual main2a733383 materializes sampled visuals before connecting the existing RenderProgress callback to final execution. extended_visual::prepare computes a known positive sampled frame count and calls its frame producer sequentially. Range cache hits and process completion currently report one before publication; the new reporting must make successful native completion truthful. Living rendering-export already requires bounded monotonic whole-render progress and atomic publication. Main's Windows unit timeouts remain a separate unresolved gate/investigation; PR1987f is independently accepted/all18-green and unmerged.

## Goals / Non-Goals

Expose completed visual work early, keep existing numeric wire/job compatibility, and reserve completion for published validated output. Preserve all guards, clocks, rendering output, errors, revisions, cancellation and native ownership. No ETA, new progress schema/stage field, durable jobs, normalization progress redesign, performance/budget change, CI deadline weakening, pending diagnostic import or desktop/video task access.

## Decisions

1. Core owns work accounting. Thread a borrowed callback through materialization and sampled preparation; report a completed frame only after successful composition, using the existing admitted frame count. Keep frame-preview/test callers as no-op consumers. Alternatives of external file counting or bridge-side inference were rejected because they duplicate private implementation knowledge and cannot prove work completion.
2. Reuse normalized numeric progress. Allocate observed sampled preparation0..0.5 and final encoding0.5..0.99; without sampled work map encoding0..0.99 and emit no invented preparation update; publish one only after rename and artifact validation, including cache hits. Final process values are mapped/clamped with monotonic ownership in the native reporter, preserving downstream job clamping and terminal guards. These fixed work shares are not time estimates. A new stage/count wire object was rejected as unnecessary contract expansion; using final-FFmpeg fractions for the whole job would leave preparation invisible or allow regression at the phase boundary.
3. Preserve all native semantics. Add no frame, sample, output or cache behavior; callbacks observe successful work only. Admission/revision/overwrite errors retain precedence before callbacks. Preparation/final/publish failures keep existing cleanup and canonical errors. Late cancellation updates remain ignored by the existing bridge terminal guard; no new cancellation mechanism is added. Existing artifact-side effects after a successful rename are not redesigned.
4. Manually govern a canonical render-preparation-progress-v1 behavior fixture and a separately owned registry addition. The public headless/MCP catalogs and numeric schemas stay byte-identical. Strict inverse ownership removes only that addition before historical owner projections, preserving captured pins and unrelated-drift/missing-owner controls. An exact-head disposable union with unmerged PR198 preserves its diagnostic fields and review evidence without creating a publishing dependency.
5. Distinguish evidence. Actual renderer tests use deliberately slow/failing frame production, a final-execution marker and publication-fault/cache controls. Typed headless and source/fresh-package MCP tests prove forwarding and terminal/cancellation behavior; any controlled transport fixture is labeled separately from actual native media success. Full native/golden/motion/platform gates remain required, without reference or timeout changes.

## Risks / Trade-offs

- Fixed shares are not proportional wall time: document reported work and avoid ETA claims.
- Long single-frame work has no completed-frame update: report only actual work and do not fabricate temporal heartbeats; nonsampled preparatory work retains existing behavior.
- Callback overhead: use bounded frame work already admitted; no unbounded accumulation or new resource budget.
- Cache/pub/error regressions: verify one only after validated publication and preserve all failure paths.
- Concurrent native-startup lane: exact-head overlap checks, strict independent owners and re-fetch main before publication.
- Current Windows main timing failures: keep investigation/fix isolated and require every final new-head gate; do not claim a proven cause.

## Migration Plan

No persisted migration, protocol major bump, new operation or external dependency. Existing clients accept the same normalized numeric progress and terminal status remains authoritative. Rollback removes only the callback accounting/behavior declaration while retaining upstream domains. No merge or deployment is authorized.

## Open Questions

None blocking the scoped implementation. Main Windows timeout cause is separately unconfirmed and must not be hidden by new-head verification.

Pending startup compatibility: genuine raw process-start witnesses still emit no progress when spawn fails. Renderer facades that completed visual preparation can legitimately retain preparation updates before a later startup failure; every such value must remain below one. Preserve all error/privacy/cleanup/state/pin assertions and govern any affected facade expectation through this approved preparation scenario, without changing startup diagnosis or weakening unrelated tests.
