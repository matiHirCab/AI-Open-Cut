# ADR 0005: Ephemeral jobs and conservative crash admission

## Status

Accepted for issue #85 under the user's issue-scoped specification authorization; CODEOWNER review required on the draft PR.

## Context and measurements

The merged #15 renderer observations and #73 bounded lifecycle precede this decision. On inspected main `431a5e38`, registry entries carry random UUIDs, `persistence: process`, immutable project/revision and bounded terminal retention. Providers and preview disposal have their own bounded ownership. Restart loses those maps; a project commit or export can survive the loss of its acknowledgement.

The checked-in #15 native fixture (`flat-scene-av-v1`, revision 4, schema 3/stage-definition 2, Linux x86_64 FFmpeg 7.1.5, fixture Git 67a35d05) reports median end-to-end frame 204.28 ms, audiovisual range 470.84 ms, export 458.34 ms, and peak process-tree resident memory 83,615,744 bytes. Stage timings overlap and cannot be summed. This is a one-second fixture, not long-job production telemetry.

Before implementation on 2026-10-11, this isolated Linux x86_64 checkout with Bun 1.4.0/Rust 1.97.0 debug executed six fresh headless status requests: 1401.65, 1082.86, 1069.98, 1110.60, 1072.94, 1137.96 ms (upper-middle median 1110.60 ms). They include native readiness probes. Four real frame-preview jobs on a 160×90 empty project completed in 294.94, 201.32, 188.84, 197.05 ms: one warm-up and three measured samples, median 197.05 ms. Historical reports #177/#179 describe long 720p30 production exports and a roughly 94 s decoder failure on an installed Windows build with unknown commit. They demonstrate costly work can exist, but are not a controlled distribution or evidence of resumable checkpoints. No comparable production long-job distribution, full optional-provider cold-load timings or third-party client retry telemetry is available. These debug observations must not be compared as regressions against release/platform baselines.

Repository MCP helpers inspect errors and poll explicitly; revision conflicts are retried with refreshed state, while SDK reconnection does not reconstruct job state. Existing failure paths include deadlines/abort, sanitized failures, cancelled unsettled capacity, failed disposal debt, and noncancellable atomic commit. Inspection found that repeated abrupt exits could leave predecessor files outside current-process accounting.

## Decision

Keep jobs ephemeral. Advertise `ephemeral_jobs_v1` in bridge MCP status and CLI health, governed by `contracts/job-restart-v1.json`. Do not add a durable job database, checkpoint inference/rendering, rehydrate private artifacts/tokens or automatically replay side effects. Durable project/history/drafts remain authoritative. The measured inexpensive sample workloads and absence of a product requirement for resumption do not justify durable private job payload storage, schema migration, ownership ACLs and exactly-once reconciliation.

A minimal exclusive admission marker under the configured TTS work root prevents cumulative producer admission after a process crash. Acquire it before the first job producer; it contains only a random nonce and is created mode 0600. An occupied, replaced or unsafe marker fails with existing retryable `JOB_REGISTRY_FULL`. Status and project reads remain available. The owning process removes only its unchanged marker after successful graceful producer/provider cleanup. Doctor synthesis uses the same protection. No PID-based takeover, directory sweep or private output adoption is allowed.

After a crash, operator reconciliation verifies termination of all runtime descendants and checks durable project/export effects before quarantining/removing verified disposable files and finally removing the marker. Without that reconciliation, repeated bridge restarts cannot produce more orphan output and leave one guard file. This deliberately trades availability for bounded additional accumulation and private ownership safety.

## Alternatives

- Durable job registry/checkpoints: deferred until production telemetry demonstrates costly lost work and product requirements define resumable provider/renderer and side-effect ownership semantics. A persisted status row alone cannot resume native execution or prove a commit did not occur.
- Automatic deletion or PID/age takeover: rejected because another runtime/descendant may still own those artifacts, PIDs can be reused, and age does not prove safe ownership.
- Unrestricted ephemeral restart: rejected because repeated crashes could accumulate private files despite bounded process-local retention.

## Compatibility, privacy and limitations

The capability is additive. Protocol 1, project schema, request/response/error envelopes and headless capability catalogs stay unchanged; no project migration is required. Restart makes every predecessor status/cancel/resource lookup return `JOB_NOT_FOUND`; the marker is not a job registry. Existing HTTP callers sharing an authorized runtime retain shared job access. Private clients require separate bridge processes and work roots. No new private path, media text, prompt or token appears in the marker, discovery or safe errors.

In-process #73 limits still apply (1000 default jobs, one-hour terminal TTL, 32 preview files/64 MiB). Exports, committed assets, interrupted workspace sizes, generated-media providers and disposal debt are not a global disk quota. The guard bounds new admission across process crashes rather than guaranteeing automatic reclamation or host-power-loss durability. Files left by older versions require an initial operator reconciliation. Trusted local filesystem writers are outside the application boundary; symlink roots/markers and replacement identities are rejected, and no recursive shared-root cleanup is added.

See [job restart and reconciliation](../job-restart.md) for clients and operators. Revisit this ADR only with measured long-job cost and an explicit durable recovery product requirement.
