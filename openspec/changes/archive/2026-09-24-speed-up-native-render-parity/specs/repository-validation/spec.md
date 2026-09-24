## ADDED Requirements

### Requirement: Protected optimized golden command
Repository policy MUST pin the exact optimized native golden command inside the existing required Render parity step and MUST keep all other protected step commands, approved environments, dependency ordering, failure propagation, strict report validation, and report publication guarantees unchanged.

#### Scenario: Accept the reviewed optimized command
- **WHEN** the Render parity workflow contains the approved optimized golden command and otherwise matches the protected policy
- **THEN** CI policy validation accepts the workflow without weakening the required render or foundation gates

#### Scenario: Reject altered render evidence
- **WHEN** the golden command is removed, made optional, changed to an unapproved profile or target, or moved outside the protected step
- **THEN** CI policy validation rejects the workflow before it can satisfy the protected gate
