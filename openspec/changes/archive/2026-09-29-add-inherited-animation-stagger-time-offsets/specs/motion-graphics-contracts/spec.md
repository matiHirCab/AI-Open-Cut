## ADDED Requirements

### Requirement: Governed inherited timing contract
Canonical versioned fixtures and catalogs MUST describe optional group/component-instance staggerMs, optional repeater timeOffsetMs, activated parent channel targets, and any capability/version discriminator needed by clients. New fields MUST be additive for valid older requests and absent fields MUST retain zero-offset behavior. Rust request and project declarations, TypeScript/Zod bridge schemas, MCP structural schemas/annotations, and governed Python consumers MUST agree with canonical fields, bounds and typed failures. Existing simple operations, error codes and retryability MUST remain compatible; unsupported future persisted schemas MUST fail closed.

#### Scenario: Discover and submit timing support
- **WHEN** a client checks capability reporting and submits valid standalone or alias-aware batch timing edits
- **THEN** it sees declared support and every governed layer accepts the same typed values and returns compatible results

#### Scenario: Reject contract drift
- **WHEN** a new field is absent from a governed consumer or malformed, out-of-range or unknown input is supplied
- **THEN** parity checks fail for drift and runtime input fails with non-retryable INVALID_ARGUMENT without a commit

#### Scenario: Preserve older clients
- **WHEN** an older request omits all new fields and uses a previously valid simple operation
- **THEN** its wire shape, result compatibility and evaluated behavior remain unchanged

