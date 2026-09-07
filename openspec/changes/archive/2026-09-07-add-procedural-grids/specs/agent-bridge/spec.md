## ADDED Requirements

### Requirement: Typed discoverable grid workflows
Headless MUST expose add_grid and MCP MUST expose timeline_add_grid using existing protocol-1 project/revision envelopes and mutation results. Request unions, update fields, batches, drafts, component inputs and project responses MUST carry strict canonical grid records. Adapters MUST submit typed values to core for domain validation and atomicity. Core and aggregate capabilities MUST advertise grid_items; rendering and aggregate capabilities MUST advertise grid_rendering only when complete rendering is ready. Existing operations, aliases, errors/retryability and provider protocols MUST retain their meaning; clients MUST use the new capabilities to distinguish grid support, and documentation MUST explicitly state that schema-16 grid-bearing projects require readers supporting the new item variant.

Canonical procedural-grid, operation, MCP structural schema/annotation, capability and ownership catalogs MUST match all governed Rust and TypeScript consumers and receive designated CODEOWNER review. Shared fixtures MUST distinguish representation rejection from semantic rejection and cover exact limits and geometry. Client documentation MUST define dimensions, lattice origin/orientation, spacing, paints, timing, ordering, clipping, limits, aliases, failures, migration and readiness.

#### Scenario: Exercise source and packaged clients
- **WHEN** real source and packaged MCP clients discover support and create/update each pattern standalone and in aliased batches, read projects, undo/redo, reopen and render
- **THEN** typed responses reflect core semantics and shared fixtures, schema/operation/capability parity and workflow assertions pass

#### Scenario: Reject transport failures without state change
- **WHEN** real clients send invalid structure or semantics, missing references, stale revisions, locked targets or a batch that fails after grid creation
- **THEN** the existing validation stage, error/retryability and full rollback behavior are preserved

#### Scenario: Distinguish editing from rendering readiness
- **WHEN** the renderer is unavailable while core grid support exists
- **THEN** grid_items remains advertised, grid_rendering is absent, and rendering returns DEPENDENCY_UNAVAILABLE without degraded output
