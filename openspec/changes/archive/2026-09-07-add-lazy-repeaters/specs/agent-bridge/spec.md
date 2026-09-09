## ADDED Requirements

### Requirement: Typed discoverable repeater workflows
The public protocol MUST add headless edit operation `add_repeater`, batch union membership, repeater-bearing project/draft/component/item responses, and MCP tool `timeline_add_repeater` using strict mirrored schemas for the canonical descriptor. `timeline_batch_edit` MUST accept the same operation and alias rules. Canonical operation, MCP structural schema/annotation, capability, fixture, ownership, Rust, and TypeScript parity evidence MUST be updated together. Capability `repeater_items` MUST report editing support and `repeater_rendering` MUST be available only when a configured local renderer can execute the complete evaluated scene. Existing protocol-1 envelopes, simple operations, errors, and retryability MUST retain their meaning.

#### Scenario: Discover and invoke standalone and batch edits
- **WHEN** a client inspects operations, MCP tools, annotations, schemas, and capabilities and then submits equivalent valid standalone or batched repeater edits
- **THEN** every surface reports the exact canonical identifiers/fields, forwards typed input to editor-core, and returns equivalent revision, changed-ID, alias, item, and typed-error behavior

#### Scenario: Reject malformed and semantic failures consistently
- **WHEN** headless or MCP receives unknown/duplicate/missing fields, invalid values, missing/unsupported/cyclic sources, a locked track, or stale expected revision
- **THEN** transport decoding or core translation returns the established non-retryable `INVALID_ARGUMENT`, `ITEM_NOT_FOUND`, `TRACK_LOCKED`, or retryable `REVISION_CONFLICT` behavior without adapter-side domain rules or partial mutation

#### Scenario: Report readiness without degraded fallback
- **WHEN** editing support exists but no configured local renderer supports every instruction in the repeated evaluated scene
- **THEN** `repeater_items` remains discoverable, `repeater_rendering` is unavailable, and render readiness fails with `DEPENDENCY_UNAVAILABLE` rather than dropping or approximating copies

#### Scenario: Preserve additive compatibility boundaries
- **WHEN** an existing protocol-1 client continues using simple operations against projects without repeaters
- **THEN** its accepted requests and responses retain their meaning, while documentation states that schema-17 repeater-bearing projects and the new item variant require a repeater-aware decoder
