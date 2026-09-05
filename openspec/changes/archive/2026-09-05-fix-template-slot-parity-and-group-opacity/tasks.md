## 1. Approval and canonical evidence

- [x] 1.1 Obtain and record explicit approval of the proposal, design, three delta specifications and tasks before implementation; recheck active changes and preserve the original archive.
- [x] 1.2 Add canonical special-key maps, malformed/unknown-key cases and group-opacity absent/present Transform2D examples to template-slots-v1; keep existing fixture assumptions intact (Govern special-key and group-opacity regressions).

## 2. Bridge map parsing

- [x] 2.1 Add failing TypeScript request/response regressions for __proto__, constructor and toString, own-key/prototype preservation, JSON/null-prototype inputs, parsed value copying, malformed values and key-prefixed error paths (Preserve special slot identifiers; Validate special-key values without dropping entries).
- [x] 2.2 Implement the shared public-Zod validated map parser with Object.fromEntries, typed output and metadata derived from the original record schema. Verify identical input/output JSON Schema and unchanged registered MCP structural catalog (Preserve override maps through real transports).

## 3. Core effective group opacity

- [x] 3.1 Add failing native validation and persistence tests for group default/override opacity 0, 0.5 and 1 with absent/present Transform2D; assert preserved other transform fields and unchanged stored base tracks (Apply group opacity without requiring explicit Transform2D).
- [x] 3.2 Initialize/update Transform2D opacity for groups alongside component instances in derived candidates; retain complete validation (Apply group opacity without requiring explicit Transform2D).
- [x] 3.3 Consume canonical special-key records in native round-trip tests; cover required/no-default, overridden defaults, missing/unknown values, undo/redo/reopen. Cover out-of-range group values, locks, stale revisions and byte-identical batch rollback (Preserve special slot identifiers; Validate special-key values without dropping entries; Preserve group opacity failure atomicity).

## 4. End-to-end evidence and documentation

- [x] 4.1 Extend the shared source/packaged component workflow with canonical special-key and group-opacity cases through standalone create/update/define and aliased batches, typed reads, errors, undo/redo and reopen (Preserve override maps through real transports; Govern special-key and group-opacity regressions).
- [x] 4.2 Update template-slot documentation to state corrected special-key preservation and group opacity behavior, retaining schema 12, protocol 1, ID grammar, errors and renderer scope.

## 5. Verification and finalization

- [x] 5.1 Run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` with established FFmpeg6/ffprobe and DejaVuSans native configuration. Record all outcomes and resolve failures.
- [x] 5.2 From apps/agent-bridge run `bun run contracts:check`, `bun run typecheck`, `bun run lint`, `bun run test`, `bun run test:integration`, `bun run test:smoke` and `bun run scripts/run-python-tests.ts`. Confirm no MCP structural catalog drift.
- [x] 5.3 Run `bunx @fission-ai/openspec@1.5.0 validate fix-template-slot-parity-and-group-opacity --strict --no-interactive` and `git diff --check`; use openspec-verify-change and record scenario-to-test evidence in verification.md. Resolve every mismatch; no failed required check may be marked complete.
- [x] 5.4 Obtain designated CODEOWNER review of completed canonical evidence and consumer changes; record explicit approval.
- [x] 5.5 Use openspec-sync-specs and openspec-archive-change to synchronize accepted requirements and archive this verified follow-up; run `moon run root:openspec-validate` afterward (pinned Moon wrapper if needed), record the result and mark complete only if it passes. Do not weaken the archive-only gate or modify the original archive.
