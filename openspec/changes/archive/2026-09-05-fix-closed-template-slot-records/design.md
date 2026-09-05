## Context

The existing closed slot contract is enforced by Rust's Serde declarations, but Zod strict objects can discard an own enumerable `__proto__` field. The previous safe override-map parser correctly retains arbitrary string keys; record closure must not restrict those keys. Both previous archived changes and the group-opacity correction remain intact.

## Goals / Non-Goals

**Goals:** Restore structural rejection parity for every closed slot record, preserve inferred types and nested diagnostics, retain parsed results and published schemas, and prove transport failure atomicity using canonical raw JSON.

**Non-Goals:** Domain validation in the bridge, broader schema refactoring, renderer changes, migrations, dependency changes, catalog schema drift, commits or pushes.

## Decisions

### Validate structure before delegating value parsing

Add one generic wrapper in the owning bridge schema module using public Zod APIs. For non-null, non-array objects, inspect `Object.keys` before invoking the supplied schema. Reject every key outside an explicit allowed-key set using an `unrecognized_keys` issue; otherwise call the existing schema's safe parser, return its parsed data, and forward its issues without discarding nested paths. Non-objects delegate to the existing parser for existing type validation. Preserve the supplied schema's input/output TypeScript types; any generic assertion must be supported by the delegate's parsed result, not unchecked original input. Do not mutate the input or its prototype.

Derive allowed keys from existing object shapes for definitions, bindings, constraints, rich-text documents/runs and managed-asset references. Wrap the complete discriminated value union with exactly `type` and `value` allowed; preserve ordinary object variants within the union so discriminator handling remains valid. Every request/response consumer must use these shared guarded declarations. Unknown-key issues retain the full containing-record path and name all offending keys; value errors retain their full leaf path, including map IDs and array indices.

Relying on `.strict()` alone is the demonstrated defect. A special-name blacklist would miss ordinary extras and could accidentally reject valid slot IDs. Private Zod internals and dependency upgrades are excluded.

### Keep maps open and schemas stable

Retain the existing slot override-map validation and `Object.fromEntries` reconstruction. Map keys are IDs, not record fields. Malformed values under special IDs still pass through guarded value parsing, and structurally valid unknown IDs still reach core for ITEM_NOT_FOUND.

Derive JSON Schema metadata from the existing declarations using public `z.toJSONSchema` and metadata APIs. Compare guarded and original declarations for both input and output schemas, and run registered MCP structural catalog parity. Do not edit the published MCP catalog to mask differences. Metadata describes the existing closed contract; it does not introduce new fields or semantics.

### Canonical negatives use raw JSON

Extend the existing template-slot catalog with a dedicated closed-record negative fixture collection. Give each case a stable name, record location, offending key and expected structural rejection. Include complete slot/default or override data suitable for consumers, with expected bridge containing-record paths and offending keys. Cover each of `__proto__`, `constructor`, `toString` and an ordinary unknown field at definition, binding, constraints, every one of the eight value envelopes, rich-text document/run and asset-reference locations. Exercise defaults and overrides wherever the record can occur; definition/binding/constraints occur only on definitions.

Load canonical fixture files from raw JSON bytes in TypeScript and Rust; do not rely on executable object literals or bundler JSON transformations for special properties. Rust asserts structural deserialization rejection; bridge asserts rejection with complete nested issue paths and named unknown keys. Native decoding messages need not adopt Zod's error format. Existing semantic error codes and retryability remain unchanged.

### Prove real failure atomicity

Extend the shared source integration/packaged smoke component workflow to submit canonical malformed defaults and overrides in standalone edits and aliased batches, including a valid operation preceding a malformed batch operation. Capture state, revision and project/history bytes before each failure and compare afterward. Retain positive all-eight-kind, special-ID/prototype, required/default precedence, undo/redo/reopen and group-opacity evidence. Core production code should remain unchanged unless approved requirements expose a missing structural rejection; do not duplicate semantic validation in transports.

## Risks / Trade-offs

- Wrapper typing or metadata could hide drift → type checks, parsed-result assertions and exact input/output/registered-MCP schema equality are mandatory.
- Special fields could disappear while constructing tests → raw JSON loading plus explicit own-property assertions before parsing.
- Nested issue prefixes could be lost → assert full containing-record and leaf paths in definition, override and batch contexts.
- Broad fixture combinations increase runtime → use exhaustive unit/contract matrices and the shared transport workflow, keeping established positive regressions.
- Previously accepted malformed records become rejected → this restores the already published closed contract and native behavior; no persisted schema or migration changes are warranted.

## Migration Plan

No migration or deployment action is required. Schema 12, protocol 1 and published structural schemas remain unchanged. The correction adds no external resource access or sensitive data. If verification fails, retain the change as incomplete and correct it within the approved scope; any rollback must preserve unrelated work and both existing archives.

## Verification

Record named regression evidence for each new scenario and retain the prior positive regressions. Run Rust formatting, strict workspace Clippy, workspace tests with established FFmpeg 6/ffprobe/font configuration and serialized tests where required; bridge contracts:check, typecheck, lint, unit, source integration and packaged smoke; hermetic Python tests; strict OpenSpec validation and diff checks. Confirm the MCP catalog remains byte-identical and registered schemas match. Run openspec-verify-change, resolve mismatches, obtain completed designated CODEOWNER review, synchronize and archive, then run the final Moon OpenSpec gate. Failed or skipped required checks block completion.

## Open Questions

No implementation decision is outstanding. Artifact approval was received on 2026-09-05; completed contract-owner review was explicitly approved on 2026-09-05.
