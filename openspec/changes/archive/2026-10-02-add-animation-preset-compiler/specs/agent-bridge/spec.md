## ADDED Requirements

### Requirement: Typed discoverable preset edit parity
Headless MUST expose `apply_animation_preset` in its typed nested edit union under the existing `edit`/`edit_batch` request envelopes. MCP MUST register `timeline_apply_animation_preset` and accept the same operation in `timeline_batch_edit`. All surfaces MUST carry typed `itemId`, mandatory `presetId`/`presetVersion`, closed scalar-tween parameters and optional collision policy, use current project/revision envelopes and `WriteResult`, and preserve core result/error/retryability/changed-ID/alias semantics. Exported TypeScript types MUST reject missing mandatory fields, wrong parameter/value/curve tags and unsupported property names. Adapters MUST delegate catalog dispatch, compatibility, timing, finite semantic bounds, collisions, migration and publication to core.

Status/capability reporting MUST add `animation_presets_v1`, report project schema 29, and retain protocol major 1 and all existing identifiers/meanings. Tool input metadata, public documentation and canonical fixtures MUST expose the exact seed/version, units, absolute item-local timing, default curve/policy, collision rules, provenance lifecycle and draft/direct-definition exclusions. Project/component responses MUST accept the optional provenance sidecar while preserving the old channel shape. Valid older request payloads MUST remain accepted.

#### Scenario: Match standalone and alias batch results
- **WHEN** equivalent valid preset requests run through headless edit and MCP standalone/batch, including an earlier creation alias
- **THEN** results expose the same committed primitive/provenance state, revision, changed IDs and batch alias mapping

#### Scenario: Return core failures through every transport
- **WHEN** a structurally valid unknown preset/version, collision, missing target, locked track, stale revision or final candidate rejection occurs
- **THEN** headless/MCP preserve the core stable error and retryability without partial mutation; malformed wire structures fail before mutation

#### Scenario: Type-check caller input and discover support
- **WHEN** bridge type fixtures construct malformed preset edits or a client inspects capability/tool metadata
- **THEN** malformed edits fail TypeScript checking and clients can distinguish the one-entry preset compiler, schema 29 and documented authoring exclusions under protocol 1
