## ADDED Requirements

### Requirement: Atomic schema38 speech provenance adoption
Core MUST migrate genuine schema37 and all earlier supported current states and retained undo/redo snapshots to38 through the existing project lock, staged complete-generation validation and crash-consistent journal. Absent alignment MUST remain absent; IDs, revisions, timestamps, resources, existing provenance and rendered behavior MUST remain unchanged. Raw speech generation sources below38 MUST reject alignment presence including null before defaults. Every retained generation MUST validate before publication; unknown future schemas MUST fail closed. Generated-speech commit/replacement SHALL stage legacy adoption and the mutation together through existing prepared generation/resource ownership; failed validation/reference/revision/resource admission MUST preserve authoritative current/history/draft/managed-file bytes. Lossless raw asset buffering SHALL retain duplicate member rejection while checking pre38 alignment presence before defaults. Existing historical guards and draft semantics SHALL remain. Repeated valid38 reads MUST not rewrite bytes.

#### Scenario: Adopt genuine legacy current and history
- **WHEN** a supported project and retained undo/redo contain old generated speech provenance without alignment
- **THEN** atomic migration advances every schema marker to38 while preserving all other values/resources and a second reopen performs no rewrite

#### Scenario: Reject malformed retained source without partial adoption
- **WHEN** any current/undo/redo source has a pre38 alignment field, invalid schema38 alignment or future version
- **THEN** migration fails with an existing typed error before changing authoritative files or managed resources

#### Scenario: Recover migration publication faults
- **WHEN** an existing journal publication fault interrupts adoption
- **THEN** existing recovery produces a complete old or committed new generation with established committed-warning behavior and unchanged resources

#### Scenario: Reject failed generated-speech requests without publishing legacy adoption
- **WHEN** a valid aligned commit/replacement on a genuine legacy project fails its target/revision/resource checks
- **THEN** no migration, new asset/provenance or partial history is published and authoritative files/resources remain byte-identical through existing rollback ownership
