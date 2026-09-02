## Context

Adversarial mutation of `soundDefinition.busId` to an undeclared ID is accepted by all canonical invalid audio fixtures because definition-bus membership is never checked. Shape validation succeeds, then the fixture's existing variant, bus, sound, or marker failure is classified and matches the independent expectation matrix.

The catalog remains fixture-only. The correction belongs in mirrored test-support validation and does not alter any runtime contract.

## Goals / Non-Goals

**Goals:**

- Resolve every invalid-audio sound-definition bus before failure derivation.
- Preserve the canonical bus-ambiguity exemption without broadening it.
- Prove ordering and parity across every invalid audio fixture.
- Keep diagnostics exact and test-only.

**Non-Goals:**

- Add or modify canonical catalog fixtures.
- Activate motion graphics in runtime models, persistence, headless, MCP, capabilities, providers, evaluation, preview, export, or packaging.
- Add dependencies, public errors, ownership movement, or a new catalog version.
- Modify an archived change.

## Decisions

### Resolve definition buses during common preflight

After bus identity and ambiguity-exemption checks, both validators will build the declared project-bus key set and require every `soundDefinition.busId` to be present. Failure will use the exact test-only invariant `audio invalid envelope sound definition bus missing reference`. This check runs before resource and semantic classifiers, so no declared fixture failure can hide it.

### Preserve the intentional bus ambiguity

Membership, rather than exact cardinality, is required for each sound-definition bus. Ordinary fixtures already require unique bus definitions. `audio_event.ambiguous_bus` may retain multiple definitions for its one event-referenced key, and a sound definition may legally reference that present ambiguous key.

### Exercise every failure family

Mirrored table-driven mutations will replace a sound-definition bus in every canonical invalid audio fixture and assert the exact preflight invariant. A paired control will declare the replacement bus and prove that the original fixture again reaches its exact expected concept, classification, and reason.

## Risks / Trade-offs

- [A duplicated intentional bus resolves to more than one definition] → Treat presence as sufficient only after the existing exact-key ambiguity preflight has rejected every unrelated duplicated bus.
- [The new check could shadow the fixture's declared failure] → This is intentional when the bus is absent; restored-bus controls prove the original failure remains unchanged.
- [A catalog change could make this a public error] → Keep the invariant inside test-support validators and leave the canonical catalog unchanged.

## Migration Plan

No runtime or data migration applies. After approval, update both validators and mirrored regressions, synchronize documentation, run all required gates, verify requirement/design/task/code alignment, and archive only this corrective change.

## Open Questions

None.
