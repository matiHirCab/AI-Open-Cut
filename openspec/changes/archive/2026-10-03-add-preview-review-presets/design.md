## Context

Range previews already share the export evaluator/render pipeline. The MCP legacy tool requires custom resolution and fps and defaults audio off; headless requires explicit numeric dimensions, fps and audio. Issue #71 is a range review convenience change with no project mutations.

## Goals / Non-Goals

**Goals:** typed presets, retained custom dimensions, audio-enabled review default, canonical validation, discoverability, deterministic export parity and unchanged legacy callers.

**Non-goals:** frame/draft resizing, export behavior changes, saved settings or migrations, compositor redesign, job semantics, speech clocks, pending unmerged issues.

## Decisions

1. Add MCP `preview_review_range`, retaining `preview_render_range` exactly. Its resolution is `540p|720p|project|{width,height}`, default `project`; fps is optional; includeAudio defaults true with explicit false respected. A unique tool avoids changing legacy omission semantics or inventing a policy selector. Changing the existing default would require a major contract under ADR 0002; a custom-only legacy exception in one tool would make the new audio default inconsistent.
2. Add headless `render_review_range` accepting `resolution:540p|720p|project|{width,height}` (omission project), optional fps and optional includeAudio default true. Preserve legacy `render_preview_range` fields and behavior exactly, including its historically broader positive-only core dimension acceptance. Unknown enum/type/unknown fields fail existing `INVALID_ARGUMENT` at typed transport; canonical invalid bounds and time intervals fail `VALIDATION_FAILED`. New MCP schema rejects malformed selection before queuing. No adapter computes dimensions.
3. Core preset resolver sets **height** to 540 or 720 and preserves project width/height aspect, including portrait and square. Width is rounded to nearest even integer with ties upward, minimum 2, using checked u64 integer arithmetic: `max(2,2*floor((projectWidth*targetHeight+projectHeight)/(2*projectHeight)))`. Project and custom dimensions are exact, without rounding. Resolved dimensions must be 1..7680 wide and 1..4320 high; fps 1..120. Unsupported aspect ratios that exceed bounds fail instead of silently clamping or cropping. The approved inward renderer→validation dependency edge is added explicitly to ADR0003 and its architecture matrix. New resolver invokes canonical settings validation before the existing numeric range API; legacy numeric callers retain their historical behavior.
4. Introduce a small canonical request-options resolver in editor-core, resolving against the immutable revision before using existing PreviewRangeOptions and render_preview_range. Do not modify all existing options literals or reconstruct scene semantics. MCP translates selection shape into typed fields, then queues the existing range worker. Project-dependent fps/dimensions remain core-owned.
5. Advertise `preview_review_presets_v1` in ready rendering subsystem and aggregate headless/MCP capability lists. Add reviewed fixtures for defaults, each preset, custom, explicit false and invalid types. Keep protocol major1; #71 introduces no persisted-schema change (the initial base was schema30, and authoritative main now owns schema31 migration/history). Catalog changes remain manual and narrowly scoped; update expanded MCP digest deliberately after parity evidence, not from a production auto-generator.

## Risks / Trade-offs

- Two range tools → recommend the audio-enabled review tool in instructions/docs while describing legacy compatibility explicitly.
- Height-based portrait presets can be small in width → documented deterministic aspect rounding, tests include landscape, square, portrait and extreme ratios.
- Odd project/custom MP4 sizes can encounter existing codec restrictions → preserve requested dimensions exactly and existing FFmpeg failure semantics; don't introduce new implicit rounding.
- Unmerged #59/#74 alter shared contracts → test a disposable semantic union, keep preview fields isolated, recompute union expanded digest, exclude unrelated commits from this branch.

## Migration Plan

Clients may adopt the new tool after detecting its capability. Existing MCP tools and all previously valid headless requests preserve explicit choices; silent review opts out with includeAudio=false. No persisted migration. Rollback removes the additive capability/tool and restores the request additions, leaving project state untouched.

## Verification

Focused core resolver boundaries and immutable rendering failure order, native audiovisual default/opt-out and export parity, transport deserialization/defaults, MCP workflows and contract parity. Verify unchanged revision/history/reopen, missing media and unsafe paths before side effects, cancellation/job envelope reuse. Run all mandated suites, independent implementation review and final protected gates; no fabricated CODEOWNER acceptance.

## Open Questions

Independent reviewer must approve range-only scope and the additive separate-tool compatibility decision before implementation.
