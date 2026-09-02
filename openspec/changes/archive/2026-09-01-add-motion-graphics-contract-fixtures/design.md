## Context

ADR 0004 fixes the motion-graphics architecture but intentionally leaves exact public type names to later milestones. Issue #11 must turn the architectural concepts into canonical cross-language examples before issue #12 introduces `EvaluatedScene` and before milestones M1-M5 add persisted or public behavior. ADR 0002 requires fixture-governed manual synchronization rather than generated native declarations.

The existing canonical contracts are JSON artifacts under `contracts/`. The new catalog must be useful to Rust, TypeScript/Zod, and MCP authors without pretending that unimplemented examples are already accepted by project persistence or public transports.

## Goals / Non-Goals

**Goals:**

- Define one versioned, language-neutral wire vocabulary for the nine concept groups named by issue #11.
- Encode ADR 0004's observable coordinate, timing, ordering, transform, compositing, safety, and deterministic-reference rules in machine-readable metadata and representative fixtures.
- Provide positive and negative examples that later native parity tests can consume without rewriting the catalog.
- Make the activation boundary explicit: catalog presence does not imply runtime, persisted, capability, headless, MCP, preview, or export support.

**Non-Goals:**

- Implement editor-core structs, validation, migrations, evaluation, rendering, mutations, batch aliases, undo/redo behavior, or reopen behavior for the new concepts.
- Add speculative public operations, responses, MCP schemas, resources, stable errors, or capability identifiers.
- Select the final rasterizer, effect implementation, numeric complexity constants for features not yet implemented, or a new schema version.
- Duplicate current project fixtures or use the catalog as executable renderer input.

## Decisions

### Use one canonical catalog with closed fixture records

Add `contracts/motion-graphics-v1.json` as the canonical artifact for the vocabulary. Its top-level records are `version`, `status`, `semantics`, `limits`, `identifiers`, `validFixtures`, and `invalidFixtures`. Fixture records are closed objects with stable `id`, `concept`, and `value` fields; negative records additionally contain one expected failure class and a stable reason key.

One catalog is preferred to nine unrelated files because cross-concept references are central: component instances bind slots, keyframes use marker-relative time and curves, layers own ordered masks/effects, and audio events resolve markers and sound definitions. Stable fixture IDs let tests reference complete cases without relying on array positions. Separate files were rejected because they make referential examples and version changes harder to review atomically.

The catalog is fixture-governed, not a JSON Schema and not code generation input. JSON Schema was rejected as the canonical owner because Serde, Zod refinements, fixed-width integer semantics, reference resolution, graph cycles, and MCP annotations do not translate losslessly. Later code may derive test inputs from examples but native declarations remain hand-authored under ADR 0002.

### Lock lower-camel wire names and tagged variants

Fixture object fields use lower camel case. Tagged unions use a closed lower-snake-case `type` or concept-specific discriminator. The initial vocabulary includes:

- transforms: explicit position units (`pixels` or `normalized`), anchor X/Y, independent scale, rotation/skew degrees, and opacity;
- layers: stable IDs, composition scope, parent reference, integer `zIndex`, blend mode, clip, ordered masks/effects, animation channels, and hidden state;
- components and slots: component dimensions/duration/tracks/markers, typed stable slot bindings, defaults/constraints, and instances with finite positive time scale;
- markers and time expressions: unique scoped marker names and either absolute integer milliseconds or a marker name plus signed integer offset;
- curves: `hold`, `linear`, `cubic_bezier`, and finite positive-parameter `spring` variants;
- masks: alpha/luma sources, ordered add/subtract/intersect/exclude operations, inversion, feather, expansion, transform, and typed managed references;
- effects: ordered tagged effects with typed finite parameters and no renderer expressions;
- audio events: semantic event identifier, marker/absolute placement, finite dB gain, deterministic variant seed, and typed bus reference.

Exact feature-specific limits remain implementation decisions, but the catalog defines named limit keys and bounded fixture values so every later owning layer must replace the preparatory limit with an explicit reviewed constant before activation. Omitting a required limit, using an unbounded collection, or treating a non-finite string token as a number is an invalid fixture.

