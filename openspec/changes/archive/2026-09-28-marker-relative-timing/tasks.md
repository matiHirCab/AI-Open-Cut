## 1. Canonical contracts and migration evidence

- [x] 1.1 Update canonical marker activation, persisted-project, headless, capability and MCP catalogs and fixtures; identify every consumer in `contracts/contract-ownership-v1.json` and update the MCP schema digest deliberately (motion-graphics-contracts and agent-bridge requirements).
- [x] 1.2 Add failing Rust migration tests for schema 23 current state, nonempty undo/redo, forbidden pre-24 fields, malformed/future snapshots, transaction fault recovery and legacy frame/audio equivalence (project-persistence requirement).
- [x] 1.3 Implement schema-24 root/component marker defaults and optional item-start expression migration in editor-core, then make 1.2 pass (project-persistence requirement).

## 2. Editor-core marker and timing behavior

- [x] 2.1 Add failing tests for scoped marker CRUD, ID/name rules, 4096 limit, duplicate-name ambiguity, same-scope lookup, signed bounds and dangling-reference rejection (marker-relative-timing requirements).
- [x] 2.2 Implement marker model, validation, scoped lookup and live item-start resolution in editor-core; make 2.1 pass (marker-relative-timing requirements).
- [x] 2.3 Add failing tests for numeric clearing, duration-only trim retention versus changed-start clearing, split clearing, checked duplicate offset adjustment and overflow in both duplication workflows, component replacement, component bounds, stale revisions, batch aliases, rollback, undo/redo and reopen (marker-relative-timing and timeline-editing requirements).
- [x] 2.4 Implement core standalone/batch operations and expression behavior through the existing transaction and alias machinery; make 2.3 pass (marker-relative-timing and timeline-editing requirements).
- [x] 2.5 Add deterministic preview, range, draft and export tests for a moved marker plus unchanged numeric-only output, and make them pass through shared evaluated timing (rendering-export requirement).

## 3. Typed agent surfaces

- [x] 3.1 Update the Rust headless request/response union and status capability, with transport tests for standalone and aliased batch marker edits and typed failures (agent-bridge requirement).
- [x] 3.2 Update TypeScript unions, Zod schemas, MCP registration and project responses, with integration tests for discovery, old numeric requests, missing/ambiguous lookup, conflicts and rollback (agent-bridge requirement).
- [x] 3.3 Update the user-facing marker timing and migration documentation; document exact scope, offsets, bounds, clear/retain rules and failure codes (agent-bridge requirement).

## 4. Verification and completion

- [x] 4.1 Run `bun run contracts:check` in `apps/agent-bridge`, `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `bun run typecheck`, `bun run lint`, `bun run test`, affected MCP integration and packaged smoke suites, and relevant hermetic Python worker tests; capture full logs and resolve all failures.
- [x] 4.2 Run strict OpenSpec validation and the pre-archive `moon run root:openspec-validate` gate; record the expected active-change-only rejection and resolve any other failure.
- [x] 4.3 Use `$openspec-verify-change` to check every requirement, scenario, design decision, task and test against implementation; resolve mismatches and record evidence limits.
- [x] 4.4 Synchronize the accepted delta specs with `$openspec-sync-specs`, archive the verified change with `$openspec-archive-change`, then rerun strict all-spec validation and `moon run root:openspec-validate` until both pass.
