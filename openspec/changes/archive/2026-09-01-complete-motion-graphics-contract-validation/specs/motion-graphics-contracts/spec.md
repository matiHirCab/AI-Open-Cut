## MODIFIED Requirements

### Requirement: Versioned canonical motion-graphics vocabulary
The project MUST maintain one versioned, checked-in canonical motion-graphics fixture catalog that defines closed wire identifiers and deterministic examples for transforms, layers, components, slots, markers, time expressions, curves, masks, effects, and audio events; every valid and invalid concept payload MUST have a strict closed test-only Rust Serde declaration and mirrored strict TypeScript Zod schema, catalog wrappers and identifier sets MUST be closed and identical in both languages, and the catalog MUST identify whether those records are fixtures only or activated runtime contracts.

#### Scenario: Locate every foundation concept
- **WHEN** a Rust, TypeScript/Zod, MCP, persistence, evaluation, or renderer milestone prepares to adopt a motion-graphics concept
- **THEN** the canonical catalog provides its version, exact field and variant identifiers, observable semantics, valid examples, invalid examples, required fields, value types, and bounds without requiring inference from renderer-specific code

#### Scenario: Reject malformed valid payloads
- **WHEN** a purported valid fixture or catalog wrapper has an unknown field, missing required field, wrong value type, unknown tagged variant, non-finite value, out-of-range value, or divergent identifier set
- **THEN** both focused language validators reject it instead of accepting its outer wrapper

#### Scenario: Keep preparatory fixtures inactive
- **WHEN** version 1 of the catalog is corrected before editor-core implements the concepts
- **THEN** it remains `fixture_only` and no project schema, headless operation, MCP tool, capability, provider contract, preview, export, revision, or history behavior changes

### Requirement: Canonical hierarchy and slot fixtures
Component, layer-parent, instance, and slot fixtures MUST use structured typed stable IDs and legal exact `project`, `root`, or `component:<id>` scopes; validators MUST derive unique definitions and references from strictly parsed payloads, reject duplicate payload or metadata definitions before normalization, and require exact agreement with fixture metadata; fixtures MUST define component-local integer time and finite positive instance time scale, MUST bind slots to typed stable properties rather than arbitrary JSON paths, MUST count constrained text by Unicode scalar values, and MUST classify missing, cross-scope, cyclic, depth-limit, type, required/default, and constraint failures.

#### Scenario: Resolve a reusable component instance
- **WHEN** a valid component instance supplies typed slot overrides and references a component with local tracks and markers
- **THEN** every definition, binding target, slot value, parent, marker, and time mapping derives from the payload and resolves uniquely within its declared tuple scope

#### Scenario: Detect metadata and payload drift
- **WHEN** fixture metadata omits, adds, duplicates, or changes a definition/reference represented by its payload
- **THEN** both language validators reject the fixture before global reference closure is evaluated

#### Scenario: Reject an invalid hierarchy
- **WHEN** complete parent or component scenarios contain missing or cross-scope references, direct or indirect cycles, or depth beyond the named inclusive limit
- **THEN** both validators derive and report the fixture's exact classification and reason without relying on an unrelated structural parse failure

#### Scenario: Apply slot values identically
- **WHEN** a slot default or supplied tagged value is missing, has the wrong kind, violates its constraint, or contains Unicode text at a length boundary
- **THEN** Rust and TypeScript derive the same result using Unicode scalar length and never use an arbitrary property path fallback

### Requirement: Canonical animation, mask, and effect fixtures
The catalog MUST define closed tagged variants for hold, linear, cubic-Bézier, and spring curves; ordered alpha/luma mask operations; and ordered typed effects, and every numeric parameter and represented collection MUST be finite and subject to its explicit named inclusive complexity limit before later runtime activation. The catalog MUST NOT declare an inline-resource complexity limit when no version-1 payload can represent that resource. Resource-bearing fields MUST accept only typed managed identifiers and MUST reject filesystem paths, traversal, URI schemes, network resources, executable SVG, event handlers, and raw renderer expressions without applying resource restrictions to ordinary text content.

#### Scenario: Preserve declared evaluation order
- **WHEN** a valid layer contains multiple masks and effects plus animated properties
- **THEN** consumers preserve array order for mask operations and effects and use the selected typed curve without substituting renderer expressions

