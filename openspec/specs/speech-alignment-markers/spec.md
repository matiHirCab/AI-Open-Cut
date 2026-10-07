# Speech Alignment Markers Specification

## Purpose
Generate bounded named timeline cues from validated speech alignment using core-owned policies and atomic publication.

## Requirements

### Requirement: Core-owned bounded alignment marker policies
Core SHALL expose a closed `none`, `sentence`, `selected_word`, and `all_word` policy through `speech_markers_generate`, resolving an existing audio-bearing asset, positive known duration, root or existing component scope, nonnegative JavaScript-safe startMs and optional non-null SpeechAlignment. Omitted alignment MUST use the asset's persisted speech alignment; explicit alignment MUST satisfy existing intrinsic and source-duration rules without altering quality or producer identity. None MUST create no markers and require no alignment; every other policy MUST reject missing alignment or empty requested granularity without fabricating timestamps. Selected_word MUST accept at most 4096 distinct zero-based valid word indices and produce markers in source order, independent of input index order. Selection, count and time validation MUST remain in core, including the existing 4096-marker total scope limit, safe offset plus segment start/end and component-duration bounds. No new provider inference or persisted schema SHALL be introduced.

#### Scenario: Generate sentence and word policies
- **WHEN** valid sentence/all-word or selected-word alignment is submitted for an audio asset
- **THEN** markers use exactly the selected supplied start timestamps plus placement offset in the requested scope, with selected indices ordered by source occurrence

#### Scenario: Preserve none and legacy speech insertion
- **WHEN** none is used or an existing speech insertion omits markerPolicy
- **THEN** no marker is generated, alignment is not required, and existing insertion results remain compatible

#### Scenario: Reject invalid or unsupported selection unchanged
- **WHEN** alignment is missing/malformed/outside the asset, the requested granularity is empty, selection indices repeat/are absent/out of range, or count/offset/component bounds are exceeded
- **THEN** the existing typed validation error is returned before publishing any marker, revision, history, alias or managed resource

#### Scenario: Reject missing source or scope
- **WHEN** the source asset is absent, lacks audio, lacks positive duration, or the component scope is absent
- **THEN** existing ASSET_NOT_FOUND, UNSUPPORTED_MEDIA, VALIDATION_FAILED or ITEM_NOT_FOUND respectively is returned without mutation

### Requirement: Deterministic unique generated cue names
Core SHALL derive generated names from each selected segment's text by retaining ASCII letters/digits, replacing other runs with underscore, trimming outer underscores, prefixing `speech_` for a non-letter first byte, falling back to `sentence_<ordinal>` or `word_<ordinal>` for empty results and truncating the ASCII base to 120 bytes. It MUST reserve every existing same-scope name and every name generated earlier in source order; a collision MUST select the smallest free suffix starting at `_2`. All resulting names MUST satisfy existing ASCII grammar, fit 128 bytes and be unique within the scope. Marker kind MUST remain cue. IDs MUST use existing stable marker identity across undo/redo/reopen; names and times MUST be deterministic for the same source and candidate, without imposing deterministic UUIDs.

#### Scenario: Name repeated or colliding words
- **WHEN** repeated source words collide with each other or existing same-scope names
- **THEN** generated names use deterministic smallest free numeric suffixes and remain unambiguous for marker-relative timing

#### Scenario: Normalize bounded arbitrary wording
- **WHEN** text contains punctuation, a numeric prefix, Unicode-only wording or an overlong ASCII name
- **THEN** generated names retain deterministic documented normalization/fallback, valid first character and bounded size

### Requirement: Transactional standalone and batch marker generation
Speech marker generation SHALL be an ordinary standalone edit and ordered timeline_batch_edit operation. It MUST resolve existing component-scope aliases, return generated changedIds in source order and permit a scalar resultAlias only for exactly one generated marker. Unknown/forward aliases or zero/multiple aliased results MUST preserve existing validation failure and atomic rollback. Standalone and batch edits MUST use existing optimistic revision, one revision/undo step, history retention, reopen and evaluated-scene behavior. A standalone none edit MUST retain ordinary edit revision semantics. Markers MUST not create visual/audio instructions; binding existing item timing to a generated unique name MUST use the same evaluated start in preview/export.

#### Scenario: Bind one generated marker through an alias
- **WHEN** a batch creates a component, generates one local marker with a result alias and subsequently updates that marker through the alias
- **THEN** all operations resolve in order and commit together as one revision/history entry

#### Scenario: Reject multi-result or forward alias transaction
- **WHEN** generation is given a scalar result alias for zero/multiple markers or references an unknown/forward component alias, or a later batch operation is invalid
- **THEN** the entire candidate, alias results and durable generation remain unchanged

#### Scenario: Conflict and exact lifecycle
- **WHEN** a stale standalone/batch request is retried at the current revision, then undone, redone and reopened
- **THEN** the stale request returns retryable REVISION_CONFLICT unchanged and valid transitions retain exact marker IDs, names and times

#### Scenario: Shared rendering after marker binding
- **WHEN** an existing visual item references a generated marker name
- **THEN** preview/export share its resolved timing while unbound markers leave existing render output unchanged

### Requirement: Atomic speech insertion marker policy
Generated speech insertion and speech preview insert placement SHALL accept optional default-none markerPolicy and forward it unchanged into core's existing commit_generated_asset transaction. Non-none policies MUST consume the retained generated asset's alignment using the item's root startMs and MUST atomically publish asset, item and markers in one revision/history entry. Invalid policy/alignment/count/naming/placement or persistence failure MUST preserve tracked rollback and existing cleanup without partially publishing media, marker or history. Conflicted preview retry MUST reuse the retained owned audio/alignment and policy without rerunning inference. Existing speech result shapes, replacement/regeneration, transcription-caption commit and omitted-policy clients MUST retain current behavior.

#### Scenario: Insert aligned preview with selected cues
- **WHEN** an aligned preview is committed with selected_word at a current revision
- **THEN** its audio asset/item and selected root cues are published atomically at the item's offset and restored exactly by undo/redo/reopen

#### Scenario: Retry aligned insertion after conflict
- **WHEN** a marker-bearing preview commit conflicts and is retried within retention
- **THEN** the owned original audio/alignment and policy are reused without inference and no markers escaped the conflict

#### Scenario: Roll back generated resources on marker failure
- **WHEN** marker validation or persistence fails during generated speech insertion
- **THEN** no partial asset/item/marker/revision/history remains and existing owned cleanup applies while preserving retry behavior
