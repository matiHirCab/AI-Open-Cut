## ADDED Requirements

### Requirement: Bounded real crash verification
Private restart verification MUST kill and settle each actual predecessor before checking cold successors, preserve every job-state/admission assertion, and complete within unchanged test and helper time limits.

#### Scenario: Repeated real predecessor crashes
- **WHEN** verification exercises queued, running and terminal predecessor jobs followed by repeated cold admission attempts
- **THEN** complete fixture records and confirmed predecessor death support all existing guard inventory and lookup assertions without additional retries or larger limits

#### Scenario: Prepare a self-contained crash fixture
- **WHEN** real crash verification prepares the original fixture and production dependencies
- **THEN** a separately bounded5000ms setup requires a successfully built self-contained JavaScript bundle, every existing successor starts cold with real production behavior, and unavailable or failed preparation fails without source fallback or larger controlled limits

#### Scenario: Fixture fails or the test ends during a live child
- **WHEN** setup, output completeness, launch or the existing watchdog fails, or a test ends with an owned fixture child
- **THEN** verification fails without fallback, closes owned child processes before deleting roots, and refuses successor launch during teardown while preserving diagnostics and all existing time bounds
