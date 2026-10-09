## MODIFIED Requirements

### Requirement: Deterministic interrupted-transaction recovery
The editor core MUST recover a valid interrupted transaction deterministically under the project lock before returning or mutating project state, MUST remove all managed transaction artifacts after successful recovery, and MUST fail closed with non-retryable `PROJECT_RECOVERY_FAILED` when recovery metadata is corrupt, unsupported, or inconsistent. Orphan transaction cleanup MUST identify only existing UUID-suffixed transaction temporary names before inspecting entry types. It MUST neither inspect nor remove unrelated render workspaces, outputs, or other entries; their concurrent disappearance MUST NOT prevent an overlapping state read. Entry inspection/removal failures for recognized transaction temporary names MUST retain existing fail-closed recovery errors, and recognized directories/symlinks MUST remain untouched.

#### Scenario: Recover every interrupted publication phase
- **WHEN** a project is opened after termination between any two persistence phases following the commit point
- **THEN** recovery publishes the transaction's project and history together, completes any recorded draft consumption, and removes managed transaction artifacts

#### Scenario: Repeat interrupted recovery
- **WHEN** recovery itself is interrupted and the project is opened again
- **THEN** replay converges on the same committed generation without duplicating a mutation or pairing history from another generation

#### Scenario: Reject irrecoverable metadata
- **WHEN** transaction recovery metadata has an unsupported version, invalid content, or a project identity inconsistent with its directory
- **THEN** opening fails with `PROJECT_RECOVERY_FAILED` without guessing, defaulting history, or rewriting the live project documents


#### Scenario: Overlap recovery with owned normalization cleanup
- **WHEN** cancellation removes a listed unrelated render workspace before recovery would inspect its entry type
- **THEN** recovery does not inspect that unrelated entry, succeeds without changing project/history and leaves unrelated published outputs untouched

#### Scenario: Preserve actual transaction failures and entry ownership
- **WHEN** a recognized UUID-suffixed transaction temporary entry cannot be inspected or removed, or is a directory or symlink
- **THEN** inspection/removal errors remain non-retryable PROJECT_RECOVERY_FAILED while directories/symlinks remain untouched and ordinary recognized files are durably removed

