## 1. Canonical fixture scenarios

- [x] 1.1 Reshape component, slot, marker, and audio valid/invalid scenario envelopes so every negative fixture contains all unrelated required data and exactly one intended structural or semantic defect.
- [x] 1.2 Remove unsupported inline-SVG complexity limits, retain fixture-only version 1, and keep the exact identifier, semantic, managed-resource, definition, and reference metadata synchronized.
- [x] 1.3 Add independently owned expected failure tuples and complete boundary cases for all represented limits, safe integers, Unicode scalar lengths, legal scopes, duplicates, cycles, and reference resolution.

## 2. Mirrored deterministic validators

- [x] 2.1 Replace the Rust boolean invalid check with closed catalog/wrapper Serde types, deterministic fixture failures, complete semantic graph/resolution validation, exact metadata uniqueness, and every represented catalog limit.
- [x] 2.2 Replace the TypeScript boolean invalid check with mirrored strict Zod catalog/payload types, deterministic fixture failures, complete semantic graph/resolution validation, exact metadata uniqueness, and every represented catalog limit.
- [x] 2.3 Align both validators on JavaScript-safe integer bounds, Unicode scalar length, identifiers, dimensions, numeric ranges, collections, enum catalogs, required fields, kind/scope legality, and field-specific resource safety.

## 3. Regression evidence and governance

- [x] 3.1 Add Rust and TypeScript regressions for swapped/mislabeled invalid payloads, inclusive/overflow limits, direct/indirect cycles, missing/ambiguous/cross-scope references, safe-integer overflow, Unicode boundaries, and duplicate managed/payload/metadata definitions.
- [x] 3.2 Preserve URL-like ordinary text while rejecting POSIX, Windows, UNC, traversal, URI, executable SVG, event-handler, and renderer-expression inputs only through resource-bearing or unsupported closed fields.
- [x] 3.3 Update fixture documentation, the ownership manifest, and CODEOWNERS only if corrected fixture/helper consumers change; keep runtime/public/persisted catalogs unchanged.

## 4. Verification and closure

- [x] 4.1 Run `moon run openspec-validate` and `bun run contracts:check` from `apps/agent-bridge`.
- [x] 4.2 Run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`.
- [x] 4.3 Run `bun run typecheck`, `bun run lint`, `bun run test`, `bun run test:integration`, and `bun run test:smoke` from `apps/agent-bridge`.
- [x] 4.4 Run `git diff --check`, verify no runtime/bootstrap/provider/renderer/persistence surface changed, use `$openspec-verify-change`, resolve every finding, sync the living spec, and archive with `$openspec-archive-change`.
