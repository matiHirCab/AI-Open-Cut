## ADDED Requirements

### Requirement: Marker-aware item start edits
Core SHALL add a typed item-start edit that accepts either absolute nonnegative milliseconds or a marker-relative expression for a root or component-local item while preserving all existing item move, trim, split and duplicate behavior. An item carrying an expression MUST retain or clear it by documented edit semantics: a numeric move, component instance timing update, or numeric trim that changes `startMs` clears it; a duration-only trim whose supplied `startMs` equals the current effective start or a non-timing edit retains it. Same-composition `duplicate_items` and `component_instance_duplicate` MUST copy the marker reference and add their numeric duplication offset to the signed marker offset with checked safe-integer arithmetic, preserving the existing duplication shift; overflow MUST reject the entire edit. Splitting a relative item MUST clear the expression on both resulting items because each split has an independently determined start. Every mutation MUST revalidate affected item intervals before publication. Existing component replacement MUST retain its marker collection and validate all replacement-track expressions against that collection.

#### Scenario: Replace a relative start with an absolute start
- **WHEN** a numeric move or absolute item-start edit targets an item with a marker expression
- **THEN** its stored expression is cleared and its new absolute time follows existing bounds and revision rules

#### Scenario: Retain expression in compatible edits
- **WHEN** a non-timing edit or same-composition duplication with a nonzero offset targets an item with a valid marker expression
- **THEN** the original expression remains unchanged and a duplicate receives a distinct ID and a checked adjusted marker offset that produces the requested shifted start

#### Scenario: Reject a duplicate offset overflow
- **WHEN** a duplication offset would exceed the allowed signed marker offset or item interval
- **THEN** core rejects the complete standalone edit or batch without publishing a copy

#### Scenario: Disambiguate trim and split
- **WHEN** a trim supplies the current effective start or changes that start, or a relative item is split
- **THEN** the expression is retained only for the duration-only trim and is cleared for a changed numeric start or either split result