#### Scenario: Enforce every represented limit
- **WHEN** a component, layer, slot, marker, keyframe, mask, effect, audio-event, or hierarchy collection is exactly at its named limit or exceeds it by one
- **THEN** both validators accept the inclusive boundary and derive the exact overflow failure for the excess case

#### Scenario: Reject unsafe graphics input
- **WHEN** a resource-bearing field includes executable SVG, an event handler, external URI, POSIX/Windows/UNC path, traversal, or raw renderer expression
- **THEN** both language validators reject it before backend-specific execution is possible

#### Scenario: Preserve ordinary text
- **WHEN** a text or rich-text slot contains URL-like prose that is not interpreted as a resource
- **THEN** resource safety validation does not reject or rewrite the text

### Requirement: Canonical marker and semantic audio-event fixtures
Marker and audio-event fixtures MUST use unique structured scoped marker references, absolute or marker-relative JavaScript-safe-integer timing, semantic sound-event identifiers, finite decibel gain, deterministic JavaScript-safe unsigned variant seeds, and typed project-scoped audio-bus and managed-asset references, and MUST provide complete deterministic missing and ambiguous scenarios for markers, sound definitions, variants, and buses.

#### Scenario: Resolve a deterministic audio event
- **WHEN** a valid audio event references a defined semantic event, unique marker-relative time, managed variant, and defined bus
- **THEN** its payload-derived scoped references resolve exactly and its placement, gain, bus, and saved variant seed are sufficient for deterministic later evaluation

#### Scenario: Reject every missing audio reference class
- **WHEN** a complete audio scenario references an absent marker, sound definition, variant, or audio bus
- **THEN** both validators derive the exact matching missing-reference classification and reason

#### Scenario: Reject every ambiguous audio reference class
- **WHEN** a complete audio scenario contains duplicate markers, sound definitions, variants, or audio buses within the same scope
- **THEN** both validators derive the exact matching ambiguous-reference classification and reason

#### Scenario: Enforce cross-language integer bounds
- **WHEN** timing or a variant seed equals JavaScript's maximum safe integer or exceeds it by one
- **THEN** both validators accept the maximum and reject the overflow with identical fixture semantics

### Requirement: Cross-language fixture evidence and adoption
Rust and TypeScript tests MUST consume the same canonical motion-graphics catalog and verify its version, fixture-only status, strict closed catalog and concept payloads, exact identifier catalogs, finite values, enforced represented limits, unique payload-derived structured reference closure, legal kind/scope combinations, field-specific safety invariants, and actual exact per-concept failure IDs/classifications/reasons; a later milestone that activates any concept MUST update every affected native declaration, public catalog, capability, stable error, migration, and parity consumer in the same approved change where applicable.

#### Scenario: Prove the shared fixture succeeds
- **WHEN** the focused Rust and TypeScript tests load the checked-in version-1 catalog
- **THEN** both strictly accept the same valid payloads and observe identical concept identifiers, scoped references, ordering metadata, numeric/string boundaries, safety rules, and invalid-case expectations

#### Scenario: Prove malformed payload rejection
- **WHEN** either focused test mutates a valid payload with string opacity, unknown fields, missing required fields, unsafe resource input, duplicate metadata, illegal scope, or an out-of-range boundary
- **THEN** that test rejects the payload with the violated shape, safety, uniqueness, scope, or range invariant

#### Scenario: Prove causal negative coverage
- **WHEN** either focused test validates or deliberately swaps complete invalid hierarchy, slot, marker, sound-definition, variant, bus, graphics, limit, and safety payloads
- **THEN** the observed validator-produced IDs, classifications, and reasons exactly equal the required test-owned matrix and unrelated payloads cannot satisfy another fixture's expectation

#### Scenario: Activate a concept later
- **WHEN** a later milestone makes a catalog concept persisted, agent-addressable, or renderer-observable
- **THEN** its approved change adds editor-core ownership and validation plus all applicable migration, revision-conflict, atomic rollback, undo/redo, reopen, batch-alias, headless, MCP/Zod, capability, stable-error, preview/export, and parity evidence while preserving existing simple behavior
