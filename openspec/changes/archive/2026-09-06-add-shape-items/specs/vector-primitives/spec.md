## MODIFIED Requirements

### Requirement: Governed parity and deferred activation
A version-1 vector catalog MUST define exact identifiers, named limits, activation status `core_primitives_only`, and valid/invalid examples. Rust production types and mirrored strict TypeScript schemas MUST consume the same fixtures and agree on decoding, bounds, and semantic acceptance; core MUST remain the runtime domain authority. The catalog activation status core_primitives_only MUST continue to describe this reference-free primitive contract. ShapeItem activation MUST be governed separately by the shape-items contract and shape_items/shape_rendering capabilities, with documentation identifying issue #28 as that activation milestone. Existing primitive values, limits, legacy color strings, and reference-free validation MUST remain unchanged; shape activation MUST NOT introduce resource resolution into primitive validation.

#### Scenario: Verify shared fixture evidence
- **WHEN** both language suites validate the catalog including malformed wrappers, all variants, inclusive limits, overflow, and wrong-type fixtures
- **THEN** they agree on every expected outcome and fail on catalog or identifier drift

#### Scenario: Preserve existing workflows
- **WHEN** legacy projects are opened, edited with standalone/batch operations, subjected to stale revisions and failed batches, undone/redone, reopened, and previewed/exported
- **THEN** existing regression suites retain their established state, errors, atomicity, schema, and render behavior while shape support is advertised only according to the separately governed activation capabilities
