## ADDED Requirements

### Requirement: Schema32 mask model migration
Persisted schema 32 MUST activate optional mask metadata with empty-stack omission defaults. Source schemas1…31 MUST reject a present masks field, including empty/null, on any root/component item before defaults can erase premature data. Supported earlier current state and every retained undo/redo snapshot MUST migrate deterministically to 32 under the existing project lock, preserving IDs, revisions, provenance, timing and managed resources while introducing empty masks only. Future/zero versions MUST retain existing fail-closed compatibility errors. Reading valid schema 32 with omitted masks MUST NOT rewrite solely to materialize defaults.

#### Scenario: Migrate retained history and reopen deterministically
- **WHEN** a valid earlier project has both undo and redo snapshots and legacy-only retained drafts
- **THEN** all applicable documents migrate together, undo/redo retain prior content, masked schema 32 state reopens identically and repeat reads do not rewrite authoritative files

#### Scenario: Reject premature and future retained fields
- **WHEN** current state is valid but any retained source contains masks before 32, malformed schema 32 masks or an unsupported future version
- **THEN** open fails before publishing any current/history/draft migration or changing managed resources

### Requirement: Atomic mask migration and draft validation
Core MUST validate the complete cloned current/history envelope and retained drafts before one existing crash-consistent migration publication. Applicable draft operations MUST replay against the matching current/retained base revision, never an unrelated generation. A draft in an earlier-than 32 current-project envelope MUST reject present masks operations before migration; schema 32 drafts with unavailable bases MUST retain structural/catalog/resource validation and existing stale REVISION_CONFLICT behavior without candidate replay. Draft envelope version 2 MUST remain unchanged. Failed migration, recovery, validation or draft mutation MUST preserve authoritative current/history/draft bytes and resources; existing journal recovery MUST precede new migration.

#### Scenario: Preserve an invalid applicable draft generation
- **WHEN** an applicable retained draft has invalid mask metadata or a missing owning item during migration
- **THEN** its existing structural/core failure blocks publication with every authoritative document/resource unchanged

#### Scenario: Preserve unavailable-base stale drafts
- **WHEN** a structurally valid schema 32 draft with typed masks has had its base evicted from retained history
- **THEN** project reads/edits/discard remain available and preview/apply return existing REVISION_CONFLICT without replaying onto current state

#### Scenario: Preserve crash-consistent generation boundaries
- **WHEN** migration fails before commit or journal recovery encounters an interrupted publication
- **THEN** readers observe a complete old/new generation under existing recovery rules, never a mixed current/history/draft publication
