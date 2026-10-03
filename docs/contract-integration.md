# Coordinating parallel contract changes

Canonical contracts remain manually synchronized under [ADR 0002](adr/0002-cross-language-contract-ownership.md). Parallel work should declare the owned sections before editing shared files. A clean textual merge still requires semantic parity review.

The current reconciliation uses main `5e6472a110bc15dc699d47252533bec337d5d16e`, which includes PR #135's motion pack and project schema 30. PR #138 originally branched from `8c6c7c936b1d6b5a14768b4dd9f8cd7b742973f7`; its ordinary main merge preserves those additions. Pending PRs are compatibility targets, not implicit dependencies to copy into this PR. The disposable speech/artifact experiment uses reconciled PR #137 head `584df7ed516274e32bc1acaaafa9691ea86aa9be` and #74 source head `e4e3e4499f8d9ee4e9123e3f2fb42e279b98d634` with the current-main reconciliation; published reconciliation heads and test outcomes belong in the PR/delivery evidence.

| Lane / section owner | Shared sections | Integration relationship |
| --- | --- | --- |
| #74 / PR #138 artifact delivery lane | `artifactResourceSchema`, `jobSchema.artifactResource`, `jobGetStatus.includeBinary`; MCP job descriptor/status input/resource template; `artifact_resources_v2`; ownership `artifactDelivery`; artifact response tests | Depends on current main; does not require #59 or #47 to ship |
| #59 speech provider lane | Provider capability metadata, `ttsStatusSchema`, corresponding MCP definitions/capability registration, provider ownership and parity assertions | Independently review the combined #59 + #74 surface on exact heads before choosing order |
| #46 / merged PR #135 motion pack lane | Preset compiler/schema 30, preset request/metadata definitions, `initial_motion_preset_pack_v1`, ownership `initialMotionPresetPack` | Already part of the reconciliation base; preserve every reviewed section |
| #47 / PR #136 animation lane | Animation core, project schema 31 and migrations, animation MCP definitions/ownership/parity | Owns persisted-core integration with #46; artifact/speech lanes do not resolve its domain semantics |
| Integrating lane | Shared capability arrays, derived expanded-catalog digest, combined contract assertions | Coordinate ordering and recompute from the semantic union; no lane owns another lane's entries |

Before publishing a branch or advancing a draft:

1. Record the base SHA, branch/head SHA, section owner and any actual depends-on PRs in the change approval/conformance record. Announce planned shared-section edits to the other lanes.
2. Fetch latest main and compare it with the recorded base. Reconcile main through an ordinary merge or agreed rebase; preserve each lane's declarations, fixtures, registrations and ownership entries. Inspect the diff against latest main for unintended domain changes.
3. Resolve schemas and registration semantics first. Combine capability identifiers and ownership categories without dropping another lane's entries. Keep canonical declarations manually scoped. Regenerate any derived union/digest only after this semantic merge, then check the actual registered MCP surface against the combined catalog.
4. For pending overlapping PRs, use a disposable worktree with exact heads to exercise the combined contract gate. Record textual conflicts, semantic mismatches, the digest and validation evidence. Keep those experiments out of the publishing branch unless an explicit dependency is approved.
5. Obtain independent specification and implementation review of the resolution, then run the affected parity, type/lint, behavioral and required protected checks. Record delegated approval separately from human CODEOWNER review.
6. Re-fetch main immediately before publish. If main advanced, repeat reconciliation and affected review/checks. Push normally, update the PR's exact head/evidence, and wait for terminal CI on that head. Recheck latest main and the reviewed diff before merge; merging requires separate authorization and required human review.

Merge order follows verified dependencies. For independent artifact and speech lanes, prefer the first reviewed, green, conflict-free PR; reconcile the second with the resulting main and deliberately recalculate its combined digest. A successful disposable combination check supports this order but does not replace exact-head CI or owner review. This checklist adds no CI or security configuration and cannot prevent all textual conflicts.
