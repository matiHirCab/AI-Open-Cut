## ADDED Requirements

### Requirement: Atomic schema43 explicit bus ducking adoption
Opening supported genuine schema1..42 current and retained undo/redo generations SHALL atomically adopt43 under existing lock/recoverable journal, preserving all prior migration guards/defaults and every genuine DSP/event/definition/route/ID/revision/timestamp/asset/provenance/font/marker/role-ducking/resource value. No ducking SHALL be invented. Actual pre43 bus ducking presence including null MUST fail before defaults, while dynamically named user values outside bus envelopes remain valid. Schema43 null/malformed/mixed invalid retained controls, source-mismatched drafts and future44 MUST fail without rewriting. Source-matched drafts,18 publication phases and complete interrupted-generation recovery SHALL remain canonical; failed edits MUST NOT publish adoption and valid43 reopen MUST NOT rewrite bytes.

#### Scenario: Adopt every supported current and retained source
- **WHEN** a genuine supported1..42 current/undo/redo project with existing DSP/event/draft resources is opened
- **THEN** all snapshots migrate atomically with exact prior content and no invented ducking

#### Scenario: Reject malformed retained or interrupted generations safely
- **WHEN** premature/null/future controls, mixed invalid history, source-mismatched drafts or publication faults occur
- **THEN** original precommit bytes remain unchanged or durable journals recover one complete target generation without partial resources
