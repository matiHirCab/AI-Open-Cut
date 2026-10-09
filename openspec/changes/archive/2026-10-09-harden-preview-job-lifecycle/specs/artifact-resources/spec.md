## ADDED Requirements

### Requirement: Bounded disposable bridge preview artifacts
The process-local bridge MUST retain at most32 completed PNG/MP4 preview artifacts and67108864 total declared encoded bytes by default, accepting both inclusive bounds and evicting oldest settled preview jobs before admitting a new output. An individually oversized preview MUST be safely discarded and fail with existing retryable JOB_REGISTRY_FULL. Failed disposal MUST remain charged and cannot silently allow capacity growth. Expiration, eviction, cancelled late completion and graceful close MUST delete only the originating process's confined renderer-generated preview files; exports, speech/analysis policies, project/media/history, foreign files and predecessor-process artifacts SHALL remain unchanged. Missing outputs SHALL be treated as already disposed; unsafe paths/symlinks and I/O failures MUST fail safely without unrelated deletion or sensitive diagnostics. Expired/evicted resource access MUST retain JOB_NOT_FOUND, metadata polling MUST not extend TTL and registry restart MUST remain cold.

#### Scenario: L4 Enforce exact artifact budgets
- **WHEN** successful preview completions meet/exceed count or byte capacity or a single preview exceeds byte capacity
- **THEN** exact bounds succeed, oldest settled outputs are disposed before replacement and oversized outputs fail safely without exceeding retained accounting

#### Scenario: L5 Dispose only owned expired outputs
- **WHEN** preview retention expires, eviction or close occurs, or a cancelled producer returns an output
- **THEN** its safe disposable file is removed, unavailable jobs retain JOB_NOT_FOUND and unrelated files/exports/project state remain intact

#### Scenario: L6 Fail closed during unsafe disposal
- **WHEN** output deletion encounters traversal, a symlink, a foreign filename/project or an I/O failure
- **THEN** unrelated files remain untouched, diagnostics stay safe and failed retained disposal cannot free charged capacity
