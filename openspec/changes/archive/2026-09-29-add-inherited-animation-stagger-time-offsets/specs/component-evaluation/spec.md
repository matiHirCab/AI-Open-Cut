## ADDED Requirements

### Requirement: Component child stagger composes with local time
A component instance MUST apply its optional staggerMs after the existing affine parent-to-definition clock mapping and before evaluating each direct top-level visual child branch. Nested instances MUST compose their own local clock and stagger independently. Source trim, timeScale, source-duration validation, slot values, occurrence identity, stacking and half-open parent activity MUST preserve their existing meanings. Zero or absent stagger MUST retain exact prior output. Component-instance duplication MUST copy staggerMs unless explicitly replaced, without mutating the shared definition.

#### Scenario: Resolve staggered component occurrences
- **WHEN** two instances share a definition but have different staggerMs, trimStartMs and timeScale values
- **THEN** each child branch resolves its own local activity and animation phase without changing the shared definition or other occurrence

#### Scenario: Keep nested bounds
- **WHEN** nested instance mapping and stagger yield a valid exact boundary versus unsafe derived time
- **THEN** valid boundaries evaluate with exclusive ends and unsafe time returns INVALID_ARGUMENT before publication


