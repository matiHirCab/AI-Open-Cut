## 1. Approval and canonical contracts

- [x] 1.1 Obtain explicit user/reviewer approval of proposal, design, delta specs and tasks before implementation; record the approval evidence.
- [x] 1.2 Add component-lifecycle-v1 fixtures and update contract ownership, headless protocol and MCP catalogs for the governed additive lifecycle requirement before updating consumers. Cover every specified success, boundary, alias and rejection stage.

## 2. Editor core

- [x] 2.1 Add automated scenarios for atomic duplication: copied fields, omitted/replaced/empty overrides, all eight kinds, special keys, defaults, missing/invalid references, wrong source type, locks and inclusive numeric/text/scene bounds.
- [x] 2.2 Implement the typed single-instance duplication edit and batch conversion/alias resolution in the existing owning core modules; reuse candidate validation and store publication. Preserve generic duplication behavior.
- [x] 2.3 Add complete lifecycle transaction tests for earlier/result aliases, literal slot data, stale revision, duplicate/forward/unresolved aliases, trailing failure rollback, single-step history and exact reopen. Retain schema-13 and supported older current/history migration and future-version rejection evidence.
- [x] 2.4 Add shared-evaluation regression coverage comparing duplication with explicit placement for frame, range, draft preview and export, including nested content and independent overrides.

## 3. Headless transport

- [x] 3.1 Expose the standalone typed edit and additive status capability; extend protocol fixture consumption and real JSON-lines lifecycle success/failure/history tests without transport-owned domain rules.

## 4. Agent bridge

- [x] 4.1 Synchronize TypeScript unions, closed Zod request/batch schemas, MCP tool registration and capability validation with canonical fixtures, preserving special own slot keys and validation issue paths.
- [x] 4.2 Extend source integration and packaged smoke with standalone and aliased lifecycle success, override independence, invalid input/reference/lock/revision/trailing failure, undo/redo/reopen and representative preview/export.

## 5. Documentation and verification

- [x] 5.1 Update public API/contract documentation for existing creation/slot/instantiation operations plus duplication, exact override semantics, coordinates, timing, order, bounds, errors, aliases and schema-13 compatibility. Map every delta scenario to automated evidence in verification.md.
- [x] 5.2 Run from repository root: `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace`. Resolve all failures, including architecture and retained migration regressions.
- [x] 5.3 Run from apps/agent-bridge: `bun run typecheck`, `bun run lint`, `bun run test`, `bun run contracts:check`, `bun run test:integration` and `bun run test:smoke`. Record results; failed or skipped required checks block completion. Python workers/contracts are unchanged, so no worker-specific test change is planned; reassess if implementation affects them.
- [ ] 5.4 Run `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive`. Obtain designated @matiHirCab review of canonical contracts and consumers; do not substitute automated parity for owner approval.
- [ ] 5.5 Use `$openspec-verify-change add-atomic-component-lifecycle`, resolve every mismatch and record evidence. Once verification and review are complete, use `$openspec-archive-change add-atomic-component-lifecycle` to synchronize living specs.
- [ ] 5.6 Run `moon run openspec-validate` after archival (the protected policy rejects active changes); record final validation and clean scoped diff checks before reporting completion.
