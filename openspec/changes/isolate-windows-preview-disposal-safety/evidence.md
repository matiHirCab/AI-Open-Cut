# Investigation evidence — 2026-10-21 UTC

## Current state and ownership

- Saved cloud host is Linux, not Windows. Initial clean branch `work` was at `ae0a6cbc01726a1b4123303a49ec38783cebf32d`; `git fetch origin main` succeeded and independently confirmed `8cb21a6bab2f5a0134ab6af3f91a018f0ae9bde8`. Created `fix/windows-preview-disposal-safety` from that main. No pre-existing dirty files, additional worktrees or local #208 WIP appeared; nothing was reconstructed or removed.
- `github_search_prs` confirmed only open PR229 (artwork) and PR228 (durable requests). `github_search_issues` for Windows disposal returned no matching issue. PR229 metadata gives head `aeea97a749132cb9acddb455af7262ffb56d546f`; its verification report records an earlier packaged-disposal ENOENT in Windows job118401719665, followed by passing subsequent heads without a disposal fix. That is historical recurrence, not proof of a baseline cause.
- Native disposer and preview-disposal-workflow.ts are byte-identical between saved checkout ae0a6cbc and current main8cb21a6b (`git diff` empty). No blame to narration or other unrelated changes is established.
- Shell `gh auth status` reports invalid GH_TOKEN; API/GraphQL and unauthenticated API curl fail. The existing GitHub connector and Git transport work independently. No credentials/security settings changed. A workflow-run lookup initially omitted required commit_sha; a corrected lookup returned an empty list and is not counted as workflow verification. Original exact-job logs were successfully fetched via the connector.

## Exact Windows observations

Original log: https://github.com/matiHirCab/AI-Open-Cut/actions/runs/39586606594/job/119176954562

- Windows Server2025 build10.0.26100, runner image20261007.264.1. Bun1.4.0 and Rust1.97.0 installed successfully; Chocolatey FFmpeg7.1.1 and dependency setup tests passed.
- Source renderer acceptance passed in435759ms. Packaged renderer acceptance failed in770ms. ENOENT reads a synthetic preview UUID filename directly beneath opencut-preview-disposal temporary root at preview-disposal-workflow.ts101, called by platform-renderer-native.test.ts158.
- The error occurs after twelve native disposer invocations and after the race promise settles. No operation-level trace identifies the remover. Failure is before ordinary deletion, foreign-path, static reparse and close/debt cases later in that helper.
- Bun reports its own version in the orchestration failure, but Vitest's checked-in dependency entrypoint has a node shebang. Actual Windows test runtime/version is not recorded; do not infer Bun executed the fixture simply from the orchestrator.

## Competing paths requiring controls

- Fixture: starts race before sentinel publication; repeatedly renames previews, creates a junction to the temporary root, recursively rm's the junction, and restores previews. This is a second deletion path outside the native adapter. Fixture-only Windows controls are needed before attributing target traversal to runtime cleanup.
- Native: opens checked root/project/previews with FILE_FLAG_OPEN_REPARSE_POINT and no FILE_SHARE_DELETE; verifies reparse/type metadata; deletes the opened regular target with SetFileInformationByHandle. Existing native test expects ancestor rename refusal while handles are held. Static inspection is not Windows race execution.
- Both fixture/root and native possibilities remain hypotheses. A passing Linux adapter test does not validate Windows sharing or junction semantics. The sentinel loss proves a test failure, not production data loss.

## Local preparation and checks

All temporary dependencies/logs remain outside tracked repository files.

- Bun/Moon/proto/Rust initially absent from PATH. Installed official npm distribution `@oven/bun-linux-x64@1.4.0` into /tmp/opencut-tools with cache in /tmp; observed Bun1.4.0. Initial npm pack without a writable cache failed ENOENT under the default cache; repeated with /tmp cache succeeded. No global config modified.
- Frozen bridge install passed (147 packages), lockfile unchanged.
- Pinned OpenSpec CLI created this change using `$openspec-propose`; proposal/design/spec/task instructions were read and artifacts follow their templates. Focused strict change validation passed. Strict all-spec validation passed79/79, log `/tmp/opencut-preview-strict-all.log`.
- Existing Linux bridge disposal/lifecycle baseline:19/19 passed across2 files, log `/tmp/opencut-preview-baseline-focused.log`. This covers adapter/lifecycle tests, not the native Windows race.
- Baseline TypeScript typecheck passed, log `/tmp/opencut-preview-baseline-typecheck.log`.
- Baseline lint passed321 files without fixes, log `/tmp/opencut-preview-baseline-lint.log`. `git diff --check` passed; only this new proposal directory is untracked. Final `git ls-remote origin refs/heads/main` still returns8cb21a6b.

## Remaining gates

