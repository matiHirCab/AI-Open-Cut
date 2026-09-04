# motion-graphics-contracts Specification

## Purpose

Define the canonical fixture-only motion-graphics vocabulary, strict cross-language payload evidence, scoped reference semantics, deterministic failure cases, and adoption rules that future persisted, public, evaluation, and rendering milestones must follow.
## Requirements
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

### Requirement: Canonical coordinate, timing, and ordering fixtures
The motion-graphics catalog MUST encode a top-left origin, positive X to the right, positive Y downward, explicit pixel or normalized position units, finite transform values, integer-millisecond half-open intervals, scoped marker-relative time, deterministic bottom-to-top layer ordering, and ADR 0004's transform and compositing pipeline.

#### Scenario: Interpret a valid visual layer
- **WHEN** a consumer reads the canonical layer fixture
- **THEN** it can resolve the layer's composition scope, parent, explicit z-index, stable tie-break order, position units, local transform order, ancestor order, mask/effect order, matte, inherited opacity, blend, premultiplied alpha, and linear-light output conversion deterministically

#### Scenario: Reject ambiguous or non-finite timing and transforms
- **WHEN** a fixture contains a non-finite token, a non-integer absolute time, an unresolved or duplicate scoped marker name, or conflicting transform fields
- **THEN** the catalog classifies it deterministically as invalid, missing, or ambiguous rather than allowing a language or renderer to infer behavior

### Requirement: Canonical hierarchy and slot fixtures
Component, layer-parent, instance, and slot fixtures MUST use structured typed stable IDs and legal exact `project`, `root`, or `component:<id>` scopes; validators MUST derive unique definitions and references from strictly parsed payloads, reject duplicate payload definitions and invalid-envelope context identifiers at their source collection before set or map normalization, reject duplicate metadata definitions before normalization, and require exact agreement with fixture metadata; fixture IDs MUST be globally unique across valid and invalid catalog entries before any result map is constructed; fixtures MUST define component-local integer time and finite positive instance time scale, MUST bind slots to typed stable properties rather than arbitrary JSON paths, MUST count constrained text by Unicode scalar values, and MUST classify missing, cross-scope, cyclic, depth-limit, type, required/default, and constraint failures. Component dependency validation MUST preserve every directed edge, reject duplicate edges and missing endpoints before traversal, detect cycles across all reachable branches, and enforce component depth against the longest reachable path.

#### Scenario: Reject a duplicate payload definition before normalization
- **WHEN** a layer set or component definition repeats the same scoped layer ID
- **THEN** both validators report the duplicate-definition invariant before metadata comparison, aggregate counting, or reference closure can erase it

#### Scenario: Strictly validate an invalid hierarchy or slot envelope
- **WHEN** an invalid component, layer, or slot candidate also contains a malformed unrelated ID, scope, name, time, collection, default, or constraint
- **THEN** both validators reject the unrelated malformed field instead of reporting the fixture's declared semantic failure

#### Scenario: Reject duplicate invalid hierarchy context
- **WHEN** an invalid component or slot envelope repeats a component ID, dependency edge, or target-layer ID unrelated to its declared defect
- **THEN** both validators reject the exact duplicate-definition invariant before fixture-specific classification

#### Scenario: Classify a branching component graph
- **WHEN** a reachable component has multiple outgoing dependencies and any branch contains a direct or indirect cycle or exceeds the inclusive component-depth limit
- **THEN** both validators preserve every edge and report the exact cycle or named depth-limit failure deterministically

### Requirement: Canonical animation, mask, and effect fixtures
The catalog MUST define closed tagged variants for hold, linear, cubic-Bézier, and spring curves; ordered alpha/luma mask operations; and ordered typed effects, and every numeric parameter and represented collection MUST be finite and subject to its explicit named inclusive complexity limit in valid and invalid envelopes before later runtime activation. Every catalog limit MUST be a positive JavaScript-safe integer in both language validators. Duplicate payload-derived mask and effect definitions and duplicate invalid-envelope context identifiers MUST be rejected before set normalization or semantic classification. Project-level limits MUST count all payload-derived project definitions, and composition limits MUST count all payload-derived records sharing the same exact `root` or `component:<id>` owner even when those records occur in separate fixtures. The catalog MUST NOT declare an inline-resource complexity limit when no version-1 payload can represent that resource. Resource-bearing fields MUST accept only typed managed identifiers and MUST reject filesystem paths, traversal, URI schemes, network resources, executable SVG, every ASCII SVG event-handler attribute, and raw renderer expressions without applying resource restrictions to ordinary text content. Safety classification MUST inspect every mask in declaration order.

