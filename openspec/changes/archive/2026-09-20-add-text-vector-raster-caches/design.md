## Context

Issue #36 builds on existing shaped text, advanced layout, typed vector primitives, shapes and sanitized SVG. `render_artifact::prepare_render_resources` rasterizes shaped glyph layers; `prepare_shape_resources` rasterizes evaluated shapes, including normalized SVG. Both write PAM bytes into an owned temporary workspace. The existing shaping memo is local to measurement and is not a raster cache.

## Goals / Non-Goals

**Goals:** Reuse identical validated raster work across renderer calls and cloned renderers; make every pixel-affecting input part of identity; bound retained memory; preserve all cold-render guarantees.

**Non-Goals:** Persistent caches, layout or geometry changes, caching composed video/audio, new user controls or transport fields, migrations, and additional graphics backends.

## Decisions

1. **Ownership and lifetime.** Add a private cache under `render_artifact`, held by `Renderer` through shared synchronized ownership. Clones share it; separately constructed renderers start cold. It stores immutable raster bytes, never workspace paths, and uses the existing renderer-to-artifact dependency. A global singleton was rejected because of project privacy and lifetime coupling; a workspace-only memo would miss repeated previews.

2. **Complete identity.** Use a versioned typed canonical key and a streaming SHA-256 digest rather than debug output or arbitrary JSON maps. Include project identity and revision, raster kind/version, effective text/document/style, positioned glyphs and layout geometry, verified face digests and indices/shaping profile, actual raster dimensions/origins/density and all raster-affecting vector geometry/paint/stroke/SVG viewport/order. Obtain identity from the evaluated/binding envelope, not downstream traversal of persisted records. Include requested output scale and effective composed sampling scale. No approximate float rounding is allowed. Full content prevents same-revision materialized drafts or copied project IDs from aliasing. Keep composition-only timing/position/opacity out of the reusable local raster identity when they cannot affect raster pixels or density. A revision-only or item-ID key was rejected as unsafe; path-based font identity was rejected because bytes can change.

3. **Precise invalidation.** A changed key misses without clearing unrelated entries. Revision changes deliberately invalidate reuse across revisions, as required by the issue; no reuse across undo/redo revisions is promised. Within one revision, equal effective sources can share bytes across component/repeater occurrences and render intents. If frame and interval evaluation select different density, they correctly use separate entries. No final semantic plan, transform, diagnostics or warnings are cached.

4. **Bounded retention.** Use deterministic least-recently-used eviction with inclusive limits of 128 entries and 64 MiB total retained key plus raster payload. Account with checked arithmetic before insertion. A valid raster too large for this cache renders normally without retention. The cache budget is separate from, and cannot relax, existing scene/surface/work budgets. Retained bytes are immutable shared buffers to avoid copying under the lock. Rasterization runs outside the lock; duplicate concurrent misses are allowed, while insertion rechecks identity and bounds. In-flight renders remain governed by existing limits. A poisoned cache is bypassed rather than becoming a new user-visible error. Unbounded maps were rejected.

5. **Validation and materialization.** Lookup occurs only after existing readiness, evaluation, path safety, font-byte integrity, measurement and aggregate work validation. Hits still create the normal workspace-local file through ArtifactIo and preserve all write/publication failures. Cache only successfully completed rasters; never store an error or partial buffer. Resource warnings remain recomputed on every request. Legacy drawtext paths continue unchanged when there is no core raster result. Skipping preflight on hits was rejected because a previously valid project can later have missing or corrupted resources.

6. **Compatibility and fixtures.** This internal optimization exposes no new addressable behavior and needs no capability flag, protocol/schema version, transport fixture or migration. Existing contract fixtures remain unchanged and pass parity checks. Add deterministic core cache fixtures and warm/cold render golden evidence. Keep coordinates, local offsets, anchors, half-open timing, stacking, fallback and visual/audio tolerances as defined in the living specifications. No code outside core gains validation or cache policy.

## Risks / Trade-offs

- Omitted key fields cause stale output: use an explicit field audit beside key construction and independent one-field-at-a-time invalidation tests, including layout, font face, SVG order and composed scale.
- Revision scoping reduces reuse after edits: accept the conservative behavior; optimize across revisions only through a separately approved requirement.
- Hashing and integrity verification still cost work: prove avoided raster calls with counters, not fragile timing assertions; make no unmeasured latency claims.
- Large rasters can bypass the cache and concurrent misses can duplicate work: test retention bounds and preserve existing per-render allocation limits.
- Font or workspace failures after a warm render can be obscured: inject missing/corrupt fonts and failing ArtifactIo after population, and assert unchanged typed errors and no partial publication.

## Migration Plan

No persisted data changes. Ship through the existing core build. Reverting this optimization restores uncached rendering without converting projects, history or drafts. Closing the renderer releases cache retention; reopening with a new renderer starts cold and yields identical pixels.

## Verification and traceability

Map each scenario in `specs/raster-caching/spec.md` to named tests in a verification report. Use independent expected pixel checks and cold/warm byte comparison, instrument actual raster invocations, exercise all production render routes, and retain existing golden tolerances for lossy output. Include font tampering, invalid finite/complexity input, missing references, stale revisions, failed atomic batches, drafts at the same revision, undo/redo and fresh reopen. No normative scenario is exempt from automation. Record exact commands, exits and external log paths; no implementation is complete until required suites and final protected validation pass.

## Open Questions

The user explicitly approved this proposal, limits and revision policy in this task. Any need to change public contracts or dependency edges must be brought back into these artifacts before implementation.
