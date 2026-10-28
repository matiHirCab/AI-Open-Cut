## Context

The existing artwork branch has a clean working tree at d70120792511f50f99c363206620211787999638. Main is b350b5cc85b4a9d31578992fde83f598e4392c68. A read-only `git merge-tree --write-tree` found conflicts in narration-profiles-projection.ts, short-candidates-projection.ts and smoke.test.ts. Automatic catalog merges still need semantic verification. Main's durable spec says it MUST NOT introduce artwork/schema48, so the old narration-only integration approval is insufficient.

## Goals / Non-Goals

**Goals:** one reviewed integration on PR229, preserving both approved runtime features, exact historical contract evidence, current48/history/main47 migration and portable package46. Keep bridge orchestration outside core and core validation/persistence authoritative.

**Non-Goals:** all exclusions in proposal.md apply. In particular no Windows disposal/descendant work, changed producer/storage semantics, dependency upgrades, workflow or timeout changes, merges or deployments.

## Decisions

1. After explicit artifact approval, merge freshly verified main into the existing feature branch with no automatic merge commit. Resolve conflicts minimally, retaining both feature assertions and additive smoke inventory. A separate duplicate feature PR or choosing an entire side would lose review continuity or evidence.
2. Keep schema48 and durable catalog1. Durable intent contains typed render requests and captured revisions, not a competing project dialect; retaining schema47 would discard approved artwork provenance. Do not add a new schema or migration. Existing atomic main47-to48 and future49 rejection remain mandatory.
3. Before consumers, independently capture both exact input heads' affected canonical files, raw SHA256 and compact/expanded representations. Add a new integration witness rather than replacing any existing fixture/pin. Compose guarded inverses so removal of durable-only additions recovers exact PR229 inputs, and removal of artwork integration recovers exact main inputs, preserving the other peer. Both orderings must recover their independently frozen shared predecessor after explicit ownership-version normalization. Never use live registrations to rewrite canonical catalogs or replace raw comparisons with parse/reserialize equality.
4. Reconcile predecessor wrappers for narration profiles, short planning, artwork history and durable-main47. Each path must retain exact-field/multiplicity checks, preserve unrelated bytes and reject missing, narrowed, duplicate or unrelated changes. New negative controls exercise both canonical peers. Existing native headless/schema markers, mandatory consumer lists and dual native-error witnesses remain enforced.
5. Add a focused actual producer acceptance using synthetic managed artwork and narration in schema48: durable admission/start plus successful render, reopen/history, stale revision and restart terminal inventory. Assert native project state and original source hashes independently. No automatic restart dispatch, job/token adoption or altered uncertainty settlement. Reuse existing source and packaged producers, registry and test-owned roots. No new external asset, model, service or public surface.
6. Update only current operator compatibility guidance and the successor living requirements during synchronization. Preserve every archived approval and historical ledger verbatim. Main47 describes accepted legacy input and frozen predecessor evidence; current project48 does not change portable package46 or durable catalog1.

## Risks / Trade-offs

- Textually clean catalogs can have incompatible inverse order or required consumers → independently freeze both input heads, test both recovery paths and negative controls before aggregate parity.
- Main47-only durable witnesses can wrongly strip artwork or fail at schema48 → introduce explicit integration adapters at the reviewed boundary, preserving original fixtures and strict guards.
- Existing Windows startup fixture failure or other required gate failure can block completion → retain original failures, leave the change active, report exact CI/step/SHA; never change tests/timeouts or archive to clear the gate.
- Linux cannot establish Windows behavior → unchanged exact-published-head platform CI is required and remains separate from local evidence.

## Migration and rollback

No new persisted format or migration is introduced. Preserve the approved project48 migration and rejection of development artwork47 dialects, future schemas and invalid retained snapshots before publication. Reverting a source commit cannot downgrade an already written schema48 project; older binaries continue to reject it. Durable records remain catalog1 and retain captured intent/outcome semantics. No data rewrite, registry relocation, deletion or lock bypass is authorized.

## Verification and approval boundary

Run tasks.md checks and map every scenario to automated evidence. Existing CI observations are evidence of the old heads only. Any mandatory unavailable/failed check blocks completion and archival. Exact-head platform CI follows authorized publication with pending limits; it is never counted passed in advance. Genuine CODEOWNER acceptance remains separate. If a required suite is prohibited by a standing execution restriction, report it as blocked rather than modifying or bypassing the suite. Implementation is pending explicit review of this complete artifact set.
