## ADDED Requirements

### Requirement: Atomic schema 34 scoped matte adoption
Schema33 current state and all retained undo/redo snapshots MUST deterministically migrate to34 defaulting absent matte/matteOnly to None/false, under the project lock and existing staged/journal transaction. IDs/revisions/timestamps, ordering, media/font resources/provenance and unrelated semantics MUST be preserved; repeated reads SHALL NOT rewrite adopted files. Future schema versions MUST reject. Raw pre34 source envelopes MUST reject presence of matte/matteOnly on root/component items even null/false before applying defaults, and equivalent authored retained-draft operation fields MUST be guarded at their actual source bases.

#### Scenario: Adopt all retained generations and resources
- **WHEN** genuine schema33 current/undo/redo/applicable drafts coexist with managed media/fonts/provenance
- **THEN** all generations adopt34 atomically with unchanged unrelated values/resources and reopened undo/redo reproduce correct state without further read rewrite

#### Scenario: Reject premature or malformed retained data atomically
- **WHEN** any pre34 current/history/component source or draft contains new fields, or any source version is future/malformed/cyclic/dangling
- **THEN** full preflight rejects before adoption and current/history/draft/resource bytes remain unchanged

### Requirement: Source-matched matte drafts and journal recovery
Applicable drafts MUST be validated against their own current/matching retained bases before staged adoption, including hidden/unused graph and complexity checks. A valid schema34 unavailable-base draft MUST remain byte-preserved and current project read/edit/discard available, but draft preview/apply/commit MUST return REVISION_CONFLICT; it SHALL NOT be replayed onto current. Mask/matte-bearing legacy draft create/update failures MUST not incidentally adopt project/history/resources. Existing prepublication rollback and post-journal recovery MUST preserve one complete valid generation.

#### Scenario: Validate stale available and unavailable draft bases
- **WHEN** a valid stale draft's target exists only in its retained base, or no matching base exists for a valid schema34 draft
- **THEN** available-base validation uses that base, unavailable-base operations stay revision-conflicted and both preserve draft bytes without current-state replay

#### Scenario: Preserve recovery across publication phases
- **WHEN** draft/graph failure or injected publication interruption occurs before or after journal publication
- **THEN** prepublication failure preserves exact authoritative/resources and postpublication reopen recovers one complete valid current/history/draft/resource generation
