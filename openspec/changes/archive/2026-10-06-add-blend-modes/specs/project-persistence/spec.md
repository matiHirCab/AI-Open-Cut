## ADDED Requirements

### Requirement: Atomic schema 35 default-normal blend migration
Verified schema34 current and all retained undo/redo snapshots MUST adopt35 with absent blendMode default normal in one existing project-locked staged transaction. IDs/revisions/timestamps/order and all unrelated mask/matte/media/font/provenance values/resources MUST remain; reopened reads SHALL NOT rewrite adopted state. Raw pre35 root/component item sources MUST reject blendMode presence even normal/null before defaults. Retained draft new fields MUST be guarded against actual current/matching retained source bases, and future/malformed/unsupported sources MUST reject atomically.

#### Scenario: Adopt genuine predecessor current history and resources
- **WHEN** genuine schema34 current/undo/redo/applicable drafts contain old masks/mattes and managed media/fonts/provenance
- **THEN**35 adoption preserves semantic default-normal output, resources and revisions across all retained generations, with deterministic undo/redo/reopen and no read rewrite

#### Scenario: Reject premature malformed or future retained fields
- **WHEN** any pre35 root/component/current/history/draft source contains blendMode normal/null/non-normal or future/invalid retained data
- **THEN** staged preflight rejects before publication and every authoritative/resource byte remains unchanged

### Requirement: Source-matched blend drafts and complete journal recovery
Applicable drafts MUST validate against their own matching retained bases before adoption; valid35 unavailable-base drafts MUST be byte-preserved with current read/edit/discard available but preview/apply/commit REVISION_CONFLICT. Blend-bearing create/update failures on legacy bases SHALL NOT incidentally adopt current/history/resources. Existing prepublication exact rollback and post-journal recovery MUST retain one complete valid generation.

#### Scenario: Validate stale and unavailable bases without current replay
- **WHEN** a draft target exists only in retained state or no matching base exists for a valid35 draft
- **THEN** available-base validation uses that base, unavailable-base operations stay revision-conflicted and draft bytes remain unchanged

#### Scenario: Preserve failure and interrupted publication atomicity
- **WHEN** invalid later edits or injected persistence interruptions occur around all journal/publication phases
- **THEN** failures before publication preserve all bytes/resources and reopen after published journal recovers one complete valid current/history/draft/resource generation
