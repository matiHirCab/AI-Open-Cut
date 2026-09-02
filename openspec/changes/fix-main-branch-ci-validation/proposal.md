## Why

The required OpenSpec CI job fails on pushes to the repository's `main` branch before validation starts because Moon defaults its VCS comparison branch to `master`, which does not exist. The repository must declare its actual default branch so required validation runs consistently in clean CI checkouts.

## What Changes

- Declare `main` as Moon's canonical VCS default branch for repository task hashing and revision comparison.
- Add regression evidence that the workspace configuration does not fall back to a nonexistent `master` revision.
- Verify the exact OpenSpec task succeeds from the repository root and remains compatible with pull-request and push workflows.
- Non-goals: changing OpenSpec requirements, altering application behavior or public contracts, changing the GitHub default branch, or modifying CI job coverage.
- This change is non-breaking and does not affect public, persisted, or cross-language compatibility surfaces.

## Capabilities

### New Capabilities

- `repository-validation`: Defines reliable repository validation behavior when Moon executes required gates against the canonical Git default branch.

### Modified Capabilities

None.

## Impact

- Affects Moon workspace VCS configuration, repository validation tests, and CI verification evidence.
- Does not affect runtime code, APIs, dependencies, persisted data, migrations, or media behavior.
