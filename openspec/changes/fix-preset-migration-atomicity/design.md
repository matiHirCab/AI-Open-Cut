## Context

The eager loader publishes migrated current/history before edit validation. The existing animation-presets transactional requirement already forbids such publication on rejection. Asset migration also copies managed files, and fonts/drafts require preparation before an edit can be certified.

## Goals / Non-Goals

Restore existing preset rejection guarantees across legacy current/components/retained history and ordered alias batches. Preserve schema 29, all canonical typed errors, successful edit revisions and undo semantics, and existing journal recovery. Do not add presets, schemas, transport validation, or a storage backend.

## Decisions

Use an internal prepared-project result owned by store, carrying migrated documents, staged font bytes and draft upgrades. Verified asset migration copies are staged speculatively under the lock at the existing loader point, with a rollback ledger owned by the entire preset request. Assets owns prepare/publish for asset migration. Normal reads/non-preset edits retain eager loading. Preset-containing standalone/batch edits prepare under the same lock, run existing operation owners exactly once, prepare fonts and inherited/extended safety for the complete candidate, push one undo snapshot and bump once, then publish staged fonts and one existing transaction containing current/history/draft upgrades. Roll back new asset files on every subsequent preparation, operation or pre-journal error.

Do not use an unsupported-version-only guard: it misses collisions, aliases and later candidate failures. Do not perform a second speculative application: generated IDs and complete candidate preflight must be authoritative. Do not implement an overlay storage backend, partial font-path aliases, or duplicate transport checks. Speculative asset staging preserves the complete existing font resolver behavior: explicit/default paths, family discovery, enumeration bounds, canonicalization and fallback priority. A reproduced future extensionless asset selector must retain DEPENDENCY_UNAVAILABLE rather than being accepted with fallback.

Before publication inspect destination entry kinds without following symlinks, reject invalid preexisting entries, and track only newly created managed asset/font paths; on a pre-journal failure remove only those files, preserving preexisting bytes. Never remove published resources once the existing journal commits; return existing recovery-pending warnings and recover on reopen. Retain current font-publication checkpoints. Asset publication verifies planned hash/size against source at publication. A source race or I/O error fails before journal; journal ownership remains unchanged. Recovery of an already committed transaction before a request remains required, distinct from publishing this rejected request.

## Risks / Trade-offs

- Migration assets must exist for source-font resolution during certification: speculative verified copies and request-wide rollback preserve resolution parity. Asset destination integrity failures retain loader-before-operation priority for malformed persisted resources; unsupported versions on valid resources remain INVALID_ARGUMENT. Verify explicit/default selectors and later retained/draft failures.
- Cleanup after failed storage operations can itself fail: preserve the primary typed error and report existing cleanup diagnostics where supported; fault tests must establish document safety and no removal of preexisting resources.
- Draft upgrades must publish alongside accepted edits, never before rejected edits; test both versions and retained bases.
- One combined migration/edit transaction changes internal write count, not the approved observable contract. Verify fault checkpoints and redo clearing.

## Migration Plan

No new schema or format. Valid supported legacy state migrates deterministically only when the preset transaction is accepted, or when existing read/non-preset paths load it. Unknown future/malformed documents fail before publication. Undo/redo snapshots retain canonical migrated schema. Reverting this correction does not change accepted documents' readability.

## Open Questions

No fresh public contract approval is intended. Independent reviewer approval of this exact restoration scope is required before implementation; any expanded semantics or new dependency edge must return for approval.

Independent reviewer (gpt-6-sol, medium) explicitly accepted this amended internal staging design before implementation on 2026-10-02. No public/schema/error meaning is added.
