## MODIFIED Requirements

### Requirement: Single-revision commit and independent discard
Committing a draft SHALL transactionally apply all draft operations as one project revision and one history entry and consume the draft as part of the committed outcome, while discarding SHALL remove only the draft without changing the project. A repeated commit attempt for the same consumed draft MUST NOT apply its operations as another revision. If the project and history generation commits but draft cleanup remains pending, the commit SHALL return the committed revision with stable `DRAFT_CLEANUP_FAILED` and `PERSISTENCE_RECOVERY_PENDING` warnings rather than report rejection.

#### Scenario: Commit a valid draft
- **WHEN** a current draft is committed successfully
- **THEN** all candidate operations are published as one project revision, one history entry is retained, and the consumed draft is removed

#### Scenario: Retry an interrupted draft commit
- **WHEN** a caller retries a draft commit after termination or an I/O failure at any persistence phase
- **THEN** recovery either completes the single committed revision or preserves the uncommitted draft, and the draft operations are never applied twice

#### Scenario: Report committed draft cleanup failure
- **WHEN** project and history publication commits but the consumed draft cannot yet be removed
- **THEN** the commit returns the committed revision with `DRAFT_CLEANUP_FAILED` and `PERSISTENCE_RECOVERY_PENDING`, and later project open deterministically completes cleanup

#### Scenario: Discard a draft
- **WHEN** a caller discards an existing draft
- **THEN** the draft is removed and the project state and revision remain unchanged
