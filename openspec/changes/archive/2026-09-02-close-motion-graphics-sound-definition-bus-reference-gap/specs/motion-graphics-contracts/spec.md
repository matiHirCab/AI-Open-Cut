## MODIFIED Requirements

### Requirement: Canonical marker and semantic audio-event fixtures
Marker and audio-event fixtures MUST use unique structured scoped marker references, absolute or marker-relative JavaScript-safe-integer timing, semantic sound-event identifiers, finite decibel gain, deterministic JavaScript-safe unsigned variant seeds, and typed project-scoped audio-bus and managed-asset references; invalid marker and audio-event definition identity MUST use the exact `{ scope, id }` tuple, permitting equal IDs in different legal composition scopes while rejecting repeats within one scope; every invalid-audio sound-definition bus reference MUST resolve to a declared project bus during common preflight before resource or semantic failure classification; the catalog's managed-resource collection MUST contain only unique `{ kind: "asset", scope: "project", id }` tuples; valid and invalid scenario definition and context identifiers MUST be unique before normalization except for exactly one semantic lookup key intentionally repeated by a fixture whose independent expectation declares the corresponding ambiguity reason; `maxMarkersPerComposition` and `maxAudioEventsPerComposition` MUST be applied independently to records grouped by exact composition scope; and fixtures MUST provide complete deterministic missing and ambiguous scenarios for markers, sound definitions, variants, and buses.

#### Scenario: Reject an unrelated missing sound-definition bus
- **WHEN** any invalid audio fixture references an undeclared bus from a sound definition
- **THEN** both validators reject the exact missing sound-definition bus invariant before the fixture's declared failure can be classified

#### Scenario: Preserve a resolved intentional bus ambiguity
- **WHEN** the bus-ambiguity fixture's sound definition references its one present event-referenced duplicated bus key
- **THEN** both validators preserve the canonical `audio_bus_ambiguous` classification instead of rejecting the resolved definition-bus reference

### Requirement: Cross-language fixture evidence and adoption
Rust and TypeScript tests MUST consume the same canonical motion-graphics catalog and verify its version, fixture-only status, strict closed catalog and concept payloads, globally unique fixture identities, exact identifier catalogs, finite values, project-wide and owner-scoped aggregate represented limits, named collection limits in valid and invalid envelopes grouped by their semantic owners, pre-normalization payload, context, dependency-edge, and metadata uniqueness, scoped invalid audio-event and marker definition identity, exact-key ambiguity exemptions, invalid-audio sound-definition bus closure before failure derivation, unique payload-derived structured reference closure, exact project-scoped managed-asset tuples, legal kind/scope combinations, field-specific safety invariants, complete branching dependency graphs, and actual exact per-concept failure IDs, concepts, classifications, and reasons. Every invalid fixture MUST pass mirrored common-field validation for all fields except its one fixture-ID-specific intentional defect before that defect is classified. A later milestone that activates any concept MUST update every affected native declaration, public catalog, capability, stable error, migration, and parity consumer in the same approved change where applicable.

#### Scenario: Prove definition-bus preflight across audio failures
- **WHEN** the sound-definition bus is removed and restored in each canonical invalid audio fixture
- **THEN** mirrored tests first observe the exact missing-reference invariant and then recover the fixture's original exact concept, classification, and reason
