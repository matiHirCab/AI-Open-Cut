## MODIFIED Requirements

### Requirement: Canonical JSON representation enforcement
VectorColor, VectorPoint, GradientStop, Stroke, CornerRadii, and VectorPath MUST decode only from JSON objects with their existing exact required fields, including at every nested occurrence. LineCap, LineJoin, and FillRule MUST decode only from their existing canonical JSON strings. Positional arrays, object-form unit enums, scalar substitutes, and missing, unknown, or duplicate fields MUST fail structural decoding before semantic validation. Raw JSON decoding MUST preserve duplicate-key detection rather than normalize duplicates away. Paint and PathCommand MUST retain their existing internally tagged object representations. Valid payloads, public Rust fields, serialization, semantic errors, numeric limits, path grammar, and radius resolution MUST remain unchanged.

Raw JSON buffering through activated shape consumers MUST preserve duplicate entries until the strict vector records decode. Validation MUST remain in core. Already-parsed objects MUST retain existing behavior; duplicates erased by a caller are not recoverable.

#### Scenario: Reject positional records at any nesting depth
- **WHEN** a correctly sized positional array replaces any vector record at top level or inside paint, stroke, gradient stops, or any path-command point field
- **THEN** Rust raw-string and Value decoders reject the representation and the TypeScript schemas reject the equivalent parsed value

#### Scenario: Reject object-form string enums
- **WHEN** a cap, join, or fill rule uses an object such as {"butt":null}, either standalone or nested
- **THEN** structural decoding rejects it while all existing exact string identifiers remain valid

#### Scenario: Preserve strict object parsing and valid round trips
- **WHEN** valid objects have reordered keys, or malformed objects contain missing, unknown, or duplicate fields
- **THEN** valid values preserve canonical serialized fields and semantic behavior, missing/unknown fields fail both decoding paths, and duplicate keys fail raw JSON decoding including nested records

#### Scenario: Reject duplicates through every shape consumer
- **WHEN** raw JSON for a single edit, batch, draft, component definition, project document, or retained history contains duplicate fields in nested vector records, whether identical or invalid-first/valid-last
- **THEN** typed structural decoding rejects the input before mutation or normalization, preserving project revision, current state, history, and batch/draft atomicity

#### Scenario: Preserve compatibility and conflict behavior
- **WHEN** valid schema-14 or supported legacy documents and valid operations pass through duplicate-preserving decoding, including omitted/null fields and stale revisions
- **THEN** existing migrations, serialization, null semantics, revision conflicts, error mappings and valid legacy behavior remain unchanged, with no new wire fields or schema version
