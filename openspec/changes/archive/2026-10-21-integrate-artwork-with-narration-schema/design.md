## Context

Verified live refs: #229 source matiHirCab/AI-Open-Cut:feat/issue-220-artwork-derivatives at aeea97a749132cb9acddb455af7262ffb56d546f; main at 8cb21a6bab2f5a0134ab6af3f91a018f0ae9bde8. PR is open, draft=false despite historical body wording. Checkout was clean. No local #208 WIP is present or accessed. GitHub connector reads succeeded; gh REST/GraphQL returned Forbidden.

Read root AGENTS.md, docs/spec-driven-development.md, original artwork approval/proposal/design/tasks/deltas, and main's narration migration/design and channel-package boundary. Dry `git merge-tree --write-tree HEAD origin/main` found eleven conflicted paths, including catalogs, store, projections and project-persistence spec. Automatic merges also need semantic review: both branches set PROJECT_SCHEMA_VERSION=47 and MCP schema constants to47, so an apparently clean merge can be wrong.

## Goals / Non-Goals

Preserve the approved artwork raster/transaction workflow and all main narration, package and contract behavior through one reviewed integration. Scope exclusions in proposal.md are binding; Windows disposal diagnostics and deletion behavior must remain untouched. No inference/model/privacy/publishing expansion.

## Decisions

### Schema48 after main's schema47

Keep main's meaning of47: narrationManifests must be a real array at47 and later, and any explicit introduced field on pre47 (including null/empty) remains rejected. Artwork derivation first becomes legal at48. Old supported schema1..46 migrates under existing rules with empty narration and no fabricated derivation; genuine main47 migrates to48 retaining manifests and all native bindings exactly. Validate each current/undo/redo snapshot before publishing any generation. Existing lock, staged edit, resource ledger, journal, revision and typed-error owners remain authoritative.

Reject artwork metadata on schema<=47, including unpublished predecessor artwork47 documents. Do not recognize a second schema47 dialect, fabricate a narration array for a malformed main47 document, strip metadata or rewrite before validating. This intentionally does not provide automatic migration for unpublished #229 runtime data. Source backups are the rollback path; no silent downgrade. Alternative reusing47 would change an existing version's contract; permissive dialect guessing would weaken main's malformed-document rejection. Any need to support predecessor branch data requires a separate reviewed import format.

Update current project schema markers in Rust/TS, canonical current-schema fixtures and operator docs to48. Preserve historical narration introduction guards at47, narration fixtures representing47, package componentSchemaVersion46, historical pin files and all older introduction guards. Blind global replacement is forbidden. Keep both Project narration fields and Asset artwork fields in every initializer, with None only where no artwork provenance exists.

### Compose exact inverses around a main47 predecessor

Use an explicit integration-owned inverse layer for current48 -> exact main47 where appropriate: remove only artwork-owned operations, metadata/ownership and schema48 marker changes, preserving narration and package additions. Preserve the original artwork predecessor hashes and original narration predecessor hashes unchanged. A separately explicit projection of peer additions can recover original artwork-branch targets, while removing artwork before narration can recover main's pre-narration targets. Each path must state its starting inventory and order; validate exact expected field/fragment multiplicity before inversion and compare complete raw bytes/hashes afterward. Add integration pins derived from the independently fetched main47 artifacts, not from candidate output. Existing historical inverses stay strict at their original input boundaries. Never pass schema48 directly to an inverse expecting47 and relax its guard to make it work.

Canonical headless list is the union with original order constraints and exactly one copy of every old/new operation. MCP retains both required narrationManifests and optional artworkDerivation, unchanged tool count/aliases/annotations except the approved current project schema marker. Expanded MCP digest changes must have exact additive predecessor proof; do not recapture old digests or omit assertions. Ownership/CODEOWNERS retain both feature inventories. Renderer release fixtures only change actual current project markers, not independent format versions.

### Preserve package46 and explicit artwork rejection

Main's v1 channel package componentSchemaVersion46 is independent of current project48. Ordinary legacy images/components keep existing capture/import/apply behavior; native package validation initializes new optional Asset provenance to None. A selected derived artwork asset cannot be encoded by v1 without losing transitive source identity/bytes. Detect that condition in the existing core capture/validation owner and reject VALIDATION_FAILED before library/project/resource publication; do not silently flatten, remove provenance or bump/rewrite package46. Test source/derivative bytes, history/revision and package-library contents unchanged after rejection. An unrelated derivative elsewhere in the project must not block an ordinary selected recipe. This guard is already required by the original artwork approval's package integration addendum; no v2 package is introduced.

### Verification and lifecycle

Add combined migration/history/future/forged metadata and native artwork+narration preservation regressions plus exact inverse positive/negative cases before considering integration complete. Keep original raster, CLI, revision-conflict and cancellation scenarios. Run all required unchanged checks; a prearchive policy rejection is acceptable only for this active change and is never recorded as success. Do not claim unrun native/platform/GUI evidence. Existing Windows failures remain blockers without diagnosis or a test/workflow change.

After conformance and all required implementation evidence pass, synchronize only this delta and archive through required skills, then rerun protected gate/strict validation. Refresh live remote refs before commit/push; concurrent source changes require preservation/reintegration rather than overwrite. Push a verified merge descendant of both original feature head and main, without force; retain PR draft=false. Observe exact published-head CI and report every failed/skipped/pending required check. No merge/deploy/issue closure.

## Risks / Trade-offs

- Current-schema ripple through fixtures -> inventory actual markers separately from immutable historical introductions and package formats; preserve negative tests.
- Unsupported draft schema47 artwork files -> explicit typed rejection and retained backup; no guessed automatic repair.
- Native asset construction added on main -> compile and scenario tests retain narration and explicit optional provenance initialization.
- Silent provenance loss in package capture -> selected-resource rejection before mutation and normal package regression coverage.
- Host lacks Rust/Bun/Moon initially -> install exact pinned tools into /tmp only; report unavailable required evidence. Isolated Bun1.4.0 is now available for specification validation. No original caches/data need removal.

## Migration Plan

Approve these artifacts, merge fetched main without committing, reconcile semantic and textual conflicts through tasks, and validate. Current/history adopts48 under existing locked recoverable migration; schema48 old-build rollback requires original full generation backups. Original archived approvals remain historical evidence; this change's synchronized requirement supersedes the artwork schema47 allocation.

## Open Questions

Resolved: the parent with delegated specification-approval authority explicitly approved schema48 allocation and nonmutating rejection of development-branch artwork47 documents before implementation; see approval.md. No open implementation decision is deferred.
