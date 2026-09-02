## Why

Issue #11 is the contract-first boundary between the accepted motion-graphics architecture and the persisted/public model work in later milestones. The repository currently has no canonical, versioned vocabulary that Rust, TypeScript/Zod, MCP, and future project migrations can use to agree on transforms, layers, components, slots, markers, curves, masks, effects, and audio events. Implementing those concepts independently would make wire names, enum values, ordering semantics, validation failures, and compatibility behavior drift before the parity gate in issue #16 can enforce them.

## What Changes

- Add a versioned canonical motion-graphics fixture catalog containing closed, typed examples and identifier sets for transforms, layers, components, slots, markers and time expressions, animation curves, masks, ordered effects, and semantic audio events.
- Record the observable coordinate, timing, ordering, transform, compositing, reference, fallback, and finite-value rules that the examples encode.
- Include deterministic valid and invalid cases for type/range failures, missing references, cycles, unsafe resources, non-finite-number encodings, and explicit complexity-limit violations.
- Register the fixture catalog in the contract ownership manifest and document how later persisted or agent-addressable milestones adopt it through the existing fixture-governed synchronization workflow.
- Add focused Rust and TypeScript tests that consume the same checked-in catalog and prove its version, category coverage, identifiers, references, safety invariants, valid examples, and intended failure classifications.
- Non-goals: add runtime motion-graphics domain types, change schema version 6, expose headless operations or MCP tools, advertise a capability, migrate projects/history, evaluate or render the fixtures, or change current preview/export output.

## Capabilities

### New Capabilities

- `motion-graphics-contracts`: Defines the canonical pre-implementation vocabulary, examples, observable semantics, validation classifications, and adoption rules for motion-graphics contracts shared across languages.

### Modified Capabilities

- None. The catalog is preparatory contract evidence and does not activate a public or persisted feature.

## Impact

- Affects `contracts/`, the contract ownership manifest, focused Rust/TypeScript contract tests, and contract documentation.
- The change is additive and does not alter the headless protocol, MCP surface, stable errors, capability report, persisted project schema, provider protocols, or renderer behavior.
- Later milestones must either adopt the version-1 vocabulary exactly or propose a reviewed versioned fixture change together with all affected native declarations and compatibility evidence.
