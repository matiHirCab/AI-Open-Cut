## Context

Current main b3173e7f preserves ephemeral_jobs_v1, private runtime guards and core-owned atomic rendering/export. #215 has no hard prerequisite, comments or competing PR. Initial supported durable producers are frame preview, audiovisual/silent review range and final export. The independent reviewer confirmed that an explicitly documented initial render scope can satisfy #215 without speech recovery, provided real producer integration and all durability/uncertainty/bounds criteria are delivered. #208 speech/audition implementation and its unauthorized device/WIP remain excluded. This design and the detailed contract/oracles were independently approved before executable edits; see approval.md. Implementation is present; final required verification and closure status is recorded in verification.md.

## Original workload observation before implementation

2026-10-19 Linux x64, Bun1.4.0/Rust1.97.0 debug native SHA256491cfa6845568ec224352c901452d028bdf972d500748089d78acabeb69217cb; core inputs on fa5 unchanged in main b317. Original320x180/10fps/2000ms solid #123456 project, captured revision1, native serial calls: frame0ms, frame1000ms, silent review0..2000ms, distinct final export (overwrite=false). Three batches produced twelve real outputs and left authoritative project state exactly unchanged. Batch totals3607.55ms(warmup),3121.09ms,3157.50ms. Per-output times respectively [467.92,434.41,1453.54,1251.56], [273.30,253.43,1329.42,1264.89], [249.27,250.57,1498.45,1159.16]ms; output sizes634/634/2179/4010bytes per batch. Safe scratch driver cleaned only its original owned project/exports root.

Full evidence/driver remain /tmp/opencut-215-second-native-measurement.log and /tmp/opencut-215-measure.ts. An initial measurement omitted the existing required allowed-media-root setting and failed before workload creation; it was corrected in scratch configuration only, without changing product validation. Concurrent unchanged Rust reference-scene tests were active; load averages1.29/1.36/1.80. These are small debug workloads, not production long-job distribution, isolated benchmarks, storage latency proof or a claim that12 outputs justify resumable execution. They support a cheap bounded metadata ledger for requested inventory; execution remains ephemeral and no resume/exactly-once claim follows.

### Expanded original audiovisual multi-output observation

A second preimplementation run completed on2026-10-19: Linux x64/Bun1.4.0/Rust1.97.0, debug native SHA256`dfd3bce7b7e43f5be9f36a8511395a8f5377310dbb5d009d91151d6e85be8ac7`. Original640x360/24fps/8000ms solid #123456, timed semiopaque rectangle and original660Hz/48kHz mono PCM; captured revision3. Three serial batches each rendered frames0/4000ms, an audio-enabled full-range review and a distinct final MP4 export with overwrite=false. All twelve outputs succeeded; final authoritative project state equalled its captured pre-render state, and the scratch driver cleaned only its owned root. Batch totals were97590.71ms(warmup),90890.92ms and87800.06ms. Per-output durations were[1081.93,1441.64,44730.13,50336.00], [931.95,704.91,44521.10,44730.92], [640.01,713.12,43198.45,43247.48]ms; each batch produced1511/2205/139870/140395bytes. Initial load averages0.01/0.02/0.08, final1.27/0.76/0.38; no deliberate concurrent local tests, remoteCI on separate runners. Full driver/evidence remain `/tmp/opencut-215-expanded-measure.ts` and `/tmp/opencut-215-expanded-native-measurement.log`.

This is a larger original audiovisual fixture and shows useful multi-output inventory at a longer observed cost. It is still one small authored scene and three debug samples, with warmup and shared-host conditions; it is not production telemetry, a regression comparison, release throughput, recovery latency or evidence of resumable encoding. The expanded observation uses the same debug SHA256 captured in the final #218 native acceptance evidence; the earlier small measurement used a different captured debug hash. Preserve those exact distinctions rather than treating all observations as one identical binary. The explicit #215 inventory requirement justifies bounded opt-in metadata; these timings do not justify replay/checkpoints/exactly-once promises.

## Goals / Non-Goals

Opt-in durable request intent and known outcome metadata connected to existing real producers, stable identity and truthful reconnect inventory, with bounded private storage and no automatic dispatch. Keep legacy defaults/jobs/tokens unchanged. No execution replay, resumable encoder, exactly-once, host-power-loss durability, credential storage, provider changes or project persistence behavior.