#### Scenario: Reject duplicate definitions in an invalid envelope
- **WHEN** an invalid layer, mask, or renderer-expression candidate repeats a scoped definition ID
- **THEN** both validators reject the duplicate invalid envelope before deriving its declared semantic failure

#### Scenario: Inspect every executable SVG source
- **WHEN** executable inline SVG appears in any mask through a script element or a case-insensitive ASCII event-handler attribute
- **THEN** both validators classify the candidate as executable SVG regardless of the mask's array position or handler spelling

#### Scenario: Enforce safe catalog limits
- **WHEN** any catalog limit equals JavaScript's maximum safe integer or the first larger integer
- **THEN** both wrapper validators accept the inclusive maximum and reject the larger value before semantic validation

#### Scenario: Enforce an invalid animation-envelope limit
- **WHEN** an invalid curve, mask, or effect candidate contains its named inclusive collection limit or one additional otherwise valid item
- **THEN** both validators accept the boundary for fixture-specific classification and reject the overflow with the exact named limit invariant

### Requirement: Canonical marker and semantic audio-event fixtures
Marker and audio-event fixtures MUST use unique structured scoped marker references, absolute or marker-relative JavaScript-safe-integer timing, semantic sound-event identifiers, finite decibel gain, deterministic JavaScript-safe unsigned variant seeds, and typed project-scoped audio-bus and managed-asset references; invalid marker and audio-event definition identity MUST use the exact `{ scope, id }` tuple, permitting equal IDs in different legal composition scopes while rejecting repeats within one scope; every invalid-audio sound-definition bus reference MUST resolve to a declared project bus during common preflight before resource or semantic failure classification; the catalog's managed-resource collection MUST contain only unique `{ kind: "asset", scope: "project", id }` tuples; valid and invalid scenario definition and context identifiers MUST be unique before normalization except for exactly one semantic lookup key intentionally repeated by a fixture whose independent expectation declares the corresponding ambiguity reason; `maxMarkersPerComposition` and `maxAudioEventsPerComposition` MUST be applied independently to records grouped by exact composition scope; and fixtures MUST provide complete deterministic missing and ambiguous scenarios for markers, sound definitions, variants, and buses.

#### Scenario: Reject an unrelated missing sound-definition bus
- **WHEN** any invalid audio fixture references an undeclared bus from a sound definition
- **THEN** both validators reject the exact missing sound-definition bus invariant before the fixture's declared failure can be classified

#### Scenario: Preserve a resolved intentional bus ambiguity
- **WHEN** the bus-ambiguity fixture's sound definition references its one present event-referenced duplicated bus key
- **THEN** both validators preserve the canonical `audio_bus_ambiguous` classification instead of rejecting the resolved definition-bus reference

#### Scenario: Preserve scoped definition identity
- **WHEN** invalid audio-event or marker definitions reuse an ID in different legal composition scopes or repeat it within the same scope
- **THEN** both validators accept the cross-scope identities and reject the same-scope duplicate tuple before semantic classification

#### Scenario: Enforce composition-owned audio limits
- **WHEN** invalid audio events or markers collectively exceed a per-composition limit across independent scopes or exceed the inclusive limit within one scope
- **THEN** both validators accept the distributed records and reject the overflowing owner with the exact named limit invariant

#### Scenario: Permit only the declared ambiguity key
- **WHEN** a marker, bus, sound-definition, or variant ambiguity fixture contains its expected repeated lookup key and also repeats a different semantic key
- **THEN** both validators reject the unrelated duplicate before reporting the fixture's declared ambiguity

#### Scenario: Reject an invalid managed-resource tuple
- **WHEN** the managed-resource collection contains a non-asset kind, a non-project scope, a duplicate asset tuple, or a payload references an unmanaged asset
- **THEN** both validators reject the catalog before the resource can participate in reference closure

#### Scenario: Enforce marker field parity
- **WHEN** a marker ID or name violates the identifier grammar or its timestamp exceeds JavaScript's maximum safe integer
- **THEN** Rust and TypeScript both reject the marker payload with identical boundary semantics

