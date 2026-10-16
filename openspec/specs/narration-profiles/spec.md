# Narration Profiles Specification

## Purpose

Define bounded shared local narration and pronunciation profiles, explicit versioned application, and immutable generated speech intent through the existing provider-neutral workflow.

## Requirements

### Requirement: Shared bounded versioned local narration library
The core SHALL own a local versioned profile library shared by projects within one configured projects root. Profiles MUST have UUID identity, trimmed unique case-sensitive names of 1..80 Unicode scalars without controls, monotonically increasing versions starting at one, language, voice, positive finite speed and complete existing speech text options. At most 100 profiles SHALL exist. List/read MUST not invoke providers. Creation/update SHALL use the existing root lock and atomic durable publication; update MUST compare expectedVersion and reject stale writes with REVISION_CONFLICT without changing authoritative bytes. Unknown/future/corrupt/oversized library data or symbolic links MUST fail closed. The library SHALL reject duplicate trimmed literal pronunciation terms and preserve overlapping terms in caller order with no Unicode normalization, using existing scalar and pause limits.

#### Scenario: Reuse one profile across two projects
- **WHEN** two projects explicitly apply the same profile version
- **THEN** both receive identical resolved defaults and independent full request snapshots

#### Scenario: Update and stale writer
- **WHEN** one writer updates a profile and another submits its old version
- **THEN** the first publishes the next version and the second receives REVISION_CONFLICT without changing library or project state

#### Scenario: Reject invalid or unavailable storage
- **WHEN** profile values exceed limits, contain duplicate terms or invalid names, or library storage is corrupt, oversized, a link or a future version
- **THEN** a typed nonmutating failure occurs before publication and existing authoritative data remains unchanged

### Requirement: Explicit deterministic profile application
Headless and provider-neutral MCP SHALL expose create/read/list/update/apply. Application MUST pin identity and expectedVersion, resolve once, and return exact effective speech settings with identity/version/name attribution. Explicit language, voice, speed or whole textOptions SHALL replace the corresponding profile field; an explicit empty pronunciation list SHALL clear it. Missing fields SHALL use saved defaults. An unavailable profile/version MUST fail before inference; provider readiness and selected voice compatibility SHALL be checked by the existing bridge workflow before apply/generate succeeds. Application SHALL not edit a project or synthesize audio.

#### Scenario: Explicit field precedence and empty list
- **WHEN** application provides a speed override and complete textOptions containing an empty pronunciation list
- **THEN** those fields replace saved values while omitted language and voice retain profile defaults

#### Scenario: Missing version or voice
- **WHEN** application requests a missing profile, stale version, unavailable provider or unsupported saved voice
- **THEN** existing typed actionable lookup/conflict/readiness errors are returned without inference or project mutation

### Requirement: Legacy and immutable generated profile intent
Legacy speech sources without profiles MUST retain existing behavior. Explicit profile application through preview/generate/regenerate SHALL retain the exact resolved request independently from future library/provider mutations. Later profile updates MUST affect only future explicit applications; retained preview conflict retry, existing assets, item regeneration without a newly selected profile, undo/redo and reopen SHALL not consult the current library. Domain ownership, resource cleanup, locking, aliases, project revision conflicts and rollback MUST remain unchanged.

#### Scenario: Update after preview and retry
- **WHEN** a resolved preview is retained, its library profile changes and commit conflicts before retry
- **THEN** retry uses the retained audio and original exact resolved request/version without inference or reapplication

#### Scenario: Regenerate legacy or profiled item
- **WHEN** an existing speech item is regenerated without a new profile selection
- **THEN** persisted effective values and existing optional attribution are retained with explicit request overrides and the existing single-revision replacement semantics

#### Scenario: Cancel during provider preflight
- **WHEN** an existing speech generation is cancelled while its executable preflight awaits and a later request is queued
- **THEN** the cancelled generation rejects with JOB_CANCELLED before starting its worker request, and the successor completes without leaked output or blocked provider ownership
