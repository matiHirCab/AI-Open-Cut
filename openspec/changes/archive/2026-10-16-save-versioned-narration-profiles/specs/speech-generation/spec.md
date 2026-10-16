## ADDED Requirements

### Requirement: Versioned narration attribution compatibility
Persisted schema46 speech requests SHALL optionally contain resolved profile identity/version/name beside their exact effective fields. Legacy current state and retained undo/redo snapshots SHALL migrate deterministically under existing locks and transactions without inventing attribution or changing effective settings. Metadata declared on an older schema, malformed attribution and unknown future versions MUST fail closed before authoritative publication. Reopen/history/render SHALL not require the profile library or provider. Canonical public and persisted contracts and all governed consumers MUST agree.

#### Scenario: Reopen profiled speech independently
- **WHEN** profiled generated speech is committed and reopened after its library profile is edited or unavailable
- **THEN** current and retained history preserve the original exact effective request and profile attribution

#### Scenario: Migrate legacy history or reject future metadata
- **WHEN** a legacy project with undo/redo history is opened, or incompatible future/older-version metadata is supplied
- **THEN** valid legacy snapshots migrate atomically without new attribution and incompatible snapshots fail without partial publication
