## ADDED Requirements

### Requirement: Shared renderer-neutral publication preflight
Editor-core MUST expose one canonical validation-only scene preflight consumed by store orchestration and render evaluation. It MUST share occurrence clock, interval, inherited transform and complexity rules with render evaluation, validate complete retained projections before generated materialization, and perform no backend execution or artifact publication. Store MUST coordinate pure candidate validation and resource staging before durable publication; scene evaluation MUST NOT own persistence or filesystem policy. The reviewed store-to-evaluated_scene dependency MUST be documented and enforced with the canonical architecture map. Transports and providers MUST NOT duplicate these domain rules.

#### Scenario: Share an unsafe candidate rejection
- **WHEN** the same candidate is evaluated for mutation publication and rendering
- **THEN** both use the same canonical derived bounds and return matching typed domain failures, while publication preflight creates no generated copies or artifacts

#### Scenario: Enforce the reviewed dependency boundary
- **WHEN** module dependencies and preflight responsibility are checked
- **THEN** the architecture test accepts the documented store-to-evaluated_scene edge and continues rejecting outward dependencies and transport or provider domain-validation copies
