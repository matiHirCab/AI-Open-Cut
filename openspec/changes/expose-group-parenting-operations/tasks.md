## 1. Approval and contracts

- [x] 1.1 Obtain explicit approval of proposal, design and all delta scenarios; record evidence before implementation and use `$openspec-apply-change`.
- [x] 1.2 For Governed additive ungroup contract, extend canonical group-parent, headless, MCP and ownership fixtures before updating consumers; include structural, semantic, alias and legacy compatibility examples without changing schema 10 or protocol major.

## 2. Editor core

- [x] 2.1 Add automated tests and implement Atomic local-preserving ungroup for root/nested/empty groups, every supported immediate-child kind, cross-track and hidden/inactive children, exact local-property preservation and ordinal normalization.
- [x] 2.2 Add tests and implement Ungroup failures preserve the complete transaction: missing/non-group target, every affected lock, permitted read-only locks, stale revision precedence and existing finite/node/depth/batch boundaries.
- [x] 2.3 Add tests and implement Ungroup batch aliases and reversible results: earlier/forward/missing/deleted aliases, prohibited resultAlias, deterministic deduplicated changed IDs, later-operation rollback, standalone/batch undo/redo/reopen and media/provenance preservation.
- [x] 2.4 For Preserve evaluated semantics, compare against an explicit reparent/delete oracle and exercise frame/range/export consistency and existing group/legacy render regressions without changing renderer semantics.
- [x] 2.5 For Preserve compatibility and persistence, run existing schema-10 and older mixed-history migration/future-version/recovery regressions; demonstrate no new persisted schema is needed.

## 3. Typed transports

- [x] 3.1 Update Rust headless operation acceptance/capability reporting and protocol tests for Governed additive ungroup contract and both Complete typed group workflows scenarios.
- [x] 3.2 Update TypeScript headless requests, Zod operation/batch schemas, MCP registration and capability consumers; consume canonical fixtures in parity tests, including strict unknown fields and resultAlias rejection.
- [x] 3.3 Add real MCP integration and packaged smoke coverage for create/reparent/z-index/ungroup standalone and aliases in batches, canonical failure propagation, rollback, history and reopen.
- [x] 3.4 Update docs/group-parenting.md and applicable public contract documentation with operations, capability, local-preserving promotion, timing, flat ordering, errors, bounds and schema compatibility.

## 4. Verification and review

- [x] 4.1 Create verification.md mapping every delta requirement/scenario to named automated tests and outcomes; document any technically impossible automation and resolve mismatches.
- [x] 4.2 From repository root run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`.
- [x] 4.3 From apps/agent-bridge run `bun run typecheck`, `bun run lint`, `bun run test`, `bun run contracts:check`, `bun run test:integration`, and `bun run test:smoke`.
- [ ] 4.4 Run `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive` and `moon run root:openspec-validate`; report failed/skipped checks explicitly. Python worker tests are unaffected because no Python/provider behavior or contract changes are proposed; reassess if that scope changes.
- [ ] 4.5 Use `$openspec-verify-change`, resolve every mismatch, and obtain designated CODEOWNER review for canonical public contracts, consumers and parity evidence.
- [ ] 4.6 Use `$openspec-archive-change` after verified implementation to synchronize living requirements; rerun `moon run root:openspec-validate` with the required archive-only inventory before reporting completion.

## 5. Approved review correction

- [x] 5.1 Preserve alias field presence during core batch decoding and reject it for ungroup, including null; retain other operation behavior and serialization.
- [x] 5.2 Add canonical null/wrong-type fixtures, Rust/TypeScript standalone and batch parity, raw JSON duplicate/compatibility tests, headless no-publication regression, and shared MCP/packaged coverage.
- [ ] 5.3 Repeat required verification, update the report, and complete remaining owner-review/archive gates.

## Current verification state

The approved null-alias correction and all automated verification are complete. Full native Rust workspace tests, formatting, strict Clippy, bridge typecheck/lint/unit/contract/integration/packaged suites and strict OpenSpec content validation pass. OpenSpec correctness/coherence verification found no remaining implementation mismatch. Task 4.4 awaits archive-only inventory; tasks 4.5, 4.6 and 5.3 await designated owner review and consequent archive/final Moon validation.
