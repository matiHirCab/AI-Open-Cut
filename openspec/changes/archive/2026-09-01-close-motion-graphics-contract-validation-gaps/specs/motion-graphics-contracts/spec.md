## MODIFIED Requirements

### Requirement: Canonical hierarchy and slot fixtures
Component, layer-parent, instance, and slot fixtures MUST use structured typed stable IDs and legal exact `project`, `root`, or `component:<id>` scopes; validators MUST derive unique definitions and references from strictly parsed payloads, reject duplicate payload or metadata definitions before normalization, and require exact agreement with fixture metadata; fixture IDs MUST be globally unique across valid and invalid catalog entries before any result map is constructed; fixtures MUST define component-local integer time and finite positive instance time scale, MUST bind slots to typed stable properties rather than arbitrary JSON paths, MUST count constrained text by Unicode scalar values, and MUST classify missing, cross-scope, cyclic, depth-limit, type, required/default, and constraint failures.

#### Scenario: Reject duplicate fixture identity
- **WHEN** two valid fixtures, two invalid fixtures, or one fixture from each collection use the same fixture ID
- **THEN** both language validators reject the catalog before parsing results can overwrite or normalize that identity

#### Scenario: Validate every component identifier
- **WHEN** a component track ID or supplied slot-value key violates the closed identifier grammar
- **THEN** Rust and TypeScript both reject the component payload before reference closure

### Requirement: Canonical animation, mask, and effect fixtures
The catalog MUST define closed tagged variants for hold, linear, cubic-Bézier, and spring curves; ordered alpha/luma mask operations; and ordered typed effects, and every numeric parameter and represented collection MUST be finite and subject to its explicit named inclusive complexity limit before later runtime activation. Project-level limits MUST count all payload-derived project definitions, and composition limits MUST count all payload-derived records sharing the same exact `root` or `component:<id>` owner even when those records occur in separate fixtures. The catalog MUST NOT declare an inline-resource complexity limit when no version-1 payload can represent that resource. Resource-bearing fields MUST accept only typed managed identifiers and MUST reject filesystem paths, traversal, URI schemes, network resources, executable SVG, event handlers, and raw renderer expressions without applying resource restrictions to ordinary text content.

#### Scenario: Enforce aggregate limits across fixtures
- **WHEN** payload-derived components or same-owner layers, markers, slots, or audio events are distributed across multiple fixture records
- **THEN** both validators accept the inclusive aggregate boundary and reject the first record beyond that owner's named limit

#### Scenario: Reject malformed animation fields identically
- **WHEN** a layer has an empty animation-channel string or a curve set has no curve definitions
- **THEN** the closed Rust and Zod validators both reject the payload

### Requirement: Canonical marker and semantic audio-event fixtures
Marker and audio-event fixtures MUST use unique structured scoped marker references, absolute or marker-relative JavaScript-safe-integer timing, semantic sound-event identifiers, finite decibel gain, deterministic JavaScript-safe unsigned variant seeds, and typed project-scoped audio-bus and managed-asset references; the catalog's managed-resource collection MUST contain only unique `{ kind: "asset", scope: "project", id }` tuples; and fixtures MUST provide complete deterministic missing and ambiguous scenarios for markers, sound definitions, variants, and buses.

#### Scenario: Reject an invalid managed-resource tuple
- **WHEN** the managed-resource collection contains a non-asset kind, a non-project scope, a duplicate asset tuple, or a payload references an unmanaged asset
- **THEN** both validators reject the catalog before the resource can participate in reference closure

#### Scenario: Enforce marker field parity
- **WHEN** a marker ID or name violates the identifier grammar or its timestamp exceeds JavaScript's maximum safe integer
- **THEN** Rust and TypeScript both reject the marker payload with identical boundary semantics

### Requirement: Cross-language fixture evidence and adoption
Rust and TypeScript tests MUST consume the same canonical motion-graphics catalog and verify its version, fixture-only status, strict closed catalog and concept payloads, globally unique fixture identities, exact identifier catalogs, finite values, project-wide and owner-scoped aggregate represented limits, unique payload-derived structured reference closure, exact project-scoped managed-asset tuples, legal kind/scope combinations, field-specific safety invariants, and actual exact per-concept failure IDs/classifications/reasons; a later milestone that activates any concept MUST update every affected native declaration, public catalog, capability, stable error, migration, and parity consumer in the same approved change where applicable.

#### Scenario: Prove corrected wrapper and field parity
- **WHEN** either focused suite mutates aggregate counts, fixture identities, managed-resource tuples, component identifiers, animation-channel strings, marker fields, or curve collection presence
- **THEN** Rust and TypeScript accept the same inclusive boundaries and reject the same malformed catalog or payload
