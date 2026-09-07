## Context

The approved and archived SVG fixes retain f64 composition, viewport-based surfaces and post-Z normalization. Review reproduced two gaps in editor-core: finite f32 coordinates can fail tiny-skia's outward-rounded integer bounds conversion and silently produce no coverage; authored-document preflight computes component-local magnification without enclosing instances and rejects scale cancellation before final scene refinement.

## Goals / Non-Goals

**Goals:** Reject backend-unrepresentable SVGs before artifacts; render valid composed occurrences; retain hidden/unused validation and all existing budgets; demonstrate public rendering and lifecycle conformance.

**Non-Goals:** No new SVG syntax, clipping algorithm, per-point clamping, standalone shape change, wire/schema migration, external resource access or dependency edge.

## Decisions

### Shared SVG raster representation checks

Keep numeric semantics inside editor-core. Introduce shared SVG conversion/validation helpers used by evaluation and artifact preparation. Validate the same raster-space f32 values supplied to tiny-skia, their outward-rounded integer bounds and representable dimensions, and arithmetic used to construct those bounds. Check conservative stroke envelopes including caps and miter expansion; validate dash conversion, sums, normalized offset and backend dash construction. Failed construction must not silently select an undashed stroke. Handle degenerate and offscreen paths as legitimate empty coverage when representable.

Return non-retryable INVALID_ARGUMENT before artifact preparation. Retain artifact-side defensive error propagation but do not rely on backend logging or a successful blank render as validation. Test the pinned backend's conversion boundaries directly, including stroke expansion; document the conversion rationale alongside the shared helper. Preserve the standalone shape encoding behavior.

Alternative: clip arbitrary mapped geometry before rasterization. Rejected for this follow-up because correct fill-rule and stroke clipping adds a new algorithm beyond the approved scope. Individual point clamping changes geometry and is not acceptable. Merely checking f32 finiteness does not cover integer conversion failures.

### Separate stored-document and occurrence validation

Validate normalized documents independently of instance transforms, including hidden content, definitions, drafts and history through existing validation paths. For transformed raster checks, use a bounded occurrence traversal carrying complete outer component and group transforms, preserving existing keyframe scale bounds and clocks. Compose matrices before computing magnification. Hidden occurrences participate in validation even when omitted from visual output.

Compute reachability through all project instances, including hidden instances. Validate project occurrences with their actual ancestry. For definitions not reachable from the project, validate each as a virtual root with identity outer transform and traverse nested instances. Referenced definitions must not also receive isolated-root sizing. Keep graph cycle/depth/occurrence guards before recursive expansion.

Remove premature isolated SVG surface sizing from intermediate flat/component evaluation. Ensure final compilation receives the complete transform and remaining scene budget. Reuse compiled occurrence results where practical; intermediate representations must not consume the same segment/memory allocation budget twice. Account each expanded occurrence, preserve document bounds, curve limits, 16384 dimension / 16777216 area limits, 44 bytes per SVG pixel and existing aggregate ceilings. Unreachable virtual-root checks must also remain bounded and cannot create an unbounded validation traversal.

Alternative: drop all definition or hidden checks and rely on visible final refinement. Rejected because stored/hidden content must not bypass bounds. Continuing to size every definition in isolation preserves the confirmed false rejection.

### Evidence and compatibility

Add unit tests for numeric conversions, fills/strokes/dashes and transformed occurrence graphs. Include the two exact review reproductions. Accepted raster tests require independent pixel expectations; the scale-cancelled scene compares with an identity-scale scene and analytic expected content. Extend existing headless/MCP and native SVG conformance coverage across frame, range, draft, export and undo/redo/reopen. Verify failed rendering publishes no artifacts and preserves state, revision and history; retain stale-revision precedence tests. Synchronize canonical evidence without changing public wire shapes or introducing transport-owned validation.

## Risks / Trade-offs

- Conservative stroke envelopes can reject extreme artwork that clipping might render. Mitigation: explicitly choose fail-closed behavior, derive bounds from actual cap/join semantics and test inclusive numeric boundaries.
- Backend conversion rules can change on dependency upgrades. Mitigation: shared conversion helpers and backend-boundary regressions, rather than a finiteness-only assumption.
- Traversing hidden and virtual occurrences can expand work. Mitigation: check remaining occurrence/segment/memory capacity before expansion and preserve cycle/depth guards.
- Intermediate evaluation may still accidentally size a local SVG. Mitigation: nested cancelling scales and multiple differently scaled instances exercise the full public pipeline.

## Migration Plan

No migration: schema 15, protocol 1, APIs, aliases and stored models stay unchanged. Existing stored scenes receive corrected rendering or explicit numeric rejection. Rollback is a code revert, with no persistent rewrite required. Do not regenerate unrelated golden references.

## Verification

Run Rust formatting, workspace strict Clippy and tests; bridge typecheck, lint, unit, contract, MCP integration and packaged smoke checks; hermetic Python tests; required native conformance and transform suites. Record commands/results and requirement-to-test traceability. Use openspec-verify-change, resolve mismatches, synchronize and archive, then run the protected Moon openspec validation gate. Required failures or skipped checks block completion.

## Open Questions

None. User explicitly approved these artifacts on 2026-09-07.
