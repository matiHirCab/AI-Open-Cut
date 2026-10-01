## MODIFIED Requirements

### Requirement: Atomic schema-27 extended visual animation migration
Core MUST migrate every supported older project to schema 27 under the project lock, adding full-source crop and empty effect stacks without altering IDs, provenance, revisions, legacy channels/curves/loops, timing, stacking, or rendered output. Migration MUST validate current state, every retained undo/redo snapshot, and retained draft state/candidates before atomically publishing one complete generation. Old source schemas containing prematurely authored crop/effect fields or newly activated channels MUST fail closed rather than gaining retrospective validity. Schema zero, malformed source/history/drafts, unknown future schemas, and interrupted writes MUST preserve established stable failures and recovery guarantees. Older binaries MUST reject schema 27. Valid schema-27 reopen MUST be deterministic and MUST NOT rewrite authoritative bytes.

Retained version-2 draft validity MUST NOT depend on extended channels, crop or effects being present in the draft or its base. Validation MUST use an applicable matching retained base where available and preserve established valid-stale-draft validation and REVISION_CONFLICT behavior without replaying onto an unrelated current revision. An invalid target in a legacy-only retained draft with an applicable base MUST fail with its existing typed error before any migration publication.

When a structurally valid retained version-2 draft's base revision is absent from current state and retained undo/redo history, core MUST preserve its established stale-draft behavior without attempting candidate replay against current state. Structural, catalog and resource validation MUST remain enforced; migration MUST preserve that draft's authoritative bytes. Project reads, edits and draft discard MUST remain available after history eviction. Applying or previewing that stale draft MUST retain REVISION_CONFLICT without authoritative writes.

#### Scenario: Migrate retained history and drafts
- **WHEN** an older project contains nonempty undo and redo histories and retained drafts
- **THEN** current and retained state migrate atomically with identity defaults and unchanged undo/redo, revisions, provenance, and legacy output

#### Scenario: Reject invalid retained state before publication
- **WHEN** any source snapshot or draft contains premature fields, an unsupported channel, invalid target, or future schema
- **THEN** open fails without changing authoritative project, history, draft, or managed resource bytes

#### Scenario: Recover and reopen deterministically
- **WHEN** migration is interrupted at a supported fault phase or a valid migrated project is reopened repeatedly
- **THEN** recovery exposes one complete old or new generation and subsequent valid reopen preserves bytes and sampled output

#### Scenario: Reject invalid legacy-only retained draft target
- **WHEN** current state and all retained undo/redo snapshots are at schema 26 and a retained version-2 legacy-only trim draft with an applicable base references a missing item
- **THEN** open returns ITEM_NOT_FOUND before schema-27 publication and preserves authoritative project, history, draft and managed-resource bytes

#### Scenario: Preserve a valid stale legacy draft
- **WHEN** a schema-26 project contains a valid stale legacy-only version-2 draft with an applicable retained base
- **THEN** schema-27 migration preserves draft bytes and IDs without applying its operations to current state, and draft access retains the established REVISION_CONFLICT behavior

#### Scenario: Evict a valid stale draft base during ordinary edits
- **WHEN** a valid trim draft refers to an item deleted after its base revision, later current state has extended effects, and ordinary edits evict the base from bounded undo/redo history
- **THEN** the edit causing eviction and subsequent project reads, edits, reopen and draft discard succeed without replaying the stale draft against unrelated current state

#### Scenario: Migrate a stale draft whose base is no longer retained
- **WHEN** a supported older project has a structurally valid version-2 draft with an unavailable base, including operations referencing an item absent from current state
- **THEN** schema-27 adoption preserves draft bytes, migrates current and history atomically, and draft application or preview returns REVISION_CONFLICT without changing authoritative bytes
