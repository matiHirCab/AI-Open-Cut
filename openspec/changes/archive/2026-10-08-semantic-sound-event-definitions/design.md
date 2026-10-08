## Context

Verified issue65 head321dc3583df2d95380efb613153b0135bb32268d passes all11 CI37721029887 plus Windows startup37721029884. Its tested mergeb0b5486d6b042efa380af329b341a9454f973b6f has source-identical tree2a477d7ed4224bbad45741b1eed55cde464fb3cb. This branch starts there. Main remains e22c3436; PR157/158 are draft/unmerged and both target main. Issue63 dependencies11/17 are closed. The roadmap explicitly specifies a project-level sound library; the fixture-only motion-graphics catalog supplies event/variantAssetIds/defaultGainDb/busId vocabulary, gain bounds and safe seeds. Schema39 has content-addressed managed assets and four built-in buses but no sound library.

## Goals / Non-Goals

Goals: bounded closed named definitions, deterministic core selection, atomic standalone/batch/draft registration with existing aliases, schema40 migration, centralized asset references and complete independent compatibility/native evidence. Non-goals: timeline sound items/marker placement/render activation (#64), DSP/side-chain (#66/#67), analysis/normalization/inspection (#68–70), registry deletion/custom bus lifecycle, provider changes, new dependencies/owners or backend expressions.

## Decisions

### Project registry and pure bounded model

Project gains soundDefinitions: Vec<SoundEventDefinition>. Every closed record requires event, variantAssetIds, defaultGainDb, busId and variantSeed. Event uses the existing foundation ASCII identifier grammar [A-Za-z][A-Za-z0-9_-]{0,127}; names are unique case-sensitive identities. Maximum512 definitions and32 variants per definition bound scanning and retained history. Variants preserve declared order, contain at least one existing asset and reject duplicate asset IDs or duplicate content hashes. Gain is finite within inclusive[-120,24]dB; seed is an unsigned integer within[0,9007199254740991]. Bus must resolve through the four persisted built-in IDs.

Use existing asset IDs rather than duplicate content records or paths. Variant references use bounded ASCII [A-Za-z0-9][A-Za-z0-9_-]{0,127}, which includes canonical UUIDs; batch aliases resolve before this check. Separators, URL syntax and oversized identity input are invalid, rather than resource locations. Each resolves through the containing snapshot's canonical asset catalog to audio or audio-bearing video with a canonical lowercase64-digit sha256 hash and positive recorded size; actual path confinement/bytes/integrity remain in the existing assets owner. No transport accepts a path/URL for registration. A model sibling owns closed DTOs, pure numeric/identity/reference invariants and deterministic selector; validation adapts it and migrations call a model method. This preserves ADR0003's migrations→none/model→error matrix, including recursive sibling checks. Do not repeat issue65's forbidden migration→validation edge.

The pure selector takes event and an optional explicit seed, defaults to the saved seed, checks safe bounds and selects ordered index seed % variant count using integer arithmetic. It resolves to the existing selected asset metadata plus definition gain/bus. Missing definition uses nonretryable INVALID_ARGUMENT. Missing variant for a mutation uses ASSET_NOT_FOUND; persisted/draft dangling roots use existing ASSET_INTEGRITY_FAILED with ownership classification. Other invalid model/reference/type/hash/gain/seed/bus conditions use INVALID_ARGUMENT. This avoids new public error identifiers and duplicated I/O policy.

### Named registration and existing transaction/alias semantics

Add SoundEventRegister {event,variant_asset_ids,default_gain_db,bus_id,variant_seed}. All fields are required remotely; no implicit defaults clear or reinterpret input. Register creates a new name at the end or replaces the same name at its stable index; replacement does not grow the registry. Both return existing WriteResult with exactly that event identity. Support existing resultAlias by classifying registration as a single-ID producer; aliases name the returned semantic identity rather than allocating a second UUID. Ordered later event/asset references reuse resolve_alias, and unresolved/missing/forward aliases retain existing VALIDATION_FAILED behavior; resolved literal identities retain the existing domain error classification. Literal event input retains the bounded grammar.

Standalone/batch/draft registration must use the existing requires_composition_staging prepared path through creation/update/rebase/preview/commit, because the ordinary loader can publish migration before a rejected edit. Stage all legacy current/history/assets/fonts once, validate candidate operations in order and publish one journaled generation only on success. Preserve retryable REVISION_CONFLICT before mutation/reference errors, atomic whole-batch rollback, one revision/history entry, drafts and exact undo/redo/reopen. No parallel persistence mechanism or transport domain validation is introduced.

### Centralized sound asset roots

Extend assets::project_asset_references with every registered variant under a sound-definition reference class, including unused definitions. Extend draft_asset_references with each pending registration variant under the existing draft-operation class. Reuse blocking_asset_reference, retained snapshot validation, managed path collection and actual integrity verification. Current and pending-draft references block deletion with ASSET_IN_USE; replacing definitions releases only current roots, while history still protects old assets/bytes until existing eviction. All previous media/caption/slot/draft roots and error classifications remain intact. Do not introduce a second hash/path collector or let registry IDs masquerade as file paths.

### Version-aware empty-registry adoption

Schema40 serialization adds exactly one field and only emits soundDefinitions at>=40. Presence-aware ProjectDocument decoding rejects actual top-level presence below40 including null, requires a present nonnull array in40, and leaves dynamically named slot/user values untouched. Typed pre40 projects reject nonempty registries. Validate every retained40 registry before durable publication. For supported schemas1..39, initialize only an empty registry and advance to40 while preserving all previous fields, introduced bus rules, resources, IDs/revisions/timestamps and old migration guards. Validate unknown future schema through the established failure path without rewriting. Invalid current/history/draft references and failed legacy registration cannot publish defaults, media copies or partial documents. Expand every original fault phase with direct/draft registration/migration cases; retain every previous case.

### Governed manual contracts and unchanged rendering

Capture all44 prechange canonical catalog hashes, exact verified issue65 MCP/headless/ownership raw files and expanded MCP semantic digest before executable edits. Add independent semantic-sound-events-v1, category/CODEOWNER coverage, semantic_sound_event_definitions_v1 capability, schema40 reporting, one uniquely named MCP tool and exact typed standalone/batch/draft declarations. Advance only the seven existing active reporting headers39→40, preserving every other byte; leave frozen audio-buses/speech/foundation feature catalogs unchanged. Precise predecessor projections remove only independently captured additions and restore validated40→39 reporting. Layer all older issue65/62 proofs below this projection; retain frozen counts/digests and unrelated drift negatives. Never regenerate catalogs from live producers.

Update every governed native/desktop/current schema/tool-count consumer explicitly; old frozen78/79/81 assertions remain independent. Existing mandatory commands and controls retain all earlier consumers; append new contract suites plus consumer-omission/masking negatives. Required real headless/MCP/package workflows cover valid replacement/aliases, limits/missing references/stale/late rollback, drafts, root deletion protection and exact history/reopen. Compare pre40/default/registered states through exact evaluated plans/filter graphs and required native RGB/PCM preview/export/draft oracles. Registry metadata does not make sound audible or alter existing roles/ducking/resources. Capability advertises definitions only; #64 owns placement/activation.

## Risks / Trade-offs

- Named replacement could release media still used by history/drafts → reuse the one existing asset-root owner and assert deletion/collection/reopen behavior.
- Lowered current fixture JSON could fabricate premature registry fields → remove only introduced registry model fields for genuine legacy envelopes and preserve every original assertion; add explicit premature-field negatives.
- Active header or native count updates could weaken older evidence → pin actual verified predecessor bytes before edits, approve exact transitions, retain frozen catalogs/negatives and require full all11 exact-head CI.
- Shared local memory/disk can disrupt large suites → preserve original failures/test data, reclaim only inactive compiler caches with receipts; diagnostic adjustments do not supply standard acceptance. Standard external CI remains mandatory.
- Registry-only feature does not play sounds yet → explicit issue63 scope and unchanged-output evidence, with separate #64 activation.

## Migration Plan

No deployment. Locked open or successful prepared registration adopts supported documents/history to40 with an empty registry and existing recoverable journal semantics. Failed requests retain old authoritative bytes/resources. Downgrade is unsupported and old readers reject future40. Required implementation acceptance and conformance precede sync/archive; final protected validation and all11 exact-head CI precede completion and successor branching.

## Open Questions

None requiring new user input. Original delegated issue-scoped specification approval and separate substantive same-agent CODEOWNER review remain authorized. Review must be transparently a COMMENT pass, not independent human/GitHub APPROVED or a merge-protection bypass. Future draft base is main; cumulative merge order157→158→successor remains explicit.
