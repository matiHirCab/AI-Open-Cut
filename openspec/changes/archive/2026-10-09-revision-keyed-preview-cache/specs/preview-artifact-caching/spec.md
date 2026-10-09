## ADDED Requirements

### Requirement: Complete immutable preview cache identity
Editor-core SHALL reuse successful encoded frame and range previews using exact project identity, revision, frame/range intent, requested timestamp/range, output dimensions, FPS and audio option. The identity MUST additionally distinguish same-revision effective draft/candidate content, verified media/font dependencies and rendering implementation/backend identity. Finite numeric distinctions MUST remain exact; no approximate rounding or incomplete path-only identity is allowed. A changed identity MUST miss and produce the same result as a fresh renderer. Persistent worker requests and renderer clones with equal configuration SHALL share eligible entries; changed backend/adapters or a new worker SHALL start cold.

#### Scenario: P1 Reuse exact immutable previews
- **WHEN** equal frame/range previews are requested repeatedly through a renderer, its clone or the same persistent worker
- **THEN** later requests reuse byte-identical encoded media without executing the ordinary final render again, while result metadata and current diagnostics remain correct

#### Scenario: P2 Invalidate every cache dimension
- **WHEN** projectID, revision, time/range, width, height, FPS, audio choice, same-revision draft content or a verified rendering dependency changes independently
- **THEN** the changed key misses, its output equals independently cold output and unrelated eligible retained entries remain reusable

### Requirement: Bounded disposable encoded retention
The preview cache MUST remain process-local with at most32 entries and67108864 accounted key/payload bytes, using checked arithmetic and LRU eviction before exceeding either inclusive limit. Successful oversized previews SHALL render uncached. Capacity admission MUST precede allocating a retained output copy; unsupported optional bounded-reader adapters, unusable synchronization or optional cache failures SHALL fall back to ordinary uncached behavior without a new public error. Concurrent requests MUST preserve immutable bytes and inclusive retention bounds; I/O/rendering MUST occur outside cache locks. Failed or partial rendering/publication MUST NOT enter the cache. Cache state MUST NOT be persisted or alter project/history/draft bytes.

#### Scenario: P3 Enforce entry and byte limits
- **WHEN** successful inserts reach/exceed either bound or a single valid preview exceeds retention capacity
- **THEN** exact limits are accepted, LRU entries are evicted as required, oversized work is bypassed and media remains equal to cold output

#### Scenario: P4 Recover from cache and process failures
- **WHEN** a render/publication fails, concurrent clones miss, a reader is unsupported or cache synchronization is unusable
- **THEN** failures are not cached, later valid requests can render successfully, and existing errors/output/retention invariants are preserved

### Requirement: Warm preflight and fresh safe publication
Every preview MUST repeat existing canonical input, scene, dependency/readiness, resource-integrity/path, normalization and aggregate complexity preflight before cache lookup. A hit MUST NOT mask invalid numbers, missing references, unsafe/tampered resources, unavailable tooling or invalid effective drafts. Cold and warm errors/retryability/stages and preflight precedence MUST remain equivalent, without prohibited output/process side effects. Hits SHALL publish retained bytes through a fresh owned temporary preview path with normal atomic publication and cleanup; old preview file deletion/tampering MUST NOT become retained cache content or an unsafe returned path. Current warnings, layout diagnostics, MIME/hash/response shape and bounded monotonic completion progress MUST be preserved.

#### Scenario: P5 Reject invalid warm input
- **WHEN** a cache is warmed and subsequent work has invalid input, stale/missing authoritative references, unsafe/tampered resources, missing tooling or over-limit complexity
- **THEN** cold and warm typed failures agree, cache counters do not advance before rejection, no partial preview is published and authoritative state/history/resources remain unchanged

#### Scenario: P6 Publish fresh immutable hits
- **WHEN** an eligible preview is requested after its prior artifact is deleted/changed, or writing/publishing a hit fails
- **THEN** successful hits publish original immutable media at a fresh safe path and failures retain established typed cleanup behavior without a partial final output

### Requirement: Shared media and lifecycle compatibility evidence
Frame/range/materialized-draft previews and final export MUST retain one evaluated behavior. Cold/warm preview encoded bytes MUST be identical; independent preview/export media evidence MUST retain SSIM>=0.99, aligned decoded float-PCM RMS<=0.0001 and timing within one output video frame. Existing standalone/batch alias authoring, stale revisions, invalid/missing references, atomic rollback, undo/redo and deterministic reopen MUST remain unchanged. Schema44, protocol1, provider/speech-worker contracts and existing public catalogs MUST remain byte-unchanged; cache instrumentation MUST be explicitly test-gated and absent from normal protocol results. No migration, new endpoint or capability is required for this internal optimization.

#### Scenario: P7 Prove native and lifecycle invariance
- **WHEN** representative deterministic audiovisual fixtures are rendered cold/warm through direct and actual source/packaged MCP requests, edited, failed atomically, undone/redone and reopened with a fresh renderer
- **THEN** independent references and cold-render comparisons preserve semantic/media/error/history behavior, instrumented evidence proves actual avoided ordinary native execution, and restart remains cold
