## ADDED Requirements

### Requirement: Atomic schema 30 motion pack migration
Core MUST migrate every supported schema to30 under the existing lock and recoverable atomic journal, including current project, component items and every retained undo/redo snapshot. Schema29 scalar provenance, channels, revisions/timestamps, media/font integrity and rendered output MUST remain unchanged; migration MUST infer no labels. New tagged source shapes in pre30 item envelopes, malformed source generations and unknown future schemas MUST fail closed before migration publication. Recovery MUST preserve existing committed-warning semantics. Older builds MUST reject schema30; rollback MUST use a complete pre-migration project/history backup.

#### Scenario: Migrate retained scalar sources unchanged
- **WHEN** schema29 current/components/undo/redo contain valid scalar sources and channels
- **THEN** all snapshots become30 atomically with identical source/channel content and render output, and second reopen does not rewrite files

#### Scenario: Fail closed across the generation
- **WHEN** current/component/undo/redo has malformed or premature tagged source data, duplicate raw fields/maps, array records or a future schema
- **THEN** open fails through the canonical persisted-input path without changing either authoritative file or managed resources

#### Scenario: Recover interrupted migration
- **WHEN** a fault occurs at each existing migration publication phase
- **THEN** reopening recovers one complete accepted generation and retains project/history/source consistency without recompilation

## MODIFIED Requirements

### Requirement: Atomic schema 29 preset provenance migration
Editor-core MUST migrate every supported older project through the schema29 provenance stage and onward to the current schema30 under the existing project lock, including current state, all retained undo/redo snapshots and items in root/component-definition tracks. Earlier documents MUST gain empty provenance without inferring preset authorship from existing channels. Migration MUST preserve existing IDs, revision/timestamps, keyframes/channels/curves/loops, hierarchy, assets/fonts and managed resource bytes. Source documents below schema 29 MUST reject any `animationPresetProvenance` field, including empty/null values, rather than silently drop or activate it. Valid schema29 projects MUST complete the schema30 upgrade without relabeling or recompilation, while valid current-schema projects MUST reopen idempotently without rewriting their stored generation.

#### Scenario: Migrate current and complete retained history
- **WHEN** a supported older project with undo/redo history and component-definition animation opens
- **THEN** current state and every retained snapshot migrate together through schema29 to schema30 with empty provenance and equal prior primitive/render semantics

#### Scenario: Reject premature provenance
- **WHEN** current state or a retained snapshot below schema 29 contains a provenance field, including an empty object
- **THEN** migration fails closed without publishing current state/history or modifying managed bytes

#### Scenario: Repeat a completed migration
- **WHEN** a valid migrated current-schema project is opened repeatedly
- **THEN** its authoritative project/history bytes, revision/timestamps, resolved channels and provenance remain unchanged

### Requirement: Fail-closed complete provenance generations
Migration and publication MUST validate all current/retained provenance shapes, identities, finite parameters, primitive/reference safety and existing candidate limits before publishing a complete project/history generation. A malformed current document, malformed retained undo or redo snapshot, unsupported future project schema, or pre-commit I/O failure MUST preserve the previous authoritative generation. Unknown future project schemas MUST retain the established `INTERNAL_ERROR` failure and MUST NOT be rewritten; a historical preset source version absent from the compilation catalog MUST remain distinguishable from a future project schema. Valid interrupted commits MUST retain existing recovery and `PERSISTENCE_RECOVERY_PENDING` behavior. Rolling back an upgrade MUST require a prior complete backed-up generation rather than dropping fields or downgrading the current schema30.

#### Scenario: Reject invalid retained provenance atomically
- **WHEN** a retained undo or redo snapshot contains null/orphaned/non-finite provenance or invalid resolved primitives
- **THEN** the entire migration/publication fails through the existing typed validation path and current state/history/resources remain the previous generation

#### Scenario: Preserve future-version rejection
- **WHEN** current state or retained history declares a project schema newer than the current supported schema30
- **THEN** core returns `INTERNAL_ERROR` and leaves the complete generation unchanged

#### Scenario: Recover a committed provenance generation
- **WHEN** project/history publication is interrupted after the existing durable commit point
- **THEN** recovery converges on the complete saved primitives/provenance/history generation without compiling or applying the preset twice
