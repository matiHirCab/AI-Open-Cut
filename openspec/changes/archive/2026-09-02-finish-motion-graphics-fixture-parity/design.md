## Context

Independent review mutations found Rust accepting a component payload with a repeated layer definition because `BTreeSet` erased the duplicate, while TypeScript rejected it. A second mutation made the name of `slot.required_value_missing` empty: Zod rejected the invalid envelope, while Rust ignored the field and still returned the declared failure. The aggregate regression helpers also discard caught errors, so unrelated failures can satisfy an overflow test.

The catalog remains fixture-only. The correction therefore belongs entirely to test support and governance evidence and requires no compatibility migration.

## Goals / Non-Goals

**Goals:**

- Reject every duplicate payload-derived definition before normalization.
- Validate every unrelated field in an invalid scenario before classifying its intentional defect.
- Preserve exact deterministic negative classifications and reasons.
- Make mutation tests prove the intended invariant rather than any failure.

**Non-Goals:**

- Activate motion graphics in runtime models, persistence, headless, MCP, capabilities, providers, evaluation, preview, export, or packaging.
- Introduce public error codes, dependencies, schema generation, ownership movement, or catalog version changes.
- Modify an archived change.

## Decisions

### Reject duplicates at their payload collection

Rust derivation will treat a failed `BTreeSet::insert` as a validation error for layer IDs, component-local layer IDs, mask IDs, and effect IDs. Detection occurs before metadata equality, aggregate counting, and reference closure. TypeScript retains its array-derived definitions and duplicate insertion failure. Both suites will assert the duplicate-definition invariant for each definition-bearing collection.

### Preflight negative scenarios before classification

Each invalid fixture family will have a strict common-field preflight in both languages. The fixture ID selects the one field or relationship allowed to violate its normal invariant; every other identifier, scope, name, time, finite number, range, required collection, and constraint is validated first. The preflight then hands a fully checked scenario to the existing deterministic classifier.

This applies to component dependency graphs, ordinary and cross-scope layer graphs, slots, marker candidates, masks including intentionally unsupported sources, effects including renderer expressions, audio resolution scenarios, and the transform/curve special cases. Relaxed types needed to represent an intentional defect remain closed and are not reused as proof that unrelated fields are valid.

Names use Unicode scalar counts: Rust `chars().count()` and TypeScript `Array.from(value).length`. Native `.max()` string length is not used where astral characters could diverge.

### Assert causal mutation failures

Failure helpers will return or inspect the caught diagnostic and require a stable test-support invariant key or named limit substring. Aggregate tests assert their exact catalog limit. Duplicate-definition, fixture-ID, managed-resource, reference-closure, and malformed-envelope mutations assert their own invariant. These diagnostics remain private test evidence and do not become runtime error codes.

## Risks / Trade-offs

- [Intentional defects can be rejected during common preflight] → Key exemptions by the independently hardcoded fixture ID and add one passing canonical test for every negative fixture.
- [Handwritten scenario checks can drift] → Use table-driven mirrored mutations and shared native helper functions for IDs, scopes, safe integers, finite ranges, and scalar lengths.
- [Exact tests can become coupled to prose diagnostics] → Assert stable test-support keys or named invariants, not full Serde or Zod messages.

## Migration Plan

No runtime or data migration applies. After approval, update the two test helpers and regressions together, clarify living documentation, run all gates, sync the modified specification, and archive only this corrective change. Rollback removes these corrective edits without touching prior archives.

## Open Questions

None.
