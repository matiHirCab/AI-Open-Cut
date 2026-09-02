## ADDED Requirements

### Requirement: Versioned canonical motion-graphics vocabulary
The project MUST maintain one versioned, checked-in canonical motion-graphics fixture catalog that defines closed wire identifiers and deterministic examples for transforms, layers, components, slots, markers, time expressions, curves, masks, effects, and audio events, and the catalog MUST identify whether those records are fixtures only or activated runtime contracts.

#### Scenario: Locate every foundation concept
- **WHEN** a Rust, TypeScript/Zod, MCP, persistence, evaluation, or renderer milestone prepares to adopt a motion-graphics concept
- **THEN** the canonical catalog provides its version, exact field and variant identifiers, observable semantics, valid examples, and invalid examples without requiring inference from renderer-specific code

#### Scenario: Keep preparatory fixtures inactive
- **WHEN** version 1 of the catalog is added before editor-core implements the concepts
- **THEN** it reports `fixture_only` status and no project schema, headless operation, MCP tool, capability, provider contract, preview, export, revision, or history behavior changes

### Requirement: Canonical coordinate, timing, and ordering fixtures
The motion-graphics catalog MUST encode a top-left origin, positive X to the right, positive Y downward, explicit pixel or normalized position units, finite transform values, integer-millisecond half-open intervals, scoped marker-relative time, deterministic bottom-to-top layer ordering, and ADR 0004's transform and compositing pipeline.

#### Scenario: Interpret a valid visual layer
- **WHEN** a consumer reads the canonical layer fixture
- **THEN** it can resolve the layer's composition scope, parent, explicit z-index, stable tie-break order, position units, local transform order, ancestor order, mask/effect order, matte, inherited opacity, blend, premultiplied alpha, and linear-light output conversion deterministically

#### Scenario: Reject ambiguous or non-finite timing and transforms
- **WHEN** a fixture contains a non-finite token, a non-integer absolute time, an unresolved or duplicate scoped marker name, or conflicting transform fields
- **THEN** the catalog classifies it deterministically as invalid, missing, or ambiguous rather than allowing a language or renderer to infer behavior

### Requirement: Canonical hierarchy and slot fixtures
Component, layer-parent, instance, and slot fixtures MUST use typed stable IDs and scoped references, MUST define component-local integer time and finite positive instance time scale, MUST bind slots to typed stable properties rather than arbitrary JSON paths, and MUST classify missing, cross-scope, cyclic, depth-limit, type, required/default, and constraint failures.

#### Scenario: Resolve a reusable component instance
- **WHEN** a valid component instance supplies typed slot overrides and references a component with local tracks and markers
- **THEN** every definition, binding target, slot value, parent, marker, and time mapping resolves uniquely within its declared scope

#### Scenario: Reject an invalid hierarchy
- **WHEN** parent or component references are missing, cross scope, cyclic, deeper than the named explicit limit, or a slot override violates its declared kind or constraint
- **THEN** the fixture records one deterministic failure classification and never supplies an arbitrary property path fallback

### Requirement: Canonical animation, mask, and effect fixtures
The catalog MUST define closed tagged variants for hold, linear, cubic-Bézier, and spring curves; ordered alpha/luma mask operations; and ordered typed effects, and every numeric parameter and collection MUST be finite and subject to an explicit named complexity limit before later runtime activation.

#### Scenario: Preserve declared evaluation order
- **WHEN** a valid layer contains multiple masks and effects plus animated properties
- **THEN** consumers preserve array order for mask operations and effects and use the selected typed curve without substituting renderer expressions

#### Scenario: Reject unsafe or unbounded graphics input
- **WHEN** an example includes executable SVG, an event handler, an external URL, an arbitrary path, a raw FFmpeg expression, a non-finite parameter, an unknown variant, or a collection exceeding its named limit
- **THEN** the catalog classifies the example as invalid input before any backend-specific execution is possible

### Requirement: Canonical marker and semantic audio-event fixtures
Marker and audio-event fixtures MUST use unique scoped marker names, absolute or marker-relative integer timing, semantic sound-event identifiers, finite decibel gain, deterministic unsigned variant seeds, and typed audio-bus references, and MUST classify missing or ambiguous markers, sound definitions, variants, and buses.

#### Scenario: Resolve a deterministic audio event
- **WHEN** a valid audio event references a defined semantic event, a unique marker-relative time, and a defined bus
- **THEN** its placement, gain, bus, and saved variant seed are sufficient for deterministic later evaluation without a path, URL, raw filter, or map-iteration dependency

#### Scenario: Reject a missing audio reference
- **WHEN** an audio event references an absent or ambiguous marker, sound-event definition, variant, or audio bus
- **THEN** the catalog records a deterministic missing-reference or ambiguous-reference failure

### Requirement: Cross-language fixture evidence and adoption
Rust and TypeScript tests MUST consume the same canonical motion-graphics catalog and verify its version, fixture-only status, concept coverage, unique identifiers, finite values, explicit limits, reference closure, safety invariants, and failure classifications; a later milestone that activates any concept MUST update every affected native declaration, public catalog, capability, stable error, migration, and parity consumer in the same approved change where applicable.

#### Scenario: Prove the shared fixture succeeds
- **WHEN** the focused Rust and TypeScript tests load the checked-in version-1 catalog
- **THEN** both accept the same valid fixtures and observe the same concept identifiers, references, ordering metadata, safety rules, and invalid-case classifications

#### Scenario: Prove the fixture checker fails
- **WHEN** either focused test evaluates a deterministic malformed in-memory catalog with a missing category, duplicate fixture ID, unresolved valid reference, unsafe resource, non-finite token, or absent named limit
- **THEN** that test rejects the catalog with the violated invariant instead of silently normalizing it

#### Scenario: Activate a concept later
- **WHEN** a later milestone makes a catalog concept persisted, agent-addressable, or renderer-observable
- **THEN** its approved change adds editor-core ownership and validation plus all applicable migration, revision-conflict, atomic rollback, undo/redo, reopen, batch-alias, headless, MCP/Zod, capability, stable-error, preview/export, and parity evidence while preserving existing simple behavior
