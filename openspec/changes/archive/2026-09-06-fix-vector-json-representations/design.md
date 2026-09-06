## Context

Review reproduced Rust acceptance of positional records such as [1,0,0,1] and enum objects such as {"butt":null}, while the matching Zod schemas reject them. Existing tests use empty arrays and miss correctly sized alternatives. The original archived change remains historical evidence.

## Goals / Non-Goals

Goals: restore the exact approved JSON representations at every nesting depth, preserve strict duplicate/missing/unknown field rejection, and provide cross-language negative evidence.

Non-goals: changing valid data, semantic validation, TypeScript production schemas, project schema, headless/MCP operations, rendering, dependencies, or unrelated models.

## Decisions

- Implement a private generic object visitor in vector.rs. Its only data visitor is visit_map; invoke deserialize_map and delegate MapAccessDeserializer to a strict private field struct. Implement Deserialize for the six record types with those field structs while retaining Serialize and public fields. This avoids whole-Value buffering that would erase raw duplicate keys and keeps ownership within the existing module.
- Implement Deserialize for LineCap, LineJoin, and FillRule by deserializing a String and matching only the current exact identifiers. Unknown strings use Serde unknown_variant errors. Keep Paint and PathCommand internally tagged, with strict private tagged enum decoders behind the same outer map-only visitor; their nested public records also enforce object shape. Verification reproduced Serde's additional internally tagged sequence aliases, so guarding the outer enums is necessary to fulfill the already-approved object representation requirement.
- Apply equivalent strict decoding to test-only Catalog and Fixture wrappers and string-only Kind. Retain strict metadata comparison, fixture identity checks, and the required value field.
- Extend the existing canonical fixture catalog with structural-rejection evidence without changing version or activation. Make both language consumers read that evidence; Rust explicitly exercises raw JSON and Value decoding. Keep duplicate-key cases in raw Rust JSON tests because parsed JSON values cannot retain duplicate keys and TypeScript schemas receive already-parsed values.
- Keep parse failures as Serde errors and pure semantic errors as existing INVALID_ARGUMENT. No transport is activated, so no new public error mapping is introduced.

## Risks / Trade-offs

Manual field-helper declarations can drift from serialization: valid/reordered round-trip tests cover every record and enum. Nested tagged decoding can differ from top-level decoding: test all nested point/color/stop/enum sites. Empty-array rejection alone is insufficient: fixtures must use correct-length positional alternatives. Wrapper tests must exercise actual parsing rather than only metadata validation after normalization.

## Migration and rollback

This is a correction to the existing approved contract, not a new supported wire format. Valid JSON, catalog version 1, schema 13, retained history, revisions, and rendering remain unchanged; migration and new alias/reference tests are inapplicable. Rollback is a coordinated revert of correction code, fixtures, and docs, with no persisted data changes.

## Approval and verification

The detailed plan was explicitly approved for implementation on 2026-09-06. Tests are added and observed failing before decoder changes. Run all checks in tasks.md, use openspec-verify-change, obtain completed contract-owner review, and sync/archive before the final Moon gate. Preserve original archival artifacts.
