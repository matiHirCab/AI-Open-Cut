## Why

Review of issue #27 proved that Rust's derived Serde decoders accept positional arrays for vector records and object-form string enums. Those inputs violate the existing vector JSON contract and are rejected by TypeScript, despite all existing fixtures passing.

## What Changes

- Require map-only decoding for vector records, including nested records, and string-only decoding for caps, joins, and fill rules.
- Preserve duplicate-key detection through streaming map visitors rather than buffering JSON values.
- Harden Rust catalog/fixture wrappers and their kind discriminator against the same representation alternatives.
- Add shared negative fixtures and raw-string/value decoding tests; retain all existing canonical serialization and semantic validation.

## Capabilities

### New Capabilities
None.

### Modified Capabilities
- `vector-primitives`: Explicit canonical JSON representations, strict structural rejection, and cross-language regression evidence.

## Impact

Changes are confined to the existing core vector module, its fixture/test consumers, the canonical vector catalog, and documentation. This repairs undocumented acceptance rather than narrowing the approved contract. Catalog version 1, core_primitives_only activation, schema 13, Rust public type names/fields, valid wire shapes, headless/MCP surfaces, and renderer semantics remain unchanged. No new dependency or migration is required.

## Non-goals

No project/timeline activation, ShapeItem rendering, public operation, domain-rule change, commit, push, or rewrite of the original archived change.

## Approval

The user explicitly instructed implementation of the complete reviewed fix plan on 2026-09-06. These artifacts transcribe that approved scope, approach, and acceptance criteria before implementation. Completed contract-owner review remains a finalization step.
