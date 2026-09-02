## Context

The version-1 motion-graphics catalog remains test-only, but its current negative evidence is not causal: each invalid value only has to fail a schema, after which tests copy the catalog's expected reason. Most negative values are incomplete outer shapes, so reference, ambiguity, cycle, and limit rules are never executed. The catalog also declares limits that are not consumed, TypeScript normalizes some duplicate metadata away, and native boundary semantics differ for safe integers, strings, IDs, and dimensions.

The correction remains fixture-governed under ADR 0002. It must preserve the two prior archives, add no production declarations or dependencies, and keep version 1 inactive.

## Goals / Non-Goals

**Goals:**

- Make every catalog negative case fail for exactly one derived classification/reason.
- Exercise graph, resolution, slot-value, limit, scope, duplicate, and resource rules with complete scenario data.
- Make Rust and TypeScript accept and reject the same catalog and boundary mutations.
- Keep catalog metadata, validators, documentation, ownership, and living requirements synchronized.

**Non-Goals:**

- Activate motion graphics in persistence, editor-core production code, headless, MCP, capabilities, providers, evaluation, preview, export, or packaging.
- Add stable runtime errors, migrations, schema generation, or dependencies.
- Edit either archived predecessor change.

## Decisions

### Return deterministic fixture failures

Both test helpers will use a closed fixture-level failure value containing `classification` and `reason`. Parsing and semantic validation will return that value for catalog negative cases, and the test-owned expectation matrix will compare the observed tuple with the fixture ID and declared tuple. Merely failing deserialization is insufficient for a semantic fixture.

Invalid scenario envelopes will contain all unrelated required data. Layer scenarios retain complete layer arrays. Component scenarios carry definition and instance collections so missing references, direct/indirect cycles, and depth overflow are representable. Slot scenarios carry a strict definition plus an optional tagged supplied value so absence and kind mismatch are semantic states. Marker/audio scenarios carry candidate collections so missing and duplicate marker, sound, variant, and bus resolution can be evaluated.

Keeping the current boolean check was rejected because it cannot associate a failure with its declared reason. Parsing error strings was rejected because Serde and Zod diagnostics are not a stable shared vocabulary.

### Treat fixture metadata as an exact unique set

Each structured reference has a legal kind/scope combination. Managed resources are project-scoped assets. Components, sound definitions, and buses are project scoped; layers, transforms, markers, curves, masks, effects, and audio events are root or component scoped; slots are component scoped. Metadata arrays are checked for duplicates before normalization, then compared exactly with payload-derived unique tuples. Global closure and graph checks run only after metadata agrees with payloads.

Silently deduplicating arrays was rejected because it hides malformed canonical evidence.

### Consume every represented limit

Validators will read limits from the catalog and apply them to the corresponding scenario collections and graph traversal. Every limit receives inclusive-boundary and overflow evidence. The three inline-SVG limits are removed because version 1 accepts no inline-SVG payload; attempted SVG fields remain closed-schema failures and executable SVG remains an explicit unsafe-resource regression. Retaining inactive numeric promises was rejected because they cannot be verified against a representable value.

### Use one cross-language numeric and string model

Unsigned fixture integers are limited to `0..=9_007_199_254_740_991`; signed offsets use the symmetric JavaScript-safe range. Rust may deserialize fixed-width integers but must apply the safe bound semantically. Length constraints count Unicode scalar values with Rust `chars().count()` and TypeScript `Array.from(value).length`. IDs remain ASCII and closed. Dimensions, names, ranges, variants, and required fields use identical constants and checks.

UTF-8 byte length was rejected because it diverges from TypeScript for ordinary Unicode. Grapheme counting was rejected because it requires a new dependency and is not required by the fixture contract.

### Keep manual native declarations

Rust will add closed Serde types for the complete catalog and wrappers, while TypeScript keeps mirrored strict Zod schemas. Both retain independent hardcoded identifier and invalid-expectation sets so changing the JSON alone cannot redefine the test oracle.

Generation or a third schema system was rejected because this fixture-only correction must preserve ADR 0002 and add no dependency.

## Risks / Trade-offs

- [Larger fixture scenarios make the catalog more verbose] → Keep one canonical complete scenario per failure and factor only validator code, not JSON evidence.
- [Hand-authored validators may drift again] → Add the same boundary mutation matrix in both suites and exact identifier assertions.
- [Removing SVG limits could appear to weaken safety] → Document that inline SVG is not accepted in version 1 and retain explicit rejection tests for SVG, event handlers, scripts, paths, URIs, and expressions.
- [Fixture reshaping could look like a breaking contract] → Keep `fixture_only` version 1 and state that no consumer has activated or persisted these shapes.

## Migration Plan

No data or runtime migration exists. After approval, update the catalog and both test helpers together, run all gates, verify the change, sync the living specification, and archive it. Rollback removes this corrective change's catalog/test/spec edits together; the prior archives remain untouched.

## Open Questions

None.
