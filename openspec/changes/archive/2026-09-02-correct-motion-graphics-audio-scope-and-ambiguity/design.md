## Context

Adversarial mutations proved two remaining gaps. Invalid audio envelopes reject otherwise legal event and marker IDs distributed across different composition scopes and apply `maxAudioEventsPerComposition` and `maxMarkersPerComposition` to total envelope length. Separately, ambiguity fixtures skip uniqueness for an entire key collection, so a second unrelated duplicate is accepted as long as the canonical ambiguity is classified first.

The catalog remains fixture-only. These corrections affect test-support validation and governance evidence only, with no public or persisted compatibility impact.

## Goals / Non-Goals

**Goals:**

- Use structured composition ownership for invalid audio-event and marker definition identity and counting.
- Restrict each ambiguity fixture to one duplicated semantic lookup key while keeping definition IDs unique.
- Preserve every canonical exact classification and reason.
- Prove equivalent behavior with mirrored Rust and TypeScript mutations that assert exact invariants.

**Non-Goals:**

- Activate motion graphics in runtime models, persistence, headless, MCP, capabilities, providers, evaluation, preview, export, or packaging.
- Add dependencies, generated schemas, public errors, ownership movement, or a new catalog version.
- Modify an archived change.

## Decisions

### Use scoped identities for composition definitions

Invalid audio-event and marker definitions will use `{ scope, id }` as their uniqueness key. Reusing an ID in a different legal composition scope is valid; repeating it within the same scope is rejected before classification. Project-scoped assets, buses, and sound definitions remain globally keyed by their project identity.

### Enforce limits per exact owner

Both validators will group invalid audio events by `event.scope` and markers by `marker.scope`, then apply the respective inclusive catalog limit to every group. Total envelope size may exceed a per-composition limit when no owner exceeds it. Errors retain the exact camel-case catalog limit name.

### Validate one exact ambiguity key

Ambiguity preflight will collect duplicated semantic keys before classification. The standalone marker fixture must have exactly one duplicated `{ scenario.scope, lookupName }`. Audio bus and sound-definition fixtures must have exactly one duplicated project key referenced by an event; marker ambiguity must have exactly one duplicated `{ event.scope, markerName }` referenced by marker-relative timing; and variant ambiguity must have exactly one duplicated asset key in one sound definition referenced by an event. Multiplicity above one is allowed for that single key, but any second or unrelated duplicated key fails the duplicate-definition invariant. The expected ambiguity must remain present.

## Risks / Trade-offs

- [Several events could reference the same declared ambiguous key] → Treat that as one semantic ambiguity key, independent of how many references observe it.
- [A fixture could contain both its expected ambiguity and another malformed reference] → Complete common preflight, including exact-key uniqueness, before semantic classification.
- [Cross-scope IDs look duplicated when inspected as strings] → Build explicit scope-and-ID keys in both languages rather than normalizing bare IDs.

## Migration Plan

No runtime or data migration applies. After approval, update both test validators and mirrored regressions, update documentation, run all required gates, verify requirement/design/task/code alignment, sync the delta into the living specification, and archive only this corrective change.

## Open Questions

None.
