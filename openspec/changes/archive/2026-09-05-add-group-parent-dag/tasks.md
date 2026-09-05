## 1. Approval and canonical fixtures

- [x] 1.1 Obtain explicit user/reviewer approval of proposal, all delta specs, and design before implementation; record approval evidence in proposal.md (user: “Approve”).
- [x] 1.2 Add group-parent-v1 canonical fixtures and ownership mapping for every scenario in Governed runtime group contract, including defaults, structural boundaries, graph failures, and reader compatibility; retain fixture-only roadmap catalog.

## 2. Core model and persistence

- [x] 2.1 Implement Non-drawing group timeline nodes and Scoped bounded parent graph in editor-core with automated placement, finite-value, identity, duplicate-ID, same/cross-track scope, missing/non-group parent, hidden/self/indirect cycle, depth 32/33, and count 4096/4097 coverage.
- [x] 2.2 Implement Schema-v10 group migration and Hierarchy migration fails closed with current/mixed-history fixtures, all supported old versions, omitted parent, invalid/future history, exact media/provenance preservation, and all persistence fault-injection tests.

## 3. Core mutations and evaluation

- [x] 3.1 Implement Transactional parenting lifecycle with standalone and evolving-batch alias resolution, null detachment, local-transform preservation, lock/revision/rollback, surviving-reference deletion checks, node duplication, child split/move/duplicate, undo/redo, and reopen tests; cover every timeline delta scenario.
- [x] 3.2 Implement Canonical group ancestor evaluation with independent matrix oracle, group composition-sized anchor, opacity multiplication, ancestor interval/visibility, unchanged audio and flat order, and immutable repeated evaluation tests.
- [x] 3.3 Implement Bounded derived group geometry through existing preparation, with finite/overflow boundaries and side-effect instrumentation; retain path-safe font/media measurement and missing-asset precedence.

## 4. Typed consumers and rendering

- [x] 4.1 Update governed headless/MCP catalogs, Rust requests, TypeScript/Zod schemas, typed group responses, capability reporting, standalone/batch adapters, and parity evidence for Governed runtime group contract without transport graph validation.
- [x] 4.2 Add headless and MCP integration for group creation/parenting aliases, unparenting, invalid/missing inputs, stale revisions, atomic rollback, history, reopen, and unchanged simple clients.
- [x] 4.3 Adapt necessary desktop exhaustive matches to the non-drawing group variant without adding group-authoring UI or domain validation.
- [x] 4.4 Prove Shared parented visual rendering for frame/range/draft/export with nested asymmetric geometry, transparency, caption/media, visibility/timing clipping, independent occlusion checks, legacy goldens, backend readiness failures, and documented SSIM/PCM/timing tolerances.
- [x] 4.5 Document group operations, scope, timing, anchor, deletion, reader compatibility, errors, and limits; update ADR 0004 activation notes. Obtain designated CODEOWNER review for governed contracts and consumers.

## 5. Validation and archival

- [x] 5.1 Maintain a verification report mapping every delta requirement and scenario to named automated tests and outcomes; resolve all mismatches before completion.
- [x] 5.2 Run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` from repository root.
- [x] 5.3 From apps/agent-bridge run `bun run typecheck`, `bun run lint`, `bun run test:unit`, `bun run contracts:check`, `bun run test:integration`, and `bun run test:smoke`.
- [x] 5.4 Run `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive` and `moon run root:openspec-validate`; record failed/skipped checks. Python workers have no changed behavior or contracts in this proposal, so worker-specific tests are not affected; reassess if scope changes.
- [x] 5.5 Use `$openspec-verify-change`, resolve every requirement/design/task/code/test mismatch, then use `$openspec-archive-change` to synchronize living requirements and archive the verified change.
- [x] 5.6 Rerun `moon run root:openspec-validate` with the archive-only change inventory required by repository policy; report final verification and any remaining blocker without claiming incomplete work is done.

## 6. Approved review corrections

- [x] 6.1 Preserve all nine legacy text anchors in static and animated ancestor evaluation and sampling; add independent and native regressions.
- [x] 6.2 Separate object geometry limits from motion sampling; precompute clipped 4096-by-4096 tiles and test travel, re-entry, seams, empty intersections and overflow before side effects.
- [x] 6.3 Reject group transition endpoints in shared validation; cover current/undo/redo, hidden records, drafts, mutations and renderer input without publication.
- [x] 6.4 Rerun the required Rust/native, bridge, contract, integration, smoke and OpenSpec checks; update verification and complete review/archive gates.
