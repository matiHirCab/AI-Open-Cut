## MODIFIED Requirements

### Requirement: Bounded exact MCP predecessor conformance proof
The canonical MCP conformance suite MUST prove complete deterministic current expansion and the exact schema35→34→33→32→31→pre-linear predecessor digests within the existing single-test5000ms deadline on supported CI platforms. The proof MAY reuse its already independently expanded read-only module fixture and one already-checked compact predecessor source, but MUST independently expand the current source again, compare complete serialized JSON bytes, retain every nested field/scalar/array/key order and assert the unchanged78-tool count, all six pinned SHA256 values (current schema35 and every unchanged schema34/schema33/schema32/schema31/pre-linear predecessor) and exactly-one linear capability removal. The current serialized bytes MAY be reused for their digest assertion. Fresh JSON-tree copying MAY avoid temporary entry tuples using direct own-key copying, but MUST preserve every own key (including literal __proto__ as data), ordinary object prototypes, scalar/array/key order and independently fresh nested identities without serialization loss or cached aliases. Strict projection helpers, source/result isolation, all malformed/missing/cyclic/nonlocal/sibling/unused-reference and unrelated/schema/description/annotation drift controls, actual registered schema parity and public catalog changes MUST be limited to the separately approved issue53 blend additions and exact marker transitions. No timeout extension, test splitting, suppression, helper aliasing change or CI-policy weakening SHALL substitute for this proof.

#### Scenario: Preserve the complete current and predecessor proof
- **WHEN** the canonical schema35 MCP catalog is checked using two independent expansions and the checked35→34→33→32→31 chain
- **THEN** complete current bytes agree and the approved current digest and all five unchanged predecessor digests,78tools and linear-capability multiplicity pass within the unchanged5000ms deadline, with source/projection inputs unmodified

#### Scenario: Preserve malformed and unrelated drift rejection
- **WHEN** malformed references, missing/malformed/duplicated mask/matte/blend additions, misplaced matching keys, unrelated fields/descriptions/annotations or unexpected fixture mutation is introduced
- **THEN** unchanged expansion/projection/drift/current-registration controls and exact current-byte or predecessor-digest assertions reject it without narrowing compared content or weakening the deadline

#### Scenario: Preserve fresh own-key JSON copies within bounded proof
- **WHEN** repeated expanded references contain nested arrays/objects, ordered keys, null/booleans/strings/finite numbers including negative zero and a literal own __proto__ field
- **THEN** each reference and each expansion has independent fresh nested copies with ordinary prototypes, exact scalar/order/own-key values and no source or sibling mutation, while every complete canonical digest remains asserted in the unchanged single5000ms test

