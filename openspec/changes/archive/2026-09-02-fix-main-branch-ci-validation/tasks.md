## 1. Workspace VCS Configuration

- [x] 1.1 Add `vcs.defaultBranch: 'main'` to `.moon/workspace.yml` and confirm Moon resolves changed files without a local or remote `master` ref using `bunx @moonrepo/cli@2.3.3 query changed-files --default-branch`.
- [x] 1.2 Update `docs/spec-driven-development.md` to state that Moon's validation task uses the configured canonical `main` branch in local and CI checkouts.

## 2. Validation

- [x] 2.1 Run `bun run scripts/normalize-openspec-workflows.ts --check` and `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive` from the repository root.
- [x] 2.2 Run `bunx @moonrepo/cli@2.3.3 run openspec-validate` from the repository root and verify Moon executes both pinned OpenSpec commands instead of resolving `master`.
- [x] 2.3 Run `git diff --check` and inspect the final diff for unrelated files, runtime behavior, public contracts, persisted data, migrations, and generated build output.

## 3. Integration and Completion

- [x] 3.1 Apply `$openspec-verify-change`, resolve any mismatch among the requirement, design, tasks, configuration, documentation, and validation evidence, then archive the verified change with `$openspec-archive-change`.
- [x] 3.2 Merge through the normal pull-request workflow and verify the resulting GitHub Actions push run on `main` passes OpenSpec validation; no Rust, TypeScript, Python, contract, render, or packaged-smoke commands are added because their code and job definitions are unchanged.
