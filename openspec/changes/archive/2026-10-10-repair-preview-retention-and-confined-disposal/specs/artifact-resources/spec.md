## MODIFIED Requirements

### Requirement: Bounded disposable bridge preview artifacts
The process-local bridge MUST retain at most32 completed PNG/MP4 preview artifacts and67108864 total declared encoded bytes by default, accepting both inclusive bounds and evicting oldest settled preview jobs before admitting a new output. An individually oversized preview MUST be safely discarded and fail with existing retryable JOB_REGISTRY_FULL. Failed disposal MUST remain charged and cannot silently allow capacity growth. Expiration, eviction, cancelled late completion and graceful close MUST delete only the originating process's confined renderer-generated preview files; exports, speech/analysis policies, project/media/history, foreign files and predecessor-process artifacts SHALL remain unchanged. Missing outputs SHALL be treated as already disposed; unsafe paths/symlinks and I/O failures MUST fail safely without unrelated deletion or sensitive diagnostics. Expired/evicted resource access MUST retain JOB_NOT_FOUND, metadata polling MUST not extend TTL and registry restart MUST remain cold.

The bridge SHALL reserve every admitted unsettled preview producer against preview count before dispatch, including visibly cancelled/expired producers, and SHALL settle required owned eviction before invoking a replacement producer. Failed disposal debt MUST remain owned and charged and MUST prevent further preview producer dispatch until safely settled; retries MUST NOT produce new previews while debt remains. Inclusive successful-retention count/byte limits SHALL remain unchanged. An already-produced oversized or failed output that cannot be removed SHALL remain explicit cleanup debt and MUST NOT be forgotten or treated as successful retention. Nonpreview work SHALL remain independently usable. Close MUST settle producers, report unsuccessful cleanup with existing safe errors, retain debt and permit cleanup-only retry without reopening admission.

#### Scenario: L4 Enforce exact artifact budgets
- **WHEN** successful preview completions meet/exceed count or byte capacity or a single preview exceeds byte capacity
- **THEN** exact bounds succeed, oldest settled outputs are disposed before replacement and oversized outputs fail safely without exceeding retained accounting

#### Scenario: L5 Dispose only owned expired outputs
- **WHEN** preview retention expires, eviction or close occurs, or a cancelled producer returns an output
- **THEN** its safe disposable file is removed, unavailable jobs retain JOB_NOT_FOUND and unrelated files/exports/project state remain intact

#### Scenario: L6 Fail closed during unsafe disposal
- **WHEN** output deletion encounters traversal, a symlink, a foreign filename/project or an I/O failure
- **THEN** unrelated files remain untouched, diagnostics stay safe and failed retained disposal cannot free charged capacity


#### Scenario: L7 Refuse replacement production during persistent disposal failure
- **WHEN** retained previews fill the count or byte budget and required disposal persistently fails while sequential or overlapping replacement requests arrive
- **THEN** the replacement producers are not dispatched, reservations/debt remain charged, safe retryable JOB_REGISTRY_FULL is preserved and no additional replacement files are produced

#### Scenario: L8 Recover debt and retry graceful close
- **WHEN** cancelled, oversized or failed work leaves owned disposal debt, later disposal becomes possible, or close is retried after cleanup failure
- **THEN** outstanding ownership is preserved, further preview production remains blocked until debt settles, safe cleanup failure is observable, recovery releases charge only after actual deletion and successful close leaves no owned preview debt

#### Scenario: L9 Anchor deletion throughout ancestor replacement
- **WHEN** an actor replaces or renames a checked project/previews ancestor at the deletion boundary, or introduces a symlink/reparse target
- **THEN** native ownership anchoring either safely disposes only the originally confined output or refuses with the existing safe error, and unrelated outside files are not moved or deleted

#### Scenario: L10 Preserve ordinary concurrent retention and protocol behavior
- **WHEN** previews overlap, cancel, expire, retry, finish at exact count/byte bounds, or run alongside nonpreview jobs
- **THEN** producer reservations and settled accounting preserve existing success/error/revision/retention semantics, no cancelled producer releases capacity early and public schemas/catalogs/schema44/protocol1 remain unchanged
