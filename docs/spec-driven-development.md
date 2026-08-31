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

1. Read the existing specs and the root `AGENTS.md` before proposing behavior.
2. Use a stable kebab-case capability name based on user-visible behavior, not a crate, app, or implementation detail.
3. Start new behavior with `$openspec-propose`; do not edit living requirements to describe unimplemented roadmap work.
4. Review `proposal.md`, delta specs, `design.md`, and `tasks.md` before applying the change.
5. Implement and verify the tasks, then archive the change so accepted deltas become the new living source of truth.

Every requirement uses `SHALL` or `MUST`. Every requirement has at least one `#### Scenario:` expressed with `WHEN` and `THEN`. Public and persisted contract changes also cover compatibility, revision conflicts, migrations, and typed failures.

## Cross-language public contracts

ADR 0002 selects fixture-governed manual synchronization. Before changing a public request, response/event, stable error, capability, MCP tool or resource, provider protocol, persisted project shape, or version rule:

1. Find its single canonical owner and governed consumers in `contracts/contract-ownership-v1.json`.
2. Classify the change as additive or breaking in the OpenSpec proposal and delta requirements. Breaking changes require a new major contract and explicit migration path.
3. Update the canonical fixture/catalog, every affected native declaration, and the shared parity tests in the same change.
4. Run `bun run contracts:check` from `apps/agent-bridge`, followed by the affected Rust, TypeScript, integration, and smoke checks.
5. Obtain review from the CODEOWNER designated for both the canonical artifact and its governed consumers.

An implementation-local test is not equivalent evidence: the parity gate must consume the checked-in canonical artifacts and cover each affected Rust, TypeScript/Zod, MCP, or provider surface.

## Validation

After installing the pinned toolchain with `proto use`, validate all specs and active changes from the repository root:

```sh
moon run openspec-validate
```

The underlying pinned command is:

```sh
bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive
```

CI runs the same Moon task. A malformed requirement, scenario, or change blocks the pull request.

## Upgrading OpenSpec

This repository uses OpenSpec's custom profile with all eleven Codex skill workflows enabled. Upgrade deliberately and keep every version-sensitive surface synchronized:

1. Choose the new version and update `OPEN_SPEC_VERSION` in `scripts/normalize-openspec-workflows.ts`, the package pin in `moon.yml`, and all version references in this guide.
2. Run `bunx @fission-ai/openspec@<version> update` from the repository root while preserving the custom profile.
3. Run `bun run scripts/normalize-openspec-workflows.ts` to replace generated prompt aliases with repository-local `$openspec-*` skill names and route generated CLI calls through the pinned package.
4. Review every generated `.codex/skills/openspec-*/SKILL.md` change and confirm its `generatedBy` metadata names the new version. Do not hand-edit generated workflow bodies.
5. Confirm `.github/workflows/bun-ci.yml` still runs `moon run openspec-validate`, so CI inherits the same pin instead of declaring a second one.
6. Run the doctor, spec listing, strict Moon validation, and workflow syntax checks before merging the upgrade.
