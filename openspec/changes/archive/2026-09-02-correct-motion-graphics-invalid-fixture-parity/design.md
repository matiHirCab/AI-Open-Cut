## Context

Review mutations proved that a negative fixture's `concept` is currently ignored, TypeScript accepts repeated IDs in three invalid definition collections that Rust rejects, and Rust's mask classifier considers only `masks[0]` and only the exact `onload=` spelling. The Rust catalog wrapper also uses unrestricted `u64` limits while Zod uses `Number.MAX_SAFE_INTEGER`.

The catalog remains fixture-only, so the correction belongs to test support and governance evidence and requires no migration or public compatibility change.

## Goals / Non-Goals

**Goals:**

- Bind each invalid fixture ID to its exact concept, classification, and reason.
- Reject duplicate definitions before invalid classification in both languages.
- Apply identical mask safety classification to every declared mask.
- Give all catalog limits the same positive JavaScript-safe-integer range.
- Prove each correction with mirrored mutations that assert the intended invariant.

**Non-Goals:**

- Activate motion graphics in runtime models, persistence, headless, MCP, capabilities, providers, evaluation, preview, export, or packaging.
- Add dependencies, generated schemas, public errors, ownership movement, or a new catalog version.
- Modify an archived change.

## Decisions

### Treat concept as expected negative evidence

The Rust and TypeScript hardcoded negative matrices will store an exact concept beside classification and reason. Validation will compare the declared concept with that expectation before dispatching the payload to its fixture-specific classifier. The observed result comparison will continue to prove the exact classification and reason.

### Preflight invalid definition uniqueness

TypeScript will perform the same pre-normalization ID uniqueness checks Rust already performs for invalid layer, mask, and renderer-expression effect arrays. Duplicate IDs fail as an invalid envelope before the intended semantic defect can be reported.

### Mirror mask safety scanning without dependencies

Rust will inspect every mask in declaration order and use a small ASCII predicate equivalent to TypeScript's case-insensitive `\son[a-z]+\s*=` detection. Per mask, executable inline SVG is checked before arbitrary file input and unresolved layer input, matching TypeScript's existing classification order. No regex or parsing dependency is added.

### Bound limits at the wrapper

Every Rust limit must be within `1..=9_007_199_254_740_991`. Both the strict Serde-backed validator and the outer catalog evidence validator will enforce this bound before limit values participate in payload or graph validation. Wrapper-level tests will exercise the maximum and first rejected integer without changing semantic invalid fixtures that depend on small canonical limits.

## Risks / Trade-offs

- [Exact concept checks duplicate information encoded in fixture IDs] → Keep the independent matrix authoritative so relabeling cannot silently change scenario ownership.
- [Multiple defects could make mask classification order observable] → Preserve TypeScript's declaration-order and per-mask precedence; canonical negatives continue to contain one intentional defect.
- [Wrapper tests at the maximum cannot run the full semantic catalog] → Test the closed wrapper/limit validator directly, then retain full-catalog tests at canonical limits.

## Migration Plan

No runtime or data migration applies. After approval, update both test validators and regressions, update documentation, run all gates, sync the delta into the living specification, and archive only this corrective change.

## Open Questions

None.
