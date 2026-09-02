## Context

The golden harness validates a staged generation, renames it under `generations/<digest>`, and immediately replaces `CURRENT`. It synchronizes the pointer temporary and, on Unix, the pointer's parent after commit, but it does not synchronize the retained generation files or the directory entries that make the installed digest reachable. A system crash can therefore preserve `CURRENT` while losing all or part of its selected generation.

The harness is compiled only for tests. Linux CI and Windows development must retain their current behavior, the checked-in Linux digest must remain unchanged, and no dependency or public contract may be added.

## Goals / Non-Goals

**Goals:**

- Establish a strict ordering: validate, persist generation content, durably install the digest directory, then commit `CURRENT`.
- Keep every generation-durability error on the uncommitted side of the pointer boundary.
- Preserve preexisting generations and safely reconcile only newly created recognizable orphans.
- Exercise content and directory durability boundaries through deterministic fault injection.

**Non-Goals:**

- Recapturing media or changing manifest, pointer, performance, or project schemas.
- Changing post-pointer-commit durability classification or cleanup semantics.
- Providing a general production filesystem transaction abstraction.

## Decisions

### Synchronize the validated generation as a tree before selection

The harness will derive the retained files from the validated manifest, add `manifest.json`, and call `sync_all` on every file. On Unix it will then synchronize unique directories deepest-first through the generation root. This keeps the durability set identical to the strict manifest rather than following unknown filesystem entries.

Alternative considered: changing each `fs::write` to a synced writer spreads transaction policy across capture serialization and still omits directory-entry durability.

### Install with platform-specific durable primitives

On Unix, the already synchronized stage will be renamed to its digest, followed by synchronization of `generations/` and the fixture container before pointer preparation. On Windows, retained files will be synchronized and a new digest directory will be installed with `MoveFileExW` plus `MOVEFILE_WRITE_THROUGH`, reusing the existing development dependency. Unsupported targets will fail before pointer commit rather than claim durability they cannot establish.

Alternative considered: retaining `fs::rename` on every platform cannot express the requested Windows write-through guarantee.

### Treat every generation-durability failure as uncommitted

Content-sync and directory-install/sync fault points will return an error before `atomic_replace_pointer`. If this invocation installed the digest, it will remove it with best effort; failure leaves a strictly recognized inactive generation for startup reconciliation. A digest that existed before the invocation is never deleted. `CURRENT` is not reread because the pointer operation has not started.

Alternative considered: exposing generation durability as another pending warning would permit a committed pointer to select data that may not survive reopening.

### Preserve the established pointer commit model

Once generation durability succeeds, the existing pointer temporary, atomic replacement, ambiguous-result reread, post-commit durability warning, and two-generation retention behavior remain unchanged.

Alternative considered: combining generation and pointer outcomes into one state machine would enlarge the patch without changing the existing correct pointer boundary.

## Risks / Trade-offs

- [Some filesystems or devices provide weaker guarantees than requested synchronization] -> Fail on reported synchronization errors and document that guarantees are bounded by the platform filesystem.
- [A best-effort rollback can leave an inactive digest] -> Keep the pointer unchanged and let strict next-invocation reconciliation remove only the recognized orphan.
- [More synchronous I/O slows explicit updates] -> Accept the cost because updates and recaptures are deliberate test-maintenance operations, not ordinary rendering.
- [Windows directory moves may fail after partially changing state] -> Validate the destination, never commit the pointer after a reported install failure, and remove only a destination created for this invocation.

## Migration Plan

No data migration or fixture recapture is required. Existing selected generations are revalidated and resynchronized only when an explicit update attempts to reuse their digest. Rollback consists of reverting the harness and documentation; fixture bytes remain compatible.

## Open Questions

None.
