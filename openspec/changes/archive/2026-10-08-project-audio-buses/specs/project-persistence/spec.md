## ADDED Requirements

### Requirement: Atomic schema39 audio bus adoption
Opening a genuine supported schema1-through38 project SHALL atomically migrate current and every retained undo/redo generation to39 under the existing project lock and recoverable journal. Only schemaVersion and introduced default audioBuses SHALL change; optional explicit track routes MUST remain absent and every ID/revision/timestamp, asset/provenance/font/marker, item/role/ducking, resource and historical guard MUST remain exact. Pre39 raw project audioBuses presence and actual root/component track audioBusId presence, including null, MUST fail before defaults; dynamic user values with coincidentally named keys MUST remain valid. Schema39 MUST require explicit non-null complete audioBuses and validate every retained route/model before any publication. Future schemas MUST fail closed without rewriting. Failed routing requests against legacy sources MUST not publish migration; interrupted committed migration MUST recover one complete generation with existing PERSISTENCE_RECOVERY_PENDING behavior and no partial files/resources. Repeated valid39 reopen MUST not rewrite bytes.

#### Scenario: Adopt mixed legacy current and retained history
- **WHEN** supported current/undo/redo sources have role-based audio tracks and no introduced fields
- **THEN** all generations gain exactly the four default buses in one journaled adoption, previous data/resources remain exact and second reopen is byte-stable

#### Scenario: Reject malformed retained or premature fields
- **WHEN** any current/undo/redo source contains premature introduced fields, malformed39 buses/routes or future schema
- **THEN** opening fails with established errors before authoritative project/history/resource bytes change

#### Scenario: Preserve dynamically named user values
- **WHEN** valid legacy slot/user values contain audioBuses or audioBusId outside the actual project/track model envelopes
- **THEN** structural field guards preserve them without mistaken migration rejection

#### Scenario: Recover migration faults without mixed generations
- **WHEN** any original persistence fault phase interrupts migration or its recovery
- **THEN** established rollback/commit-warning/recovery rules yield a complete original or committed generation with every existing fault case retained

#### Scenario: Reject failed routing without publishing adoption
- **WHEN** a stale/invalid/missing-reference/locked routing mutation targets genuine legacy files
- **THEN** no migration or route escapes the failed request and authoritative current/history/draft/managed bytes remain unchanged
