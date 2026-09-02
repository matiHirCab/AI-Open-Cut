## MODIFIED Requirements

### Requirement: Canonical hierarchy and slot fixtures
Component, layer-parent, instance, and slot fixtures MUST use structured typed stable IDs and legal exact `project`, `root`, or `component:<id>` scopes; validators MUST derive unique definitions and references from strictly parsed payloads, reject duplicate payload definitions at their source collection before set normalization, reject duplicate metadata definitions before normalization, and require exact agreement with fixture metadata; fixture IDs MUST be globally unique across valid and invalid catalog entries before any result map is constructed; fixtures MUST define component-local integer time and finite positive instance time scale, MUST bind slots to typed stable properties rather than arbitrary JSON paths, MUST count constrained text by Unicode scalar values, and MUST classify missing, cross-scope, cyclic, depth-limit, type, required/default, and constraint failures.

#### Scenario: Reject a duplicate payload definition before normalization
- **WHEN** a layer set or component definition repeats the same scoped layer ID
- **THEN** both validators report the duplicate-definition invariant before metadata comparison, aggregate counting, or reference closure can erase it

#### Scenario: Strictly validate an invalid hierarchy or slot envelope
- **WHEN** an invalid component, layer, or slot candidate also contains a malformed unrelated ID, scope, name, time, collection, default, or constraint
- **THEN** both validators reject the unrelated malformed field instead of reporting the fixture's declared semantic failure

### Requirement: Canonical animation, mask, and effect fixtures
The catalog MUST define closed tagged variants for hold, linear, cubic-Bézier, and spring curves; ordered alpha/luma mask operations; and ordered typed effects, and every numeric parameter and represented collection MUST be finite and subject to its explicit named inclusive complexity limit before later runtime activation. Duplicate payload-derived mask and effect definitions MUST be rejected before set normalization. Project-level limits MUST count all payload-derived project definitions, and composition limits MUST count all payload-derived records sharing the same exact `root` or `component:<id>` owner even when those records occur in separate fixtures. The catalog MUST NOT declare an inline-resource complexity limit when no version-1 payload can represent that resource. Resource-bearing fields MUST accept only typed managed identifiers and MUST reject filesystem paths, traversal, URI schemes, network resources, executable SVG, event handlers, and raw renderer expressions without applying resource restrictions to ordinary text content.

#### Scenario: Reject duplicate mask or effect definitions
- **WHEN** one payload repeats a scoped mask or effect ID
- **THEN** both validators reject the duplicate before normalization or aggregate validation

#### Scenario: Prove the named overflow invariant
- **WHEN** a mutation exceeds one represented aggregate or local limit by one
- **THEN** both focused tests observe the exact corresponding named limit rather than accepting an unrelated validation failure

### Requirement: Cross-language fixture evidence and adoption
Rust and TypeScript tests MUST consume the same canonical motion-graphics catalog and verify its version, fixture-only status, strict closed catalog and concept payloads, globally unique fixture identities, exact identifier catalogs, finite values, project-wide and owner-scoped aggregate represented limits, pre-normalization payload and metadata uniqueness, unique payload-derived structured reference closure, exact project-scoped managed-asset tuples, legal kind/scope combinations, field-specific safety invariants, and actual exact per-concept failure IDs/classifications/reasons. Every invalid fixture MUST pass mirrored common-field validation for all fields except its one fixture-ID-specific intentional defect before that defect is classified. A later milestone that activates any concept MUST update every affected native declaration, public catalog, capability, stable error, migration, and parity consumer in the same approved change where applicable.

#### Scenario: Reject malformed unrelated fields in negative evidence
- **WHEN** a negative candidate contains its declared defect plus an empty or invalid ID, illegal scope, invalid Unicode-scalar name length, unsafe integer, non-finite or out-of-range number, missing required collection, or invalid unrelated constraint
- **THEN** both validators reject the malformed envelope before deriving the declared classification and reason

#### Scenario: Preserve causal canonical negatives
- **WHEN** each unchanged canonical invalid fixture is validated after strict common-field preflight
- **THEN** both languages still derive its independently expected fixture ID, classification, and reason exactly
