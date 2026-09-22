## Why

Issue #36 now has renderer-local text/vector raster reuse, but the bridge's specified process-per-request transport discards it between agent renders. The independent review also found that hash-only invalidation tests and incomplete warm-cache failure assertions do not establish all claimed C2/V1 conformance.

## What Changes

- Add an opt-in persistent headless render worker and one warm worker per bridge client; overlapping renders continue through one-shot processes without queueing.
- Preserve legacy one-shot execution for all operations and use it for non-render requests. Update the living bridge requirement explicitly to permit the persistent-render exception.
- Introduce a versioned, fixture-governed worker envelope and request-scoped artifact identity without changing existing MCP requests, results, or project schemas.
- Add production-path evidence of successive-request cache reuse, dependency invalidation, and validation/error precedence with warm caches.
- Document retention versus in-flight memory, failure/restart behavior, and the actual verification environment.

## Capabilities

### New Capabilities
None. Worker transport is an extension of the existing bridge capability.

### Modified Capabilities
- `agent-bridge`: Persistent render-only worker transport, request identity, concurrency, cancellation, timeout, cleanup, and legacy compatibility.
- `raster-caching`: Cross-request renderer lifetime and meaningful production-path invalidation and warmed-preflight evidence.

## Impact

Changes affect editor-core renderer/artifact request scoping and tests, headless dispatch and transport, bridge client lifecycle and tests, canonical worker fixtures/ownership, and documentation. Core remains the sole cache and domain owner. No new private-owner dependency edge is planned; if one becomes necessary, update ADR 0003 and its architecture test before implementation.

Compatibility is additive: a new explicit CLI mode and separately versioned worker protocol preserve the existing protocol-1 single-request contract. Paired updated bridge/headless binaries use the worker; an unsupported worker does not trigger an automatic replay. A small additive Rust renderer method scopes artifact identity while retaining shared cache ownership. ADR 0002 governs new cross-language fixtures and designated-owner review. No persisted-schema change or migration is needed; existing current-state/history migration tests remain mandatory.

## Non-goals

Persistent disk caches, multiple retained workers or a render queue, cross-process cache sharing, public cache controls/statistics, rendering-quality changes, new graphics features, altered mutation semantics, global environment mutation, and automatic request replay are excluded. The prior archive remains historical evidence and is not rewritten to imply that this follow-up was already implemented.

## Approval

The user selected persistent rendering with one warm worker plus one-shot overflow, reviewed the linked proposal/design/specs/tasks, and explicitly replied "Yes" to the artifact approval request on 2026-09-21. Implementation is authorized through these tasks.

Final contract-owner review: on 2026-09-21 the user explicitly replied "Yes" to the request for CODEOWNER approval of the completed worker contract, registered consumers and parity evidence, and authorized specification synchronization and archival. This is separate from the earlier planning approval.
