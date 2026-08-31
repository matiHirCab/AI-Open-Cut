# Project persistence and recovery

OpenCut treats the current project, retained undo/redo history, and consumption of a committed draft as one logical transaction. This document describes the operational filesystem contract implemented by `crates/editor-core`.

## Lock and transaction boundary

Every project read, mutation, migration, and recovery runs while holding the project's exclusive `project.lock`. Callers must use the editor core rather than reading or writing the JSON files concurrently.

For each logical write, the core constructs the complete target `project.json`, target `history.json`, and optional committed draft identifier in memory. It then performs these phases in order:

1. Atomically write and synchronize `.project-transaction.json`.
2. Atomically replace `project.json` with the target project.
3. Atomically replace `history.json` with the matching target history.
4. Remove the consumed draft when the transaction commits a draft.
5. Remove `.project-transaction.json`.

The durable publication of `.project-transaction.json` is the commit point. Before that point, a failure rejects the mutation and the previous project/history pair remains authoritative. After that point, the journal is authoritative even if one or both destination documents still contain the previous generation.

Each JSON replacement uses a unique same-directory temporary file, synchronizes file contents, renames it atomically, and removes the temporary file after failure. Successful completion or recovery leaves no journal or managed temporary transaction file.

## Recovery

On every locked project access, the core checks for `.project-transaction.json` before reading revision state or running migrations. A valid journal is replayed in the same fixed order. Every phase is idempotent, so termination during recovery can be recovered again on the next open without duplicating a mutation.

Recovery validates the journal version, project identity, schema versions, retained snapshot identities, and optional draft identifier before modifying live documents. Unsupported, corrupt, or inconsistent recovery metadata fails closed with non-retryable `PROJECT_RECOVERY_FAILED`. The core preserves the journal and live documents for diagnosis; it does not select a document by revision, default the history, or guess which generation should win.

Do not manually delete a journal after `PROJECT_RECOVERY_FAILED`. Stop processes using the project and copy the complete project directory before diagnosis or repair. A backup must contain the entire directory, including hidden files, `project.json`, `history.json`, `project.lock`, drafts, and assets.

## Mutation results and warnings

A mutation returns an error only when it fails before the journal commit point. Once the journal is durable, the result reports the committed revision even if materialization has not completed:

- `PERSISTENCE_RECOVERY_PENDING` means a later locked access must finish journal replay.
- `DRAFT_CLEANUP_FAILED` appears with `PERSISTENCE_RECOVERY_PENDING` when project and history publication committed but the consumed draft still needs removal.

These warnings describe a committed mutation, not a rejection. Clients must advance to the returned revision and must not reapply the operation. Retrying the same draft identifier cannot create another revision: recovery consumes the recorded draft before the retry is evaluated, after which existing `DRAFT_NOT_FOUND` behavior applies.

## Migrations and compatibility

The core loads project and history as one pair. Supported migrations update the current project and every retained undo/redo snapshot in memory, then publish the migrated pair through the same transaction protocol. Unknown future project schemas and unsupported journal versions fail closed.

The journal format is private recovery metadata with its own version. Before downgrading to a build without journal support, open every project with the current build and confirm no `.project-transaction.json` remains. If a journal exists, let the current build recover it; do not downgrade first.

## Durability limits

The protocol prevents mixed logical generations caused by process termination or ordinary I/O failure. It relies on same-filesystem atomic rename and the synchronization guarantees exposed by the operating system and filesystem. It cannot recover from total storage loss, filesystem or hardware behavior that violates acknowledged synchronization, simultaneous out-of-band file modification, or corruption of both authoritative recovery metadata and the required target data. Use whole-directory backups for protection from those failures.
