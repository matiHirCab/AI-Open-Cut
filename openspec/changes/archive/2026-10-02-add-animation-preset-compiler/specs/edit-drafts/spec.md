## ADDED Requirements

### Requirement: Bounded preset authoring boundary for drafts
For the initial versioned compiler milestone, core MUST reject `apply_animation_preset` in draft create/update operation lists and when materializing an injected persisted preset intent with non-retryable `INVALID_ARGUMENT`, before draft/resource writes or project/history publication. Existing accepted draft operations, draft format/version 2, revision conflicts, preview/rebase/commit/discard and recovery semantics MUST remain unchanged. A valid draft based on a committed preset project MUST use its saved primitives without compilation; existing raw channel edits in a draft MUST obey provenance clearing only in that candidate until commit.

#### Scenario: Reject new preset intent before draft side effects
- **WHEN** draft creation/update supplies a preset edit after other valid operations
- **THEN** core returns `INVALID_ARGUMENT`, creates no draft/resources, and preserves prior draft/project/history bytes and revisions

#### Scenario: Reject injected replay intent
- **WHEN** a persisted draft includes a preset application intent and a caller previews, rebases or commits it
- **THEN** core rejects without compiling or rewriting the draft or project/history/resources

#### Scenario: Preserve ordinary draft editing of saved output
- **WHEN** a current draft previews a committed preset project, then uses a valid low-level channel replacement and is committed or discarded
- **THEN** preview consumes saved channels, candidate replacement clears labels in isolation, commit publishes one undoable primitive/provenance change, and discard leaves the committed labels intact
