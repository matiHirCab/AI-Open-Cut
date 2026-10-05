## RENAMED Requirements

- FROM: `### Requirement: Schema32 mask model migration`
- TO: `### Requirement: Schema33 mask model and channel migration`
- FROM: `### Requirement: Atomic mask migration and draft validation`
- TO: `### Requirement: Atomic mask activation migration and draft validation`

## MODIFIED Requirements

### Requirement: Schema33 mask model and channel migration
Persisted schema33 MUST activate closed mask animation targets/properties and approved static/animated mask rendering. Optional stored masks MUST retain empty-stack omission defaults and all #50 model fields/limits/IDs. Source schemas1…31 MUST reject any present masks field, including empty/null, on root/component items before defaults; source schemas1…32 MUST additionally reject new mask target/property identifiers, including empty-key channels. Schema32 static masks MUST remain legal and migrate without loss. Supported earlier current state and every retained undo/redo snapshot MUST migrate deterministically to33 under the existing project lock, preserving IDs, revisions, provenance, timing and managed resources. Newly activated renderability/work/precision checks MUST validate the complete migrated generations before publication. Valid nonempty32 masks MUST intentionally acquire approved mask output; absent/empty masks MUST preserve previous output. Unknown future/zero versions MUST retain existing fail-closed compatibility errors. Reading valid33 with omitted masks MUST NOT rewrite solely to materialize defaults.

#### Scenario: Migrate retained history and reopen deterministically
- **WHEN** valid earlier current state has both undo and redo snapshots, schema32 stored masks and legacy-only or applicable mask-bearing drafts
- **THEN** all applicable documents migrate together to33, authored masks/legacy clocks persist exactly, undo/redo reproduce the correct activated generation and repeat reads do not rewrite authoritative files

#### Scenario: Reject premature and future retained fields
- **WHEN** any retained source contains masks before32, mask targets/properties before33, malformed current mask/channel data or an unsupported future version
- **THEN** open fails before publishing current/history/draft migration or changing any managed resource

#### Scenario: Reject newly unsafe prior metadata atomically
- **WHEN** valid earlier metadata in any current/history/applicable-draft candidate exceeds activated renderability/work/inverse-precision bounds
- **THEN** adoption fails with existing typed errors and old authoritative/resource bytes unchanged, without clipping, repairing or dropping stored models

### Requirement: Atomic mask activation migration and draft validation
Core MUST validate the complete cloned current/history envelope and retained drafts before one existing crash-consistent migration publication. Applicable draft operations MUST replay against the matching current/retained base revision, never an unrelated generation, and complete replayable candidates MUST satisfy activated renderability. Earlier-than32 source envelopes MUST reject present masks operations before migration; earlier-than33 source envelopes MUST reject new mask target/property operations, including unavailable-base drafts. Valid schema33 drafts with unavailable bases MUST retain structural/catalog/resource validation and existing stale REVISION_CONFLICT behavior without candidate replay or owner-dependent certification against unrelated current state. Draft envelope version2 MUST remain unchanged. Failed migration, recovery, validation or draft mutation MUST preserve authoritative current/history/draft bytes and resources; existing journal recovery MUST precede new migration. Existing staged pre-publication selector/resource validation MUST cover channel-only mask edits as well as masks-bearing edits, preventing invalid create/update/rebase/commit/preview from publishing a resource rewrite or migration prefix.

#### Scenario: Preserve an invalid applicable draft generation
- **WHEN** an applicable retained draft has invalid mask/channel metadata or a missing owning item/target during migration
- **THEN** its existing structural/core failure blocks publication with every authoritative document/resource unchanged

#### Scenario: Preserve unavailable-base stale drafts
- **WHEN** a structurally/catalog/resource-valid schema33 draft with typed masks or mask channels has had its base evicted from retained history
- **THEN** reads/edits/discard remain available and preview/apply retain existing REVISION_CONFLICT without replay or owner-dependent certification on unrelated current state

#### Scenario: Preserve crash-consistent generation boundaries
- **WHEN** migration fails before commit or journal recovery encounters interrupted publication
- **THEN** readers observe a complete old/new generation under existing recovery rules, never a mixed current/history/draft publication

#### Scenario: Preserve source-matched channel-only staged requests
- **WHEN** a channel-only mask draft/edit has invalid targets/renderability on its applicable base while resource selectors could otherwise be rewritten or migration could occur
- **THEN** final-candidate failure publishes no project/history/draft/selector/resource prefix and retains existing acknowledged-commit/recovery behavior
