## Why

#215 requires truthful production inventory after reconnecting, while current process-local job IDs/tokens intentionally disappear. A lost acknowledgement cannot establish whether publication happened; stable bounded durable intent and known terminal metadata must remain separate from execution handles.

## What Changes

- Add an explicitly enabled bridge-owned durable request/outcome registry with stable caller request identity, captured project/revision and typed bounded intent. Initial executable intents reuse existing frame/range/review rendering and final export workflows; supported kinds are explicit. Existing speech/audition behavior and unpublished #208 are excluded.
- Persist queued admission before any producer, require explicit execution, record running ownership before dispatch and known terminal metadata after completion. Restart preserves queued and terminal records and reports interrupted running work as uncertain, without replay, old token adoption or claiming failure from a lost acknowledgement.
- Add governed opt-in MCP admission/status/list/start/queued-cancel surfaces and a bridge capability, without changing old ephemeral job envelopes or default behavior. Existing core validates project/revision/rendering/export semantics and retains domain ownership.
- Bound record count/bytes/age, fixed crash staging/ownership debris and retained metadata. Fail closed before effects when storage is unavailable; preserve private-root isolation, foreign files and existing authorization.
- Measure original representative multi-output rendering workloads and document trade-offs, retention, operator recovery and limits. Independently verify storage fault injection, process crashes, compatibility and genuine source/packaged/native workflows.

## Capabilities

### New Capabilities

- `durable-production-requests`: opt-in bounded bridge-owned request intent and known outcome metadata, distinct from ephemeral execution/artifact handles.

### Modified Capabilities

None. Existing `job-restart-semantics` continues to govern all process-local jobs and private runtime admission. Update ADR0005 with this separately opted-in metadata registry; do not alter its no-replay/no-adoption guarantees.

## Impact

Bridge-only registry/application orchestration and thin MCP adapters; canonical versioned durable request fixture plus additive MCP/ownership governance and independent historical projections; runtime wiring/health and explicit opt-in configuration; ADR/documentation and tests. Existing native/headless/project/provider wire schemas and persisted project/history remain unchanged. Compatibility is additive; disabled clients retain current behavior and job/error semantics. No new dependencies or security policy changes are planned. Shared central integration files can conflict textually with future #208; preserve semantic unions without accessing/reconstructing its WIP.

## Non-goals

No automatic replay/resume, checkpointed encoder/inference, exactly-once or host power-loss guarantee, credential persistence, legacy job/token adoption, arbitrary tool replay, cloud storage, change to project ownership/GC or native renderer semantics, Windows embedded playback or #214. A durable row does not prove resumability or failure of an unacknowledged effect. The complete artifacts and measurements received independent approval before executable edits; implementation verification and closure are recorded separately.

## Planning detail

[Approved exact contract and failure boundaries](contract-design.md) records the tools, normalized intents, state/catalog projection, bounded retention/readback, safe initial storage and confirmed native-error provenance. The complete artifact set received independent implementation-scope approval recorded in approval.md; syntax validation alone was not treated as approval. Implementation is present; final required conformance/closure checks are recorded in verification.md.
