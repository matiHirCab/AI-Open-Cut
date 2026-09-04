## Context

`openspec validate --all` validates active changes but does not require them to be archived. The protected bootstrap can therefore attest a repository whose implementation is complete while accepted deltas remain outside the living specifications. The repository currently contains `harden-render-benchmark-migration-and-sampler-cleanup`, whose implementation and non-Linux checks are complete but whose recorded Linux parity verification is pending.

## Goals / Non-Goals

**Goals:**

- Fail the protected policy whenever any unarchived OpenSpec entry exists.
- Detect the state before Moon launches or the bootstrap can attest success.
- Preserve local authoring of active changes while making archive-only state mandatory for merge readiness.
- Verify and archive the existing completed implementation change without bypassing its pending checks.

**Non-Goals:**

- Prevent developers from creating active changes locally.
- Automatically archive incomplete or failing changes.
- Change application behavior, public interfaces, fixtures, goldens, or GitHub branch protection.

## Decisions

### Treat every non-archive entry as active

The validated inventory will require `openspec/changes` and its `archive` child to be ordinary directories. Every other file, directory, or symbolic link directly beneath `openspec/changes` will be reported as an unarchived change and will block policy validation. This avoids naming conventions or metadata that could hide incomplete state.

### Validate the inventory during bootstrap preflight

The active-entry inventory will join the existing policy sources consumed by `validateCiPolicy`. Validation will reject a nonempty inventory before `runMoon`, so the output writer remains unreachable. Unit tests will use explicit synthetic inventories; the real repository validation will become green only after all changes, including this follow-up, are archived.

### Archive the outstanding change only after Linux evidence

The existing renderer hardening change will retain its current requirements and implementation. Its Ubuntu native conformance and report validation will be run, remaining tasks and verification notes will be updated, and only then will it be synchronized and archived. Failure of that gate blocks archival rather than weakening the new rule.

## Risks / Trade-offs

- **The protected gate is intentionally red during active development.** Local focused validation remains available through unit tests and direct OpenSpec commands; the merge-ready bootstrap becomes green only after archival.
- **Parallel active changes block unrelated merges.** This is the requested repository-wide invariant; contributors must finish, archive, or remove abandoned changes before merge.
- **Archival cannot be used to hide incomplete work.** The outstanding change must satisfy its recorded Linux checks before being moved.

## Migration Plan

Implement and test the inventory rule with synthetic sources, complete and archive the existing renderer change, synchronize this delta, archive this follow-up, and then run the real bootstrap against the archive-only repository state. Rollback removes the inventory rule and restores documentation; no data migration is required.

## Open Questions

None.
