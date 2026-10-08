## Context

Verified predecessor33732919 is schema40 with definitions, buses, markers and prepared current/history/draft transactions. Root/component media already carry numeric start and marker startTime and share one audio evaluator. ADR0003 ownership and existing imports remain authoritative.

## Goals / Non-Goals

Goals: deterministic absolute/scoped placement, stable chosen content after library updates, atomic standalone/batch/draft edits and native shared preview/export. Non-goals: new bus DSP/ducking/loudness, dynamic variant reselection, event removal API, arbitrary expressions/resources, providers or new top-level dependency.

## Decisions

Persist optional nonnull closed `audioEvent` on existing type:media, represented by AudioEventItem provenance: event, gainDb, defaultGainDb, busId, variantSeed, variantIndex, contentHash. Media assetId/sourceInMs/durationMs/startMs and existing startTime remain canonical item timing/ownership. This avoids a parallel timeline/renderer path and permits all existing item inspection, move, split, duplicate, mute, fade, history and asset-root behavior. A separate enum item was considered; sharing the media path keeps exactly one evaluated behavior and additive old-media compatibility. Present metadata requires schema41; omission retains byte-identical old items.

Place only on audio tracks in root or component:<id>. Resolve earlier track/component/event aliases; named marker timing remains exact scoped name (not marker-ID alias). Missing scope/component or marker retains ITEM_NOT_FOUND, absent track TRACK_NOT_FOUND, locked track TRACK_LOCKED, missing event/invalid bounds INVALID_ARGUMENT; alias failures VALIDATION_FAILED. Stale revision precedence unchanged. Marker ambiguity, negative/unsafe end, component end beyond duration fail atomically. Absolute at uses existing closed milliseconds variant; persisted marker at uses startTime and existing reconciliation, absolute at numeric startMs. Resolve function reused in markers owner.

Choose using existing seed modulo ordered variants. Snapshot current definition gain/bus/selection/contentHash at creation; future re-registration does not rewrite placed events. A required registered event name stays referenced, but captured asset need not remain a current variant after replacement. This makes sound design deterministic across reopen/history. Capture defaultGainDb and placement gainDb separately, each finite[-120,24], render product audio.volume * 10^((defaultGainDb+gainDb)/20); no clamping or relaxation of old audio.volume[0,4]. Empty keyframes/default fades initially; existing volume/fade/mute controls apply as independent modifiers. Explicit/default-selected seed safe[0,MAX_SAFE], index0..31. Optional durationMs defaults full known positive safe asset duration; supplied duration may truncate from source0; unknown/nonpositive duration cannot default and invalid ranges fail closed. Later generic split/trim preserve metadata and existing media source timing rules.

VideoWithAudio variants are audio-only even though original assets contain video: suppress visuals, retain audio resource and canonical PCM. No arbitrary backend filters. Event busId is captured routing intent for later issue66; current role ducking remains exact. Preflight/resource/animation/audio classification must consistently treat event-bearing media as audio-only. Schema41 new metadata is validated in pure model methods and validation owner; no model→validation dependency.

Migrate1..40 current/undo/redo on cloned locked generation. Reset definitions only source<40 and buses only source<39; preserve populated40 registry/routes/media. Reject forged event metadata in pre41 sources and unknown future versions before rewriting. Prepared store classification includes new operation for all standalone/batch/draft/resource transactions. Existing materialized items and definition roots preserve selected managed bytes and history/GC. Version2 placement-bearing drafts additionally retain derived optional nonnull audioEventAssetIds captured roots through the existing collector. Create/update/rebase refresh roots; stale pending drafts preserve old selected bytes after definition replacement. Rebase explicitly re-resolves intent against the new base and recaptures selection; same-base replay requires exact roots. Old drafts omit the field.

Manually author timeline-audio-events-v1 canonical catalog from approved semantics. Before executable edits freeze exact337 raw headless/MCP/ownership bytes and hashes outside checkout; add independently pinned structural projection, retaining all old proofs. Seven active schema headers40→41 only; frozen feature catalogs unchanged. Add capability timeline_audio_events_v1 and thin MCP tool; protocol1 stays1. Register every affected consumer/CODEOWNER path. No generation of canonical catalog from live producer.

## Risks / Trade-offs

Snapshot gains/bus may differ from a replaced definition → explicitly documented immutable placement; new placements use new definition. Video/audio classification divergence → native equivalent-audio golden, complete-scene admission tests. Migration erasure of populated registry → original40 retained-current/undo/redo/recovery/draft negative controls. Runtime OOM/zombies → preserve failures, proper docker-init child reaping; standard unmodified exact-head CI required.

## Migration Plan

No deploy/merge authorized. Verify all standard checks, native original RGB/PCM oracles, conformance and separate transparent same-agent reviewer COMMENT before sync/archive. Final protected/all11 exact-head CI source/tested-merge equality blocks successor work. PR base main; cumulative order157→158→159→successor. No outstanding decision requires user input.
