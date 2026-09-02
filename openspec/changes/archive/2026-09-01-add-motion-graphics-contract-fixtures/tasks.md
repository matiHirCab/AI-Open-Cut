## 1. Canonical fixture catalog

- [x] 1.1 Add `contracts/motion-graphics-v1.json` with version/status metadata, ADR 0004 coordinate/timing/ordering/compositing semantics, explicit named limit keys, and closed identifier catalogs.
- [x] 1.2 Add deterministic valid fixtures for transforms, layers, components, slots, markers and absolute/marker-relative time expressions, hold/linear/cubic-Bézier/spring curves, ordered masks, ordered effects, and semantic audio events.
- [x] 1.3 Add deterministic invalid fixtures for type/range/non-finite input, explicit limit violations, missing references, direct/indirect cycles, ambiguous marker/binding references, unsafe SVG/resources, paths/URLs, and renderer-expression injection.

## 2. Ownership and documentation

- [x] 2.1 Register `motionGraphicsVocabulary` in `contracts/contract-ownership-v1.json`, naming the catalog as canonical and the focused Rust and TypeScript fixture tests as its current governed consumers.
- [x] 2.2 Document the fixture-only activation boundary, compatibility/versioning rule, field/tag casing, concept-to-future-milestone mapping, and the obligation for later changes to update native declarations, public catalogs, capabilities, errors, migrations, and parity evidence when applicable.

## 3. Cross-language fixture evidence

- [x] 3.1 Add an editor-core Rust test that consumes the canonical catalog and verifies version/status, nine-category coverage, identifier/fixture uniqueness, finite numbers, explicit limits, valid-reference closure, safety invariants, and negative failure classifications; prove the checker rejects one deterministic malformed in-memory case.
- [x] 3.2 Add an agent-bridge TypeScript test over the same catalog with equivalent structural assertions and a deterministic malformed in-memory failure case, without adding runtime Zod/MCP support for unimplemented concepts.
- [x] 3.3 Keep `headless-protocol-v1.json`, `mcp-surface-v1.json`, `error-codes-v1.json`, capability reporting, project schema version 6, and renderer output unchanged; assert/document that no public or persisted activation occurred.

## 4. Verification and closure

- [x] 4.1 Run the pinned OpenSpec 1.5.0 strict validator directly because `moon` is unavailable, and run `bun run contracts:check` from `apps/agent-bridge`.
- [x] 4.2 Run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` from the repository root.
- [x] 4.3 Run `bun run typecheck`, `bun run lint`, and `bun run test` from `apps/agent-bridge`.
- [x] 4.4 Integration and packaged-smoke suites are unaffected and were not run because this fixture-only change modifies no runtime, transport, bootstrap, packaging, provider, or rendering path.
- [x] 4.5 Use `$openspec-verify-change`, resolve every requirement/design/task/test mismatch, obtain the designated contract-owner review, then archive with `$openspec-archive-change` so the living specification remains current.