#### Scenario: Reject unrelated duplicate marker or audio context
- **WHEN** an invalid marker or audio candidate repeats a marker ID, asset ID, event ID, or other context definition unrelated to its declared ambiguity
- **THEN** both validators reject the exact duplicate-definition invariant before fixture-specific classification

#### Scenario: Enforce invalid marker and audio limits
- **WHEN** an invalid marker or audio-event envelope is at its named inclusive limit or contains one additional otherwise valid record
- **THEN** both validators accept the boundary for fixture-specific classification and reject the overflow with the exact named limit invariant

### Requirement: Cross-language fixture evidence and adoption
Rust and TypeScript tests MUST consume the same canonical motion-graphics catalog and verify its version, fixture-only status, strict closed catalog and concept payloads, globally unique fixture identities, exact identifier catalogs, finite values, project-wide and owner-scoped aggregate represented limits, named collection limits in valid and invalid envelopes grouped by their semantic owners, pre-normalization payload, context, dependency-edge, and metadata uniqueness, scoped invalid audio-event and marker definition identity, exact-key ambiguity exemptions, invalid-audio sound-definition bus closure before failure derivation, unique payload-derived structured reference closure, exact project-scoped managed-asset tuples, legal kind/scope combinations, field-specific safety invariants, complete branching dependency graphs, and actual exact per-concept failure IDs, concepts, classifications, and reasons. Every invalid fixture MUST pass mirrored common-field validation for all fields except its one fixture-ID-specific intentional defect before that defect is classified. A later milestone that activates any concept MUST update every affected native declaration, public catalog, capability, stable error, migration, and parity consumer in the same approved change where applicable.

#### Scenario: Prove definition-bus preflight across audio failures
- **WHEN** the sound-definition bus is removed and restored in each canonical invalid audio fixture
- **THEN** mirrored tests first observe the exact missing-reference invariant and then recover the fixture's original exact concept, classification, and reason

#### Scenario: Reject ambiguity-exemption drift
- **WHEN** either language broadens an ambiguity fixture's exemption beyond its one event-referenced or lookup-named duplicated key
- **THEN** mirrored unrelated-duplicate mutations fail the contract evidence instead of allowing the declared ambiguity to hide the drift

#### Scenario: Reject a relabeled invalid fixture
- **WHEN** an invalid fixture keeps its ID and payload but declares a different valid concept
- **THEN** both validators reject the concept mismatch before fixture-specific classification

#### Scenario: Preserve exact corrected negative evidence
- **WHEN** every unchanged canonical invalid fixture is validated after concept, uniqueness, safety, and wrapper-bound corrections
- **THEN** both languages still derive its independently expected ID, concept, classification, and reason exactly

#### Scenario: Reject an unrelated invalid-envelope defect
- **WHEN** an invalid fixture adds a duplicate definition, duplicate context identifier, duplicate dependency edge, or collection overflow unrelated to its declared failure
- **THEN** both validators reject the exact common invariant before the declared semantic failure can be observed

### Requirement: Governed runtime Transform2D contract
A versioned transform2d-v1 runtime catalog MUST define the complete closed Transform2D payload, defaults, bounds, switching rules, coordinate semantics, valid and invalid fixtures, and its mapping to existing fixture vocabulary. The remaining motion-graphics-v1 catalog MUST stay fixture-only. Contract ownership, typed headless requests/responses, MCP Zod schemas, existing update/batch surfaces, persisted consumers, and parity evidence MUST agree. Ready implementations MUST advertise the additive transform2d capability without removing existing capabilities or changing protocol major version. No new provider or stable-error contract SHALL be introduced.

#### Scenario: Discover and use support
- **WHEN** a client reads capabilities from a runtime with complete Transform2D support
- **THEN** it sees transform2d and can submit a complete typed update standalone or in a batch and read the resulting value

#### Scenario: Enforce cross-language parity
- **WHEN** Rust and TypeScript validate canonical success, boundary, unknown-field, invalid-number, unsupported-unit, and conflicting-update fixtures
- **THEN** both agree on the documented payload acceptance and failure semantics

#### Scenario: Keep roadmap concepts inactive
- **WHEN** the client inspects contracts after this milestone
- **THEN** only static Transform2D is newly addressable and other motion-graphics concepts remain fixture-only

#### Scenario: Preserve old request compatibility
- **WHEN** an existing valid legacy request is sent to the new runtime
- **THEN** its shape, transform meaning, and stable error/retryability behavior remain supported
