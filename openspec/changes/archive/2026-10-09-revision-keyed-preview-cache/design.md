## Context

The existing renderer evaluates each immutable snapshot once, verifies media/font bindings, performs complexity/readiness/normalization preflight, materializes owned resources and invokes its process adapter. The existing raster cache and worker are suitable predecessors but do not retain encoded PNG/MP4 previews. This change must avoid duplicating canonical project/timeline rules and preserve error precedence over output side effects.

## Goals / Non-Goals

Reuse identical successful frame and audiovisual range previews, with complete identity and bounded disposable retention. Preserve current frame/range/draft/export semantics, source integrity, errors, warnings, text-layout diagnostics, progress, revisions/history and atomic publication. No persistent cache, exported-file reuse, inferred speech data, new endpoint, public cache diagnostics or migration.

## Decisions

1. A private artifact-owner cache holds immutable encoded bytes behind Arc/Mutex and is shared by renderer clones. Renderer orchestration supplies versioned validated identity; artifact ownership handles retention, bounded reads and fresh publication. Keep the existing ownership/import matrix. If implementation needs a new edge, update ADR0003 and its architecture test before introducing that edge, rather than embedding project validation in cache code.
2. Stream exact validated snapshot/evaluated identity into SHA256 without materializing whole JSON. Include projectID/revision, frame versus range intent, exact start/end, dimensions/FPS/includeAudio, same-revision effective draft content, renderer implementation identity and verified media/font/backend binding identity. Retained resource integrity checks precede lookup. Do not round key floats, use paths as a substitute for verified content, or let drafts alias merely because revisions match.
3. Complete ordinary validation/readiness/resource/complexity preflight before lookup. Existing normalization preparation and raster/materialization may still run; the concrete optimization is avoiding the ordinary final render execution on an encoded-artifact hit. Do not claim that every native preparation process disappears on warm requests. Preserve the existing evaluated semantics and current warnings/layouts for each request.
4. Retain at most32 entries and67108864 bytes including key and payload accounting; evict LRU before exceeding either inclusive limit. Use a capacity-admitted bounded read of successful controlled output. Oversized media, unsupported optional adapter reads, arithmetic failure or poisoned cache bypass caching. Cache unavailable is not a new public error. Never hold a cache lock while doing I/O/rendering; tolerate concurrent misses without unbounded retention. No failed/partial publication may populate the cache.
5. A hit publishes immutable retained bytes into a fresh owned temporary/output path under the established preview publication policy. Deleting or changing an old published preview must not corrupt the cache or make a hit return that mutable old path. Preserve existing warnings/layouts/MIME/hash/result shape and bounded monotonic completion progress. Write/publication failures preserve normal typed failure and cleanup behavior.
6. Export stays uncached and shares the unchanged evaluated scene. Existing source/packaged MCP requests use the persistent worker without added public fields. Optional counters/trace needed to prove actual avoided native execution stay behind the existing explicit test-feature boundary and never appear in normal protocol results. Reset cache when renderer backend/adapters change; request correlation must not clear eligible semantic entries.

## Implementation ownership and eligibility

`render_artifact/preview_cache` owns immutable key/payload retention and streaming identity hashing; its optional bounded-reader port is implemented by `FileSystemArtifactIo` and delegated by request-scoped artifact I/O. `render_process/preview_identity` owns binary resolution/content fingerprints. `renderer` supplies the full validated project snapshot, closed request intent, owner-produced media/font digests and backend identity, repeats ordinary preflight/materialization, and publishes through the existing artifact owner. These siblings stay inside the existing ADR0003 import matrix; no new owner or dependency edge is introduced.

System backend reuse requires identifiable direct FFmpeg-family CLI native ELF/PE executable bytes (including its embedded version formatter) and canonical paths for both FFmpeg and FFprobe. Forwarding native wrappers, scripts, non-UTF8 bindings, unresolved/ambiguous bare PATH candidates and unsupported adapters bypass reuse. Canonical PATH aliases of one executable are permitted. This conservative eligibility does not alter ordinary rendering for those bindings. Legacy drawtext requires explicit resolved font files; shaped text uses verified in-memory face bytes. Configured fallback-font bytes are fingerprinted too. Changing adapter or font roots resets retention; correlation IDs retain it.

The64MiB bound accounts for retained key and encoded payload bytes, with32 entries. Mutex/deque/Arc bookkeeping is outside that accounting. Independent raster retention, currently materialized workspaces and concurrent immutable hit references or pending copies remain in-flight memory rather than a global memory ceiling. The reader checks regular-file type and size against remaining single-entry admission before allocation; a growth/empty/nonfile/optional-read failure skips insertion. Backend/content hashing and I/O/rendering occur outside the retention lock. Test diagnostics are emitted only to stderr under the existing `raster-cache-test-hooks` feature; normal schema/protocol/result/catalog/provider files stay unchanged.

## Risks / Trade-offs

- Same-revision drafts or direct immutable candidates differ: full effective identity plus independent cold output tests.
- Warm requests could mask invalid input/tampered resources: mandatory canonical preflight and cold/warm typed-error/side-effect comparison.
- Encoded videos are large: inclusive entry/byte bounds, one bounded retained copy, oversized bypass and documented independent raster plus in-flight memory.
- Fresh warnings/tool state can differ: verify dependencies before lookup, report current diagnostics rather than trusting stale metadata.
- Optional artifact adapters cannot perform bounded reads: uncached fallback preserves old adapter behavior.
- Native/GUI/CI tools can be unavailable: report required evidence blockers and do not weaken or regenerate frozen references.

## Migration Plan

None. Cache is process-local; restart is cold and stored schema44/protocol1 remain unchanged. Follow approved OpenSpec implementation, required checks, conformance review, synchronization/archival and protected validation. Publish only a draft PR targeting main after verification.

## Open Questions

The user explicitly approved this scope on 2026-10-09. Work begins from verified main in a separate worktree so its active proposal cannot invalidate issue #70's protected gate. GitHub API access remains unavailable in the current session; public issue/dependency snapshots and Git refs are available.
