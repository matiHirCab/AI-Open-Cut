## ADDED Requirements

### Requirement: Shared marker-resolved render timing
Frame preview, range preview, draft materialization and export SHALL consume one editor-core evaluated item timing for marker-relative starts. Marker records MUST not draw or emit audio. All preexisting numeric-only projects MUST retain their previous pixel and audio output, and invalid persisted marker timing MUST fail before destination inspection or render side effects.

#### Scenario: Compare render intents
- **WHEN** a marker moves an item's effective start and the same revision is rendered through preview and export
- **THEN** each intent uses the same resolved half-open item interval and repeated output is deterministic

#### Scenario: Reject invalid persisted timing before output
- **WHEN** a persisted expression is missing, ambiguous or out of bounds
- **THEN** every render intent returns a typed error before creating or overwriting output
