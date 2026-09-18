# Spec-driven development with OpenSpec

OpenCut uses OpenSpec 1.5.0 to keep reviewable behavior requirements beside the code. The living source of truth is under `openspec/specs/`; proposed changes live under `openspec/changes/` until they are implemented and archived.

## Codex workflows

Restart Codex after initial setup so generated workflows are discovered. Invoke the repository-local skills directly:

- `$openspec-explore` to investigate an unclear problem without creating artifacts.
- `$openspec-propose` to create a proposal, delta specs, design, and tasks for a change.
- `$openspec-new-change` to create a named change without generating every artifact immediately.
- `$openspec-continue-change` to create the next ready artifact for an active change.
- `$openspec-apply-change` to implement an approved change and track its tasks.
- `$openspec-ff-change` to fast-forward an active change through all planning artifacts.
- `$openspec-sync-specs` to reconcile a change's delta requirements into living specs without archiving it.
- `$openspec-archive-change` to close a completed change and update the living specs.
- `$openspec-bulk-archive-change` to archive several completed, non-conflicting changes together.
- `$openspec-verify-change` to check that implementation, specs, design, and tasks agree before archival.
- `$openspec-onboard` for a guided walkthrough of the complete OpenSpec lifecycle.

These repository-local Codex workflows are skills, so invoke them with the `$openspec-*` names above. Every CLI call in the skills uses `bunx @fission-ai/openspec@1.5.0`; a global `openspec` installation is neither required nor accepted as the repository workflow. The generated skill bodies are owned by OpenSpec, then normalized by `scripts/normalize-openspec-workflows.ts` for pinned CLI and Codex skill compatibility rather than edited by hand.

## Adding or changing a capability

