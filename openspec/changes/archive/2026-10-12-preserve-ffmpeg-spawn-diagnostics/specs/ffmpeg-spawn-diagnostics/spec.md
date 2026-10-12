## ADDED Requirements

### Requirement: Bounded sanitized final process startup diagnostics
Core MUST retain safe OS diagnostics when actual final FFmpeg process creation fails. Optional `ffmpegSpawn` SHALL contain only a closed normalized error kind, signed32 native code or null, and the producing platform. Kinds SHALL be not_found, permission_denied, invalid_input, invalid_filename, argument_list_too_long, resource_busy, out_of_memory or other; unlisted I/O kinds SHALL map to other. Platform SHALL be linux, macos, windows or other. The metadata MUST NOT contain error text, paths, arguments, environment values, project/media identifiers or private content; native codes MUST NOT be represented as child exit codes.

#### Scenario: Missing executable or denied access
- **WHEN** final process creation fails with missing-executable or access-denied I/O errors
- **THEN** the error identifies the normalized kind and available native code/producing platform without exposing executable or output paths, and child exit/stderr remain null

#### Scenario: Native code fallback and privacy
- **WHEN** process creation yields a platform-specific raw code, an unlisted I/O kind, or a custom error containing hostile private text without a native code
- **THEN** metadata preserves only the safe kind/native-code/platform values, unlisted kinds become other, missing codes remain null, and no private text is serialized

#### Scenario: Preserve bare-name lookup without a native code
- **WHEN** actual bare-name command lookup yields NotFound without a raw OS code
- **THEN** core MUST preserve the actual kind and nullable raw code without inventing an OS value, while a separate absolute missing-executable witness retains available native-code and no-progress evidence

### Requirement: Compatible typed public forwarding
Core/headless standalone and persistent worker/MCP errors and failed jobs SHALL preserve the same optional startup metadata from the owning core error. Existing FFMPEG_FAILED code, spawn stage, generic message, non-retryability, null child diagnostics, request meaning and project/protocol versions MUST remain unchanged. Ordinary, readiness, pipe and child-exit errors MUST retain their existing public shapes without fabricated startup metadata.

#### Scenario: Forward actual typed startup failure
- **WHEN** the typed core startup failure crosses standalone/worker headless error serialization and MCP error/job translation
- **THEN** each governed surface exposes equal bounded startup metadata with the existing stable error identity and no artifact

#### Scenario: Preserve errors that are not process creation failures
- **WHEN** readiness fails, a pipe is unavailable, a child exits unsuccessfully, or an ordinary domain error occurs
- **THEN** existing diagnostics and retryability remain unchanged and ffmpegSpawn is absent

### Requirement: Startup failure preserves publication and state safety
Startup diagnostics MUST NOT change failure precedence, cleanup, revision behavior or artifact ownership. A final startup failure SHALL remove temporary output/workspace data, publish no partial artifact and preserve an existing destination, project revision/state, retained history and drafts. Invalid/stale requests MUST fail with existing canonical errors before final execution and SHALL NOT gain startup metadata.

#### Scenario: Fail final startup through renderer intents
- **WHEN** an injected final-start failure reaches frame, range, draft or export renderer facades
- **THEN** cleanup removes temporary output/workspace residue, no artifact or completion is published, and authoritative state/history/draft/destination bytes remain unchanged

#### Scenario: Preserve stale revision and admission precedence
- **WHEN** a stale revision or invalid render request is submitted to otherwise failing process infrastructure
- **THEN** the existing revision/validation error wins before final execution, has no startup metadata and leaves state/artifacts unchanged

### Requirement: Governed additive startup error contract
Canonical fixture/catalog ownership and every listed Rust, headless, TypeScript/Zod and MCP consumer MUST remain synchronized under ADR0002. The unchanged complete contract gate SHALL exercise canonical valid and invalid metadata, actual typed producer serialization, exact predecessor additions and existing negative controls. Pending overlapping diagnostic changes MUST retain their independently reviewed fields and receive an exact-head semantic-union contract check before publication.

#### Scenario: Detect synchronized additions and malformed metadata
- **WHEN** valid canonical metadata or malformed kind/platform/code/extra-field witnesses reach governed consumers
- **THEN** all valid cases agree, malformed wire metadata is rejected, the optional addition projects exactly to the recorded predecessor, and unrelated drift remains rejected

#### Scenario: Preserve overlapping diagnostic lanes
- **WHEN** this change is combined in a disposable verification checkout with the exact pending budget-diagnostic head
- **THEN** both optional fields, existing catalog contracts and predecessor negative controls are preserved by the complete contract gate, with conflicts and independent resolution review recorded separately from publication

### Requirement: Truthful platform evidence and causal limits
Verification MUST distinguish actual OS startup tests, deterministic injected adapter failures, transport fixtures and actual native media regression evidence. Public documentation SHALL describe the confirmed diagnostic loss without claiming a path-length defect from the historical longer/shorter-root association. Any path-length causal claim MUST first have an authorized controlled Windows root comparison; unavailable private-project reproduction SHALL be recorded explicitly.

#### Scenario: Record cross-platform diagnostic evidence
- **WHEN** Linux, macOS and Windows required jobs exercise actual missing-executable and deterministic platform-code controls
- **THEN** reported results identify the exercised platform and retain no-publication/privacy evidence without presenting transport fixtures as successful native renders

#### Scenario: Document historical cause as unconfirmed
- **WHEN** this fix is described without the original private project/media and installed-build commit
- **THEN** documentation states the missing evidence and unconfirmed cause, and makes no path-length root-cause claim

#### Scenario: Preserve reviewed main domains during startup verification correction
- **WHEN** current main advances with approved color slots, persisted-schema migrations or another reviewed diagnostic domain
- **THEN** startup diagnostic reconciliation MUST preserve its declarations, fixtures, registrations and strict predecessor proofs, reject missing metadata and unrelated drift without changing pins, and receive independent resolution review before publication
