## ADDED Requirements

### Requirement: Atomic schema31 retained-clock migration
Core MUST persist retained animation clocks only in schema31. Supported schema1–29 documents MUST migrate deterministically without changing source animation/provenance/media records, including root tracks, component definitions and every retained undo/redo snapshot, under the existing project lock and atomic publication protocol. Pre31 clock records, reserved schema30 and unknown future schemas MUST fail closed without modifying disk/current/history. Schema30 remains reserved for independent motion-pack provenance; integration MUST add a validated30→31 adapter preserving tagged provenance and all history rather than relabeling or dropping pack metadata.

#### Scenario: Migrate current components and retained history
- **WHEN** a schema29 project with component animations and retained undo/redo snapshots is reopened
- **THEN** every snapshot becomes schema31 atomically, absent clocks preserve original output and a subsequent edited reopen restores exact retained clocks

#### Scenario: Fail closed and retain disk state
- **WHEN** current or retained history contains malformed clocks, a pre31 clock, reserved30 or a future version, or staged migration publication fails
- **THEN** migration fails with the established non-retryable error and leaves the complete persisted bundle unchanged or recoverable through the existing atomic journal protocol
