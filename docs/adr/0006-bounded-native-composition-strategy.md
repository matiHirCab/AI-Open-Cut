# ADR0006: Bounded native composition strategy

- Status: Accepted bounded decision and verified conditional disposition; release gates pending
- Date:2026-10-13
- Issue:[187](https://github.com/matiHirCab/AI-Open-Cut/issues/187)
- Parent:[180](https://github.com/matiHirCab/AI-Open-Cut/issues/180)
- Extends:ADR0004; preserves ADR0003 owners and ADR0002 contracts
- Evidence:[bounded evaluation](../bounded-dense-scene-strategy.md)

## Context

The seven completed original workloads use1920x1080/30fps, a500ms frame and three-frame[0,100)ms range. Their original native shapes/text and managed artwork remain editable. The predeclared maximum-of-three fresh decision rule uses the unchanged issue181 reference thresholds5000ms/8000ms. Exact reference environment matches are recorded. Thirteen/fourteen cells pass; row4frame5414.02ms fails. All140captured outputs and state/preexisting-output assertions pass; the original record's final classification failure was separately corrected/revalidated without replacing data. Three observations do not characterize a latency distribution; sampler overhead and source evolution prevent a regression claim.

Actual raster and encoded cache counters show partial reuse. Existing source rasters are reused, but native visual preparation occurs before encoded lookup. Moving that lookup alone would affect warm behavior and cannot remedy the failed cold target. Preparation's observed2307.17ms in the failed sample provides a bounded path for investigation; its inner byte-transfer cost is inferred from source, not directly isolated.

## Decision

Select exactly one outcome: **one small identified direct-rendering optimization candidate**, with target sufficiency unverified until its separate issue passes. Retain the current authoritative editable graph/direct native architecture. Do not implement optimization or precomposition inside187.

Candidate: replace repeated byte-channel `linear(u8/255)` transfer evaluation in existing `render_artifact::extended_visual::Raster::rgba` with one256-entry **f64** lookup generated from the same function. Keep alpha multiplication in f64 and cast each product to f32 in the original order. This affects cold source-to-float preparation as well as warm cached PAM parsing. It adds approximately2KiB immutable scalar storage rather than retaining decoded user rasters or creating a source/derived lifecycle. The inference is plausible; no speedup or target success is asserted.

Follow-up issue: [203](https://github.com/matiHirCab/AI-Open-Cut/issues/203), created under specific user approval with the exact [reviewed body](../verification/bounded-dense-scene-strategy/follow-up-issue-draft.md), linked to187/180 and blocked until187's exact published-head verification. Its scope is solely this byte-transfer conversion and its conformance/measurement proof.

## Required implementation proof and resource compatibility

Before implementing, the follow-up requires its own approved OpenSpec plan. Exhaustively compare all256channel×256alpha pairs bit-for-bit against the current f64 transfer/multiply/f32cast; retain malformed-input, dimension and failure behavior. Verify independent alpha/color fixtures, original PNG/encoded oracles, cold/warm/frame/range/draft/export parity and unchanged warnings/layout/history/resources/preexisting-output behavior. Keep all existing linear-light semantics, checked admission, cache identities/limits, schema45, public declarations/errors and owner dependency edges.

Demonstrate the table's exact bounded storage and ownership; account for it within existing fixed-overhead policy without increasing any guard or cache budget. No per-source/user-content retention. No approximate transfer, lower precision, decoded-plate caching, shortcut preflight or GPU/renderer rewrite.

Reference performance acceptance: same named host/release/font/tools and unchanged fourteen-cell settings/procedure; maximum of three fresh samples in **every** cell must meet5000ms frame/8000ms range, while retaining the5414.02ms187counterexample as the recorded predecessor. Source-miss/preparation/backend timings and raster/encoded work must remain distinct and non-additive. Compare controlled before/after builds; report instrumentation and sample limitations. Portable CI timing remains report-only. If these targets are not established, do not declare sufficiency or erase the failure: record the result and reopen a bounded strategy decision instead of broadening implementation.

## Alternatives and conditional disposition

Current direct rendering already demonstrates independent output and resource admission, but fails the predeclared fresh comparison in one cell; warm success is insufficient to choose the all-targets-met outcome.

Static source-preserving precomposition would add source identity, freshness, history/persistence, managed-resource ownership and derived-output/UI work. The bounded evidence supplies no necessity for that larger lifecycle: original workloads are admitted and the measured miss is414.02ms over a5s comparison goal, with an identified small cold-path candidate. Therefore precomposition is **not selected** for this slice. [188](https://github.com/matiHirCab/AI-Open-Cut/issues/188) and [189](https://github.com/matiHirCab/AI-Open-Cut/issues/189) were explicitly closed as not_planned under specific user approval, preserving their original bodies and appending the bounded decision and203link. [Exact action receipts and fresh-read state/body proof](../verification/bounded-dense-scene-strategy/external-action-receipts.json) retain this disposition; it was not silently waived.

The unimplemented96-leaf/six-instance/3000ms envelope is outside this evidence. Future evidence may justify a separately approved precomposition decision; this ADR does not assert that larger scenes are ready.

## Release and rollback

Release remains blocked until the selected optimization is separately verified and remaining issue191 workflow acceptance is complete. Issue187's deliverable is the bounded evidence/decision and linked conditional disposition, with required local checks, conformance/archive and all18/four-workflow exact remote-head acceptance.

No production behavior, persisted contract, dependency or migration changes here. Reverting187removes only its test-only observations/evaluation and documentation/specification; original recipes/direct rendering remain valid. No merge, auto-merge or deployment authorization.
