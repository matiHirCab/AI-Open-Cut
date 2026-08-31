## Context

`editor-core` persists logical asset records in the current project and each undo/redo snapshot. Media items reference those records through `assetId`; transcription captions also retain a source `assetId`, and durable draft operations can introduce asset references when reopened or committed. The current deletion guard searches only current media items, while garbage collection derives managed-file roots only from asset records in current state and history. Reference discovery is therefore duplicated and incomplete.

The project lock already serializes project, history, draft, integrity, and garbage-collection operations. The solution must preserve that boundary, the existing revision model, content-addressed deduplication, history retention limit, draft durability, and the public error catalog without adding provider or presentation dependencies to the core.

## Goals / Non-Goals

**Goals:**

- Establish one `editor-core` ownership graph for all persisted logical asset references and managed-file roots.
- Prevent logical deletion when current media, current caption provenance, or any durable draft still references the asset.
- Retain managed content while it is reachable from current state, undo/redo history, or durable drafts.
- Validate current state, retained history, and drafts deterministically on load and before mutation.
- Preserve undo/redo and draft reopen behavior without changing valid persisted JSON shapes.

**Non-Goals:**

- Add force-delete, provenance detachment, or tombstone contracts.
- Change caption or generated-asset provenance schemas.
- Retain bytes after all current, history, and draft roots expire.
- Move ownership checks into headless, the bridge, provider workers, or desktop code.

## Decisions

### Central asset reference inventory in `editor-core`

Introduce a core-owned reference discovery abstraction that emits typed logical references from a project snapshot and draft operations. The initial inventory includes:

- media timeline item `assetId` references;
- caption source `assetId` references;
- asset records as owners of managed content paths, including imported and generated assets;
- `AddMedia` asset references retained in durable edit drafts;
- the same references and asset records inside undo/redo snapshots.

The representation will carry enough context to produce deterministic diagnostics, such as reference kind and owning item or draft identifier. Deletion, integrity validation, and garbage collection will consume this abstraction rather than maintaining independent timeline searches.

Alternative considered: add a caption-only check to `delete_asset`. This is smaller but leaves draft and garbage-collection behavior divergent and makes every future asset-bearing entity another ad hoc patch.

### Block live and draft references; retain history as a storage root

Current media references, current caption provenance, and retained draft references block logical asset deletion with `ASSET_IN_USE`. The message will identify the blocking reference class and will be stable enough for callers to act on. No reference is mutated or detached.

Undo/redo snapshots do not block logical deletion because deletion itself must remain undoable. Instead, their asset records and references remain reachability roots for managed bytes until history eviction. This preserves the current deletion semantics while making the distinction between logical deletion guards and physical retention explicit.

Alternative considered: detach caption provenance during deletion. That destroys inspection and regeneration information and requires a new persisted nullable/tombstone contract. Alternative considered: let drafts become stale and fail only on commit. That breaks durable reopen expectations and permits garbage collection to invalidate retained work.

### Validate each retained document against its own asset catalog

Project snapshots will be checked so every media and caption asset identifier resolves to an asset record in that snapshot. Durable draft references will be checked against the current retained asset catalog before the draft is returned, rebased, previewed, or committed. Loading or mutating legacy state with an unresolved reference fails with `ASSET_INTEGRITY_FAILED`; diagnostics will deterministically identify the reference class and identifier.

Validation runs under the project lock and after supported schema migration. No automatic repair is attempted, because guessing a replacement asset would corrupt provenance. Existing `ASSET_NOT_FOUND` behavior remains appropriate for a newly submitted operation that names an unknown asset; `ASSET_INTEGRITY_FAILED` distinguishes already-persisted corruption.

Alternative considered: warn and continue opening legacy projects. That would allow rendering, draft, and regeneration paths to observe inconsistent state and could let garbage collection make the corruption permanent.

### Garbage collection derives paths from all retained roots

Garbage collection will first inventory all retained project snapshots and durable drafts under the project lock, validate their logical references, then derive the set of managed relative paths owned by reachable asset records. Content-addressed files remain deduplicated: a file is removable only when no retained asset record reachable through current state, history, or drafts owns that path.

Draft enumeration failures or invalid retained references prevent unsafe collection and surface an actionable warning or integrity error according to whether project validity is affected. Cleanup failure remains an additive `ASSET_GC_FAILED` warning after the project transaction commits.

Alternative considered: copy referenced content into each draft. That duplicates storage, changes the draft schema, and creates a second asset lifecycle.

### Preserve public and persisted contracts

No persisted field is added or removed, so the project schema and draft version need not increment. Error codes remain `ASSET_IN_USE`, `ASSET_INTEGRITY_FAILED`, and `ASSET_GC_FAILED`; only previously unguarded invalid cases begin returning them. Headless and bridge layers continue forwarding core results without duplicating ownership rules.

This is a fail-closed compatibility change for corrupted legacy data, not a breaking change for valid projects. Contract fixtures require updates only if they assert exact actionable messages or reference classifications.

## Risks / Trade-offs

- [Draft enumeration adds I/O to persistence cleanup] → Keep the history and draft limits bounded, inventory once per operation, and reuse the result for validation and collection.
- [A corrupted unrelated draft could block collection or project work] → Return deterministic diagnostics naming the draft and reference, and allow explicit draft discard to remain the recovery path when its identifier is valid.
- [Future asset-bearing entities could bypass the graph] → Locate the discovery API in `editor-core`, make deletion and garbage collection depend exclusively on it, and add exhaustive tests alongside every new asset reference variant.
- [Exact error messages can accidentally become brittle contracts] → Keep stable error codes normative; use concise reference-kind details for actionability and assert only intentional message fragments.

## Migration Plan

1. Add reference discovery and validation without changing serialized shapes.
2. Validate current project and retained undo/redo snapshots after existing schema migration, with fixtures for valid caption provenance and legacy dangling references.
3. Include durable drafts in deletion guards and garbage-collection roots, with reopen and discard recovery tests.
4. Roll out through the existing core/headless/bridge binaries. No data rewrite is required for valid projects.

Rollback is code-only because no schema is written. Projects saved by the new implementation remain readable by the previous version, although rolling back would reintroduce the dangling-reference risk.

## Security, Privacy, and Failure Modes

Reference discovery operates only on project-managed metadata and paths already constrained by the path policy. It does not expose provider paths or media contents. All validation and collection remain under the project lock. A failure before commit leaves project, history, and drafts unchanged; a file-removal failure after commit is reported through `ASSET_GC_FAILED` without rolling back valid metadata.

## Open Questions

None. Force deletion, explicit detachment, and tombstones require separate approved contracts if later needed.
