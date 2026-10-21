## ADDED Requirements

### Requirement: Complete private normalization readiness publication
Private lifecycle verification MUST observe complete backend and descendant PID metadata before triggering cancellation, deadline or shutdown, preserve actual PID liveness/absence checks and retain all existing phase, case, output-integrity and time limits.

#### Scenario: Read concurrently with readiness publication
- **WHEN** a verifier observes readiness during native fixture publication
- **THEN** the published PID record is complete JSON identifying the actual backend and descendant, and incomplete publication cannot count as successful readiness

#### Scenario: Publication fails or never completes
- **WHEN** complete readiness is unavailable within the existing phase-entry bound
- **THEN** the case fails with owned-request cleanup and phase/outcome diagnostics instead of skipping the lifecycle proof

#### Scenario: Reject malformed published metadata
- **WHEN** a readiness record exists but contains malformed JSON
- **THEN** verification aborts and settles only its owned request, retains phase/outcome diagnostics and rethrows the original parse failure without accepting missing PIDs or retrying the lifecycle operation