## Decisions

### Separate application metadata from execution

Bridge owns a new DurableProductionRequests application service and private filesystem store. Existing JobRegistry and native core remain the execution/publication authorities. New thin MCP tools admit/get/list/start/cancel-queued intent. A dedicated runtime option, disabled by default, enables the registry below the configured existing private TTS work root. No arbitrary caller path/owner is accepted. Capability durable_requests_v1 is reported with enabled/readiness information; old ephemeral capability remains. Authentication, network access and security policy are untouched.

Supported intent discriminants are preview_frame, preview_range, review_range and export_video. Their request payloads reuse the existing public bridge validators/mapping for preview_render_frame, preview_review_range and project_export_video, including explicit overwrite and captured project/expectedRevision. If extracting a shared mapping is needed, preserve existing behavior and test all old handlers; do not introduce parallel project/render validation. Native core validates source state/revision/dimensions/publication. Unsupported producers fail explicitly and are documented; no generic arbitrary tool replay is allowed.

Admission is a durable queue operation, not a dispatch. A stable caller UUID requestId plus normalized validated intent is the identity. Same identity and identical canonical intent returns its current record; different intent fails VALIDATION_FAILED. start(requestId) claims only a queued request, persists running ownership before dispatch, then invokes one existing JobRegistry producer. A process-local map may return the current job to the starting live caller, but persisted records never include its opaque job ID, private resources or tokens. Start retries/concurrent starts cannot create a second producer. No reopen/reconnect/retry dispatches automatically.

### Truthful state machine and known outcomes

Persist states queued, running, completed, failed and cancelled. A running record contains its originating random runtime identity. Inventory from another runtime exposes it as uncertain, preserving its original captured request. Missing metadata is not failed work. queued can be explicitly cancelled without execution. A live execution remains subject to existing cancellation/noncancellable publication rules; this initial registry does not invent resumable cancellation. Existing job_cancel remains usable with the newly returned live job ID.

Wrap the actual producer via existing startTask semantics to capture its real outcome. Persist completed only after successful native result; retain a bounded safe summary without private artifact links/tokens. Persist failed only from a confirmed producer failure; persist cancelled only once cancellation/termination outcome is confirmed. A start/admission exception can be called failed only when dispatch is confirmed not to have begun. An exception after dispatch or unconfirmed terminal persistence produces uncertainty, even if a live ephemeral job separately knows its result. Do not turn terminal metadata failure into a false failed effect or falsely claim durable completion.

Atomic visibility is the process-crash commitment boundary; no host-power-loss claim. A write whose acknowledgement is lost may be recognized committed only by safe exact readback of the intended complete catalog, including storeId/generation and owned identities; otherwise uncertainty is mandatory. Never reexecute to repair metadata. Pending terminal persistence keeps a safe in-memory uncertain overlay and ownership debt; close may retry metadata only, and failed cleanup retains the guard. Exact fault points and readback rules must be independently specified/tested before approval.

### Private bounded on-disk format and ownership

One versioned closed JSON catalog and one fixed staging pathname, under a fixed private subdirectory of the existing configured private work root. Reuse existing conservative nonce guard mechanics in that subdirectory; no PID/age takeover. Guard is acquired before the first write and released only after owned execution and metadata cleanup succeed. A predecessor guard keeps write/start admission closed until documented operator reconciliation. Safe atomic-catalog reads remain possible after crash and do not adopt/delete predecessor artifacts.

Planned limits:64 records;8KiB each;1MiB total serialized catalog including header;4KiB normalized intent;7 days from admission retention. Reserve bounded terminal metadata headroom at admission. Each write checks both record/catalog capacity before producer invocation. One fixed staging file plus catalog plus guard bounds crash metadata debris. A crash guard prevents repeated owners adding staging files. Unknown/foreign files are preserved and block unsafe writes rather than swept. Exported files, project media and native interrupted workspaces are outside this metadata quota.

