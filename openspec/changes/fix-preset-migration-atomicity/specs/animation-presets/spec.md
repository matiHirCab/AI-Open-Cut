## MODIFIED Requirements

### Requirement: Transactional preset edits
`apply_animation_preset` MUST work through existing standalone edit and ordered batch paths with optimistic revision checks, project locking, track locks, stable missing-reference failures and one atomic project/history publication. Batch application MUST resolve an earlier item's `@alias` and MUST NOT be an ID-creation operation. The final candidate MUST undergo existing channel/reference and inherited/extended safety preflight before commit. Any failure MUST publish none of the batch, return no committed alias mapping and preserve current state, history, revision, existing drafts and managed resources. Existing interrupted-transaction recovery and post-commit warning semantics MUST remain unchanged.

#### Scenario: Apply to an earlier creation alias
- **WHEN** a batch creates a compatible item with `resultAlias`, then applies a valid preset to `@alias`
- **THEN** the alias resolves to the created ID, all edits publish as one revision and one undo entry, and undo/redo restore the complete state including provenance

#### Scenario: Roll back after earlier valid operations
- **WHEN** an ordered batch has valid earlier edits followed by a colliding or invalid preset, or later candidate safety fails
- **THEN** the complete batch fails with the established typed error and no project/history/draft/resource change

#### Scenario: Preserve canonical stale, missing and lock failures
- **WHEN** the expected revision is stale, item/asset is missing, track is locked, an alias is referenced before creation, or the preset declares `resultAlias`
- **THEN** core preserves the existing respective `REVISION_CONFLICT`, `ITEM_NOT_FOUND`/`ASSET_NOT_FOUND`, `TRACK_LOCKED` or `VALIDATION_FAILED` code and retryability without a mutation

#### Scenario: Reject before publishing legacy migration
- **WHEN** a supported legacy project and retained history require migration and a preset or preset-containing ordered batch is rejected for invalid input, collision, alias, track/target, or complete-candidate safety
- **THEN** the established typed error is returned and this request changes no current/history/draft bytes, revision, or managed resource bytes

#### Scenario: Accept legacy migration and edit together
- **WHEN** a valid preset or preset-containing ordered batch edits supported legacy current/history/drafts
- **THEN** migration and the complete edit publish through the existing journal as one revision and undo entry, and undo/redo/reopen preserve migrated snapshots and provenance

#### Scenario: Preserve preset publication fault semantics
- **WHEN** resource publication or transaction persistence fails before journal commit, or a checkpoint fails after the journal commits
- **THEN** pre-commit failure preserves authoritative documents and preexisting managed bytes and removes only new uncommitted resources, while committed recovery-pending behavior and reopen recovery remain unchanged

#### Scenario: Preserve preexisting resource destination entries
- **WHEN** a planned managed asset or font destination has a dangling symlink or other invalid preexisting entry during a preset transaction
- **THEN** the request fails with the established integrity error before overwriting that entry, and rollback preserves its link target and all preexisting project/history/draft/resource bytes

#### Scenario: Preserve migration asset font selection behavior
- **WHEN** a preset-containing request selects an extensionless content-addressed migration asset as an explicit or configured-default font source
- **THEN** the existing font owner returns nonretryable DEPENDENCY_UNAVAILABLE rather than accepting a pinned fallback, and all project/history/draft/resource bytes remain unchanged
