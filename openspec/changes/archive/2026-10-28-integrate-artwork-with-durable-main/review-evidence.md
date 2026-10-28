# Pre-implementation review evidence

Observed 2026-10-28 through the authorized GitHub connector and local fetched refs. This is planning evidence, not implementation verification.

- Current main: b350b5cc85b4a9d31578992fde83f598e4392c68, merged durable requests PR228. Both local `work` and fetched origin/main initially matched.
- PR229: open, non-draft, conflicting; branch feat/issue-220-artwork-derivatives, d70120792511f50f99c363206620211787999638. Linked #220 remains open and scoped to source-preserving artwork.
- PR233: open draft, mergeable at observation; branch fix/windows-preview-disposal-safety, 2768cd00bec17b681e8342a4a7c51e3750bd28ec. Proposed native-boundary extension explicitly unapproved; production edits remain excluded.
- #234: dependency maintenance, requires its own approved OpenSpec; expressly preserves disposal/deletion/junction/crash/descendant restrictions. No implementation was selected from this issue.

## Terminal CI at existing PR heads

PR229: OpenCut CI39613333026, platform renderer39613333030 and motion release39613332784 succeeded. Windows startup39613332788 failed in job119267696248. The unchanged original fixture failed at render_worker.rs:571: PID record, shell-entry and renderer-stderr files missing after10.007s, cmd.exe and powershell.exe present. The native comparison published PID after31.7372ms. This does not establish an artwork failure or authorize a fixture/production repair.

PR233 OpenCut CI39598451825 failed Windows correctness job119216545488 solely in the hermetic TypeScript unit step: contracts.test.ts:569, deterministic MCP catalog expansion test,5000ms timeout;1050passed,18skipped,1failed. Rust formatting/Clippy/tests, typecheck/lint passed in that job. Render parity119216545455 and contract parity119216545692 succeeded. OpenSpec119216545168 and aggregate119242806744 failed separately. Native renderer39598451844, motion release39598451738 and Windows startup39598451832 succeeded. Passing diagnostic controls are not cause attribution or a verified disposal repair.

Main's exact SHA is independently verified. Connector commit-workflow lookup returns only pull-request-triggered runs and returned none for main; combined legacy status returned an empty array. Therefore the parent's main CI failure summary was not independently verified here and is not claimed as fresh observed evidence.

## Read-only integration result

`git merge-tree --write-tree origin/feat/issue-220-artwork-derivatives origin/main` reported three textual conflicts:

- apps/agent-bridge/tests/fixtures/narration-profiles-projection.ts
- apps/agent-bridge/tests/fixtures/short-candidates-projection.ts
- apps/agent-bridge/tests/smoke.test.ts

No actual merge or executable edit was performed. Catalog automatic merges still require exact semantic and predecessor verification. Main's living durable requirement `Preserve main47 contracts and independent durable proofs` explicitly prohibits unmerged artwork/schema48. The archived PR229 integration approved reconciliation with main8cb21a6b and narration, not durable-main compatibility. This successor proposal resolves that specification conflict and awaits explicit approval.

## Tool and access limits

Git fetch succeeded. CLI `gh pr checks 229` and `gh pr checks 233` returned GraphQL Forbidden; neither was retried. Authorized connector PR/run/job-log reads succeeded and supplied the above evidence. No branch-protection administrative API was queried or retried. No request to loosen permissions or bypass a restriction was made.

Pinned Bun1.4.0 was downloaded into /tmp for the proposal workflow. No dependency manifest/lockfile/tool pin was changed. Rust/Moon were not installed and no implementation tests were claimed. No remote push, commit, PR state change, merge or deployment has occurred.

Planning verification passed: `bunx @fission-ai/openspec@1.5.0 validate integrate-artwork-with-durable-main --strict --no-interactive` (exit0); all four planning artifact types are complete according to pinned OpenSpec status; `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive` passed80/80, exit0, full local log `/tmp/opencut-artwork-durable-planning-validation.log`; `git diff --check` passed. These are specification checks, not protected Moon success or runtime conformance.

## Artifact identities awaiting approval

- proposal.md: f442ae2f7c2ce8e5181d66934506ba8bc1256e3c437721f47342796f0f57e249
- design.md: 1a7c50fe2ab3c1f45bb8d5361af6b6d00d388f064e871caa8bb3179c1c5baa5f
- specs/durable-production-requests/spec.md: d5183125af697aba174996185babf4436093bdfef66ff21d82316a4c94ce202a
- specs/project-persistence/spec.md: f57d199cb1695f95ef960dbfe2b305ba4975b66c429af65acdfa7fa29033ade2
- tasks.md: 15fda64657be11b0053af5c392c9489da32f6a77443e16fa71fa68cc9d6fe616
