## ADDED Requirements

### Requirement: Atomic schema-27 extended visual animation migration
Core MUST migrate every supported older project to schema 27 under the project lock, adding full-source crop and empty effect stacks without altering IDs, provenance, revisions, legacy channels/curves/loops, timing, stacking, or rendered output. Migration MUST validate current state, every retained undo/redo snapshot, and retained draft state/candidates before atomically publishing one complete generation. Old source schemas containing prematurely authored crop/effect fields or newly activated channels MUST fail closed rather than gaining retrospective validity. Schema zero, malformed source/history/drafts, unknown future schemas, and interrupted writes MUST preserve established stable failures and recovery guarantees. Older binaries MUST reject schema 27. Valid schema-27 reopen MUST be deterministic and MUST NOT rewrite authoritative bytes.

#### Scenario: Migrate retained history and drafts
- **WHEN** an older project contains nonempty undo and redo histories and retained drafts
- **THEN** current and retained state migrate atomically with identity defaults and unchanged undo/redo, revisions, provenance, and legacy output

#### Scenario: Reject invalid retained state before publication
- **WHEN** any source snapshot or draft contains premature fields, an unsupported channel, invalid target, or future schema
- **THEN** open fails without changing authoritative project, history, draft, or managed resource bytes

#### Scenario: Recover and reopen deterministically
- **WHEN** migration is interrupted at a supported fault phase or a valid migrated project is reopened repeatedly
- **THEN** recovery exposes one complete old or new generation and subsequent valid reopen preserves bytes and sampled output
