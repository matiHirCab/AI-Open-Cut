## ADDED Requirements

### Requirement: Portable shape semantic references

Stored shape golden semantic references MUST preserve exact structure and exact authored geometry, transforms, paint, density, bounds, timing and ordering. Only corresponding finite generated contour x/y coordinates MAY differ across platforms, and acceptance MUST require both an absolute difference no greater than 1e-12 and a distance no greater than 8 ULPs. Signed-zero differences, non-finite coordinates, malformed structure, missing/extra fields and all other semantic differences MUST fail. Repeated evaluation within one runtime MUST retain exact plan equality. Recipe hashes, reviewed pixel references, existing visual/audio/timing thresholds, legacy and rule-card comparisons MUST remain unchanged. This test-only portability policy MUST NOT change production evaluation, public contracts, persisted data or revision behavior.

#### Scenario: Accept measured platform contour variation
- **WHEN** the stored and evaluated shape plans differ only in generated contour coordinates within both approved numeric bounds, including the three observed Linux/Windows pairs
- **THEN** stored-reference conformance passes while repeated local evaluation still requires exact equality

#### Scenario: Reject semantic drift and excessive numeric variation
- **WHEN** a generated contour coordinate exceeds either numeric bound, or any authored geometry, transform, paint, density, bounds, timing, ordering, structure or field count changes
- **THEN** semantic reference conformance fails with a diagnostic identifying the mismatch

#### Scenario: Fail closed on unsupported coordinates and structure
- **WHEN** contour coordinates are non-finite, signed-zero variants differ, coordinate scope cannot be established, or the plan is malformed
- **THEN** portability comparison fails without recapturing references or changing rendering behavior

#### Scenario: Preserve independent golden evidence
- **WHEN** native shape and legacy conformance runs on the corrected PR
- **THEN** the committed recipe hash, unchanged pixel references and established visual/audio/timing checks pass, and hosted render/foundation parity succeeds
