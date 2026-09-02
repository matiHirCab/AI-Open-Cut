## Context

Adversarial mutations proved three gaps that canonical fixtures do not expose: duplicate component, slot, marker, mask-context, and audio identifiers can survive invalid-envelope preflight; most invalid scenario collections are not checked against their named catalog limits; and component dependencies are normalized into a single-successor map that overwrites earlier outgoing edges.

The catalog remains fixture-only. These corrections affect test-support validation and governance evidence only, with no public or persisted compatibility impact.

## Goals / Non-Goals

**Goals:**

- Make every invalid scenario valid under all common uniqueness and limit invariants except its one declared defect.
- Preserve intentional ambiguity fixtures through narrowly keyed exemptions rather than broadly permitting duplicates.
- Traverse the complete directed component graph and classify missing endpoints, cycles, and maximum depth deterministically.
- Prove equivalent behavior with mirrored Rust and TypeScript mutations that assert exact invariant names.

**Non-Goals:**

- Activate motion graphics in runtime models, persistence, headless, MCP, capabilities, providers, evaluation, preview, export, or packaging.
- Add dependencies, generated schemas, public errors, ownership movement, or a new catalog version.
- Modify an archived change.

## Decisions

### Centralize invalid-envelope collection preflight

Each scenario-family validator will check definition and context arrays before semantic classification. Component IDs and dependency pairs, slot target-layer IDs, marker IDs, mask available-layer IDs, audio asset IDs and event IDs, and equivalent definition collections must be unique before any set or map normalization. Bus IDs, marker names, sound-definition event keys, and variant asset IDs may repeat only for the fixture whose independent expectation declares the matching ambiguity reason; definition IDs remain unique even in those fixtures.

### Pass named limits through the complete invalid classifier

The Rust and TypeScript classifier entry points will receive the complete validated limits object. Each invalid scenario preflight will apply the named limit owned by the collection it represents, including components, layers, markers, masks, effects, curve/keyframe collections, and audio events. Boundary values are inclusive. An overflow returns an error containing the exact catalog limit name before fixture-specific failure derivation.

### Use adjacency lists and active-path traversal

Component dependencies will be stored as ordered adjacency lists rather than a single-successor map. Preflight first rejects duplicate component IDs and duplicate directed edges, then verifies the entry and every endpoint exist. A depth-first traversal from the entry uses an active-path set to detect direct or indirect cycles and explores every outgoing edge. If no cycle is found, the classifier compares the longest reachable node path with `maxComponentDepth`. Declaration order is retained for deterministic traversal, while missing-reference validation always precedes graph classification.

## Risks / Trade-offs

- [Ambiguity fixtures intentionally contain duplicate lookup keys] → Key each exemption by fixture ID and expected reason, while continuing to require unique definition IDs.
- [A malformed graph can contain both missing endpoints and cycles] → Validate all endpoints before traversal so missing references have deterministic precedence.
- [A graph can exceed depth and also contain a cycle] → Detect cycles during complete traversal before returning the computed depth overflow, preserving cycle evidence independently of branch order.

## Migration Plan

No runtime or data migration applies. After approval, update both test validators and mirrored regressions, update documentation, run all required gates, verify requirement/design/task/code alignment, sync the delta into the living specification, and archive only this corrective change.

## Open Questions

None.
