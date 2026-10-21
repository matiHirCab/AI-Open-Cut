## MODIFIED Requirements

### Requirement: Artwork metadata schema migration
Current and retained supported historical snapshots SHALL migrate deterministically under the existing project lock to schema48 with optional artwork derivation absent for legacy assets. Derivation-bearing snapshots MUST require schema48 and canonical asset integrity. Schema47 SHALL retain main's required nonnull narrationManifests array, exact native ownership/provenance and introduced-field guards; schema48 SHALL preserve that contract. Schema1..46 migration SHALL retain existing empty-manifest adoption and every earlier introduction guard. Old public requests, aliases, speech origins and stable errors MUST retain meaning; additive headless/CLI operations and optional MCP project-state metadata SHALL be governed by canonical fixture parity and exact predecessor inverse proofs. Unknown future schemas MUST fail closed without mutation. Unpublished artwork47 documents SHALL NOT be treated as an alternate schema47 dialect.

#### Scenario: Migrate retained legacy history
- **WHEN** schema46 or older supported project and retained undo/redo are opened and reopened, or genuine main schema47 generations containing valid narration ownership migrate
- **THEN** schema48 publication is atomic/idempotent, original assets, source/speech intent, narration identities/bindings and history are preserved and no derivation is fabricated

#### Scenario: Reject future or premature derivation
- **WHEN** a future schema, derivation metadata on schema47 or older, malformed/missing/null schema47 narration metadata, premature narration fields on schema46 or older, or an invalid retained snapshot is loaded
- **THEN** established typed failure preserves authoritative project/history/draft/media bytes without guessed downgrade, alternate-dialect detection, invented narration metadata or silent field removal

## ADDED Requirements

### Requirement: Combined artwork and narration integration conformance
Integrated schema48 SHALL preserve both approved feature contracts, both canonical headless operation additions and main's existing package operations exactly once, required narration and optional artwork MCP metadata, ownership inventories and current-schema consumers. Historical predecessor raw captures, hashes and strict inverse assertions MUST remain unchanged, with exact guarded integration inverses proving that only approved feature/version additions are removed at each stage. Ordinary package v1 componentSchemaVersion46 SHALL remain supported independently of project48. Capturing selected derived artwork into that package format SHALL fail VALIDATION_FAILED before any mutation until a separately approved format preserves transitive provenance. Ordinary selected recipes MUST remain usable when unrelated derived assets exist elsewhere.

#### Scenario: Preserve both features through transactions and history
- **WHEN** a genuine main47 project with authored narration is migrated, artwork is prepared and committed, then reopened, undone and redone
- **THEN** narration IDs, requests, spoken/display text, native bindings and media remain exact, artwork source/provenance/pixels survive their normal history transitions, and stale captured revisions retain REVISION_CONFLICT without partial publication

#### Scenario: Recover every exact predecessor without masking drift
- **WHEN** integrated catalogs are projected through explicit artwork/integration/narration/package inverse stages, including missing/duplicate additions, changed unrelated fields or altered historical pins as negative controls
- **THEN** valid paths recover complete independently pinned predecessor bytes/hashes and every malformed or unrelated drift fails rather than being erased or recaptured

#### Scenario: Preserve package compatibility without dropping artwork provenance
- **WHEN** ordinary schema46 package capture/import/apply runs in a schema48 project, or a capture selects a derived artwork asset
- **THEN** ordinary package contents/format marker and existing behavior remain valid, while selected derived artwork fails VALIDATION_FAILED without source/derivative/project/history/library mutation or provenance loss
