## ADDED Requirements

### Requirement: Atomic schema40 sound-library adoption
Opening genuine supported schema1..39 current and retained undo/redo generations SHALL atomically adopt schema40 under the existing project lock/recoverable journal, adding only an empty soundDefinitions registry while preserving every established migration rule, bus/track route, asset/provenance/font/marker, ID/revision/timestamp, role/ducking/item and resource. Actual pre40 top-level soundDefinitions presence, including null, MUST fail before defaults; dynamically named slot/user values outside that model field MUST remain valid. Schema40 MUST require a present nonnull array and validate every current/retained registry before publication. Unknown future versions MUST fail closed without rewriting. Interrupted committed migration SHALL recover one complete generation under existing PERSISTENCE_RECOVERY_PENDING rules; failed registration SHALL not publish adoption or speculative resources. Valid40 reopen MUST not rewrite bytes.

#### Scenario: Adopt mixed supported generations
- **WHEN** genuine supported current/undo/redo sources have no introduced registry field
- **THEN** all generations gain exactly an empty registry and preserve prior values/resources through one adoption, with byte-stable repeated reopen

#### Scenario: Reject premature malformed or future registries
- **WHEN** any current/retained source has premature registry presence, null/missing40 array, invalid definitions/references or future schema
- **THEN** established typed errors reject before authoritative documents/resources change

#### Scenario: Preserve dynamically named values
- **WHEN** valid legacy user/slot values contain soundDefinitions outside the project model envelope
- **THEN** structural guards preserve those values without mistaken migration rejection

#### Scenario: Recover every existing persistence phase
- **WHEN** any original fault phase interrupts adoption/registration or its recovery
- **THEN** established rollback/committed-warning/replay ownership produces a complete original or committed generation with every prior fault case retained

#### Scenario: Reject failed legacy registration without adoption
- **WHEN** stale/invalid/missing-reference registration or draft commit targets genuine legacy sources
- **THEN** current/history/draft/resource bytes remain authoritative and no empty registry, media copy or partial state escapes the failed transaction
