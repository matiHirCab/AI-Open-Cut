## Context

The golden harness stores immutable generations below one shared fixture container and performs bounded reconciliation even during ordinary conformance. Without process-wide coordination, one invocation can classify another invocation's live stage or inactive-but-needed generation as residue. Separately, generation durability currently synchronizes only each retained file's immediate parent, so nested references can leave unsynchronized ancestor directory entries before `CURRENT` commits them.

The implementation remains test-only and must preserve fixture bytes, the selected digest, public contracts, and performance report schema 2.

## Goals / Non-Goals

**Goals:**

- Serialize every native golden invocation over the complete lifetime of shared fixture access.
- Keep coordination stable across process lifetimes and release ownership through RAII.
- Synchronize every accepted directory ancestor through the generation root in deepest-first order.
- Distinguish confirmed installation ownership from an ambiguous installation result.
- Verify behavior across processes, failures, panics, nested references, and overlapping same-digest publication.

**Non-Goals:**

- Changing renderer output, reference media, digests, tolerances, schemas, or public APIs.
- Introducing a new dependency or a nonblocking lock mode.
- Generalizing the lock beyond the test-only golden fixture container.

## Decisions

### Persistent exclusive coordination file

Add a small RAII guard backed by `fs2::FileExt`. It opens a stable `.golden.lock` in the fixture container with read/write, create, and no truncation, then waits with `lock_exclusive`. `Drop` unlocks the file. The file is never unlinked and is outside all recognized stage, pointer-temporary, and generation cleanup patterns.

The native entry point validates FFmpeg, FFprobe, and the configured font before acquiring the lock. It then acquires the lock before reading `CURRENT` or reconciling and retains the guard through selection, rendering, comparison, staging, publication, reporting, and final cleanup. Ordinary conformance is protected as well as update mode because it both reconciles and relies on its selected generation remaining present.

### Complete ancestor collection

For every retained file, walk from its parent upward until the validated generation root is reached. Reject the set if any walk fails to reach that exact root. Deduplicate all ancestors, include the root, and order them by descending component count. Unix durability synchronization opens and synchronizes that deepest-first sequence before installation; the existing synchronization of `generations/` and the fixture container after rename remains unchanged.

### Confirmed installation ownership

Generation installation returns an explicit internal outcome that distinguishes a confirmed successful install from an error or ambiguous platform result. The caller may attempt best-effort removal only for a generation newly created by this invocation after installation reported success and a later pre-commit durability step failed. If installation itself reports failure, observing a valid destination digest does not establish ownership: the pointer remains unchanged and the recognized destination is retained for the next invocation's locked reconciliation. Preexisting digest generations remain protected.

### Cross-process tests

Reuse the test-binary subprocess pattern. An ignored helper announces startup, attempts the same lock, reconciles, and records acquisition. The parent holds the lock with a live stage, verifies the child remains blocked without deleting it, then releases the guard and verifies the child acquires and removes only the now-orphaned stage. Additional cases overlap same-digest publications and prove lock release after both a returned pre-commit error and a caught panic. Bounded marker polling avoids depending only on fixed sleeps.

## Risks / Trade-offs

- A hung invocation blocks later golden work. This is intentional serialization; operating-system lock release still covers process termination.
- Holding the lock through rendering reduces parallelism. Correctness requires protecting generations used by ordinary conformance, so the critical section deliberately spans the full invocation.
- Persistent lock files add one fixture-container entry. Keeping the inode stable avoids unlink races and recognized cleanup ignores the file.
- Ambiguous install failures can retain a complete inactive generation temporarily. Locked reconciliation bounds this residue without risking deletion of another owner's data.

## Migration Plan

No data migration or recapture is required. The first invocation creates the persistent coordination file. Existing `CURRENT`, generations, fixture media, and schema 2 reports remain valid. Rollback can remove the locking and ancestor traversal code; leaving the coordination file is harmless because it is an unknown preserved path.

## Open Questions

None.
