## Context

Main `431a5e38` contains #15 and #73 and has no active changes. Existing jobs use random UUIDs, `persistence: process`, immutable project/revision, a 1000-entry registry, one-hour terminal TTL and 32-file/64 MiB preview retention. Providers maintain bounded queues and expiring process-local tokens. Graceful cleanup refuses foreign/predecessor files; an abrupt exit loses accounting. HTTP shares one authorized runtime registry; separate processes remain private.

Measured before storage selection on 2026-10-11 Linux x86_64, Bun 1.4.0, Rust 1.97.0 debug: six fresh headless status calls took 1401.65, 1082.86, 1069.98, 1110.60, 1072.94, 1137.96 ms (median 1110.60 ms using upper-middle); four actual 160x90 empty-project frame-preview jobs completed in 294.94, 201.32, 188.84, 197.05 ms (first warm-up; measured median 197.05 ms). These include local readiness probes and are not release performance regressions. #15's reviewed Linux native fixture reports 204.28 ms frame, 470.84 ms range, 458.34 ms export end-to-end and 83,615,744-byte peak process-tree resident memory; stages overlap and must not be added. Historical #177/#179 reports describe long 720p30 exports and a roughly 94 s decoder failure on an unknown installed Windows commit; they show costly work exists but do not supply comparable controlled distributions or resumable checkpoints. No comparable production long-job distribution or third-party retry telemetry is available. Repository MCP helpers throw on errors and explicitly poll, cancel or retry revision conflicts; SDK reconnect does not restore a registry. Failure inspection found abort/deadline cleanup, stable failed jobs, protected commit phases and potential loss of the success acknowledgement after a durable commit.

## Goals / Non-Goals

Expose honest ephemeral semantics and bound additional accumulation across process crashes; preserve canonical revisions and isolation. No resumable storage, timeline/domain duplication, automatic side-effect replay, predecessor artifact adoption, host-power-loss guarantee or disk quota.

## Decisions

1. Keep jobs ephemeral. Measured short local render workloads and absent product demand do not justify durable private job payloads, media paths, result retention, migrations and exactly-once side-effect reconciliation. Durable project/history/drafts already provide the authoritative state. Reconsider on measured costly long-job workloads and an explicit recovery product requirement.
2. Add `ephemeral_jobs_v1` to bridge MCP status and CLI health only, backed by `contracts/job-restart-v1.json` and ownership/parity tests. MCP capabilityIdentifiers adds the same bridge-only identifier; exact predecessor rollback/digests preserve all older catalog evidence. No headless status, MCP structural schema or protocol/schema major change is needed because capabilities are open strings.
3. Before any job producer executes, acquire one exclusive private marker under configured TTS work root with mode 0600 and a random nonce. The marker is only an admission guard, not a registry; it contains no project IDs, paths, tokens, prompts or results. A stale/live/unsafe marker returns existing retryable JOB_REGISTRY_FULL before registry admission or task invocation. Read-only status and project reconciliation remain available. Never infer staleness from PIDs or automatically remove/adopt prior private files. Only the owning process can remove its unchanged marker after job and provider cleanup succeeds. Failed cleanup preserves the guard. Existing Kokoro close currently swallows file deletion failures; surface remaining owned debt with safe VALIDATION_FAILED and allow repeated closed-service/provider cleanup attempts without reopening admission. Track worker retirement and wait for confirmed exit before final owned cleanup, including cancelled generation that could write after its initial cancellation acknowledgement; force-kill after 1 s and fail closed after 2 s if exit remains unconfirmed. Doctor production follows the same guard. Doctor fixture deletion must not swallow failures and claim successful cleanup; reject safely so its guard remains for reconciliation.
4. Explicit manual crash reconciliation: stop all producers/descendants sharing roots, inspect durable project revisions/history and exports, quarantine or remove only verified disposable files, then remove the marker. Repeated bridge crashes while blocked create neither new producer outputs nor new guard files. Live producers remain subject to #73 bounds; exports/committed assets and interrupted native workspace sizes are not a global disk quota. Separate private clients use separate process/work roots; HTTP token sharing intentionally shares jobs.
5. Registry scheduling remains unchanged: admitted jobs synchronously transition to running; provider-queued work is a running registry job. Queued-state restart coverage kills the process at the existing job.admitted log boundary before producer dispatch; pending-producer coverage verifies the running registry envelope, with real-provider FIFO behavior separately covered by existing provider tests, without inventing a durable queue.

## Risks / Trade-offs

- Conservative marker requires operator recovery after crash, even after a producer had completed → report guidance and keep project reads available. No automatic resumability claims.
- Local filesystem writers are trusted → reject symlink root/marker and replacement identity; never recursively clean shared roots. Trusted parent aliases above root remain supported.
- No durable idempotency keys → client must reconcile state before retrying mutations/export; JOB_NOT_FOUND proves lost lookup, not failed commit.
- Tooling absent in saved environment → install pinned tools under workspace and use isolated native headers; report any blocked required checks rather than weaken gates.

## Migration Plan

No persisted project migration. Existing orphan files predate the guard and must be reconciled by operators on upgrade; no blanket startup deletion. Existing clients still parse jobs/errors and capability strings. Rollback retains the harmless guard file but removes enforcement, so operators must preserve recovery discipline. Reviewer is existing CODEOWNER @matiHirCab.

## Verification

Actual child process termination at registry queued/running/completed/failed/cancelled states; fresh registries/tools/resources reject predecessor IDs; blocked admissions do not invoke producers across repeated restart; distinct runtime guards are independent; symlink/occupied/replaced markers fail closed; close/cancellation/TTL and #73 tests pass. Capability parity plus MCP source and packaged smoke. Required Rust fmt/Clippy/workspace tests, bridge typecheck/lint/unit/integration/packaged smoke, contracts parity, hermetic Python, strict OpenSpec and pre/post archive protected policy. Record conformance before synchronization and archival. Exact-head CI after pushing final verified work.

## Open Questions

None in this issue scope. Production telemetry and future durable product requirements are explicitly deferred.
