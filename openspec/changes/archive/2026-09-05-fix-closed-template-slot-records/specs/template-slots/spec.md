## MODIFIED Requirements

### Requirement: Closed typed slot definitions and values
Core MUST support component-local slots with `id`, `name`, `kind`, `required`, optional `defaultValue`, `binding` and kind-specific `constraints`. Values MUST be closed `{type,value}` records with matching types `text`, `rich_text`, `color`, `number`, `boolean`, `enum`, `duration`, or `asset`; null, coercion, unknown fields and mismatched tags MUST fail with non-retryable INVALID_ARGUMENT. Slot IDs MUST match `[A-Za-z0-9_-]{1,128}`, names MUST be nonblank and at most 256 UTF-8 bytes, and IDs MUST be unique within their definition. Core MUST validate every default even when an override exists. A required slot without a default MUST be legal on a definition, but every stored instance MUST supply a value for it. Override takes precedence over default; an absent optional value MUST leave the original target property unchanged. Unknown override IDs MUST fail with ITEM_NOT_FOUND. Overrides MUST NOT modify the shared definition or be materialized into its tracks.

#### Scenario: Round-trip all eight kinds
- **WHEN** definitions and nested instances contain valid values of every supported kind
- **THEN** reads, undo/redo and reopen preserve exact typed values and deterministic override/default precedence

#### Scenario: Validate absent and invalid values
- **WHEN** a default has the wrong tag, an override is null or unknown, a required effective value is absent, or an optional value is omitted
- **THEN** core rejects the invalid candidates with the specified error and accepts optional omission without changing the target

#### Scenario: Preserve special slot identifiers
- **WHEN** definitions and instances use __proto__, constructor or toString as legal slot IDs, including required slots without defaults and overridden defaults
- **THEN** native persistence and bridge reads preserve every own typed override exactly across undo, redo and reopen, overrides take precedence and no object prototype changes

#### Scenario: Validate special-key values without dropping entries
- **WHEN** an own special-key override is malformed, a required value is missing or the key does not name a declared slot
- **THEN** malformed input is rejected with INVALID_ARGUMENT and a key-prefixed bridge validation path, missing required values fail with INVALID_ARGUMENT, and structurally valid unknown IDs reach core and fail with ITEM_NOT_FOUND without mutation

Closed slot records MUST reject every unknown own enumerable field, including __proto__, constructor, toString and ordinary unknown names, before fields can be discarded. This applies to slot definitions, bindings, constraints, each typed value envelope, rich-text documents/runs and managed-asset references. Override-map keys MUST remain unrestricted by record-field closure and retain the existing slot-ID contract.

#### Scenario: Reject unknown fields throughout closed slot records
- **WHEN** a default or override contains an unknown own field at any applicable closed-record location, or a definition, binding or constraints record contains one
- **THEN** native and bridge structural validation reject it rather than silently removing it, bridge diagnostics identify the full containing-record path and offending keys, and no project or history mutation occurs

#### Scenario: Preserve valid records and special map identifiers
- **WHEN** all eight valid value kinds and legitimate special slot IDs pass through guarded record validation
- **THEN** exact parsed values, prototypes, required/default precedence, undo/redo/reopen and group-opacity behavior remain unchanged
