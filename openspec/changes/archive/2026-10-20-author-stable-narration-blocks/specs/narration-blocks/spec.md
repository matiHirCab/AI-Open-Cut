## ADDED Requirements

### Requirement: Bounded deterministic narration planning
Core SHALL plan at most32 ordered blocks from exact source text of at most32768 UTF8 bytes using pinned UAX29 sentence suggestions or explicit strictly increasing Unicode-scalar end boundaries covering the full source. Every nonblank display span MUST retain its exact original slice and paragraph ordinal, stable UUID, manifest UUID and immutable resolved profile identity/version/settings. Every block's original/display and resolved spoken text MUST fit4096 UTF8 bytes and min(4096, advertised provider maxTextCharacters) scalars before synthesis. Automatic overlong sentences MUST fail with actionable manual-boundary guidance; no truncation, estimated alignment or invented timestamps is permitted. Blank text, empty spans, invalid/duplicate boundaries, missing/stale profiles or invalid budgets SHALL fail with existing typed errors without project mutation.

Effective request trimming for new narration blocks uses the ECMAScript String.trim whitespace/line-terminator set, matching the existing bridge resolver: U+FEFF is trimmed and U+0085 is retained beside meaningful text. Core planning and immutable span validation use that same explicitly pinned set; original display slices remain exact. Existing legacy speech preparation/blank validation, profile resolution and #208 remain unchanged. Literal manual-boundary FEFF/NEL cases are shared by native planning and bridge preparation tests.

#### Scenario: Plan punctuation paragraphs and abbreviations
- **WHEN** source includes punctuation, abbreviations, paragraph breaks, emoji or combining characters
- **THEN** automatic suggestions and manual scalar boundaries produce fixture-defined source order, exact wording and profile snapshots without timestamps

#### Scenario: Reject blank boundary and expansion limits
- **WHEN** source or a manual span is blank, boundaries are duplicate/reversed/outside source, an automatic sentence is oversized, or pronunciation expansion exceeds resolved limits
- **THEN** validation fails before synthesis or project publication without altering any existing plan or project

#### Scenario: Pin a profile before later library edits
- **WHEN** the selected profile is missing/stale at planning or is updated after a valid plan is created
- **THEN** missing/stale selection fails and an existing plan preserves its exact originally resolved effective request independently of future library state

### Requirement: Existing provider and retained preview reuse
The bridge SHALL preflight all block requests and resolved spoken segments through existing provider capabilities and text preparation before synthesis. It MUST preserve exact normalization/pronunciation/chunking semantics and retain completed previews through the ordinary ownership/retention mechanism. Partial generation/cancellation MUST never publish a manifest or timeline mutation; completed previews SHALL remain available for explicit use until original expiry. No new provider/model download, risk analysis or fabricated alignment SHALL be introduced.

#### Scenario: Generate a complete preview group
- **WHEN** every planned block passes provider preflight and synthesis
- **THEN** ordered completed previews retain exact request/profile/spoken snapshots, actual durations and any actual returned alignment without project mutation

#### Scenario: Reject unsupported provider before synthesis
- **WHEN** any block uses unavailable voice/language/speed or exceeds actual provider/resolved limits
- **THEN** the entire group fails preflight without starting inference or publishing media

#### Scenario: Cancel or fail partway through generation
- **WHEN** synthesis fails or is cancelled after some blocks complete
- **THEN** no manifest/native insertion is published and the completed tokens remain explicitly reusable within original retention without rerunning completed inference

### Requirement: Atomic approved narration publication
One complete approved distinct preview set SHALL publish its generated audio assets, independently identified native items and ordered manifest through one core transaction at the expected project revision. Actual durations and explicit startMs SHALL determine checked sequential placement; owned captions/cues SHALL be opt-in, generated only from real supported supplied alignment and explicitly referenced. Track locks, managed-resource bounds, staged migrations, aliases, failure cleanup, one revision/undo step and recoverable publication semantics MUST remain unchanged. Core MUST validate plan structure, source spans, unique IDs, exact generated request/profile correspondence and every resulting reference before publication.

#### Scenario: Publish independently addressable blocks
- **WHEN** a complete valid preview set is committed at the current revision
- **THEN** one revision atomically contains all native audio/items and blocks with stable IDs, exact source/display/spoken intent and explicit owned caption/cue references

#### Scenario: Roll back invalid late publication
- **WHEN** the last block is invalid, an ID/request mismatches, requested alignment is missing, placement overflows, a track is locked or persistence fails before its commit point
- **THEN** no partial manifest, asset, item, history, alias or managed-resource publication remains

#### Scenario: Reuse previews after conflict
- **WHEN** complete publication conflicts with a newer project revision and the caller explicitly retries within retention
- **THEN** no inference is repeated and only the successful single transaction publishes all original stable block IDs and retained provenance

