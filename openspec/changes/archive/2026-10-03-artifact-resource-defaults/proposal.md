## Why

Issue #74 requires bounded metadata responses instead of repeated unconditional PNG/WAV base64 during job polling. Clients should choose when to fetch process-owned output without gaining arbitrary file access.

## What Changes

- Default job polling returns existing structured job metadata plus an optional artifact resource descriptor and MCP resource link.
- Add optional `includeBinary: true` to recover inline PNG/WAV; explicit MCP resource reads retrieve PNG/WAV/MP4 output.
- Register an opaque job-owned artifact resource and advertise bridge capability `artifact_resources_v2`.
- **BREAKING**: default PNG/WAV content blocks are removed. Existing request and structured result fields remain compatible; clients needing the old content behavior must explicitly set `includeBinary: true`. This opt-in is the migration path required by issue #74. A separate canonical artifact-delivery-v2 contract records this major-2 content policy; structural MCP, headless, project and provider contracts retain their existing versions.

## Capabilities

### New Capabilities
- `artifact-resources`: Metadata-first process-local artifact delivery and explicit binary retrieval.

### Modified Capabilities

None. Existing safe transports, job retention and canonical rendering semantics remain authoritative.

## Impact

Bridge job MCP schemas, status capability registration, resource registration, catalog/parity tests, response tests, source/packaged smoke, and documentation. CODEOWNER is @matiHirCab; delegated issue-scoped approval is recorded separately from human CODEOWNER review. No Rust/headless request changes are needed because binary delivery is entirely MCP-owned; typed render artifact responses remain unchanged.

## Non-goals

No persisted project migrations, renderer changes, network resources, path-based download API, new authentication model, speech-provider capability work (#59), animation changes (#47), or preview presets (#71).