All timestamps are checked nonnegative safe integers. Retention uses captured admission age, never refreshes on polling. Expiration removes metadata only and cannot cancel or prove failure of effects. Never evict a still-owned active producer to admit more work; fail capacity instead. Expired uncertain/terminal metadata may be forgotten with an explicit retention warning; absence must never authorize replay. Queued expiration is not automatic dispatch. Tests must define clock rollback/future timestamp behavior conservatively.

The fixed format records schema version1, storage owner identity, request identity, canonical intent hash/full supported intent, project/revision, creation/update timestamps, original execution runtime and state, plus bounded terminal summary/error code/retryability. No credentials, raw provider diagnostics, private output tokens, base64 or project documents. Final export's already-approved relative output path is part of explicit persisted intent; document that consent. Future versions and malformed records fail closed without rewriting them. Exact keys, independent fixture examples and transitions were approved before implementation; see contract-design.md, planning-oracles.json and approval.md.

Filesystem operations validate configured roots/ordinary files, exclusive creation and unchanged ownership, preserve foreign/symlink/replaced files, publish an entire catalog atomically, and leave safe bounded debris on failure. Reuse existing safe filesystem/guard primitives where applicable. The single writer/serialized application queue prevents cross-client duplicate starts. Separate private configured roots are isolated; intentionally shared authorized HTTP runtime access remains shared as already documented. Trusted local filesystem writers remain outside application authentication; no new ACL/security configuration is introduced.

### Compatibility and governance

One new independently authored durable request fixture and owned category; additive MCP schemas/definitions/capability with actual required parity consumers and all unchanged predecessor pins/negative controls. Capture current raw catalogs before changes. Invert only exact reviewed additions in outer projection boundaries before historical proofs. Do not derive independent expected fixtures from live validators, rewrite old pins or weaken assertions. Keep native/headless/project/provider contracts and ephemeral job envelopes unchanged. Update ADR0005 to distinguish retained ephemeral execution decision from opt-in durable metadata.

## Risks / Trade-offs

- Lost terminal acknowledgement: exact safe readback can identify an already committed complete record; otherwise report uncertain and reconcile authoritative state/export, never execute again.
- Conservative old guards reduce availability: inventory remains readable but start/write requires explicit existing operator recovery.
- Metadata retention can forget uncertain effects: document expiration, and missing/expired never means safe retry.
- Renderer may finish after client cancellation/disconnect: persist only confirmed outcome and preserve core publication/cancellation semantics.
- Private shared root is not a per-project ACL: preserve existing isolation boundaries and advertise intentional sharing truthfully.
- Small debug fixtures do not characterize production long jobs: measure original representative multi-output work, report environment/load/sample limitations and avoid performance guarantees.
- Shared central-file integration can conflict textually with #208: document semantic union without reading/reconstructing WIP; keep all speech behavior unchanged.

## Verification

Independent closed contract/state/bound examples, real storage fault injection before/after admission/claim/terminal publication, exact duplicate/concurrent identity tests, real abrupt subprocess exits/reopens, count/bytes/age/debris/foreign isolation, native producer starts and terminal outcomes. Genuine MCP and packaged workflows must prove the service is wired to actual rendering/export, not just a storage helper. Measure multiple original outputs with exact tools/build/workload context before approving the implementation design. Every normative scenario needs automated coverage.

Required Rust format/strict workspace Clippy/workspace tests, TS type/lint/unit/contracts/source MCP/packaged smoke, relevant hermetic Python, OpenSpec protected prearchive/conformance/sync/archive/final protected and strict checks, then exact-head required CI. Reuse only unchanged relevant evidence with explicit inputs/toolchain/environment justification. No merge/deployment.

## Migration Plan

Additive disabled-by-default opt-in. New catalog version1 rejects unknown future versions. No project migration or token adoption. Disabling stops new durable use without deleting registry data or modifying committed exports/projects. Re-enable only against the same positively owned safe root; interrupted guard recovery remains explicit.

## Open Questions

The proposed exact inputs/outputs, canonical catalog, readback/retention/initialization rules and internal confirmed-core-error boundary are elaborated in [contract-design.md](contract-design.md). Those decisions, independent JSON transition/fault examples, expanded original workload observations, producer/transport/retention mapping and scenario traceability received independent artifact approval before executable edits; see approval.md. No unresolved design question remains at this checkpoint; any uncovered requirement ambiguity must be resolved/reviewed before expanding implementation.
