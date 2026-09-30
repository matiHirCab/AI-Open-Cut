## ADDED Requirements

### Requirement: Atomic inherited pre-publication validation
For candidates containing inherited child timing, signed copy timing or active parent animation, core MUST run canonical derived-clock, projection, known intrinsic geometry, inherited raster and complexity preflight before publishing project state, history, draft state or staged resources. Existing per-operation field and reference validation MUST retain its order. Ordered batches and draft operation lists MUST preflight their final candidate once before publication. Hidden and unused retained content MUST be checked. Unsafe derived time, overflow or excessive inherited bounds MUST return non-retryable INVALID_ARGUMENT with unchanged authoritative project/history/draft bytes, revision, aliases, resources and artifacts, and without generated-copy materialization. Stale revisions MUST retain retryable REVISION_CONFLICT and missing or locked references MUST retain their existing typed failures. Valid older zero-timing behavior and public wire shapes MUST remain compatible; no schema version change or silent repair MUST occur.

#### Scenario: Reject excessive inherited scale before an edit commit
- **WHEN** an affected candidate contains a 500-pixel-wide rectangle whose parent scale-X channel reaches 100
- **THEN** standalone or alias-aware batch publication fails with INVALID_ARGUMENT before changing revision, history, resources or artifacts

#### Scenario: Reject a hidden stagger overflow
- **WHEN** a hidden group's stagger delay pushes a ranked child ending at u64 maximum beyond representable time
- **THEN** candidate publication fails with INVALID_ARGUMENT before generated materialization and preserves authoritative bytes and revision

#### Scenario: Preserve every draft boundary
- **WHEN** draft creation, update, rebase or commit produces unsafe inherited timing or excessive derived bounds
- **THEN** the operation fails atomically without publishing draft or project changes or staged resources

#### Scenario: Validate current state and retained history before migration publication
- **WHEN** opening or migrating current state or any retained undo/redo snapshot encounters an unsafe inherited candidate
- **THEN** it fails before resource or transaction publication without rewriting authoritative state, and valid retained generations preserve undo/redo, reopen and interrupted-publication recovery

#### Scenario: Keep compatible successes and error precedence
- **WHEN** older valid requests omit timing, or an affected edit has a stale revision, missing reference, locked track or trailing invalid operation
- **THEN** valid older behavior remains unchanged and existing failure codes, retryability and rollback remain intact without exposing partially generated aliases