### Requirement: Exclusive bounded preview group ownership
Atomic publication SHALL reserve all distinct retained tokens before awaiting core and prevent concurrent consumption/discard/expiry while leased. Tokens MUST be consumed only after authoritative commit acknowledgement; precommit failure MUST restore original remaining retention, and postcommit cleanup failures MUST remain warnings. An uncertain transport outcome MUST quarantine all group tokens against ordinary/group consumption, discard and expiry. Explicit read-only reconciliation SHALL acknowledge only an exactly matching current manifest; absence/mismatch SHALL retain uncertainty and prohibit replay. Abandon/close MUST await active calls, permanently retire tokens and clean only owned temporary outputs without claiming rollback. No uncertain group SHALL ever be republished.

#### Scenario: Reject concurrent duplicate or expired use
- **WHEN** a group repeats a token, contains expired/missing tokens, or overlaps an active lease
- **THEN** it fails before core publication and does not disturb unrelated valid previews

#### Scenario: Settle expiry and cleanup around publication
- **WHEN** retention expires during an active commit or temporary cleanup fails after acknowledgement
- **THEN** the active transaction settles before owned cleanup and committed results remain authoritative with explicit warnings where needed

#### Scenario: Preserve uncertain acknowledgement truth
- **WHEN** core transport fails without a trustworthy publication acknowledgement
- **THEN** the workflow reports uncertainty and retains bounded metadata for inspection without replaying inference or automatically republishing

### Requirement: Stable native ownership and explicit detachment
Committed manifests SHALL have at most32 manifests and256 live blocks per project with globally unique block UUIDs, ordered nonoverlapping source spans, exact immutable profile/request/wording and existing correctly typed root item/asset/caption/cue references. Legacy operations that invalidate bindings MUST fail atomically. Explicit detach_narration_block SHALL remove only selected ownership metadata while preserving native content and every unrelated block byte/ID/order/provenance; removing the last block SHALL remove the empty manifest. Frozen audio track/reference/start/duration/sourceIn, owned caption text/timing and cue name/time MUST match native content; presentation-only changes preserving those snapshots SHALL remain valid. Native duplication SHALL create unowned content; splitting or moving owned intervals SHALL fail until explicit detach. Remaining spans MAY have gaps after explicit detachment, with order and exact wording preserved. No implicit resegmentation/profile propagation or duration-changing sentence repair SHALL occur.

#### Scenario: Preserve identity through reopen and history
- **WHEN** a multi-block narration is reopened, undone and redone after the profile library becomes unavailable
- **THEN** exact block/native IDs, original/display/spoken intent and supplied ownership survive without consulting providers or the library

#### Scenario: Reject invalidating legacy edits
- **WHEN** legacy deletion/regeneration/replacement would remove or change an owned binding or owned caption/cue
- **THEN** core rejects the whole candidate with existing validation semantics and all unrelated state/history/resources stay unchanged

#### Scenario: Detach one block independently
- **WHEN** a caller explicitly detaches one block and then uses a valid legacy edit on its native content
- **THEN** only that ownership entry changes, unrelated blocks retain exact identity/provenance, and undo/redo/reopen preserve each committed generation

### Requirement: Governed additive source and headless contracts
Additive typed planning, complete commit and detachment operations plus the bounded persistent source CLI SHALL agree with canonical narration-blocks-v1 fixtures and every governed Rust/TypeScript consumer. Existing MCP/provider catalogs and operations SHALL retain their meaning; no installed GUI readiness claim is implied. Malformed envelopes SHALL fail closed and source shutdown MUST await active cancellation/ownership cleanup.

#### Scenario: Consume shared positive and negative fixtures
- **WHEN** contract parity runs across native and bridge declarations
- **THEN** both consume the canonical valid/invalid planning, manifest and operation cases and retain predecessor compatibility with deliberate schema transition evidence

#### Scenario: End a source session during inference
- **WHEN** bounded JSON-lines input is malformed/oversized or a live session receives cancellation/shutdown
- **THEN** input fails typed without publication and active work plus owned preview/core processes settle through existing lifecycle boundaries

### Requirement: Bounded metadata and source session
Core and bridge SHALL enforce the exact serialized UTF8/count budgets in wire-plan.md:512KiB per plan/manifest,4MiB project manifest metadata,128KiB effective request,16MiB native/CLI input,1MiB CLI response,32 spoken segments/block,100 total created caption/cue references/group,4 session groups and128 retained group tokens. Byte counting MUST stop at the budget; rejection MUST precede media import/publication without truncation or partial output. Raw source and provider limits remain independently required.

#### Scenario: Reject envelope and retained-group overflow
- **WHEN** any byte/count bound is exceeded, including large pronunciation/alignment metadata or a fifth live group
- **THEN** typed nonpublishing failure preserves existing project/preview ownership without unbounded buffering, partial JSON output or silent truncation
