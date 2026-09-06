## ADDED Requirements

### Requirement: Canonical JSON representation enforcement
VectorColor, VectorPoint, GradientStop, Stroke, CornerRadii, and VectorPath MUST decode only from JSON objects with their existing exact required fields, including at every nested occurrence. LineCap, LineJoin, and FillRule MUST decode only from their existing canonical JSON strings. Positional arrays, object-form unit enums, scalar substitutes, and missing, unknown, or duplicate fields MUST fail structural decoding before semantic validation. Raw JSON decoding MUST preserve duplicate-key detection rather than normalize duplicates away. Paint and PathCommand MUST retain their existing internally tagged object representations. Valid payloads, public Rust fields, serialization, semantic errors, numeric limits, path grammar, and radius resolution MUST remain unchanged.

#### Scenario: Reject positional records at any nesting depth
- **WHEN** a correctly sized positional array replaces any vector record at top level or inside paint, stroke, gradient stops, or any path-command point field
- **THEN** Rust raw-string and Value decoders reject the representation and the TypeScript schemas reject the equivalent parsed value

#### Scenario: Reject object-form string enums
- **WHEN** a cap, join, or fill rule uses an object such as {"butt":null}, either standalone or nested
- **THEN** structural decoding rejects it while all existing exact string identifiers remain valid

#### Scenario: Preserve strict object parsing and valid round trips
- **WHEN** valid objects have reordered keys, or malformed objects contain missing, unknown, or duplicate fields
- **THEN** valid values preserve canonical serialized fields and semantic behavior, missing/unknown fields fail both decoding paths, and duplicate keys fail raw JSON decoding including nested records

### Requirement: Representation regression evidence
The canonical version-1 vector catalog and both native test consumers MUST cover structural representation rejection distinctly from semantic failure without changing activation status. Rust catalog and fixture wrappers MUST themselves accept objects only, the fixture kind MUST be a canonical string, and required fields, metadata and unique identities MUST retain existing validation. Canonical successful fixtures MUST continue passing both Rust decoding paths and TypeScript validation. Provider, project, headless/MCP, and render contracts MUST remain unchanged.

#### Scenario: Reject alternate catalog envelopes
- **WHEN** a catalog or fixture wrapper is a positional array or its kind is an object-form enum, or a required field is absent
- **THEN** the Rust and TypeScript catalog consumers reject the envelope instead of normalizing it into a valid fixture

#### Scenario: Prove representation parity and unchanged behavior
- **WHEN** the shared fixture suites and existing workflow regressions execute
- **THEN** both languages reject every structural counterexample, all previous valid and semantic-invalid cases retain their outcomes, and existing state/history/revision/render workflows retain their behavior
