## ADDED Requirements

### Requirement: Complete raster identity and selective invalidation

Editor-core SHALL reuse successful text, shape and normalized SVG raster results using complete effective content, style, verified font identity, output/sampling scale, project identity and revision, and a versioned raster implementation identity. Text identity MUST cover positioned glyphs, effective layout, paint layers and raster extents; vector identity MUST cover geometry, paints, strokes, viewport and ordered SVG content. Same-revision drafts MUST remain distinct when effective inputs differ. Changed keys MUST miss without clearing unrelated entries. Composition-only values MUST NOT prevent reuse when they do not affect local raster pixels or sampling density. A changed revision MUST miss. Key construction MUST preserve validated finite numeric distinctions without approximate rounding.

#### Scenario: C1 Reuse equal evaluated content
- **WHEN** a renderer or its clone renders equal effective text, shapes or SVG repeatedly, including equal component and repeater occurrences
- **THEN** after the first successful rasterization subsequent equal keys reuse byte-identical rasters without rerunning the rasterizer

#### Scenario: C2 Invalidate each raster dependency
- **WHEN** content, style, layout, font bytes/face/profile, geometry, paint, stroke, SVG viewport/order, scale, raster version, project identity or revision changes independently
- **THEN** the changed raster key misses and its result matches an uncached render while unrelated retained keys remain reusable

#### Scenario: C3 Separate drafts and composition changes
- **WHEN** different materialized drafts share a project revision, or equal local sources differ only in composition timing, position or opacity without affecting density
- **THEN** different effective raster content cannot alias while composition-only differences can reuse bytes and still receive their own correct semantic plans

### Requirement: Bounded disposable raster retention

The cache SHALL remain process-local and shareable by renderer clones, retaining at most 128 entries and 67108864 bytes of accounted key and raster payload with checked arithmetic. It MUST evict least-recently-used entries before exceeding either inclusive bound. A valid raster exceeding the retention budget MUST render uncached. Failed or partial rasterization MUST NOT be retained. Concurrent use MUST preserve immutable results and retention bounds; an unusable cache MUST fall back to uncached rendering without a new public error. Existing per-render memory and complexity limits MUST remain effective independently of cache occupancy.

#### Scenario: B1 Enforce both retention limits
- **WHEN** insertion reaches or exceeds the entry or byte budget, including a single oversized valid raster
- **THEN** exact limits remain valid, least-recently-used entries are evicted as needed, oversized entries are bypassed and pixels remain equal to cold rendering

#### Scenario: B2 Recover from misses and concurrency
- **WHEN** rasterization fails and is retried, concurrent clones request equal/different keys, or cache synchronization becomes unusable
- **THEN** failures are not cached, successful output remains deterministic and retained accounting never exceeds either bound

### Requirement: Cache hits preserve validation and error precedence

Every render MUST perform existing canonical validation, readiness, evaluated-scene and aggregate complexity checks, font integrity checks and safe resource binding before raster cache lookup. Hits MUST NOT accept invalid numbers, missing references, tampered fonts, unsafe paths or unsupported SVG. Existing error codes, retryability and preflight precedence MUST remain unchanged. Cached bytes MUST be materialized through the existing owned-workspace and publication policy, with normal write failures and cleanup. Cache operations MUST NOT mutate project state, revision, history or drafts.

#### Scenario: V1 Reject invalid input after warming
- **WHEN** a cache is populated and a subsequent render encounters non-finite or over-limit input, missing references, missing/corrupt font bytes, a path escape or unsupported SVG
- **THEN** the same typed failure as cold rendering occurs before raster lookup or output side effects, with authoritative state unchanged

#### Scenario: V2 Preserve workspace failures
- **WHEN** writing a warm cached raster into its owned workspace fails
- **THEN** the existing render failure and cleanup behavior apply and no partial final artifact is published

### Requirement: Shared deterministic rendering and lifecycle compatibility

Frame preview, audiovisual range preview, materialized draft preview and final export SHALL share the cache policy and unchanged EvaluatedScene semantics. Equal cold/warm local rasters MUST be byte-identical; composed output MUST retain documented visual SSIM of at least 0.99, aligned decoded float-PCM RMS error at most 0.0001 and timing within one output video frame. Revisions, atomic batches including aliases, undo/redo, deterministic reopen, warnings and diagnostics MUST remain unchanged. Cache state MUST NOT be persisted or require a migration, public operation, capability flag, contract version or error-code change.

#### Scenario: L1 Compare all render routes
- **WHEN** deterministic fixtures containing text, shapes and SVG are rendered cold and warm through frame, range, draft and export with matching output settings
- **THEN** raster bytes and semantic plans match their uncached equivalents, warnings and diagnostics are unchanged and decoded audiovisual results satisfy existing tolerances

#### Scenario: L2 Preserve edits and reopen
- **WHEN** warmed content undergoes successful standalone/batch alias edits, a failed batch, a stale-revision or missing-target request, undo/redo and reopening through a fresh renderer
- **THEN** existing errors and atomic revision/history semantics remain intact, each valid snapshot renders its own content and fresh cold output matches the equivalent warm snapshot

#### Scenario: L3 Preserve public and persisted contracts
- **WHEN** existing requests and canonical fixtures are exercised and projects with retained history are reopened
- **THEN** all existing contracts remain valid without added schema fields or migrations and no cache content appears in persisted state or protocol responses
