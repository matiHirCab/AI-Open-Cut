## 1. Approval and baseline

Completed boxes below record the original implementation only. Corrective work is tracked in section 5; prior passing results must not be reused where the revision or shared-tree changes invalidate them.

- [x] 1.1 Obtain explicit user/reviewer approval of proposal, requirements, design, and tasks; record approval before implementation (Preserved safeguards).
- [x] 1.2 Record working-tree baseline and hashes of global config, generated skills, and protected workflow files without exposing secrets; preserve all unrelated changes (Project-scoped defaults; Preserved safeguards).

## 2. Regression coverage and implementation

- [x] 2.1 Add scripts/agent-context-efficiency.test.ts with parsed TOML checks for the three model/effort values, exact nine plugin IDs, local isolation fixtures, and negative cases for wrong values/IDs (Project-scoped defaults; Reversible local plugin availability).
- [x] 2.2 Add focused documentation checks covering approval, scenario tests, verify/archive, required checks, ownership/compatibility, search/history behavior, refresh conditions, failure logging, and invalidated evidence; demonstrate rejection of removed safeguards (Focused context handling; Preserved safeguards).
- [x] 2.3 Add the project config exactly as designed, preserving all unrelated inherited settings (Project-scoped defaults; Reversible local plugin availability).
- [x] 2.4 Consolidate duplicate explanation and add concise guidance to AGENTS.md and docs/spec-driven-development.md, including local plugin re-enablement and manual verification (Focused context handling; all safeguard scenarios).

## 3. Automated validation

- [x] 3.1 Run `bun test scripts/agent-context-efficiency.test.ts`; run the existing Biome binary from apps/agent-bridge against the new test with `bunx --no-install biome check ../../scripts/agent-context-efficiency.test.ts`; report unavailable tools rather than install unpinned replacements.
- [x] 3.2 Run `bunx @fission-ai/openspec@1.5.0 validate reduce-agent-context-overhead --strict --no-interactive` and `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive`.
- [ ] 3.3 Run `moon run root:openspec-validate` before archival and inspect the complete result. Only rejection identifying this active change alone is expected pre-archive evidence, never a passed gate; any other failure blocks progress. Do not edit the protected task. Require the successful post-archive run in 4.5.
- [ ] 3.4 Run root `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`; capture output without modifying unrelated Rust files.
- [x] 3.5 From apps/agent-bridge run `bun run typecheck`, `bun run lint`, `bun run test:unit`, `bun run test:integration`, and `bun run test:smoke`; record each exit code and any external prerequisites.
- [x] 3.6 From apps/kokoro-tts run `bun run ../agent-bridge/scripts/run-python-tests.ts` for the hermetic worker checks. No migration or public-contract edits are planned; if scope changes, revise artifacts and obtain approval before implementation.

## 4. Runtime evidence and completion

- [x] 4.1 Record observed OpenCut model/planning and skill/tool availability, another project's behavior where observable, and the scope of local re-enablement. Distinguish fresh desktop evidence, persistent-task catalogs, CLI rendering, and static tests. Record unsupported project-local remote suppression as accepted limitations and unavailable evidence explicitly; do not substitute static tests for desktop/runtime success.
- [x] 4.2 Compare identical small read-only audits before/after only where comparable fresh-task telemetry exists; otherwise explicitly record unavailable measurement and make no quantitative savings claim (Usage evidence scenario).
- [x] 4.3 Verify global config/generated skills/protected checks and unrelated files remain unchanged relative to baseline; review documentation semantically for preserved safeguards. Comparison completed: global/protected files unchanged; concurrent unrelated drift recorded in verification.md, with no writes to those files by this task.
- [x] 4.4 Use $openspec-verify-change and record a requirement/scenario-to-test/manual-evidence matrix. Resolve mismatches and report every failed, skipped, or pending required check. Report completed; external verification blockers remain in the unchecked tasks.
- [ ] 4.5 After all required implementation checks and OpenSpec conformance verification pass, and 3.3 has only the expected rejection, synchronize and use $openspec-archive-change for this change only. Then run `moon run root:openspec-validate` and strict all-spec validation; require final gate success before completion. If Moon is absent from PATH, use the repository-pinned `bunx --package @moonrepo/cli@2.3.3 moon run root:openspec-validate`. A failed final gate remains blocking; never bypass archive-only enforcement.

