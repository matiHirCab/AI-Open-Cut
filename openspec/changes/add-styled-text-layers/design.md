## Context

Issue #34 builds on stored rich-text documents (schema 18) and pinned font shaping (schema 19). `RichTextDocument` currently owns ordered runs; `TextStyle` owns one outline and one unblurred shadow. Core shapes to byte-addressed glyph clusters and rasterizes pinned outlines. The implementation must preserve that legacy path and keep all domain rules in editor-core under ADR 0003. Headless and bridge are adapters, with contracts governed by ADR 0002.

The unrelated `reduce-agent-context-overhead` change and existing working-tree edits are outside scope. Its presence can independently block the archive-only protected gate; it must not be modified to unblock this work.

## Goals / Non-Goals

Goals: the four delta specs define grapheme-addressed styles, bounded ordered paint stacks, shared rendering and atomic schema-20 persistence, with additive public contracts and scenario coverage.

Non-goals: see proposal; particularly no alternate renderer, ambient font fallback, desktop editing controls or general effect system.

## Decisions

1. **Extend documents instead of replacing runs.** Add optional `spans` to documents and optional `paintLayers` to item/span styles. Keep absent fields absent on serialization; retain simple-text projection and all previous valid inputs. Reject overlaps instead of introducing ambiguous cascading; sorted nonoverlapping spans are sufficient to describe effective styled ranges. Store full replacement documents through existing operations rather than introducing partial indexed text mutation operations.

2. **Pin grapheme interpretation.** Use Unicode 16.0 default extended grapheme segmentation and a pinned implementation with independent Unicode conformance vectors. Segment the concatenated string, including cross-run graphemes; map grapheme ranges to bytes once in core. Span bold/italic overrides feed the existing bound-face segmentation. Paint boundaries leave shaping context intact. Map paint to complete shaped clusters using the lowest logical cluster offset, including RTL and ligatures. Splitting ligatures for color was rejected because it would violate existing paint-only shaping invariants. The profile and this cluster fallback must be documented to clients.

3. **Explicit stacks and stable legacy fallback.** Closed fill/stroke/shadow variants accept only solid colors. Array order is back-to-front. Each contiguous effective logical style segment paints its complete stack before the next logical segment; adjacent identical effective styles coalesce. Paint span boundaries that fall inside a shaped cluster move to that cluster's ownership decision, never divide outlines. An item explicit fill color takes precedence over run/span `color`; that color affects only the original legacy fill when no explicit stack applies. Span stacks replace item stacks. Empty stacks are meaningful. Keep the old per-glyph legacy raster path when stacks are absent to avoid changing overlap pixels. Automatically translating every old style to global passes was rejected because overlapping glyphs can render differently.

4. **Bounded local raster effects.** Reuse tiny-skia outline coverage and source-over. New centered strokes use round joins; shadow masks union filled glyph coverage before coloring. Use deterministic separable Gaussian convolution with normalized weights exp(-d*d/(2*sigma*sigma)) on integer taps [-ceil(3*sigma),ceil(3*sigma)], transparent extension and a final clamped rounded alpha conversion. Zero sigma bypasses convolution. Blur is performed before item transforms. Use checked arithmetic for bounds and the specified conservative full-raster pass budget. Reject excess in core preparation before allocation or destination checks. External filters and shader-dependent blur were rejected to preserve a single local deterministic rendering path.

5. **Schema 20 without a legacy visual transition.** Source-validation rejects new fields in earlier versions, including rich-text slots. Migration from 19 advances the schema while leaving new optional fields absent; older migrations retain existing intermediate steps. Current state, undo/redo and applicable retained drafts participate in the existing journal/lock transaction and recovery protocol. Do not re-resolve existing fonts or add missing defaults that alter old rasterization. Rollback before journal commit preserves original bytes; after commit use existing complete-generation recovery. Downgrading an already activated project is unsupported; restoring a complete pre-upgrade backup requires the matching older binary.

6. **Versioned contract and review evidence.** Add `contracts/styled-text-layers-v1.json` and ownership entries, then Rust/headless and bridge schema/capability consumers plus MCP surface snapshots. Preserve `text-layout-v2` shaping semantics and old fixture histories; new schema-20 fixtures reference the existing font profile plus the new paint/grapheme contract. Update latest-schema reporting without editing historical fixtures to pretend they originally supported layers. Audit all listed consumers and obtain designated CODEOWNER review. Python provider formats stay unchanged; run hermetic worker tests as repository regression evidence.

## Risks / Trade-offs

- Graphemes can span runs and ligatures can span styles -> independent Unicode, mixed bidi, combining-mark and ligature tests must assert offsets/glyphs/paint ownership explicitly.
- Blur can multiply memory/time -> validate dimensions and aggregate passes before allocation, use separable convolution and bounded reusable buffers, test exact boundaries and overflows.
- Legacy strokes interleave with glyph fills -> preserve the omitted-stack path and use decoded lossless comparisons against existing fixtures.
- Schema activation affects retained data -> source validation and all persistence fault phases need automated migration coverage, including hidden components and drafts.
- A segmentation dependency version may implement another Unicode version -> implementation must select a pin satisfying the approved profile and prove it with conformance fixtures; a profile change requires updated artifacts and approval.
- Unrelated active changes or missing render/toolchain dependencies can block mandatory checks -> record exact failures and logs; do not waive gates or change unrelated work.

## Migration Plan

Approve artifacts first. Add canonical fixtures and migration tests before consumer changes. Implement core model/validation/migration, then evaluation/rasterization, then adapters and documentation. Run all required checks and record per-scenario conformance evidence. Run the protected gate, verify, synchronize and archive only when the repository's gate rules permit it, then rerun final protected/all-spec validation. No implementation or archival is claimed while approval or required checks are unavailable.

## Open Questions

No unresolved product choices are required for this proposed scope. The user explicitly approved the specified overlap policy, solid paint variants, cluster fallback, limits, Unicode profile and schema-20 activation by replying “Approve”. Any incompatible discovery must update these artifacts before implementation proceeds.


## Implementation notes

Canonical semantic validation remains in the validation owner, including retained draft text validation before migration publication. Shaping maps checked grapheme boundaries without importing validation. Render plans retain scoped layer identities; glyph files use deterministic numeric names because component/repeater scope separators are not valid Windows filenames. This supports the approved cross-intent component/repeater scenario without changing public IDs or contracts. Gaussian evaluation divides distance by sigma before squaring to preserve finite behavior for subnormal positive sigma.

The combined styled component/repeater fixture measured range/export SSIM 0.988 with the legacy veryfast/CRF-28 preview preset. Scenes containing the new spans/paint fields therefore use the existing medium/CRF-23 detail-preserving range preset already used by procedural grids. Export commands and scenes without the new fields retain their existing behavior. This is an implementation of the approved >=0.99 cross-intent requirement.

## Approved review corrections

The user approved fixing both reproduced rendering regressions. Mixed legacy segments must call the original painter, retaining shadow order, stroke joins and per-glyph stroke/fill order. Adjacent legacy glyphs coalesce across face/color changes and paint in their original visual order. Explicit segments coalesce by effective stack and face; overridden color cannot split them. Rendering and work accounting use the same grouping. These corrections implement existing legacy fallback and explicit paint precedence requirements without changing public contracts.
