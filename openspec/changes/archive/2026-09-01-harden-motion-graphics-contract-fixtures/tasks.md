## 1. Strict fixture payload contracts

- [x] 1.1 Reshape valid and invalid catalog payloads into complete closed examples and add structured scoped definition/reference records plus managed external resource definitions.
- [x] 1.2 Add test-only `deny_unknown_fields` Rust Serde structs/enums and semantic validators for all nine concept payloads, numeric/range/collection rules, and field-specific resource safety.
- [x] 1.3 Add mirrored strict TypeScript Zod schemas and semantic refinements without adding dependencies or production runtime declarations.

## 2. Scoped reference and failure evidence

- [x] 2.1 Derive definitions/references from each parsed valid payload, compare them exactly with metadata, and validate tuple closure, duplicates, scope rules, dependency cycles, and depth limits.
- [x] 2.2 Add complete hierarchy/slot invalid fixtures for missing/cross-scope references, direct/indirect cycles, depth overflow, wrong type, required/default failures, constraint violations, missing targets, and arbitrary property paths.
- [x] 2.3 Add complete audio invalid fixtures for missing and ambiguous markers, sound definitions, variants, and buses.
- [x] 2.4 Require both language tests to reject every invalid payload with its exact ID/classification/reason and add malformed in-memory regressions for payload shape, required fields, resources, and scope mismatch.

## 3. Specification and ownership coherence

- [x] 3.1 Replace the living `motion-graphics-contracts` purpose placeholder and update fixture documentation for strict payload validation and structured scoped references.
- [x] 3.2 Keep the ownership manifest and CODEOWNERS synchronized with every focused validator/helper consumer; leave all runtime/public/persisted catalogs unchanged.

## 4. Verification and closure

- [x] 4.1 Run the pinned OpenSpec 1.5.0 strict validator and `bun run contracts:check`.
- [x] 4.2 Run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`.
- [x] 4.3 Run `bun run typecheck`, `bun run lint`, and `bun run test` from `apps/agent-bridge`.
- [x] 4.4 Run `git diff --check`; integration and packaged smoke are unaffected because no runtime, bootstrap, transport, provider, renderer, or packaging path changed.
- [x] 4.5 Use `$openspec-verify-change`, resolve all findings, sync the modified living specification, and archive with `$openspec-archive-change`.
