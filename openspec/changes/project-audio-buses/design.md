## Context

Verified predecessor is issue62 d8168ad6631d834891df9303485e1d8c1b9b1116 (CI37706235074 all11 success). The new branch starts exactly there. PR157 targets main and is unmerged; this issue's future draft must also target main and explicitly include predecessor scope/merge order. Epic8 requires bus-routed audio; issues66/67 separately own DSP and explicit side-chain rendering. Current schema38 has audioRole/ducking and no buses. Project deserialization already buffers structural model envelopes, migrations clone current/history, and prepared store transactions provide crash-consistent publication and resource rollback.

## Goals / Non-Goals

Goals: four fixed buses, bounded route DAG, explicit scoped track routing with role fallback, deterministic schema39 migration, thin additive addressable edits, complete failure/history/native compatibility and governed predecessor proofs. Non-goals follow proposal.md: no custom bus lifecycle, DSP, event registration, metering, normalization, unrelated refactors or changes to rendered output.

## Decisions

### Closed model, stable defaults and bounded graph

Project audioBuses is a Vec of closed records in exact voiceover/music/sfx/master order. Each has id and required nullable outputBusId. All stem outputs exist, master output is null, paths are unique and terminate at master in <=4 nodes. Fixed identities avoid custom creation/deletion/missing-owner complexity while allowing meaningful typed routing for later DSP. Alternatives: role strings alone cannot express explicit routing; an arbitrary submix graph unnecessarily expands issue65. No numerical DSP fields are added here.

Track audioBusId is optional nullable, omitted on serialization when None. Root and component Audio/Video tracks can route; Overlay/Caption reject explicit routes. This permits video media with audio without loosening existing role rules. Role fallback maps SoundEffects to sfx and Unassigned to master. Bus routing does not mutate role or current ducking. A single canonical validation/resolution helper under existing model/validation owners checks records and computes routes; transport layers only parse/forward. Do not add a new top-level editor-core owner or dependency edge; nested siblings inherit ADR0003 boundaries.

### Existing transaction and alias ownership

Add AudioBusSetRoute {bus_id,output_bus_id} and AudioTrackRoute {scope,track_id,bus_id: Option<String>} to the existing edit enum and strict remote definitions. Nullable track bus_id must be supplied on requests so omission cannot silently clear a route. Bus outputs are supplied as String; master route edits are rejected. Both return existing WriteResult; fixed buses are not single-ID creators. Track/component aliases reuse existing resolvers. Each ordered route edit validates its resulting graph; temporary cycles are rejected, so callers reroute in safe order. Locks protect track-route edits; project bus changes are project-level edits, not item mutations.

Read-only inspection found ordinary load_project_data can publish legacy adoption before applying an edit. Route edits and route-bearing batches/drafts therefore MUST enter the existing prepared/staged migration path, not the ordinary loader. Extend its existing edit predicate to recognize the two route variants throughout standalone/batch/create/update/preview/commit draft selection. Reuse one prepared generation, font/resource rollback and journal publication; do not introduce a parallel persistence path. Test failed routing against genuine legacy bytes explicitly, including late batch failure and draft conflict.

### Version-aware migration and complete validation

Project gains audio_buses, serialized only for schema>=39 with an updated exact field count. ProjectDocument presence-aware decoding requires complete non-null buses in39 and rejects presence below39 including null. Buffered actual root/component track envelopes reject audioBusId presence below39 before defaults; dynamically named user/slot values are not recursively scanned. Closed records retain duplicate/unknown-member rejection. Typed pre39 Project values with nonempty buses or explicit track routes are also rejected by migration. Genuine legacy decode has an empty internal bus vector until migration initializes defaults; serialization of old typed Project values omits introduced project fields. Every valid current39 candidate and retained39 snapshot validates the bus model, including empty/hidden/muted tracks.

Migrations initialize exactly four defaults for every supported old generation, preserving absence of explicit routes and every other field. Expand the existing supported range through38; unknown future remains InternalError. Migration preparation validates every generation before publication and uses the existing lock/journal. Existing fixtures that construct genuine old versions by lowering a current JSON schema marker must remove only the newly introduced project field (and actual track routing, if present), preserving each original scenario; add separate premature-field negatives instead of relaxing guards. Keep source-matched draft/recovery and old font/provenance/resource assertions.

### Preserve render evidence and historical contracts

Routing in65 changes persisted/effective routing metadata only. Current evaluated audio roles, ducking, normalized filter graphs and audio/visual output remain exact. No output-affecting evaluator/filter change is authorized. A core helper and project state expose the route for future66. Native fixtures compare genuine pre39/default/rerouted state with existing independent RGB/PCM/plan/timing oracles; all required native workflows remain included.

Capture exact issue62 raw headless/MCP/ownership files, expanded MCP digest cb0cb43b3817113887c265cd2ebd98d58c56420527f547677fcdfe367c73397b, and all43 raw canonical catalog hashes before implementation. Add audio-buses-v1, category/CODEOWNER coverage and project_audio_buses_v1 capability. Manually synchronize schema reporting39 and two uniquely named tools plus exact draft unions. Precise projections assert expected live39 before restoring predecessor38 reporting and remove independently captured exact additions. Every older proof remains layered below issue62 and retains unrelated-drift negatives. Frozen historical catalogs/counts are unchanged; current runtime consumers independently assert new reporting/tool additions while preserving exact old counts and media assertions. This includes native-only hero and desktop consumers that previously caught missed additions.

Public requests are additive protocol1; persistence is versioned schema39 with explicit migration. An older reader correctly rejects future39 rather than guessing. No major request protocol change or provider alteration is required.

## Risks / Trade-offs

- Legacy typed/JSON fixture constructors may unintentionally fabricate premature fields -> preserve original fixtures as genuine source versions and add explicit negative fixtures.
- Invalid retained routes can be hidden in inactive content -> full current/history/component validation before every publication and migration.
- Route edits can accidentally trigger early migration -> reuse prepared staging for all route-bearing standalone/batch/draft paths and assert byte equality on failure.
- Frozen native/catalog counts can be overlooked -> governed complete consumer inventory, exact predecessor projection and required all-head CI, including native-only render workflows.
- Shared local disk and large migration suites can exhaust resources -> preserve failures and test data, reclaim only inactive compiler cache with receipts if necessary; standard CI remains acceptance authority, and diagnostic runtime adjustments never replace it.
- Restricted built-ins do not support user submix creation -> intentional issue65 boundary; no incomplete custom lifecycle is advertised.

## Migration Plan

No deployment is performed. Compatible local builds adopt supported persisted generations on locked open or on a successfully staged route mutation. A failed precommit request leaves the old generation intact; committed interruption uses established recovery warning/journal ownership. Downgrading a schema39 file is unsupported and older builds fail closed. Verification precedes specification synchronization/archive; final protected gate and all11 exact-head CI precede completion/next issue.

## Open Questions

None requiring user input: the user explicitly delegated issue-scoped specification approval and same-agent separate substantive CODEOWNER review. Review is transparently not an independent human or GitHub APPROVED action and does not bypass merge protections.
