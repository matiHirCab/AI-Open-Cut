## Context

`editor-core` currently holds an exclusive project lock but replaces `project.json` and `history.json` independently. The individual JSON writes are atomic, yet the pair is not: termination after the first replacement can leave the visible project at revision N while retained history still belongs to revision N-1. Draft commit adds a third phase by deleting the consumed draft only after both documents have been written, so cleanup failure is currently surfaced as a rejected call even though the revision was committed.

The solution must remain local, synchronous, dependency-free, and compatible with existing project and history JSON. It must also compose with migration, optimistic revision checks, garbage collection, and all callers that use `WriteResult.warnings` as an additive warning channel.

## Goals / Non-Goals

**Goals:**

- Make a project mutation and its retained undo/redo history one recoverable logical generation.
- Include committed-draft consumption in that transaction outcome.
- Establish an explicit durable commit point and unambiguous pre-commit versus post-commit errors.
- Recover under the project lock before any project state is returned or mutated.
- Detect corrupt or unsupported recovery data without rewriting authoritative files.
- Exercise every persistence phase through deterministic, test-only fault injection.
- Leave no managed journal or temporary JSON file after successful completion or recovery.

**Non-Goals:**

- Cross-project or asset-file transactions.
- A new project/history schema, a new lock mechanism, or a general database abstraction.
- Changing revision arithmetic, undo/redo retention, draft operation validation, or successful wire response shapes.
- Guaranteeing recovery from physical loss or corruption of every copy on the storage device.

## Decisions

### Use a versioned write-ahead transaction journal

Each mutation constructs the complete target `Project`, target `History`, and optional committed draft identifier in memory. While holding the existing project lock, the core atomically writes and synchronizes a versioned `.project-transaction.json` containing those values. Durable publication of that journal is the transaction commit point. The core then materializes `project.json`, materializes `history.json`, removes the consumed draft when present, and finally removes the journal.

The journal is authoritative after the commit point, regardless of which destination documents were already replaced. Replaying full target values makes recovery idempotent and removes any need to infer a generation from partially updated destination files.

Alternatives considered:

- A generation directory plus manifest pointer gives cheap selection but changes the persisted layout, complicates legacy migration, and requires garbage collection for old generations.
- A directory swap is not consistently atomic or replaceable across supported platforms and conflicts with the stable lock/assets directory.
- Adding generation fields to both documents still requires an authoritative recovery source when fields differ and would force a persisted schema migration.

### Define pre-commit rejection and post-commit recovery outcomes

Failure before the journal is durably published returns an error and leaves the prior generation authoritative. Once the journal is durable, the target generation is committed. Materialization or cleanup failure after that point must not be reported as mutation rejection: the caller receives the committed revision with `PERSISTENCE_RECOVERY_PENDING`; draft cleanup failure also adds `DRAFT_CLEANUP_FAILED`. The journal remains so the next locked access can finish recovery.

Every locked operation recovers before reading revision state. Consequently, a subsequent call in the same process and a reopen after termination observe the same committed target generation.

Alternatives considered:

- Returning an I/O error after journal publication preserves existing helper signatures but makes retry ambiguous and can cause callers to apply a mutation twice.
- Deleting the journal on any materialization failure loses the only authoritative description of the committed generation.

### Recover deterministically before migration or normal reads

Project loading under the lock first checks for the journal. A supported, valid journal is replayed in a fixed order: target project, target history, optional draft cleanup, journal cleanup. Replacements and cleanup are idempotent, so termination during recovery can safely restart the same sequence. Only after journal cleanup does normal schema migration or request handling continue.

The recovery loader validates the journal version and deserialization, project identity, supported project/history contents, and committed draft identifier before modifying destination files. Invalid or unsupported recovery state returns non-retryable `PROJECT_RECOVERY_FAILED` and preserves the journal and live documents for diagnosis. It never guesses or defaults history while a journal exists.

Alternatives considered:

- Choosing whichever destination has the larger revision cannot prove that its history belongs to that revision.
- Falling back to an empty history would silently discard undo/redo data and violate compatibility guarantees.

### Use one persistence path for ordinary mutations, creation, and migration

All logical writes of project plus history route through the journal protocol. Loading reads the pair together so migrations of current project state and retained snapshots are published as one transaction rather than as independent rewrites. Project creation uses the same protocol after directory setup; a pre-commit failure is rejected and best-effort cleanup removes unpublished managed files.

The existing JSON atomic-write helper will remove its own unique temporary file on failure and synchronize publication metadata as supported by the platform. Transaction cleanup similarly synchronizes deletion before reporting a fully materialized success.

Alternatives considered:

- Limiting the protocol to edit calls leaves project creation and migrations with the same split-publication defect.
- Keeping migration writes inside independent read helpers creates hidden persistence outside the transaction boundary.

### Make draft identifiers replay barriers

Draft commit records the consumed draft identifier in the journal. Recovery always materializes the committed generation before removing that draft. If cleanup is incomplete, the successful response contains stable warnings and the journal remains authoritative. A repeated commit can therefore either finish recovery and find the draft consumed or observe a revision conflict; it cannot reapply the operations as another revision.

No permanent idempotency receipt is added: this change guarantees that retries cannot duplicate the mutation and that acknowledged post-commit cleanup failures are reported as committed. Retrying an already completed draft commit continues to use the existing `DRAFT_NOT_FOUND` behavior.

Alternatives considered:

- Permanent commit receipts would change retention and privacy characteristics, require a pruning contract, and exceed the issue's compatibility scope.
- Removing the draft before publishing the journal risks losing an uncommitted proposal.

### Fault injection is test-only and phase-oriented

A test-only hook can fail before journal publication and after each durable phase: journal, project, history, draft cleanup, and final journal cleanup. Tests reopen through a fresh `EditorCore`, assert the project/history invariants and draft behavior, and inspect the project directory for managed leftovers. The hook is thread-local so parallel tests do not interfere.

Alternatives considered:

- Permission-based I/O failures are platform-dependent and cannot reliably target every phase.
- Process-kill integration tests remain useful smoke coverage but are too nondeterministic to replace unit-level phase injection.

## Risks / Trade-offs

- [The full journal duplicates project and history data and increases write amplification] → Keep one journal per project, remove it after completion, and prefer correctness over storage efficiency for the bounded history size.
- [A post-commit warning means destination files can temporarily lag the authoritative journal] → Recover before every locked read or mutation and document the journal as authoritative during that interval.
- [Platform filesystem APIs differ in rename and directory-sync guarantees] → Centralize atomic replacement/deletion, test on supported platforms, and never acknowledge a pre-commit write that failed synchronization.
- [Corrupt journal recovery can block an otherwise readable project] → Fail closed with `PROJECT_RECOVERY_FAILED`, preserve evidence, and document manual backup/diagnostic handling instead of risking mixed generations.
- [Existing uncommitted implementation work overlaps persistence code] → Preserve it, review it against this design, and make only scoped follow-up edits after approval.

## Migration Plan

1. Add the stable error and warning catalog entries with cross-language contract coverage.
2. Introduce journal parsing, validation, durable atomic-file helpers, recovery, and phase injection without changing project/history schemas.
3. Route project creation, all paired mutations, draft commit, undo/redo, and migrations through the common transaction path.
4. Add fault-injection and compatibility tests, then document the filesystem contract and recovery behavior.
5. Rollback is code-only when no journal exists. A build that may encounter a journal must retain recovery support; operators must open/recover projects with the new build before downgrading.

## Open Questions

None. The journal format is private persisted recovery metadata, versioned independently from the public project schema, and unsupported versions fail closed.
