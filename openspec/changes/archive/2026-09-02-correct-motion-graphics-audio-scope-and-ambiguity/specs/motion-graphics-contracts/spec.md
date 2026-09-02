## MODIFIED Requirements

### Requirement: Canonical marker and semantic audio-event fixtures
Marker and audio-event fixtures MUST use unique structured scoped marker references, absolute or marker-relative JavaScript-safe-integer timing, semantic sound-event identifiers, finite decibel gain, deterministic JavaScript-safe unsigned variant seeds, and typed project-scoped audio-bus and managed-asset references; invalid marker and audio-event definition identity MUST use the exact `{ scope, id }` tuple, permitting equal IDs in different legal composition scopes while rejecting repeats within one scope; the catalog's managed-resource collection MUST contain only unique `{ kind: "asset", scope: "project", id }` tuples; valid and invalid scenario definition and context identifiers MUST be unique before normalization except for exactly one semantic lookup key intentionally repeated by a fixture whose independent expectation declares the corresponding ambiguity reason; `maxMarkersPerComposition` and `maxAudioEventsPerComposition` MUST be applied independently to records grouped by exact composition scope; and fixtures MUST provide complete deterministic missing and ambiguous scenarios for markers, sound definitions, variants, and buses.

#### Scenario: Preserve scoped definition identity
- **WHEN** invalid audio-event or marker definitions reuse an ID in different legal composition scopes or repeat it within the same scope
- **THEN** both validators accept the cross-scope identities and reject the same-scope duplicate tuple before semantic classification

#### Scenario: Enforce composition-owned audio limits
- **WHEN** invalid audio events or markers collectively exceed a per-composition limit across independent scopes or exceed the inclusive limit within one scope
- **THEN** both validators accept the distributed records and reject the overflowing owner with the exact named limit invariant

#### Scenario: Permit only the declared ambiguity key
- **WHEN** a marker, bus, sound-definition, or variant ambiguity fixture contains its expected repeated lookup key and also repeats a different semantic key
- **THEN** both validators reject the unrelated duplicate before reporting the fixture's declared ambiguity

### Requirement: Cross-language fixture evidence and adoption
Rust and TypeScript tests MUST consume the same canonical motion-graphics catalog and verify its version, fixture-only status, strict closed catalog and concept payloads, globally unique fixture identities, exact identifier catalogs, finite values, project-wide and owner-scoped aggregate represented limits, named collection limits in valid and invalid envelopes grouped by their semantic owners, pre-normalization payload, context, dependency-edge, and metadata uniqueness, scoped invalid audio-event and marker definition identity, exact-key ambiguity exemptions, unique payload-derived structured reference closure, exact project-scoped managed-asset tuples, legal kind/scope combinations, field-specific safety invariants, complete branching dependency graphs, and actual exact per-concept failure IDs, concepts, classifications, and reasons. Every invalid fixture MUST pass mirrored common-field validation for all fields except its one fixture-ID-specific intentional defect before that defect is classified. A later milestone that activates any concept MUST update every affected native declaration, public catalog, capability, stable error, migration, and parity consumer in the same approved change where applicable.

#### Scenario: Reject ambiguity-exemption drift
- **WHEN** either language broadens an ambiguity fixture's exemption beyond its one event-referenced or lookup-named duplicated key
- **THEN** mirrored unrelated-duplicate mutations fail the contract evidence instead of allowing the declared ambiguity to hide the drift
