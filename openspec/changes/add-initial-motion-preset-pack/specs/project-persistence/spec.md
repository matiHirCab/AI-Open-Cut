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
