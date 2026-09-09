## ADDED Requirements

### Requirement: Complete retained transition fact budgets
Core MUST enforce the inclusive 4,096 transition-fact limit across all ordinary and generated visual occurrences in each independent root or component-definition validation domain. Each attached endpoint role MUST count as one fact, including both roles when endpoints coincide. Hidden transitions/tracks, hidden or clipped occurrences, and nested local/exterior copies MUST participate in validation without changing visible publication. Checked accumulation MUST reject overflow or excess with non-retryable INVALID_ARGUMENT before any generated layer materialization, backend execution or artifact I/O.

#### Scenario: Reject the reproduced expanded transition excess
- **WHEN** a component with 17 attached transition facts is evaluated ordinarily and repeated 256 additional times
- **THEN** the 4,369 projected facts return INVALID_ARGUMENT with zero generated materializations and no partial scene

#### Scenario: Accept exact and reject one-over fact limits
- **WHEN** root or local/exterior repeater expansion produces exactly 4,096 versus 4,097 endpoint facts, including self-endpoint roles
- **THEN** the exact boundary succeeds subject to other limits and one-over fails before generated copies

#### Scenario: Validate retained transitions without publishing them
- **WHEN** transitions, tracks, repeaters or instances are hidden or interval-clipped and retained expansion reaches or exceeds the transition limit
- **THEN** the same inclusive boundary applies while hidden transition facts and hidden occurrences remain absent from published output

#### Scenario: Isolate domains and reject later failures
- **WHEN** independent definitions individually fit but their sum exceeds one domain, or a later domain exceeds the limit after earlier valid candidates
- **THEN** valid independent definitions are accepted regardless of declaration order and the later invalid domain fails with zero generated materializations and no artifact I/O

### Requirement: Effective visual-only repeater sources
Repeater source closures MUST be visual-only after resolving component defaults and instance overrides. Core MUST validate root closures, standalone definitions with defaults, and actual effective instances, including group descendants, nested components and local repeaters. An effective media asset with audio MUST cause non-retryable INVALID_ARGUMENT regardless of visibility, clipping or mute. Validation MUST establish references, acyclicity and slot validity before effective-audio traversal, retain active-node cycle defenses, and isolate results for distinct effective values of the same definition. Valid silent sources MUST preserve existing IDs, ordering, bindings, timing and RichText behavior without audio repetition.

#### Scenario: Reject audio introduced by slots
- **WHEN** a default or instance asset override introduces audio into a repeated component or a component descendant of a repeated group, including nested components and local repeaters
- **THEN** canonical validation and direct evaluation return INVALID_ARGUMENT before state publication, generated materialization or renderer/artifact work

#### Scenario: Preserve effective-value isolation and silent sources
- **WHEN** different instances of one definition resolve silent and audible assets, or a source resolves an authored audible asset to a silent effective asset
- **THEN** classification follows each effective value independently of declaration order, silent sources remain valid subject to independent definition validation, and audible repeated sources are rejected

#### Scenario: Validate retained closures safely
- **WHEN** an effective audible source is hidden, muted, clipped or unused, or the input contains invalid references, cycles or slot values
- **THEN** retained audio is still rejected and invalid references, cycles and slots retain their established typed failures without unbounded recursion
