# Issue 45 proposal preparation evidence

Status: proposed, unapproved, no implementation. Date: 2026-10-02 UTC. This records planning evidence; none of the implementation checklist is complete.

## Source and Git preflight

- Current issue [#45](https://github.com/matiHirCab/AI-Open-Cut/issues/45) was read through the authorized GitHub connector. [#38](https://github.com/matiHirCab/AI-Open-Cut/issues/38), [#39](https://github.com/matiHirCab/AI-Open-Cut/issues/39) and [#11](https://github.com/matiHirCab/AI-Open-Cut/issues/11) were verified closed. [#46](https://github.com/matiHirCab/AI-Open-Cut/issues/46) remains open and owns the five named creative presets.
- Remote main: `90f7884f1b58deb1a4cd5992083520b516b56fe3`. Draft [PR #133](https://github.com/matiHirCab/AI-Open-Cut/pull/133): `618357054cd6d2d40275ec6f61163996852227a3`. `git merge-base --is-ancestor origin/main HEAD` passed when HEAD was the exact PR head; `git ls-remote` confirmed both remote refs. PR #133 is open/draft/unmerged at this read. Its CI monitor is separate; no terminal CI claim is made here.
- Isolated worktree: `/workspace/issue45-proposal`, branch `codex/issue-45-proposal-20261002`, based directly on PR #133's head. Original `/workspace/AI-Open-Cut` remained clean on branch `work` at main. No correction code, tests, archives or PR metadata were edited.
- Before substantial proposal authoring, normal local Git commit succeeded for the generated `.openspec.yaml`: `26b2663e3378fe6ce92777b45623d3143562c367` (`docs(openspec): scaffold issue45 compiler proposal`).
- `git push --dry-run origin HEAD:refs/heads/codex/issue-45-proposal-20261002` exited 0. Subsequent `git ls-remote --heads origin codex/issue-45-proposal-20261002` returned no ref: no branch was published. This checks current transport/readiness, not future authorization or guaranteed later branch-protection approval.

## Planning validation

Pinned Bun 1.4.0, Moon 2.3.3 and OpenSpec 1.5.0 were installed/used locally under `/tmp`; no repository/global credentials, security or workflow configuration changed. Initial npm installation failed because its default cache location was unavailable; moving the task cache under `/tmp` resolved installation. Bun's package wrapper required its omitted postinstall, so the installed official platform binary was used directly. Neither failure changed repository files.

- PASS, exit 0: `bunx @fission-ai/openspec@1.5.0 status --change add-animation-preset-compiler` reports all four planning artifacts complete. This is artifact completeness, not owner approval.
- PASS, exit 0: `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive`: 36 passed, 0 failed (35 living specifications and this active change). Log: `/tmp/issue45-openspec-validation.log`.
- PASS, exit 0: independent local document audit `python /tmp/issue45-proposal-audit.py`: five capability files match the proposal, all 13 requirements have WHEN/THEN scenarios and coverage-map rows, all 37 scenarios are well formed, 22 unique implementation/review tasks are unchecked, both design JSON examples parse, and the example's normalized parameters/fixed primitive keys match. Script/log: `/tmp/issue45-proposal-audit.py`, `/tmp/issue45-proposal-audit.log`. This is document consistency evidence, not compiler correctness evidence.
- PASS, exit 0: staged `git diff --cached --check`; inventory restricted to this OpenSpec change directory.
- EXPECTED AUTHORING REJECTION, exit 1: isolated `bun --config=/dev/null --no-env-file run scripts/run-ci-policy.ts` rejects only `add-animation-preset-compiler` as an unarchived change. Log: `/tmp/issue45-policy-preflight.log`. No policy attestation, passed protected gate or merge readiness is claimed.
- ENVIRONMENT BLOCKED, exit 1: direct `moon run root:openspec-validate` first could not create its default `~/.moon` in the read-only home. With only task-local `MOON_HOME`/`PROTO_HOME`/cache destinations under `/tmp`, it then stopped at `proto::offline`: “Internet connection required, unable to download, install, or run tools.” Final log: `/tmp/issue45-moon-validation.log`. The direct Moon task did not execute its required checks. No skip/bypass flags, fake attestation or gate edits were used. Toolchain availability must be resolved for implementation's required checks; this does not replace the expected active-change rejection above.

Rust, bridge, migration, integration, native render, packaged smoke and Python implementation suites were not run: there is no issue-45 implementation to verify. Their required commands and scenario coverage are in `tasks.md`; all remain pending. Existing PR #133 conformance evidence is not relabeled as issue-45 verification.

## Required decision and stop point

Root [`AGENTS.md`](../../../AGENTS.md) requires: “Create or update the OpenSpec artifacts first, then obtain explicit user or reviewer approval before implementation.” [`docs/spec-driven-development.md`](../../../docs/spec-driven-development.md) applies the same lifecycle. The OpenSpec skill does not independently require this approval; the repository's hard gate does.

Approve the complete proposal/design/deltas/tasks, specifically:

1. One `scalar_tween@1` seed with the six current targetless scalar properties, explicit endpoints, item-local half-open timing and existing four curve types; retain #46 for the creative pack.
2. Default exact-channel collision rejection and explicit whole-channel replacement, preserving unrelated content and refusing implicit legacy conversion.
3. Descriptive sidecar provenance with resolved channels authoritative, raw-setter label clearing, secondary-path reconciliation and compiler-independent retired-source reopen.
4. Atomic schema 28 -> 29 migration of current/component/retained undo/redo state; no live downgrade.
5. Root standalone/batch authoring initially; reject new preset intents inside draft lists and direct component-definition authoring while preserving existing primitive workflows.

Issue #44's historical/delegated approvals do not approve these choices. No issue-45 code, public contract artifact, schema, migration, executable test or living specification was changed. No proposal archival, external branch/PR publication, merge or deployment occurred. The local documents and transport checks are the reviewable result; new owner approval is the next implementation gate.
