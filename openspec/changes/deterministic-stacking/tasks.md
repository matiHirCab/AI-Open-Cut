## 1. Approval and canonical contracts

- [x] 1.1 Obtain explicit approval of proposal, design, delta specs, and tasks before implementation; record approval evidence in proposal.md (user: "Approve", 2026-09-04).
- [x] 1.2 Add stacking-v1 runtime catalog and ownership mapping; update canonical headless/MCP catalogs and compatibility fixtures for all motion-graphics-contracts scenarios before consumer implementation.
- [x] 1.3 Create scenario-to-test evidence mapping covering every scenario in the five delta specs; update it as tests are implemented.

## 2. Core model and migration

- [x] 2.1 Add schema-v9 ordering fields and strict current-schema validation with numeric and malformed-order tests for Persisted canonical stacking values.
- [x] 2.2 Implement schema-v9 migration for current state and all retained history; cover oldest/mixed history, legacy output, reopen, undo/redo, invalid/future snapshots, and every publication fault-injection phase from project-persistence scenarios.

## 3. Core mutation and evaluation

- [x] 3.1 Implement core z-index/item-reorder/track-reorder operations and alias resolution; test signed bounds, first/last/same-position indices, missing/locked/unsupported targets, stale revisions, changed IDs, batch rollback, undo/redo, and reopen against timeline-editing scenarios.
- [x] 3.2 Maintain ordinals and source z-index across all existing creation, move, split, duplicate, delete, generated-caption, and speech-placement paths; test insertion semantics and creation alias correctness when multiple changed IDs are returned.
- [x] 3.3 Apply canonical visual sorting in EvaluatedScene; test exact comparator order, synthesized ID ties, track precedence, hidden filtering, audio/transition preservation, immutable repeated evaluation, and complexity rejection from motion-graphics-architecture scenarios.

## 4. Typed transports and contract parity

- [x] 4.1 Update headless typed operations and batch unions, dispatch, responses, and protocol tests for all three operations and legacy compatibility.
- [x] 4.2 Update bridge Zod schemas, registrations, capability discovery, and typed adapters without duplicating core semantics; cover standalone/batch creation aliases and errors in MCP integration and packaged smoke.
- [ ] 4.3 Implement canonical Rust/TypeScript fixture parity for runtime stacking including strict malformed/unsafe input; obtain designated CODEOWNER @matiHirCab review of canonical and consumer changes.

## 5. Rendering and documentation

- [x] 5.1 Add overlapping opaque/transparent render fixtures covering equal/unequal z-index, track/item reorder, captions, transitions, hidden items, legacy and Transform2D visuals; verify independent occlusion oracle and identical ordering across frame/range/draft/export.
- [x] 5.2 Verify migrated legacy render fixtures and complete-backend failure behavior; enforce SSIM >= 0.99, audio RMS <= 0.0001, and one-frame timing tolerance for rendering-export scenarios.
- [x] 5.3 Update public operation, ordering, migration, capability, and contract-fixture documentation, clarifying that reorder preserves z-index and remains within a track.

## 6. Verification and archival

- [x] 6.1 From repository root run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`; record actual outcomes in verification evidence.
- [x] 6.2 From apps/agent-bridge run `bun run typecheck`, `bun run lint`, `bun run test`, `bun run contracts:check`, `bun run test:integration`, and `bun run test:smoke`; record actual outcomes. Python provider behavior/contracts are unchanged, so worker-specific tests are not affected; reassess if implementation changes that scope.
- [x] 6.3 Run `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive`; use `$openspec-verify-change` and resolve every requirement/design/task/test mismatch before reporting completion.
- [ ] 6.4 Use `$openspec-archive-change` to synchronize accepted deltas and archive the verified change, then run `moon run root:openspec-validate` with archive-only changes inventory. Report any failed or skipped required check as blocking completion.

## 7. Approved review correction

- [x] 7.1 Add failing all-facade malformed stacking regressions, including hidden/nonvisual items, exact errors, immutability and no publication; cover valid ordinals and empty tracks.
- [x] 7.2 Share read-only stacking validation after scene complexity preflight; update ADR 0003 and architecture tests for the approved validation edge; correct valid hand-built fixtures only.
- [x] 7.3 Rerun required Rust, native, bridge and OpenSpec checks and revise verification/evidence to resolve the confirmed rendering mismatch.
