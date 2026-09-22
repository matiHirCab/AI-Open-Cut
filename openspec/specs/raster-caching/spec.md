# Raster Caching Specification

## Purpose

Define bounded deterministic reuse of validated text and vector rasters while preserving core rendering, lifecycle and failure semantics.

## Requirements

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

### Requirement: Cross-request agent raster reuse
Successive frame, range, draft and export requests served by an available persistent render worker SHALL share one editor-core renderer cache. Equal keys MUST avoid rasterization across request boundaries while every request repeats canonical preflight and loads its own immutable snapshot. Retained key/payload limits MUST remain 128 entries and 67108864 bytes per renderer. Documentation MUST distinguish retained memory from in-flight work and independent overflow processes. Cache state MUST remain disposable and absent from persisted projects and public render results.

#### Scenario: X1 Prove agent reuse through actual raster work
- **WHEN** an agent renders unchanged text, shapes and SVG through successive worker requests at matching settings
- **THEN** instrumented tests prove the same worker avoids subsequent rasterization and output matches a fresh renderer with correct plans, warnings, layout diagnostics, visual/audio tolerances and timing

#### Scenario: X2 Preserve invalidation and cold restart
- **WHEN** project identity, revision, effective draft content or pixel dependencies change, or a worker is replaced
- **THEN** changed keys or a fresh cache miss, unchanged eligible keys remain reusable, and each snapshot renders its own content without affecting history or revisions

### Requirement: Production-path dependency invalidation evidence
Complete raster identity MUST be backed by valid independently varied inputs submitted through evaluation and materialization. Changed output MUST match a separately constructed renderer, then reuse its own cached bytes, while unrelated retained keys remain reusable. Hash-sensitivity tests alone MUST NOT establish this conformance. Unsupported font profiles MUST be tested as failures rather than presented as valid font changes. Tests MUST distinguish composition-only changes from changes to composed raster sampling.

#### Scenario: X3 Vary valid pixel dependencies
- **WHEN** tests independently vary text/runs/spans, paint stacks, layout, selected valid font bytes, vector geometry/paint/stroke, SVG viewport/order, output dimensions or composed sampling scale at the same revision
- **THEN** changed rasters miss and equal fresh output, later repeats hit, and an unrelated retained raster still hits

#### Scenario: X4 Reuse composition-only changes
- **WHEN** timing, position or opacity changes leave local pixels and sampling density unchanged, including component and repeater occurrences
- **THEN** cached local bytes are reused while independent plan/placement assertions demonstrate the changed composition correctly

### Requirement: Warm preflight conformance evidence
Warm-cache validation tests MUST compare cold and warm error codes, retryability and relevant stages, demonstrate unchanged hit and miss counters, inspect process/artifact events and preserve authoritative project/history/draft state. They MUST cover invalid numbers, missing references, missing/corrupt fonts, unsafe paths, unsupported SVG, aggregate complexity and readiness failures through applicable frame/range/draft/export routes. Existing preflight precedence over export collision and existing workspace-write failure behavior MUST remain effective. Instrumentation MUST be explicitly test-gated and absent from normal wire output.

#### Scenario: X5 Reject before lookup and output side effects
- **WHEN** a populated cache is followed by any specified invalid-input, resource, aggregate-limit or dependency-readiness failure
- **THEN** cold and warm typed errors agree, neither hit nor miss counts advance, prohibited process/artifact side effects do not occur and authoritative state is unchanged

#### Scenario: X6 Preserve precedence and warm materialization failures
- **WHEN** invalid work targets an existing export destination or writing a valid cached raster fails
- **THEN** invalid work retains its canonical error before collision inspection, and a warm write failure retains the established render error, cleanup and no-partial-publication behavior
