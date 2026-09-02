## Why

The fixture-only motion-graphics catalog added for issue #11 establishes the right concepts but its validators close only the outer fixture wrapper. Payload values remain untyped, reference metadata is detached from composition scope, required negative audio and slot cases are incomplete, and the archived living specification retains a generated placeholder purpose. Those gaps allow Rust and TypeScript to accept internally inconsistent fixtures while still reporting contract parity.

## What Changes

- Add strict, closed, test-only Rust Serde and TypeScript Zod payload declarations for every valid fixture concept without activating runtime support.
- Replace string definition/reference metadata with structured `{ kind, scope, id }` records and verify metadata derived from payloads against scope-aware reference closure.
- Complete deterministic hierarchy, slot, marker, sound-definition, variant, and bus failure fixtures and require exact failure IDs, classifications, and reason keys in both languages.
- Replace global unsafe-string scanning with typed resource-field validation for managed identifiers, paths, URIs, SVG execution features, and renderer expressions while leaving ordinary text unrestricted.
- Replace the living specification's placeholder purpose and keep ownership/reviewer coverage synchronized.
- Non-goals: change project schema version 6, add editor-core runtime models, expose headless or MCP behavior, add capabilities or stable errors, migrate data, or change rendering.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `motion-graphics-contracts`: Strengthens fixture payload typing, scope-aware references, deterministic negative coverage, and cross-language evidence.

## Impact

- Affects the version-1 fixture-only catalog, focused Rust/TypeScript contract tests, the living specification purpose, ownership metadata, and documentation.
- The correction remains additive to the repository's runtime behavior and does not modify any public, persisted, provider, or renderer contract.
- Version 1 is retained because the fixture has not been merged, released, or activated as a runtime contract.
