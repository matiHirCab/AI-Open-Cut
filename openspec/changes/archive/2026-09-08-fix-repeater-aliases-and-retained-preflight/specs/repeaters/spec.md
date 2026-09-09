## ADDED Requirements

### Requirement: Retained repeater validation domains
Core MUST validate complete repeater expansion independently of visibility before generated-layer materialization. The root expansion and every component definition MUST each have an independent budget domain; independent definitions MUST NOT be added to the root total. Actual instances MUST use effective slots and composed clocks/transforms; standalone definitions MUST use their own canvas and defaults. Hidden items/tracks, hidden or interval-clipped instances, unused definitions, and local repeaters copied externally MUST undergo the existing inclusive occurrence, visual/audio/resource, segment, surface and memory checks. Empty output intervals MUST NOT bypass finite arithmetic or final parent-conjugated matrix validation. Publication MUST retain only visible occurrences and preserve existing identity, order, RichText, bindings and timing.

#### Scenario: Validate hidden and unused layer boundaries
- **WHEN** root, hidden-track, hidden-repeater, hidden-instance, clipped-instance or unused-definition expansion has exactly 4,096 versus 4,097 visual occurrences
- **THEN** the inclusive boundary passes subject to other limits and one-over returns INVALID_ARGUMENT before generated materialization; hidden content produces no visible output

#### Scenario: Validate retained geometry and conjugation
- **WHEN** retained expansion reaches an exact versus one-over segment, surface or memory boundary, or finite offset powers yield a non-finite final parent-conjugated matrix
- **THEN** exact budgets pass subject to other limits and invalid work fails with INVALID_ARGUMENT before copies, raster allocation, renderer or artifact I/O

#### Scenario: Keep independent domains independent
- **WHEN** multiple unused definitions individually fit their limits but their sum exceeds one domain's limit, or definitions are reordered
- **THEN** validation accepts each independent domain consistently without adding its work to the root budget

#### Scenario: Preserve effective nested occurrences
- **WHEN** nested instances resolve slot overrides and RichText, contain local repeaters, and are copied by two outer repeaters
- **THEN** complete projected work is validated and published visible occurrences preserve deterministic identities, numeric order, intervals, transforms, opacity and isolated bindings, with independent source copies

### Requirement: Bounded ordinary evaluation without redundant snapshots
Ordinary evaluation MUST NOT deep-clone its entire visual collection for repeater discovery. Repeater expansion MUST use one shared projection/publication path with lightweight source references and MUST complete retained-domain validation before cloning any generated layer.

#### Scenario: Evaluate ordinary geometry without copy allocation
- **WHEN** a project without repeaters evaluates ordinary shapes or nested components
- **THEN** no repeater snapshot or generated-copy materialization occurs and semantic output remains unchanged

#### Scenario: Reject before the first generated clone
- **WHEN** any later repeater or independent retained domain exceeds a budget after earlier valid candidates
- **THEN** evaluation returns INVALID_ARGUMENT with zero generated-layer materializations and no partial scene or artifact