## Historical verification blockers

- 3.3: pinned Moon executed successfully but its task failed archive-only policy for both active changes. No policy bypass or external change archival was attempted.
- 3.4: formatting and strict Clippy initially passed. Workspace test failed a sampler child-process assertion; an isolated retry passed. The full retry failed E0283 in concurrently edited renderer.rs:530. A stable shared tree and successful full checks remain required.
- 4.1: fresh OpenCut task, another project's task, and local plugin re-enablement have not been observed through desktop runtime. Static configuration tests are not substituted for this evidence.
- 4.5: archival is withheld because verification is incomplete. See verification.md for command results, log locations, and scenario traceability.

These observations describe the original run. Subsequent review found font-shaping archived and `cargo check -p opencut-editor-core --tests` passing; that targeted result is not a successful full workspace test run. The only active change is now this one. Refresh verification.md after revised-artifact approval, retaining prior failures as history.

## 5. Corrective revision

- [x] 5.1 Obtain explicit approval of the revised proposal, requirements, design, and tasks before corrective implementation. Record a fresh scoped baseline afterward.
- [x] 5.2 Retain the exact nine local-marketplace overrides and Astra/medium settings; clarify config comments and contributor guidance about local scope, remote/workspace-managed limitations, and evidence sources (Reversible local plugin availability).
- [x] 5.3 Remove the synthetic JavaScript merge test as evidence of Codex inheritance. Retain exact-value/negative configuration and safeguard tests; add documentation regression tests and negative fixtures rejecting unconditional complete-removal claims and incorrect archival ordering (Reversible local plugin availability; Ordered verification and final merge readiness).
- [x] 5.4 Clarify implementation verification, expected rejection caused only by this active change, synchronization/archival, and mandatory final gate success in contributor guidance. Preserve all required checks and protected scripts (Ordered verification and final merge readiness).
- [ ] 5.5 Rerun focused Bun tests and Biome with the exact commands in 3.1, strict validation in 3.2, and the required Rust/bridge/Python commands in 3.4-3.6 on a stable tree. Reuse evidence only when relevant inputs remain unchanged; retain all command statuses and failure logs.
- [x] 5.6 Refresh verification.md with current evidence and the expanded scenario matrix. Remove current-blocker claims disproven by the review while retaining them as historical results; record remote-control and observation limitations explicitly. Confirm global settings, generated skills, protected scripts, and unrelated work were not modified by this task.
- [ ] 5.7 Use $openspec-verify-change against the revised requirements, complete the corrected 3.3/4.1/4.5 sequence, and require final protected-gate success. No measured-savings claim without comparable telemetry.

## Corrective verification blockers — current

- All 17 focused tests, Biome, strict specification validation (27 items), Rust formatting/Clippy, bridge typecheck/lint, 392 bridge unit tests on retry, 11 MCP integration tests, 6 packaged smoke tests, and 15 Python worker tests passed in this corrective run. The initial unit timeout is retained as history in verification.md.
- The protected gate rejects `fix-font-draft-retention-and-unicode-breaks` as well as this change. Because that is not rejection caused only by this active change, 3.3 and archival remain blocked.
- `cargo test --workspace` fails `component_draft_bindings_use_scoped_local_identity` in the unrelated font_draft_retention.rs:97 test with InvalidArgument. Tasks 3.4 and 5.5 cannot be marked complete.
- OpenSpec verification produced the current report and scenario mapping, but 4.5/5.7 cannot finish until the required shared-tree checks pass and only this change blocks pre-archive readiness. No other change was edited or archived. No post-archive gate success is claimed.
