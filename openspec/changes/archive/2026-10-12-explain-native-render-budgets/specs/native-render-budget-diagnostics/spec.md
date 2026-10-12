## ADDED Requirements

### Requirement: Report existing native accounting at selected admission guards
The core MUST attach optional `renderBudget` diagnostics at the selected shape segment/raster allocation, candidate analysis, effect pixel pass, motion blur pixel work, linear composition pixel work and composition memory guards. It MUST reuse the owner's existing accounting without changing thresholds, evaluation order, supported inputs or accepted output. The diagnostic MUST name its budget, effective limit, documented units and `charged` or `conservative` measurement; work MUST describe attempted accounting at the rejecting guard, not runtime RSS or full scene cost. Other guards MUST retain their existing errors without fabricated diagnostics.

#### Scenario: Exact allowance and first excess
- **WHEN** existing accounting reaches its exact allowance and then attempts one unit over it
- **THEN** the exact allowance remains accepted and rejection reports the unchanged limit and attempted charge with its established error code and retryability

#### Scenario: Conservative allocation bound
- **WHEN** a native allocation or effect bound exceeds its existing limit
- **THEN** the diagnostic reports the computed conservative bound with its units and does not claim measured memory or exact execution cost

### Requirement: Preserve lossless bounded numeric and overflow semantics
Diagnostic limits and non-overflow work MUST be canonical unsigned decimal strings of at most 20 digits. The core MUST retain checked arithmetic. Arithmetic overflow MUST report `overflow: true` and `work: null`; finite work MUST report `overflow: false`. The diagnostic MUST never wrap, saturate into a fake estimate or lose precision in transport.

#### Scenario: Arithmetic overflow
- **WHEN** selected existing checked accounting overflows
- **THEN** the established typed rejection reports its limiting budget and limit with overflow true and no invented work value

#### Scenario: Values beyond JS safe integer precision
- **WHEN** a valid diagnostic contains u64-sized charges
- **THEN** Rust, headless and bridge preserve exactly the decimal digits

### Requirement: Bound and sanitize contributor reporting by construction
Diagnostics MUST contain at most eight anonymous numeric contributors with closed native kinds. Shape aggregate owners MUST deterministically retain the largest eight encountered charges ordered by descending charge then kind. Single-charge owners MUST report their current charge when available; unsupported decomposition MUST yield an empty list. Contributors MUST be documented as partial and MUST NOT contain names, IDs, paths, commands, source content or arbitrary labels.

#### Scenario: Deterministic bounded contributors
- **WHEN** more than eight contributors with tied charges reach a selected aggregate guard
- **THEN** repeated evaluation yields the same at-most-eight ordered entries without changing the total accounting

#### Scenario: Private source values
- **WHEN** source content contains private names, absolute paths or command-like text
- **THEN** diagnostics contain only the governed identifiers and numeric values and do not expose that content

### Requirement: Propagate additive diagnostics through governed native error transports
Headless standalone/status/worker errors, bridge jobs and MCP MUST preserve valid core-origin diagnostics. Their canonical catalogs and listed consumers MUST agree. Ordinary errors MUST omit the field and preserve their existing envelope. Unknown keys, kinds, budgets, oversized contributors or inconsistent overflow/work MUST fail contract validation. Providers MUST NOT acquire a native accounting contract. Existing stable codes and retryability MUST remain unchanged, and budget admission rejection MUST NOT become FFmpeg failure or a renderer defect.

#### Scenario: Native transport parity
- **WHEN** the same selected admission rejection is observed through core, headless and MCP/job polling
- **THEN** the diagnostic agrees structurally and retains INVALID_ARGUMENT, non-retryable behavior and existing FFmpeg-null details

#### Scenario: Ordinary and malformed errors
- **WHEN** an ordinary error or malformed diagnostic is transported
- **THEN** the ordinary error keeps the predecessor shape while malformed diagnostics fail the existing contract-validation path

### Requirement: Preserve preflight atomicity and compatibility
Budget diagnostics MUST preserve rejection before artifact/output preparation and publication, project/draft/revision/history immutability, existing destination bytes, locks and revision conflict precedence. Valid illustrated benchmark rendering MUST remain accepted with its existing expected output. No persisted schema, major protocol or error-code migration SHALL be introduced.

#### Scenario: Rejected render leaves state and artifacts unchanged
- **WHEN** an actual render request reaches a selected budget guard with an existing destination and project/history/draft snapshots
- **THEN** the typed diagnostic appears without renderer execution or artifact publication and all captured state/destination bytes remain unchanged

#### Scenario: Healthy predecessor and revision conflict
- **WHEN** a valid illustrated benchmark renders or an edit supplies a stale expected revision
- **THEN** existing rendering evidence and revision conflict behavior remain effective without native budget diagnostics

#### Scenario: Complete rollback of governed MCP budget metadata
- **WHEN** the budget definition remains but all six governed MCP error fields are removed
- **THEN** direct and raw rollback MUST reject missing budget metadata with budget-specific failure text rather than accept a predecessor

#### Scenario: Historical catalog without budget metadata
- **WHEN** an input has neither budget fields nor a budget definition
- **THEN** rollback MUST preserve the historical input and all recorded predecessor pins unchanged
