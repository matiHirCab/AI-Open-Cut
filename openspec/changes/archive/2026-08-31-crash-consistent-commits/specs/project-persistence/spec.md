## MODIFIED Requirements

### Requirement: Serialized durable persistence
Project mutations, project creation, and migrations MUST execute while holding the project lock, and each logical write of project state plus retained history MUST use one recoverable transaction whose durable commit point identifies a single authoritative generation. Each persisted JSON document SHALL be published through a synchronized temporary file and atomic replacement so readers never observe a partially written document.

#### Scenario: Publish a project generation
- **WHEN** a validated mutation commits new project state and retained history under the project lock
- **THEN** every subsequent locked read observes the project and history from that committed generation rather than a mixed pair

#### Scenario: Reject before the commit point
- **WHEN** persistence fails before the transaction commit point is durably published
- **THEN** the mutation fails and the prior project and history generation remains authoritative

#### Scenario: Interrupt after the commit point
- **WHEN** persistence is interrupted after the transaction commit point but before every destination is materialized
- **THEN** the target generation remains recoverable and the mutation is not reported as rejected

## ADDED Requirements

### Requirement: Deterministic interrupted-transaction recovery
The editor core MUST recover a valid interrupted transaction deterministically under the project lock before returning or mutating project state, MUST remove all managed transaction artifacts after successful recovery, and MUST fail closed with non-retryable `PROJECT_RECOVERY_FAILED` when recovery metadata is corrupt, unsupported, or inconsistent.

#### Scenario: Recover every interrupted publication phase
- **WHEN** a project is opened after termination between any two persistence phases following the commit point
- **THEN** recovery publishes the transaction's project and history together, completes any recorded draft consumption, and removes managed transaction artifacts

#### Scenario: Repeat interrupted recovery
- **WHEN** recovery itself is interrupted and the project is opened again
- **THEN** replay converges on the same committed generation without duplicating a mutation or pairing history from another generation

#### Scenario: Reject irrecoverable metadata
- **WHEN** transaction recovery metadata has an unsupported version, invalid content, or a project identity inconsistent with its directory
- **THEN** opening fails with `PROJECT_RECOVERY_FAILED` without guessing, defaulting history, or rewriting the live project documents

### Requirement: Unambiguous acknowledged mutation outcome
The editor core SHALL report a mutation as rejected only before its durable transaction commit point, and SHALL return the committed revision with stable `PERSISTENCE_RECOVERY_PENDING` warning when post-commit materialization remains for deterministic recovery.

#### Scenario: Report post-commit materialization failure
- **WHEN** the transaction commit point is durable but project or history materialization cannot finish before returning to the caller
- **THEN** the result identifies the committed revision and includes `PERSISTENCE_RECOVERY_PENDING` rather than returning a mutation error

#### Scenario: Access after a recovery warning
- **WHEN** a caller accesses the project after receiving `PERSISTENCE_RECOVERY_PENDING`
- **THEN** the core finishes recovery under the lock before evaluating the new request against the committed revision
