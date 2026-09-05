## ADDED Requirements

### Requirement: Explicit flat scene stacking
EvaluatedScene MUST order visible visual instructions bottom-to-top by ascending track array index, signed zIndex, stackOrder, and stable item ID as the final tie-break for synthesized equivalent keys. Canonical stackOrder MUST match item array position, preserving the existing equal-z-index array rule. Higher z-index MUST NOT cross track boundaries. Hidden filtering MUST NOT renumber persisted ordering. Audio summing/routing, transition endpoint association, half-open timing, transforms, and evaluation immutability MUST retain existing semantics.

#### Scenario: Resolve overlapping visual layers
- **WHEN** overlapping layers have negative, positive, and equal z-index across multiple tracks
- **THEN** repeated evaluation yields the exact comparator order including stable equal-z-index array order and final synthesized ID ties

#### Scenario: Preserve nonvisual semantics
- **WHEN** tracks or items with audio, transitions, and hidden content are reordered
- **THEN** visual ordering follows canonical keys without changing audio routing/gain, transition references, timing, or the source project

#### Scenario: Reject complexity before ordering
- **WHEN** the scene exceeds an existing canonical evaluation limit
- **THEN** evaluation returns its existing typed failure before sorting, rasterization, process execution, or artifact publication
