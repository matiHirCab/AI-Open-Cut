## Context

ADR 0002 and the contract-governance specification define `bun run contracts:check` as the standalone fixture-governed parity command. The render-regression specification and editor-core golden harness define deterministic native conformance, report validation, and immutable reviewed references. CI currently invokes those checks inside the cross-platform `correctness` and multi-purpose `packaged-smoke` jobs, so branch protection and reviewers cannot distinguish a contract failure from unrelated correctness failures or a golden regression from packaging/integration failures.

The change is cross-cutting only at the repository-validation layer. Production Rust and TypeScript behavior, canonical fixtures, render thresholds, persisted data, and public contracts remain untouched. The workflow must continue using pinned Moon/Bun/Rust tools, Linux FFmpeg/FFprobe, and the deterministic DejaVu Sans fixture font.

## Goals / Non-Goals

**Goals:**

- Publish stable, independently visible `contract-parity`, `render-parity`, and aggregate foundation status checks.
- Make contract drift fail through the existing standalone parity command.
- Make coordinated preview/range/export drift fail through the existing native golden harness while preserving required benchmark-observation validation and upload.
- Keep broader correctness, integration, packaged-smoke, and lifecycle coverage without running the same expensive parity checks twice.
- Add a fast structural policy checker that detects accidental weakening of the required jobs and commands during ordinary repository validation.
- Give contributors exact local reproduction commands and identify the aggregate status intended for branch protection.

**Non-Goals:**

- Change contract catalogs, native declarations, Zod/MCP schemas, capability versions, stable errors, or CODEOWNER assignments.
- Change renderer evaluation, output, fixtures, tolerances, golden update behavior, or performance budgets.
- Change project persistence, migrations, revisions, batches, aliases, undo/redo, reopen semantics, or path policy.
- Configure repository-host branch-protection settings from source control.

## Decisions

### Use dedicated jobs plus one aggregate status

Move `bun run contracts:check` into an Ubuntu `contract-parity` job and the configured native golden/lifecycle checks into an Ubuntu `render-parity` job. Add a minimal `foundation-parity` job whose `needs` list contains both jobs. The leaf jobs make failures diagnosable; the aggregate job gives branch protection one stable status that cannot succeed unless both leaf gates succeed.

Keeping the checks only inside the existing broad jobs was rejected because their status names conflate unrelated failures and their future matrix or packaging changes can accidentally weaken the foundation boundary. Making all downstream CI jobs depend on parity was rejected because it increases feedback latency without strengthening merge protection.

### Reuse authoritative commands instead of creating parallel validators

The contract job runs `bun run contracts:check`, which already covers TypeScript checking, Rust headless evidence, and the focused TypeScript contract suite. The render job calls the exact editor-core golden conformance and external-report validation tests, plus the existing headless lifecycle test that covers edit, undo, redo, reopen, and draft isolation. No contract or rendering rule moves into workflow code.

A new combined validation executable was rejected because it would duplicate ownership and make local and CI behavior diverge. Comparing encoded container bytes was rejected because the accepted golden specification compares decoded frames/audio, timing, semantic plans, and normalized filter graphs.

### Keep deterministic render setup and report publication together

`render-parity` installs FFmpeg and the declared font, sets `OPENCUT_GOLDEN_REQUIRED=1`, uses explicit FFmpeg/FFprobe/font paths, writes the report to an absolute workspace path, validates that same path, and uploads it. The native harness remains fail-closed when a dependency or reference is invalid. Performance observations remain report-only and are not used as pass/fail budgets.

Splitting report capture and validation across independent jobs was rejected because it introduces artifact transfer as a new failure boundary and weakens the guarantee that validation reads the exact file just produced.

### Enforce workflow policy through the pinned OpenSpec validation task

Add a repository script that parses the CI workflow structurally and fails unless the three stable jobs, dependency edges, authoritative commands, deterministic environment, report validation, and upload path are present. Invoke it from the existing `openspec-validate` Moon task, which already runs in a dedicated required CI job, and cover the parser's failure behavior with focused tests where practical.

Text-only substring checks were rejected because YAML reordering and multiline command formatting would create false results. Depending solely on reviewer memory was rejected because issue #16 explicitly requires regression detection before later milestones.

## Risks / Trade-offs

- **Branch protection may not yet require the aggregate status.** → Document `foundation-parity` as the stable required status; repository-host configuration remains an explicit maintainer action outside this source change.
- **The render gate is comparatively expensive.** → Run it once on Linux in its dedicated job and remove its duplicate execution from packaged smoke.
- **A workflow-policy check can become stale when commands legitimately evolve.** → Keep it focused on semantic invariants and update it in the same reviewed OpenSpec change as any intentional gate change.
- **Moving checks changes CI status names.** → Retain all broader jobs, add the aggregate status additively, and document the required-status migration before removing reliance on embedded parity steps.

## Migration Plan

1. Add and locally verify the workflow-policy checker.
2. Extract the two parity jobs, preserve their existing setup and commands, and add the aggregate status.
3. Remove only duplicate parity steps from broader jobs; keep all other checks.
4. Run OpenSpec, workflow-policy, contract, Rust, TypeScript, integration, and smoke verification.
5. Configure GitHub branch protection to require the documented aggregate status after the workflow is present on the default branch.

Rollback restores the embedded steps and removes the dedicated/aggregate jobs and policy assertions together. No application, contract, fixture, or persisted-data rollback is required.

## Open Questions

None. GitHub-host branch-protection activation is a maintainer follow-up because source changes cannot safely assume repository administration authority.
