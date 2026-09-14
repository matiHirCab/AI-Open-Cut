## ADDED Requirements

### Requirement: Complete draft matching ambiguity detection
Core MUST prioritize structural matches before font-intent matches and evaluate one-to-one alternatives in both directions with bounded analysis. Different complete retained bindings or retained versus newly unresolved outcomes MUST fail with INVALID_ARGUMENT before font publication or authoritative mutation. Equivalent retention outcomes MUST be paired deterministically in original order. Existing revision, catalog, integrity and work-limit checks MUST remain effective.

#### Scenario: One old action has several replacements
- **WHEN** one old font action can match either of two non-exact replacements
- **THEN** both replacement orders fail atomically with an ambiguity message and unchanged draft, project, history and owned font bytes

#### Scenario: Exact and equivalent assignments
- **WHEN** operations are inserted, removed, reordered or repeatedly edited
- **THEN** exact matches take priority, differing alternative bindings fail, and identical retention outcomes succeed deterministically

### Requirement: Component child font retention
Paired component actions MUST retain complete bindings per scoped local text ID with unchanged selector presence/value. Pairing MUST use operation kind and explicit component identity, or overlapping local text IDs for anonymous creation, subject to ambiguity detection. Non-font edits, track movement and ordering MUST NOT reset bindings. New IDs and changed selectors MUST resolve with current configuration; removed IDs MUST NOT contribute retention. Retention MUST NOT leak to sibling identities or other components sharing local IDs.

#### Scenario: Partial component font edit
- **WHEN** a component-create or component-update draft changes one child's selector after defaults change or source removal
- **THEN** untouched siblings retain all four font hashes through preview, reopen and commit while the changed child resolves normally

#### Scenario: Local identity changes
- **WHEN** children are inserted, removed, reordered, moved between tracks or edited without selector changes
- **THEN** unchanged identities retain bindings, new identities resolve normally and other components remain independent

### Requirement: Representable draft font steps
Core MUST preserve project schema 19, draft version 2 and opencut-text-v2 without new public fields. Each persisted selector entry MUST represent the complete binding of every newly unresolved identity sharing that selector in its step. Differing bindings MUST fail atomically with descriptive INVALID_ARGUMENT before font publication; identical bindings MUST succeed.

#### Scenario: Conflicting selector outcomes
- **WHEN** a retained component child and a new or changed child share a selector but require different bindings
- **THEN** replacement fails with unchanged draft, project, history and owned font bytes

#### Scenario: Equivalent selector outcomes
- **WHEN** multiple component children sharing a selector resolve or retain the same complete binding
- **THEN** the draft remains readable as version 2 and reproduces those bindings after reopen and commit

#### Scenario: Inherited binding conflict
- **WHEN** inserted actions cause a component child to inherit a different binding that selector-keyed draft replay cannot replace with the retained binding
- **THEN** replacement fails atomically with descriptive INVALID_ARGUMENT and publishes no staged fonts

### Requirement: Explicit component selector resolution
Matched component actions MUST explicitly resolve changed selector presence or values and newly introduced local text IDs using current configuration, even when operation application inherits a base binding. Internal resolution markers MUST participate in ambiguity comparisons and MUST NOT equal inherited or retained outcomes solely because hashes coincide. Core MUST capture and clear marked inherited bindings in memory before resolution. If the complete resolved binding differs from the captured binding and normal v2 replay would retain that binding, core MUST reject atomically with descriptive INVALID_ARGUMENT before publication. Equal complete bindings MUST succeed. Selector steps MUST only describe identities normal operation replay leaves unresolved. Markers MUST NOT alter stored operations or public fields.

#### Scenario: Selector reversion conflicts with base binding
- **WHEN** a matched replacement reverts a family or path selector, including null resets, from draft font B to a base selector whose inherited font A differs from current resolution C
- **THEN** replacement rejects with unchanged draft, project, history and font bytes, including differences confined to styled faces

#### Scenario: Equivalent selector reversion
- **WHEN** current resolution and the inherited complete binding agree after a selector reversion
- **THEN** replacement succeeds identically through preview, reopen, repeated replacement and commit while unchanged siblings retain bindings

#### Scenario: Reintroduced local identity
- **WHEN** a matched replacement reintroduces a local text ID missing from the previous operation but present in the base component
- **THEN** it explicitly resolves and accepts only representable results under the same inherited-binding comparison

#### Scenario: Resolution outcome ambiguity
- **WHEN** alternative component matches would explicitly resolve a local ID or retain or inherit its binding
- **THEN** the differing intent outcomes reject as ambiguous even when current hashes could coincide

### Requirement: Matching against prepared draft prefixes
Core MUST compare matching outcomes against actual complete bindings supplied by preceding replacement operations, including retained and freshly resolved fonts. Bounded weighted assignment MUST maximize structural matches first and total compatible matches second, retaining every globally optimal candidate and unmatched possibility. Canonical original-order assignment MUST NOT narrow later ambiguity checks. Core MUST advance chronological in-memory preparation only after all alternatives agree for a step; any later error MUST leave authoritative and owned font bytes unchanged. No assignment permutation enumeration or public format change is permitted.

#### Scenario: Equivalent inheritance from a preceding action
- **WHEN** an earlier exact or font-intent-matched action supplies a pinned font and a later non-font action is replaced by equivalent actions
- **THEN** both replacement orders succeed through preview, reopen and commit, including changed defaults or removed sources

#### Scenario: Resolution differs from preceding inheritance
- **WHEN** an earlier action supplies a font and alternatives for later replacements explicitly resolve or inherit that font
- **THEN** both orders reject as ambiguous even if current complete bindings coincide, with unchanged draft, project, history and font bytes

#### Scenario: Chronological chains and global alternatives
- **WHEN** prefixes include freshly resolved fonts, multiple component identities, or longer chains of retained actions
- **THEN** comparison uses actual scoped prefix bindings, preserves exact priority and checks all global alternatives even when earlier equivalent pairs were canonicalized
