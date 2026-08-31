## Context

PR #93 now derives Rust operation names with Strum and snapshots normalized Zod JSON Schemas. A variant-level Serde rename is independent from Strum metadata, so the catalog, Strum, and parity test can remain aligned while runtime deserialization changes. Separately, the JSON normalizer sorts schema keys but retains `description`, although ADR 0002 explicitly excludes description copy from compatibility parity.

The fix remains test-only and must not alter protocol version 1, runtime request handling, MCP registrations, or the manually governed catalog.

## Goals / Non-Goals

**Goals:**

- Verify that every Strum-derived operation name is recognized by the actual Serde `Request` deserializer.
- Make the existing description-exclusion policy executable and recursively deterministic.
- Add focused regression coverage for both gaps.

**Non-Goals:**

- Replace Strum, expose Serde's generated private variant table, or add a schema generator.
- Add a full valid request fixture for every operation.
- Exclude JSON Schema keywords other than `description` or exclude any tool annotation.
- Change the checked-in MCP definitions when their current schemas contain no descriptions.

## Decisions

### Probe actual Serde tag recognition

For every `Request::VARIANTS` value, deserialize a minimal object containing only that `operation`. A successful parse or a missing-field error proves Serde recognized the tag; an unknown-variant error proves the tag metadata drifted. The test first deserializes a reserved invalid control tag and verifies Serde's unknown-variant diagnostic, then requires that diagnostic to be absent for every derived name.

This is preferred over another duplicate name list, which recreates the original defect, and over full valid request fixtures, which would duplicate every request payload solely to discover tags. Serde does not expose the derive-generated variant table as a public API, so the controlled deserialization diagnostic is the narrowest direct check.

### Strip descriptions only while normalizing schemas

Keep the generic recursive key sorter for annotations and other JSON values. Add schema-specific normalization that recursively omits object entries named `description`, preserves all remaining keys, and preserves array order. Apply it after `z.toJSONSchema` for both input and output modes.

This is preferred over post-processing the checked-in catalog because live and expected definitions must share a clearly documented compatibility projection. It also avoids accidentally removing a future annotation field with the same name.

### Prove both boundaries with focused tests

Add a Rust regression using a small test-only enum whose Serde rename intentionally differs from its Strum snake-case name, demonstrating that the tag-recognition helper rejects the mismatch. Add TypeScript tests showing nested Zod descriptions normalize identically while a structural constraint change still differs.

## Risks / Trade-offs

- [Serde changes the text of unknown-variant diagnostics] -> Validate a reserved invalid control tag in the same test run and classify real tags relative to that observed diagnostic prefix.
- [A future schema keyword named `description` carries structural meaning] -> JSON Schema defines `description` as annotation metadata; exclude only that exact object key in schema normalization.
- [The test-only renamed enum does not exercise the production enum declaration] -> The production loop uses `Request::VARIANTS` and `Request::deserialize`; the small enum only proves the helper detects the failure mode.

## Migration Plan

1. Add the Serde tag-recognition helper and regressions without changing the production enum.
2. Add schema-specific description filtering and its structural-control regression.
3. Run the complete contract, Rust, TypeScript, integration, smoke, Python, and OpenSpec checks.
4. Synchronize and archive this follow-up; rollback is a test-only revert with no data or protocol migration.

## Open Questions

None.
