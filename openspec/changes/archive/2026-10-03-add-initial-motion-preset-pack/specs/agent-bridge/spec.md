## ADDED Requirements

### Requirement: Discoverable typed initial motion pack
Headless and MCP MUST forward the unchanged preset operation with the additive closed parameter union and report initial_motion_preset_pack_v1 alongside existing capabilities. Canonical preset, project-schema, headless and MCP structural fixtures and all governed native consumers MUST agree. Existing scalar callers MUST remain valid. Unsupported live identities/versions and semantic boundaries MUST be decided in core; transport schemas MUST reject malformed closed shapes without expansion. Standalone and batch workflows MUST preserve existing aliases, revisions, typed errors, undo/redo and reopen behavior. Source and packaged smoke MUST exercise the five entries against the same canonical fixture.

#### Scenario: Old and new typed clients
- **WHEN** an old scalar request or any valid new pack request is passed through typed headless and MCP standalone/batch surfaces
- **THEN** fixtures and generated primitive/source state agree without transport-owned motion logic

#### Scenario: Failure parity and capability discovery
- **WHEN** clients inspect status or submit malformed fields, unsupported versions, collision, missing reference or stale revision
- **THEN** status exposes exact support and Rust/TypeScript/MCP agree on accepted structure, canonical failure code/retryability and unchanged authoritative state

## MODIFIED Requirements

### Requirement: Typed discoverable preset edit parity
Headless MUST expose `apply_animation_preset` in its typed nested edit union under the existing `edit`/`edit_batch` request envelopes. MCP MUST register `timeline_apply_animation_preset` and accept the same operation in `timeline_batch_edit`. All surfaces MUST carry typed `itemId`, mandatory `presetId`/`presetVersion`, closed scalar-tween or initial-motion-pack parameters and optional collision policy, use current project/revision envelopes and `WriteResult`, and preserve core result/error/retryability/changed-ID/alias semantics. Exported TypeScript types MUST reject missing mandatory fields, wrong parameter/value/curve tags and unsupported property names. Adapters MUST delegate catalog dispatch, compatibility, timing, finite semantic bounds, collisions, migration and publication to core.

Status/capability reporting MUST retain `animation_presets_v1`, add `initial_motion_preset_pack_v1`, report current project schema 30, and retain protocol major 1 and all existing identifiers/meanings. Tool input metadata, public documentation and canonical fixtures MUST expose the exact scalar seed and five pack IDs/versions, units, absolute item-local timing, default curve/policy, collision rules, provenance lifecycle and draft/direct-definition exclusions. Project/component responses MUST accept the optional provenance sidecar while preserving the old channel shape. Valid older request payloads MUST remain accepted.

#### Scenario: Match standalone and alias batch results
- **WHEN** equivalent valid preset requests run through headless edit and MCP standalone/batch, including an earlier creation alias
- **THEN** results expose the same committed primitive/provenance state, revision, changed IDs and batch alias mapping

#### Scenario: Return core failures through every transport
- **WHEN** a structurally valid unknown preset/version, collision, missing target, locked track, stale revision or final candidate rejection occurs
- **THEN** headless/MCP preserve the core stable error and retryability without partial mutation; malformed wire structures fail before mutation

#### Scenario: Type-check caller input and discover support
- **WHEN** bridge type fixtures construct malformed preset edits or a client inspects capability/tool metadata
- **THEN** malformed edits fail TypeScript checking and clients can distinguish the six-entry catalog (scalar compiler1 and pack compiler2), current schema30 and documented authoring exclusions under protocol 1
