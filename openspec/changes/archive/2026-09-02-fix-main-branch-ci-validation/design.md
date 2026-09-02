## Context

Moon 2.3.3 uses `master` as its default VCS comparison revision when a workspace does not configure `vcs.defaultBranch`. OpenCut's only default branch is `main`. In a clean GitHub Actions checkout of a push to `main`, `moon run openspec-validate` builds its task graph by invoking Git against `master` and exits 128 before either normalization or strict OpenSpec validation runs. Pull-request checks can pass because their checkout/ref context supplies a usable comparison base, which allowed the configuration defect to reach `main`.

## Goals / Non-Goals

**Goals:**

- Make Moon use the repository's actual `main` branch for VCS comparisons in every environment.
- Preserve the existing pinned OpenSpec task and CI job coverage.
- Verify both direct pinned validation and Moon-orchestrated validation.

**Non-Goals:**

- Change the repository or GitHub default branch.
- Add a `master` compatibility branch.
- Change application, public contract, persistence, migration, security, or privacy behavior.
- Redesign CI or change which jobs are required.

## Decisions

### Configure the default branch at the Moon workspace boundary

Add `vcs.defaultBranch: 'main'` to `.moon/workspace.yml`. Moon's workspace configuration is the canonical source for revision comparison, task hashing, and affected-file behavior, so the fix applies to CI and contributor machines without per-command overrides.

This is preferred over setting `MOON_BASE=main` in GitHub Actions because an environment-only override would leave local and future CI invocations vulnerable to the same incorrect default. It is also preferred over creating a `master` branch because that would duplicate branch state and conceal the repository's actual convention.

### Use the existing required task as end-to-end evidence

The existing `moon run openspec-validate` job is the observable regression check: a clean push checkout on `main` must construct the task graph and execute both pinned validation commands. The direct `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive` command remains a diagnostic control proving that spec content is valid independently of Moon.

No application test suite is added because the failure occurs before application code or test discovery. The workspace schema plus the clean CI push run exercise the configuration at its actual boundary.

## Risks / Trade-offs

- [A future default-branch rename would make the setting stale] -> Treat the Moon workspace value as part of the branch-rename checklist and document the configured assumption beside validation guidance.
- [A local clone without a `main` ref can still fail VCS comparison] -> This is intentional; supported clones track the repository's canonical default branch, while CI fetches all branch refs for the validation job.
- [The current failed run cannot be made green retroactively] -> Merge the configuration fix and require the new `main` run to pass before closing epic #88.

## Migration Plan

1. Add the workspace VCS setting and update validation guidance.
2. Run workflow normalization, strict OpenSpec validation, and the Moon task locally.
3. Merge through the normal pull-request workflow and confirm the push run on `main` passes the OpenSpec job.
4. Roll back the workspace and documentation changes together if Moon rejects the setting; no data or runtime migration is involved.

## Open Questions

None.
