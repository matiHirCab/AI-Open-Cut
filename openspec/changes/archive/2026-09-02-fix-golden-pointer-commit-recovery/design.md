## Context

The golden harness writes and syncs a temporary pointer, renames it over `CURRENT`, and then syncs the containing directory on Unix. The rename is the observable commit point, but the current return type collapses a later directory-sync failure into the same error as a failed rename. Publication then deletes the installed generation even though `CURRENT` already selects it. Separately, bounded cleanup is called only after a successful update, so recognized debris from an interrupted run survives ordinary conformance and report invocations.

## Goals / Non-Goals

**Goals:**

- Never remove a generation after `CURRENT` may have committed to it.
- Preserve both possible generations while pointer durability is uncertain.
- Reconcile recognized orphan data on the next golden invocation without changing selected evidence.
- Test real pre- and post-commit boundaries through injectable test-only fault points.

**Non-Goals:**

- No render, fixture, tolerance, performance-report, dependency, public API, protocol, or persisted-schema changes.
- No recapture of the current Linux generation.

## Decisions

### Return an explicit pointer commit state

Pointer replacement will return either `NotCommitted(error)` or `Committed { durability_pending }`. Unix marks the operation committed immediately after a successful same-directory rename; failure to sync the parent then sets `durability_pending`. On Windows or any other ambiguous replacement error, the harness rereads strict `CURRENT` bytes and treats the operation as committed when they select the intended generation. Publication removes a newly installed generation only after a confirmed `NotCommitted` result.

Alternative considered: treating every sync failure as fatal preserves the existing dangling-pointer bug. Retrying the rename cannot establish whether an earlier call already committed and can overwrite newer concurrent state.

### Retain both generations while durability is uncertain

A committed result with `durability_pending` skips post-commit inactive-generation cleanup and reports a non-fatal warning. On reopening, either the old or new pointer may be durable; both generations therefore remain available and reconciliation preserves whichever strict pointer selects.

Alternative considered: deleting the prior generation immediately assumes the pointer rename survived a crash, defeating the purpose of directory sync.

### Reconcile before every capture

The harness will resolve and fully validate the selected generation before rendering, then attempt bounded cleanup using that digest. Cleanup failures remain non-fatal and are reported. Update mode performs the same reconciliation when a pointer exists; an absent pointer permits first publication but only recognized stages and pointer temporaries can be cleaned until an active digest exists. A second cleanup remains after a durably committed publication.

Alternative considered: cleanup only on update does not satisfy next-invocation recovery and lets interrupted output accumulate indefinitely on verification-only hosts.

## Risks / Trade-offs

- [A directory-sync failure leaves extra generations] -> Preserve availability first and remove the inactive generation after the next validated reopen.
- [Ordinary conformance performs filesystem cleanup] -> Restrict deletion to UUID-named temporaries and fully validated inactive digest generations; never change `CURRENT`, the selected generation, or unknown paths.
- [Fault injection diverges from platform primitives] -> Place injection immediately around the actual rename boundary and retain platform-native smoke coverage through existing tests.

## Migration Plan

No data migration or recapture is required. Deploy the test-harness change with the existing generation. Rollback restores the prior harness code; the pointer and generation formats remain compatible.

## Open Questions

None.
