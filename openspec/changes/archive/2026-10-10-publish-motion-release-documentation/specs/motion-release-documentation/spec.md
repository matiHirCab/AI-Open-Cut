## ADDED Requirements

### Requirement: Discoverable canonical current release documentation
Release documentation SHALL distinguish headless protocol1, current persisted schema44, individually reported feature support, catalog/example versions and artifact content policy2. It MUST link canonical owners for public operations, coordinates, timing, ordering, fallback and errors rather than introduce a parallel model catalog. Historical activation-schema and fixture-only statements SHALL remain clearly historical. README and bridge guidance SHALL expose this entry point with valid repository-local links.

#### Scenario: D1 Discover current versions and owners
- **WHEN** a reader follows the release entry point and executes its example against the current source or default compiled MCP bridge
- **THEN** actual status reports protocol1/schema44 and required editor capabilities, documented current tool/alias names are discoverable and all repository-local owner/example links resolve

#### Scenario: D2 Refuse stale or incomplete teaching metadata
- **WHEN** the private example format, required case identity/count, error claim or runtime binding is missing, unknown, duplicated or inconsistent
- **THEN** example verification fails before accepting incomplete evidence, without changing a public contract or adding transport-side domain rules

### Requirement: Executable standalone and aliased authoring examples
The same checked-in private teaching corpus MUST execute through actual source and default compiled MCP clients. It SHALL demonstrate standalone definition creation and ordered aliased atomic definition/slot/instance edits with durable returned IDs, one committed revision/undo step, history and disk reopen. Genuine schema-valid domain-invalid input, missing reference, stale revision and late-batch failure SHALL produce existing catalog errors/retryability and preserve exact project/history bytes. No mock client, new public operation, weakened deadline or duplicated core validator SHALL substitute for owning-layer execution.

#### Scenario: D3 Author and restore the documented component workflow
- **WHEN** the standalone and atomic examples execute successfully through each actual MCP client
- **THEN** aliases resolve to persisted definition/instance IDs, slot/timing state matches the authored values, the batch contributes one revision/undo step and undo/redo/reopen restore the documented state

#### Scenario: D4 Exercise the complete documented refusal corpus
- **WHEN** a definition cycle, missing definition, stale writer or failure after valid aliased edits is submitted
- **THEN** existing INVALID_ARGUMENT/ITEM_NOT_FOUND/REVISION_CONFLICT behavior and retryability are observed while both authoritative files remain byte-identical

### Requirement: Accurate migration and operational guidance
Documentation MUST identify editor-core as the atomic current/retained-history migration owner under the project lock, preserve no-downgrade/future-schema refusal, and identify canonical verification evidence. It SHALL distinguish editor readiness, optional renderer/filter support and independent speech/transcription prerequisites, process-local jobs/resources/tokens, metadata-by-default artifact delivery and compatible speech/review aliases. Diagnostic paths SHALL not be confused with backend capability readiness; public errors/logs retain existing redaction semantics.

#### Scenario: D5 Explain schema compatibility without changing storage
- **WHEN** a reader needs to open older state or retained history, or encounters a future schema
- **THEN** the guide points to canonical atomic migration/future-refusal tests and current schema44 without introducing a migration, manual JSON rewriting or changed public/persisted fields

#### Scenario: D6 Distinguish optional readiness and compatibility
- **WHEN** dependencies or a requested optional renderer capability are unavailable, or an older alias/content flow is used
- **THEN** the guide describes existing subsystem readiness, stable refusal, retained alias semantics and explicit artifact reads without promising inference quality, silent renderer degradation or durable process-local handles

### Requirement: Preserve independent complete release acceptance
Documentation examples MUST remain distinct from #70 synthetic marker fixtures, #77's complete ten-group scene, #78 default-package proof and #79's all-platform canonical/native/stress gates. Existing preview/export semantics, fixtures, golden tolerances, schema44/protocol1, public/provider catalogs, deadlines and protected workflows SHALL remain unchanged. #15 remains report-only; creative rescue-video proposals and broad fixture cleanup remain separate. Publication SHALL require verified/archive/final protected gates and successful exact-head required CI, with unavailable evidence disclosed and no merge/deployment.

#### Scenario: D7 Publish a bounded verified documentation change
- **WHEN** current examples and documentation are prepared for review
- **THEN** actual source/default-package conformance and unchanged independent native acceptance pass, the change is verified/archived, all final required checks pass and the draft targets main with verified predecessor lineage