### Encode semantics without activating behavior

The catalog status is `fixture_only`. It declares that no project schema, headless protocol, MCP tool, capability, provider response, evaluator, preview, or export path accepts these records yet. `contracts/contract-ownership-v1.json` gains a `motionGraphicsVocabulary` category whose canonical owner is the catalog and whose current consumers are the focused Rust and TypeScript catalog tests. Later milestones add their actual native consumers to the ownership manifest in the same change that activates a concept.

No changes are made to `headless-protocol-v1.json`, `mcp-surface-v1.json`, `error-codes-v1.json`, capability reporting, or `Project`. This avoids advertising support before editor-core owns validation and persistence. Adding placeholder operations was rejected because clients could interpret their presence as usable behavior.

### Classify deterministic negative fixtures

Negative cases use one of four fixture-level classifications:

- `invalid_input` for wrong tags, ranges, non-finite tokens, conflicting fields, unsafe inline/external content, and explicit complexity-limit violations;
- `missing_reference` for absent parent, component, marker, slot target, mask source, sound event, or audio bus;
- `reference_cycle` for parent, component, or mask dependency cycles;
- `ambiguous_reference` for duplicate marker names or bindings that cannot resolve uniquely in one composition scope.

These are fixture classifications, not new stable editor error codes. The later activating milestone maps them to an approved stable error catalog and tests atomic rollback, revisions, undo/redo, migrations, aliases, and reopen behavior where applicable. This avoids preselecting runtime errors before operations exist while preserving the intended failure path.

### Cross-language evidence is structural in this issue

Focused tests in Rust and TypeScript read the same catalog and verify JSON parseability, exact version/status, unique fixture IDs, all nine concept groups, known identifier sets, finite JSON numeric values, reference closure for valid examples, absence of paths/URLs/renderer expressions/executable SVG, required explicit limit keys, and deterministic classification of negative examples. Each test also uses a deliberately malformed in-memory fixture to prove the validator's failure path.

Issue #16 remains responsible for CI drift enforcement against future native Rust types, TypeScript/Zod schemas, MCP declarations, and renderer-output fixtures. This issue provides the canonical data that gate will consume; it does not collapse the two backlog items.

## Compatibility, Persistence, and Security

This is an additive catalog-only change. It does not change a public request/response, capability, resource, provider protocol, persisted document, schema version, revision, history, or renderer output. Therefore no migration, atomic persistence path, revision-conflict behavior, batch alias, undo/redo, or reopen implementation is applicable in this change.

Fixtures use only logical IDs and managed/content-addressed resource references. They contain no absolute or relative filesystem paths, network URLs, raw FFmpeg expressions, scripts, event handlers, or external SVG resources. Numeric values must be JSON numbers and finite in both native readers; strings such as `NaN`, `Infinity`, and `-Infinity` are reserved invalid inputs.

## Risks / Trade-offs

- **The vocabulary may evolve before runtime adoption.** Mitigation: version the catalog and require later incompatible changes to introduce a reviewed new major fixture or an explicit compatibility path.
- **Fixture-only types can be mistaken for supported API.** Mitigation: include machine-readable `fixture_only` status, document the activation boundary, and leave protocol/MCP/capability catalogs unchanged.
- **Examples do not prove every valid value.** Mitigation: pair representative values with closed identifier sets, semantic metadata, negative classifications, and later native parity/property tests.
- **Preparatory limit names may outlive poor initial choices.** Mitigation: lock the requirement for explicit limits now but defer exact runtime constants to the activating OpenSpec change.

## Rollout and Rollback

1. Add the versioned catalog, ownership entry, documentation, and structural cross-language tests together.
2. Validate the change and run the existing contract, Rust, and TypeScript suites.
3. Later milestones adopt individual concepts by updating the catalog if needed, native declarations, public fixtures, capability reporting, stable errors, migrations, and parity evidence together.
4. Rollback before activation removes the catalog, ownership entry, tests, and documentation together; no project or external data rollback is required.

## Open Questions

None for this fixture-only boundary. Runtime constants and stable error mappings are deliberately assigned to the OpenSpec change that activates each concept.