Explicit approval of this concrete change is pending, requested using the asynchronous input tool. AGENTS.md requires: "Create or update the OpenSpec artifacts first, then obtain explicit user or reviewer approval before implementation." No executable files have been edited. No native Windows execution, rerun, commit, push or PR creation has occurred. No automatic approval review rejected any action; no mutation has an uncertain outcome.

Required next evidence is fixture-only/native-only Windows controls followed by actual source/package race acceptance. Production code must remain unchanged unless those controls establish a native defect and an amended design is approved. Full repository implementation suites, protected Moon gate, verification/synchronization/archival and draft PR remain incomplete. Strict spec validation alone is not implementation completion. The protected gate will intentionally reject this unimplemented active change; do not archive it or unrelated work to clear that gate.

## Follow-up after explicit delegated review

The approval blocker above is superseded by the explicit delegated reviewer approval recorded in approval.md. All four complete artifacts were pasted into the parent thread and reviewed before executable edits. Reviewer authorized publishing a locally checked investigation draft before Windows evidence, while leaving this specification active and the repair incomplete.

Private acceptance changes add an independent twelve-cycle fixture-only control (no native process), an ordinary native-only control in a separate root, runtime/source/package observations, sentinel phase checks, actual replacement/sharing-refusal counts and a nonzero-replacement assertion. All twelve original race calls and original safety/idempotence/foreign/reparse/close-debt assertions remain. Recursive removal under investigation is unchanged. Safe failure recovery removes any remaining owned links before deleting the synthetic root and retains original failures; this is not a claimed teardown correction. Both primary and restoration failures remain visible together. No production code, package scripts, lockfiles, CI workflow/gates or public contracts changed.

Exact tools installed only under /tmp: Bun1.4.0 from official npm package, Rust1.97.0 rustc/cargo/std/rustfmt/clippy components from official distribution URLs with SHA256 checked against the official manifest, and Moon2.3.3 from its official npm platform package. No global/security/credential configuration changes. Temporary tool/cache paths are supplied per command.

Local observations on diagnostic tree:

- Frozen install remained unchanged. Typecheck passed (`/tmp/opencut-preview-diagnostic-typecheck-final.log`). Lint passed323 files (`/tmp/opencut-preview-diagnostic-lint-checked.log`). Initial lint/format failures are retained in diagnostic-lint.log, diagnostic-format.log and diagnostic-lint-final.log; corrected formatting, unused loop suppressions and accurate AggregateError cause handling before passing. No warning was globally disabled.
- Focused controls/adapter/lifecycle23/23 passed (`/tmp/opencut-preview-diagnostic-focused-final.log`). Controls detect missing and changed sentinel contents with phase attribution, retain simultaneous observation/restoration errors, verify12 actual fixture cycles, and verify recovery preserves directory targets and missing-link idempotence.
- Native headless build passed (`/tmp/opencut-preview-native-build.log`). Actual source/native confined-cleanup integration selection passed1/1,42 other cases filtered by explicit -t selection, not counted executed (`/tmp/opencut-preview-diagnostic-native-smoke.log`): Node24.19.0,12 native race calls,11 replacements,0 sharing refusals. This is Linux evidence only.
- Native core disposal5/5 passed (`/tmp/opencut-preview-native-core-tests.log`); private headless adapter1/1 passed (`/tmp/opencut-preview-native-adapter-tests.log`). Rust workspace formatting passed (`/tmp/opencut-preview-rust-fmt.log`).
- Strict all-spec validation passed79/79 (`/tmp/opencut-preview-diagnostic-strict-all.log`). Protected Moon, with isolated caches and exact-version global tool resolution, passed538 policy tests and79 strict items, then failed solely because this own active change is unarchived (`/tmp/opencut-preview-prearchive-moon-global.log`). This is expected rejection, not a passing gate. Earlier startup errors were read-only Wasmtime cache and proto offline resolution; logs retained as prearchive-moon.log and prearchive-moon-checked.log. No repository settings or validation code changed.
- Initial full unit run failed35/passed1020/existing skipped15 across4 files (`/tmp/opencut-preview-diagnostic-unit-all.log`). Failures concern unchanged descendant lifetime checks. Failed normalization fixture reports backend15914/descendant15916; `ps` independently confirmed both are stateZ with PPid1. Prior repository investigation documented the same container reaping limitation. A temporary launcher at `/tmp/opencut-preview-subreaper.py` sets child-subreaper status only on itself and reaps only its own adopted descendants, without changing repository code/assertions/timeouts. Full unchanged-suite result under that launcher follows after completion.

Full unchanged unit rerun under the owned reaper passed1055/existing skipped15 across83 passing files, with62 adopted descendants reaped (`/tmp/opencut-preview-diagnostic-unit-reaped.log`). The initially failing35 cases passed without any assertion/timeout/source changes; original failed output remains retained.

Windows execution and cause attribution remain pending; no repair or final conformance is claimed. Full Rust workspace/Clippy, contracts, integration, assembled smoke, Python checks and final archive/gate remain required before completion. Initial publication is investigation only, per reviewer sequencing approval.
