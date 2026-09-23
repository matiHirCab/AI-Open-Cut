## 1. Contract and persistence foundation

- [ ] 1.1 Update the canonical channel/value, schema-version, operation, MCP, and capability fixtures and contract ownership/parity expectations for the active and inactive catalog entries; classify the additions under ADR 0002 and obtain the designated CODEOWNER review. (motion-graphics-contracts)
- [x] 1.2 Add schema-22 channel storage and deterministic locked migration for current, undo, and redo documents, including malformed-history, future-schema, interrupted-commit, and reopen tests. (project-persistence)

## 2. Editor-core behavior

- [x] 2.1 Add closed channel/value types and canonical bounds, target compatibility, identity, timing, finite-value, inactive-channel, and legacy-collision validation with boundary tests. (animation-channels)
- [x] 2.2 Add the replace-channels edit to core standalone/batch transaction handling and test success, missing/locked target, stale revision, aliases, rollback, undo/redo, and unchanged legacy operations. (timeline-editing)
- [x] 2.3 Sample active position X/Y, independent scale X/Y, opacity, and audio gain channels in the shared evaluated scene; test exact/held/interpolated values and fail-closed unsupported persisted input. (animation-channels, motion-graphics-architecture)
- [x] 2.4 Prove still, range, draft, and export parity and legacy render equivalence with deterministic fixtures. (rendering-export)

## 3. Transport and documentation

- [x] 3.1 Extend the typed headless request/response union and headless tests for standalone and batch channel edits and error translation. (agent-bridge)
- [x] 3.2 Extend bridge types, Zod schemas, MCP registration, capability reporting, and MCP integration tests, including alias and failure parity. (agent-bridge, motion-graphics-contracts)
- [x] 3.3 Document channel names, active status, value tags, bounds, coordinate/timing/interpolation rules, compatibility, and migration for clients. (animation-channels, motion-graphics-contracts)

## 4. Verification and archival

- [ ] 4.1 Run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`; record full logs and exit codes.
- [x] 4.2 From `apps/agent-bridge`, run `bun run typecheck`, `bun run lint`, `bun run test`, `bun run contracts:check`, `bun run test:integration`, and `bun run test:smoke`; record full logs and exit codes. Run hermetic Python worker tests if any worker surface changes; none are planned.
- [ ] 4.3 Run pinned strict all-spec validation, then the pre-archive `moon run root:openspec-validate` gate; record the expected rejection only if it names this active change and no other failure.
- [ ] 4.4 Run `$openspec-verify-change`, resolve all requirement/design/task/test mismatches, then `$openspec-sync-specs` and `$openspec-archive-change`.
- [ ] 4.5 Rerun `moon run root:openspec-validate` and pinned strict all-spec validation after archival; require both to pass before claiming completion.
