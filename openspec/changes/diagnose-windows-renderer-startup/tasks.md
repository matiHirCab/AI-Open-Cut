## 1. Scoped approval

- [x] 1.1 Preserve issue60 checkout and create an isolated main-based fix branch; read original/retried Windows logs and applicable requirements.
- [x] 1.2 Review and approve the diagnostic-only proposal/design/specification under the newly delegated baseline-investigation authority; retain original failed evidence and prohibit speculative corrections.

## 2. Diagnostic implementation

- [x] 2.1 Add bounded portable fixture-evidence formatting and meaningful missing/unreadable/truncation/typed-event tests.
- [x] 2.2 Add owned shell/PowerShell startup evidence and correlated worker-event capture to the Windows fixture without changing its10-second startup or5-second cleanup deadlines/handle assertions.
- [x] 2.3 Add a focused native Windows workflow and automated workflow-policy test preserving all required workflows exactly.
- [x] 2.4 Add approved paired original/instrumented native startup probes, bounded owned-descendant snapshots and malformed-byte evidence coverage; collect native comparison evidence without changing standard acceptance or inferring a correction.

## 3. Verify diagnostics and collect Windows evidence

- [x] 3.1 Run `cargo fmt --check --all`, focused headless tests and Windows-target cargo check; run `bun test scripts/windows-renderer-startup-diagnostic.test.ts` and strict pinned OpenSpec validation. Distinguish cross-check from actual Windows execution.
- [x] 3.2 Review the complete diagnostic diff for assertion preservation, bounded owned evidence, unchanged production/contracts and unchanged required gates; push a draft investigation branch targeting main only as needed for real Windows evidence, with all unresolved verification explicitly pending.
- [ ] 3.3 Collect focused native Windows startup/process output, identify the proven cause and amend/re-review specifications before any corrective implementation. A failed diagnostic job is evidence, never acceptance.

## 4. Complete correction and repository verification

- [ ] 4.1 Implement and cover only the cause-specific approved correction; keep all cleanup assertions and deadlines unchanged unless a separately evidenced requirement explicitly approves a different lifecycle mechanism.
- [ ] 4.2 Run required `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`; bridge `bun run typecheck`, `bun run lint`, `bun run test`, `bun run test:integration`, `bun run test:smoke`; relevant hermetic Python checks and full policy/spec validation. Preserve full failures; no coverage suppression or standard acceptance substitution.
- [ ] 4.3 Independently review conformance with openspec-verify-change, synchronize/archive only after required implementation checks pass, then run final `moon run root:openspec-validate` and strict all-spec validation.
- [ ] 4.4 Rewrite the draft PR around the final verified fix targeting main, verify every required check on the exact final head, report material evidence and blockers, and only then resume preserved issue60 work. Do not merge or deploy.
