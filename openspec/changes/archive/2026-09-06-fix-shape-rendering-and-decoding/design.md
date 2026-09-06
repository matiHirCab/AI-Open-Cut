## Context

The original ShapeItem change is archived, but review reproduced incorrect magnified coverage, fractional-anchor displacement, and accepted duplicate vector fields. The existing working tree includes that implementation and must be preserved. Core already owns evaluated contours, PAM raster artifacts, affine sampling, typed vector decoding, and project migration preprocessing.

## Goals / Non-Goals

Goals: correct all three findings with independently checked regressions; preserve valid inputs, schema 14, migration/history behavior, and shared rendering semantics.

Non-goals: new public fields, tools, geometry, backend, dependency edge, or unrelated validation policy. No global rejection policy for unrelated legacy JSON objects.

## Decisions

1. Add process-local raster density to EvaluatedShape. Compute max(1, sigma_max) from the full effective local-to-output linear matrix, with a numerically stable singular-value calculation. Include parent/component transforms and maximum supported animated scale over the interval. Compute from authored transforms, not a density-compensated matrix, so refinement is idempotent. Keeping the existing local-resolution bitmap was rejected because increased curve subdivision alone cannot restore lost coverage; a new output-space backend is unnecessary.
2. Keep geometric bounds and authored contours in local coordinates. Convert padded bounds to raster coordinates using density, floor/ceil there, and keep an explicit local-to-raster mapping. Stroke padding scales with density; the antialias padding is one raster pixel. Rasterize paths and stroke metrics at density, evaluate paint at inverse-mapped raster pixel centers, and compose the inverse local-to-raster mapping into source-to-output transforms. Update animated sampling as well as static matrices. Preserve fill-before-stroke and premultiplied linear-light color math.
3. Validate finite dimensions, checked integer conversions, per-shape surfaces and aggregate memory before raster allocation or artifact writes. Count simultaneously allocated coverage/output buffers according to existing memory accounting. Keep existing segment and subdivision limits; reject insufficient precision or excessive work with INVALID_ARGUMENT rather than cap density. Test boundary failures independently of allocation success.
4. Derive anchors from analytic unstroked extents. Preserve every positive extent, including subpixel dimensions. Substitute one only for a zero-extent axis of path geometry; line axes and other geometry keep their actual extent. Stroke/raster padding never participates in anchors.
5. Introduce an internal Serde-compatible buffered value with sequence storage for object entries, retaining duplicates and input numeric types. Replay MapAccess into the existing typed vector deserializers. Replace lossy buffers on shape-bearing EditOperation and ProjectDocument paths, retaining preprocessing for legacy slots/stacking and existing explicit-null checks. Audit batch flattening, draft/component payloads and retained project/history wrappers for intervening Value conversion. Do not normalize before strict nested records decode. A transport-level JSON scan was rejected because it duplicates core ownership and misses direct core/persistence consumers; globally rejecting duplicate legacy objects would broaden compatibility impact.
6. Preserve raw-string and from_value support. Already-parsed Value input cannot expose duplicates removed upstream. Raw headless requests and persisted documents must retain them. Keep existing malformed-input error translation and stale-revision behavior. No new public catalog shape is needed; add raw regression evidence to existing governed consumers and retain parity checks.

## Verification and traceability

- Closed bounded shape geometry: exact evaluator matrix tests for fractional rectangles/ellipses, noncentral anchors, horizontal/vertical lines, and move-only/zero-extent paths; equivalent anchor placements must yield identical rasters.
- Scale-aware shape coverage: compare 2x2 at scale 50 against 100x100 at scale 1; raster alpha must be 255 at the center and 0 at exterior corners well outside the analytic edge. At matching density allow at most one 8-bit coverage level difference; transformed comparisons use an independently defined one-output-pixel edge band and verify no interior/exterior leakage beyond it. Native codec checks use existing golden tolerances without weakening them.
- Canonical shape evaluation: nonuniform scale, rotation/skew, inherited/component transforms, maximum animated scale, gradients, strokes and dash phase; matching timestamps across preview/range/export; density-adjusted allocation limits and refinement idempotence.
- Canonical JSON representation enforcement: raw duplicate keys at color, point, stop, stroke, radii and path nesting, both identical and invalid-first/valid-last; edits/batches/drafts/components/current/history; assert structural rejection and unchanged persisted state/revision/history. Retain missing/null, reordered keys, legacy migration and stale-revision regressions.
- Run existing Rust formatting, strict workspace Clippy, workspace tests, TS typecheck/lint/unit, contract parity, MCP integration, packaged smoke, relevant hermetic worker tests, and release native golden conformance. Record commands/results and scenario-to-test mapping in verification.md. Required failures/skips block completion.

## Risks / Trade-offs

- Higher raster density increases memory and can reject previously rendered oversized work: retain documented limits and fail before allocation, never silently reduce fidelity.
- Source/raster pixel-center mismatch can shift output: exact matrix and native equivalence regressions cover static and animated sampling.
- Buffered decoding can accidentally change migration or optional-field behavior: retain all legacy and contract fixtures, test raw nesting, and scope replacement to existing lossy shape-bearing buffers.

## Migration Plan

No persisted migration or wire-version change. Record concrete artifact approval before implementation, add failing regressions first, then complete tasks. Verify with openspec-verify-change and archive with openspec-archive-change; preserve the original archive. Run strict active-change validation before archival and the final Moon OpenSpec gate after archival, since protected policy intentionally rejects active changes. Rollback consists of reverting this follow-up only, without rewriting user data or the original implementation.

## Open Questions

No implementation decisions remain open. The user explicitly approved these concrete artifacts on 2026-09-06.
