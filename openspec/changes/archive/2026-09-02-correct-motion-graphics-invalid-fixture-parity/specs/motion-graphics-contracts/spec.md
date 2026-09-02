## MODIFIED Requirements

### Requirement: Canonical animation, mask, and effect fixtures
The catalog MUST define closed tagged variants for hold, linear, cubic-Bézier, and spring curves; ordered alpha/luma mask operations; and ordered typed effects, and every numeric parameter and represented collection MUST be finite and subject to its explicit named inclusive complexity limit before later runtime activation. Every catalog limit MUST be a positive JavaScript-safe integer in both language validators. Duplicate payload-derived mask and effect definitions, including definitions in invalid scenario envelopes, MUST be rejected before set normalization or semantic classification. Project-level limits MUST count all payload-derived project definitions, and composition limits MUST count all payload-derived records sharing the same exact `root` or `component:<id>` owner even when those records occur in separate fixtures. The catalog MUST NOT declare an inline-resource complexity limit when no version-1 payload can represent that resource. Resource-bearing fields MUST accept only typed managed identifiers and MUST reject filesystem paths, traversal, URI schemes, network resources, executable SVG, every ASCII SVG event-handler attribute, and raw renderer expressions without applying resource restrictions to ordinary text content. Safety classification MUST inspect every mask in declaration order.

#### Scenario: Reject duplicate definitions in an invalid envelope
- **WHEN** an invalid layer, mask, or renderer-expression candidate repeats a scoped definition ID
- **THEN** both validators reject the duplicate invalid envelope before deriving its declared semantic failure

#### Scenario: Inspect every executable SVG source
- **WHEN** executable inline SVG appears in any mask through a script element or a case-insensitive ASCII event-handler attribute
- **THEN** both validators classify the candidate as executable SVG regardless of the mask's array position or handler spelling

#### Scenario: Enforce safe catalog limits
- **WHEN** any catalog limit equals JavaScript's maximum safe integer or the first larger integer
- **THEN** both wrapper validators accept the inclusive maximum and reject the larger value before semantic validation

### Requirement: Cross-language fixture evidence and adoption
Rust and TypeScript tests MUST consume the same canonical motion-graphics catalog and verify its version, fixture-only status, strict closed catalog and concept payloads, globally unique fixture identities, exact identifier catalogs, finite values, project-wide and owner-scoped aggregate represented limits, pre-normalization payload and metadata uniqueness, unique payload-derived structured reference closure, exact project-scoped managed-asset tuples, legal kind/scope combinations, field-specific safety invariants, and actual exact per-concept failure IDs, concepts, classifications, and reasons. Every invalid fixture MUST pass mirrored common-field validation for all fields except its one fixture-ID-specific intentional defect before that defect is classified. A later milestone that activates any concept MUST update every affected native declaration, public catalog, capability, stable error, migration, and parity consumer in the same approved change where applicable.

#### Scenario: Reject a relabeled invalid fixture
- **WHEN** an invalid fixture keeps its ID and payload but declares a different valid concept
- **THEN** both validators reject the concept mismatch before fixture-specific classification

#### Scenario: Preserve exact corrected negative evidence
- **WHEN** every unchanged canonical invalid fixture is validated after concept, uniqueness, safety, and wrapper-bound corrections
- **THEN** both languages still derive its independently expected ID, concept, classification, and reason exactly
