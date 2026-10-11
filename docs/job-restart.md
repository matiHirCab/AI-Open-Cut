# Job restart and reconciliation

Bridge MCP `editor_get_status` and CLI `--health` advertise `ephemeral_jobs_v1`. Its canonical behavior is [contracts/job-restart-v1.json](../contracts/job-restart-v1.json); [ADR 0005](adr/0005-ephemeral-job-restart-semantics.md) records the measurement-based decision.

Jobs are private to a bridge runtime. Their random UUIDs are lookup handles, not persistent identity or idempotency keys. `projectId` and `revision` identify the originating request and never track later edits. Results can include a newer committed revision. Metadata says `persistence: process`; progress, cancellation, result metadata, artifact links and generated commit tokens do not survive process exit. A provider FIFO waiter has registry status `running`: the registry's initial `queued` state transitions synchronously at admission.

All queued-provider, running, completed, failed and cancelled work disappears from a cold registry. Status, cancellation and job resources for old IDs return `JOB_NOT_FOUND`. Retained tokens cannot be adopted by supplying old IDs or paths. Jobs expire one hour after terminal state by default; polling does not refresh TTL. Capacity is bounded by `OPENCUT_JOB_MAX_COUNT` (default 1000); unsettled cancelled producers remain charged. Preview retention is 32 files/64 MiB and can evict before TTL. Cancellation is immediate/idempotent while safe, with retryable `JOB_CANCELLED`; an atomic commit phase returns `JOB_NOT_CANCELLABLE`. See [artifact delivery](artifact-delivery.md) for file access and retention.

Separate private clients use separate bridge processes and TTS work roots. An authorized shared HTTP bridge intentionally shares its jobs; the active project is not an ACL and possessing the same bearer token is shared runtime access. Existing host/origin/body/authentication checks remain in force. Restart never adopts predecessor output into a new registry.

## Client recovery

1. Reconnect, inspect the capability, open/read the project and obtain its current revision. Drop predecessor job IDs, links and tokens; do not poll them indefinitely.
2. For a read-only preview/analysis, submit a new request against the refreshed revision after runtime recovery. A fresh job has a new UUID; cold render caches and optional provider model loading may add startup cost.
3. For an uncertain mutation, inspect authoritative state/history before replanning. Lost acknowledgement and `JOB_NOT_FOUND` do **not** prove that an atomic commit failed. Check the intended items/assets and provenance; do not blindly resubmit generation-and-insert, regeneration or a draft commit.
4. For an uncertain final export, inspect the approved output separately. A final published export is preserved. Never infer overwrite approval from retryability, and request approval again when the retry would overwrite without existing authorization.
5. Re-read after `REVISION_CONFLICT`, then replan against the new revision. Process-local preview/generated tokens from a predecessor cannot avoid resynthesis. Retryable `JOB_REGISTRY_FULL` after restart may require operator recovery; exponential retries cannot repair a crash marker.

No automatic resumption, side-effect replay or exactly-once guarantee is provided. Durable project/history/drafts retain their own existing recovery semantics.

## Operator crash recovery

Before the first job producer (or doctor synthesis), the bridge exclusively creates `.opencut-job-runtime-v1` in `OPENCUT_TTS_WORK_DIR`. It contains only an opaque nonce. If another live runtime or a predecessor owns that marker, new jobs fail with safe retryable `JOB_REGISTRY_FULL` before production. Status and ordinary project reads remain available. A successful graceful shutdown settles producers, cleans owned preview/provider outputs and removes only its own unchanged marker. A failed cleanup or unconfirmed provider termination preserves the guard and reports a safe shutdown failure. Owned provider cleanup can be retried while admission stays closed. Do not run doctor while another runtime owns this root.

An unsuccessful doctor report caused by unavailable subsystems or optional models retains its unsuccessful exit code, but releases its own guard after diagnostics return and all runtime cleanup succeeds. A thrown doctor cleanup failure or failed service shutdown retains the guard for reconciliation. Diagnostic readiness alone is not evidence of a crash or leftover output.

After an abrupt exit:

1. Stop all bridge, headless/render, FFmpeg and provider descendants sharing the configured work/project roots. Verify no owner remains; a PID or file age alone is insufficient.
2. Reconcile authoritative project state/history, pending project transactions/drafts and approved exports. Preserve committed media, exports, project/history/drafts and unclassified/foreign files.
3. Inspect/quarantine or remove **only** positively identified disposable preview, native temporary workspace and generated provider output owned by the stopped runtime. Use existing confined cleanup procedures and trusted local filesystem tooling. Never sweep an entire project, exports root or another client's work directory.
4. Only after that reconciliation, remove the admission marker from this verified work root. Start a fresh runtime and submit fresh jobs against current revisions.

Initial upgrade requires reconciliation of any orphan files created by versions that did not write the marker. Changing work roots to evade a guard without cleaning prior owned files bypasses the bound.

While blocked, repeated crashes/restarts add no producer outputs and no new marker files. The guard is an admission safety record, not a durable job registry, automatic crash cleaner, disk quota or host-power-loss journal. Existing process-local caps, cleanup debt and individual interrupted workspace sizes retain their documented limits. Recovery is deliberately conservative; unavailable filesystem cleanup requires repair before admitting more producers.
