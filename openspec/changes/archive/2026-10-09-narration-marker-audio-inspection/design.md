## Context

Issue #70 depends on existing marker, event, preset, compositing, ducking and normalization features. PR #164 is merged into main at c0031b60; this branch starts there. Living speech-alignment-provenance and speech-alignment-markers specs already govern saved alignment and optional explicit alignment. Desktop currently lists root timeline items and visual/animation controls, but does not present narration cues or comprehensive audio state. No active change or recoverable implementation authorizing #70 was found.

## Goals / Non-Goals

**Goals:** Read-only bounded inspection of the authoritative project; an integrated repeatable narration example; evidence for both alignment paths, existing transactional operations, rendering and actual GUI inspection.

**Non-Goals:** New public operations, worker changes, inferred timings, persistence changes, editable audio controls, DSP changes, merge or deployment.

## Decisions

1. Keep desktop presentation downstream of typed Project/TimelineItem/asset records. Present authored routing/settings and marker expressions directly; obtain effective timing and any effective routing from existing core evaluation rather than implementing a resolver in desktop. Mark unavailable computed data explicitly. Alternative: a desktop resolver would duplicate core rules and risk divergence.
2. Add process-local paging/selection for root and component-definition cues. Render at most 32 marker rows per page, 16 bus summaries per page, and one selected alignment segment with granularity/count/quality/producer summary. Bind cursors to scope, immutable ID and source revision; selection/refresh/history clears stale details. Alternative: dump entire arrays/JSON, which scales poorly for 100000 alignment segments and obscures scope.
3. Keep the fixture a test-only recipe composed through existing typed core edits. Use bounded local deterministic PCM and existing deterministic visual sources; no downloaded media, TTS or aligner. A versioned JSON recipe declares six sentence cues, independent word segments, estimated quality, fixture producer identity and explicit nullable model metadata. Both saved and explicit paths use the same declared alignment, with omitted alignment tested on the saved asset and explicit alignment tested on an otherwise unaligned asset. Alternative: change speech synthesis requests to add fixture markers, which would violate the unchanged worker contract.
4. Bind each cue to an existing visual preset and semantic event using stored marker expressions, with literal expected timing and captured event variant/gain/bus assertions. Include narration activity ducking on music and enabled master normalization using the approved main behavior. Independent references originate in fixed input samples and declared timing/control equations, never by capturing current implementation output as expected output. Existing native SSIM/PCM/timing thresholds remain unchanged.
5. Add no public/persisted surface. Existing schema44 and protocol1 remain current; a test fixture is not a new wire capability. Contract ownership review confirms frozen catalogs and provider contract bytes are unchanged. No migration is needed because persisted models are unchanged; current/history/reopen tests prove compatibility. Alternative: add a persisted inspection snapshot, which would introduce unnecessary migrations and stale display data.

## Risks / Trade-offs

- Large valid projects → bounded paging, selected-segment materialization tests and no whole alignment serialization.
- Synthetic timings mistaken for measured speech → label quality estimated and synthetic provenance in recipe, docs and UI; make no real-model accuracy claim.
- Native normalization may fail infeasible targets → select a finite reproducible feasible source/target and verify delivered output with independent backend evidence; preserve typed failure tests.
- GUI runtime or build dependencies unavailable → actual desktop build/manual evidence remains an acceptance blocker, reported separately from automated session tests.
- GitHub API policy currently denies api.github.com → read issue/PR through allowed github.com pages and Git refs; draft creation remains blocked until supported API access is available. Do not bypass the network policy.

## Migration Plan

No schema or data migration. Implement presentation and fixture/tests after artifact approval, run all required suites, verify conformance, synchronize/archive these specs and rerun protected validation. Publish only a draft PR targeting main. Reverting this change removes presentation/fixture support without rewriting projects.

## Open Questions

The user explicitly approved these artifacts on 2026-10-09. Required toolchain/GUI readiness and GitHub API access are unverified or unavailable; they are evidence blockers rather than permission to skip checks. Live issue pages identify #72 as open with both dependencies (#71 and #15) closed; successor implementation requires its own approved scope after #70 review.
