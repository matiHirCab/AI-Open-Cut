## MODIFIED Requirements

### Requirement: Canonical hierarchy and slot fixtures
Component, layer-parent, instance, and slot fixtures MUST use structured typed stable IDs and legal exact `project`, `root`, or `component:<id>` scopes; validators MUST derive unique definitions and references from strictly parsed payloads, reject duplicate payload definitions and invalid-envelope context identifiers at their source collection before set or map normalization, reject duplicate metadata definitions before normalization, and require exact agreement with fixture metadata; fixture IDs MUST be globally unique across valid and invalid catalog entries before any result map is constructed; fixtures MUST define component-local integer time and finite positive instance time scale, MUST bind slots to typed stable properties rather than arbitrary JSON paths, MUST count constrained text by Unicode scalar values, and MUST classify missing, cross-scope, cyclic, depth-limit, type, required/default, and constraint failures. Component dependency validation MUST preserve every directed edge, reject duplicate edges and missing endpoints before traversal, detect cycles across all reachable branches, and enforce component depth against the longest reachable path.

#### Scenario: Reject duplicate invalid hierarchy context
- **WHEN** an invalid component or slot envelope repeats a component ID, dependency edge, or target-layer ID unrelated to its declared defect
- **THEN** both validators reject the exact duplicate-definition invariant before fixture-specific classification

#### Scenario: Classify a branching component graph
- **WHEN** a reachable component has multiple outgoing dependencies and any branch contains a direct or indirect cycle or exceeds the inclusive component-depth limit
- **THEN** both validators preserve every edge and report the exact cycle or named depth-limit failure deterministically

### Requirement: Canonical animation, mask, and effect fixtures
The catalog MUST define closed tagged variants for hold, linear, cubic-Bézier, and spring curves; ordered alpha/luma mask operations; and ordered typed effects, and every numeric parameter and represented collection MUST be finite and subject to its explicit named inclusive complexity limit in valid and invalid envelopes before later runtime activation. Every catalog limit MUST be a positive JavaScript-safe integer in both language validators. Duplicate payload-derived mask and effect definitions and duplicate invalid-envelope context identifiers MUST be rejected before set normalization or semantic classification. Project-level limits MUST count all payload-derived project definitions, and composition limits MUST count all payload-derived records sharing the same exact `root` or `component:<id>` owner even when those records occur in separate fixtures. The catalog MUST NOT declare an inline-resource complexity limit when no version-1 payload can represent that resource. Resource-bearing fields MUST accept only typed managed identifiers and MUST reject filesystem paths, traversal, URI schemes, network resources, executable SVG, every ASCII SVG event-handler attribute, and raw renderer expressions without applying resource restrictions to ordinary text content. Safety classification MUST inspect every mask in declaration order.

#### Scenario: Enforce an invalid animation-envelope limit
- **WHEN** an invalid curve, mask, or effect candidate contains its named inclusive collection limit or one additional otherwise valid item
- **THEN** both validators accept the boundary for fixture-specific classification and reject the overflow with the exact named limit invariant

### Requirement: Canonical marker and semantic audio-event fixtures
Marker and audio-event fixtures MUST use unique structured scoped marker references, absolute or marker-relative JavaScript-safe-integer timing, semantic sound-event identifiers, finite decibel gain, deterministic JavaScript-safe unsigned variant seeds, and typed project-scoped audio-bus and managed-asset references; the catalog's managed-resource collection MUST contain only unique `{ kind: "asset", scope: "project", id }` tuples; valid and invalid scenario definition and context identifiers MUST be unique before normalization except for the exact lookup key intentionally repeated by a fixture whose independent expectation declares the corresponding ambiguity reason; every represented marker and audio-event collection MUST obey its named inclusive limit; and fixtures MUST provide complete deterministic missing and ambiguous scenarios for markers, sound definitions, variants, and buses.

#### Scenario: Reject unrelated duplicate marker or audio context
- **WHEN** an invalid marker or audio candidate repeats a marker ID, asset ID, event ID, or other context definition unrelated to its declared ambiguity
- **THEN** both validators reject the exact duplicate-definition invariant before fixture-specific classification

#### Scenario: Enforce invalid marker and audio limits
- **WHEN** an invalid marker or audio-event envelope is at its named inclusive limit or contains one additional otherwise valid record
- **THEN** both validators accept the boundary for fixture-specific classification and reject the overflow with the exact named limit invariant

### Requirement: Cross-language fixture evidence and adoption
Rust and TypeScript tests MUST consume the same canonical motion-graphics catalog and verify its version, fixture-only status, strict closed catalog and concept payloads, globally unique fixture identities, exact identifier catalogs, finite values, project-wide and owner-scoped aggregate represented limits, named collection limits in valid and invalid envelopes, pre-normalization payload, context, dependency-edge, and metadata uniqueness, unique payload-derived structured reference closure, exact project-scoped managed-asset tuples, legal kind/scope combinations, field-specific safety invariants, complete branching dependency graphs, and actual exact per-concept failure IDs, concepts, classifications, and reasons. Every invalid fixture MUST pass mirrored common-field validation for all fields except its one fixture-ID-specific intentional defect before that defect is classified. A later milestone that activates any concept MUST update every affected native declaration, public catalog, capability, stable error, migration, and parity consumer in the same approved change where applicable.

#### Scenario: Reject an unrelated invalid-envelope defect
- **WHEN** an invalid fixture adds a duplicate definition, duplicate context identifier, duplicate dependency edge, or collection overflow unrelated to its declared failure
- **THEN** both validators reject the exact common invariant before the declared semantic failure can be observed