Follow the root [AGENTS.md](../AGENTS.md#mandatory-spec-driven-development) for required reading, artifact approval, implementation, scenario coverage, verification, and archival. Start new behavior with `$openspec-propose`, use a stable kebab-case capability name based on user-visible behavior rather than implementation structure, and do not edit living requirements to describe unimplemented roadmap work.

Active changes remain available for local authoring and focused validation, but they are never merge-ready. The protected policy preflight requires `openspec/changes/` to contain only its ordinary `archive/` directory; any other file, directory, or symbolic link blocks Moon execution and prevents the policy attestation. Finish, synchronize, verify, and archive every change before expecting the protected CI gate to pass.

Every requirement uses `SHALL` or `MUST`. Every requirement has at least one `#### Scenario:` expressed with `WHEN` and `THEN`. Public and persisted contract changes also cover compatibility, revision conflicts, migrations, and typed failures.

## Cross-language public contracts

ADR 0002 selects fixture-governed manual synchronization. Before changing a public request, response/event, stable error, capability, MCP tool or resource, provider protocol, persisted project shape, or version rule:

1. Find its single canonical owner and governed consumers in `contracts/contract-ownership-v1.json`.
2. Classify the change as additive or breaking in the OpenSpec proposal and delta requirements. Breaking changes require a new major contract and explicit migration path.
3. Update the canonical fixture/catalog, every affected native declaration, and the shared parity tests in the same change.
4. Run `bun run contracts:check` from `apps/agent-bridge`, followed by the affected Rust, TypeScript, integration, and smoke checks.
5. Obtain review from the CODEOWNER designated for both the canonical artifact and its governed consumers.

An implementation-local test is not equivalent evidence: the parity gate must consume the checked-in canonical artifacts and cover each affected Rust, TypeScript/Zod, MCP, or provider surface. Headless operation evidence is derived from the actual Rust request enum and checked against the Serde tags accepted on the wire. MCP evidence includes normalized structural input schemas, structural output schemas, and client-visible annotations for every registered tool; schema descriptions are documentation copy and are excluded recursively. The checked-in catalogs are reviewed and updated manually; parity tests never rewrite them.

## Validation

After installing the pinned toolchain with `proto use`, validate all specs and active changes from the repository root:

```sh
moon run root:openspec-validate
```

The underlying pinned command is:

```sh
bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive
```

CI starts the bootstrap with repository Bun configuration and dotenv loading disabled, validates the complete Bun/Moon/proto boundary and archive-only OpenSpec change inventory through `run-ci-policy.ts`, and only then runs the same Moon task. A malformed requirement, scenario, configuration, active change, or archive boundary blocks the pull request.
Moon uses the workspace's configured `main` VCS default branch for revision comparison in both local and CI checkouts.
The same task also validates the dedicated contract/render parity workflow policy described in
[`ci-parity-gates.md`](ci-parity-gates.md), so weakening a required foundation gate fails repository validation.

## Project agent settings and verification

`.codex/config.toml` keeps Astra and medium reasoning/planning and contains nine local-marketplace overrides scoped to this project with `enabled = false`. Other settings and plugins remain inherited. Remote/workspace-managed plugins may remain available despite a corresponding local override; unsupported project-local suppression is an accepted limitation. These settings do not establish complete desktop plugin removal. Do not change global configuration or project trust, invent remote keys, or uninstall plugins to apply them. See the [Codex configuration reference](https://learn.chatgpt.com/docs/config-file/config-reference).

To re-enable a local installation, edit its existing table in `.codex/config.toml` to `enabled = true`, then open a fresh OpenCut task (restart Codex if needed). This does not control a separate remote installation. Restore `enabled = false` to return to the reviewed defaults. The focused configuration test intentionally checks those defaults, so restore them before final verification. A managed override or explicit task setting can take precedence; report any mismatch instead of bypassing it.

Run the focused regression test from the repository root:

```sh
bun test scripts/agent-context-efficiency.test.ts
```

This checks parsed settings and documented safeguards; static tests do not prove desktop loading, Codex inheritance, or future agent behavior. CLI-only evidence does not prove desktop availability either. Record each observation's source: a fresh desktop task, an existing task's catalog, CLI rendering, or static configuration. Check Astra/medium planning and which skills/tools actually appear in a fresh OpenCut task; record remote plugins that remain available. Check another project retains its existing settings and plugins where observable. Report unavailable runtime evidence explicitly, separately from passing static checks and accepted remote-control limitations. A JavaScript object-merge fixture is not evidence of Codex configuration loading.

For routine discovery, start with `rg --files openspec/specs` and requirement headings in the applicable file. Search current material with `rg -n '<pattern>' openspec/specs openspec/changes -g '!**/archive/**'`; remove the exclusion for historical investigation. Validators still read their entire required scope. The root instructions define instruction reuse and check-evidence invalidation.

For lengthy checks, keep full logs outside the repository and report the command, exit status, a short summary, relevant failures, and log path. For example in PowerShell:

```powershell
$checkLog = Join-Path $env:TEMP ('opencut-check-' + [guid]::NewGuid() + '.log')
bun test scripts/agent-context-efficiency.test.ts *> $checkLog
$checkExit = $LASTEXITCODE
Write-Output "Exit: $checkExit; log: $checkLog"
Get-Content -LiteralPath $checkLog -Tail 12
```

Inspect relevant failure sections when a tail lacks enough detail. Never commit logs or expose secrets from them. A blocked archive-only gate must identify the active changes; do not modify or archive unrelated work to make it pass.

When comparable telemetry exists, compare the same small read-only prompt in fresh tasks before and after, with the same model. Record input/context, cached input, output/reasoning, and allowance separately where available. If the baseline or telemetry is unavailable, do not claim measured savings or infer tokens from repository file sizes.

## Verification order

1. Run required implementation checks on a stable tree, including focused tests, formatting/linting, strict specification validation, and all repository-required suites. Passing compilation alone does not establish passing tests.
2. Run the protected gate before archival: `moon run root:openspec-validate`. Inspect its complete result. Rejection naming only this active change is expected before archival, not a passed gate. Any other failure blocks archival; never modify or archive unrelated work to clear it.
3. Verify implementation conformance with `$openspec-verify-change`, including requirements, design, tasks, scenario coverage, and explicitly recorded evidence limitations. All required implementation checks must have passed.
4. Synchronize and archive this verified change only with `$openspec-sync-specs` and `$openspec-archive-change`. Final merge-readiness validation remains pending until the next step.
5. Run the protected gate after archival and strict all-spec validation again. The unchanged protected gate must pass before declaring completion. Record any failure and keep completion blocked. Do not weaken checks or fabricate an attestation.

This sequence preserves the root instructions and archive-only CI enforcement. It separates pre-archive implementation verification from post-archive final merge readiness; it does not waive any final check.

## Upgrading OpenSpec

This repository uses OpenSpec's custom profile with all eleven Codex skill workflows enabled. Upgrade deliberately and keep every version-sensitive surface synchronized:

1. Choose the new version and update `OPEN_SPEC_VERSION` in `scripts/normalize-openspec-workflows.ts`, the package pin in `moon.yml`, and all version references in this guide.
2. Run `bunx @fission-ai/openspec@<version> update` from the repository root while preserving the custom profile.
3. Run `bun run scripts/normalize-openspec-workflows.ts` to replace generated prompt aliases with repository-local `$openspec-*` skill names and route generated CLI calls through the pinned package.
4. Review every generated `.codex/skills/openspec-*/SKILL.md` change and confirm its `generatedBy` metadata names the new version. Do not hand-edit generated workflow bodies.
5. Confirm `.github/workflows/bun-ci.yml` still invokes the isolated `run-ci-policy.ts` bootstrap and that the bootstrap launches `moon run root:openspec-validate`, so CI validates the reviewed boundary before addressing the pinned root task.
6. Run the doctor, spec listing, strict Moon validation, and workflow syntax checks before merging the upgrade.
