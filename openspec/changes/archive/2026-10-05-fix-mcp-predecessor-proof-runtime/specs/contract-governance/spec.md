## ADDED Requirements

### Requirement: Bounded exact MCP predecessor conformance proof
The canonical MCP conformance suite MUST prove complete deterministic current expansion and the exact schema33→32→31→pre-linear predecessor digests within the existing single-test5000ms deadline on supported CI platforms. The proof MAY reuse its already independently expanded read-only module fixture and one already-checked compact predecessor source, but MUST independently expand the current source again, compare complete serialized JSON bytes, retain every nested field/scalar/array/key order and assert the unchanged78-tool count, all four pinned SHA256 values and exactly-one linear capability removal. The current serialized bytes MAY be reused for their digest assertion. Strict projection helpers, source/result isolation, all malformed/missing/cyclic/nonlocal/sibling/unused-reference and unrelated/schema/description/annotation drift controls, actual registered schema parity and public catalogs MUST remain unchanged. No timeout extension, test splitting, suppression, helper aliasing change or CI-policy weakening SHALL substitute for this proof.

#### Scenario: Preserve the complete current and predecessor proof
- **WHEN** the canonical schema33 MCP catalog is checked using two independent expansions and the checked33→32→31 chain
- **THEN** complete current bytes agree and all four existing pinned digests,78tools and linear-capability multiplicity pass within the unchanged5000ms deadline, with source/projection inputs unmodified

#### Scenario: Preserve malformed and unrelated drift rejection
- **WHEN** malformed references, missing/malformed/duplicated mask additions, misplaced matching keys, unrelated fields/descriptions/annotations or unexpected fixture mutation is introduced
- **THEN** unchanged expansion/projection/drift/current-registration controls and exact current-byte or predecessor-digest assertions reject it without narrowing compared content or weakening the deadline
