## Why

#209 needs stable sentence targets before #210 can repair one sentence. Existing speech items, previews, profiles and atomic edits work, but do not record an ordered narration manifest tying original wording to immutable generated intent and owned native references.

## What Changes

- Plan bounded sentence blocks before inference with explicit manual Unicode-scalar boundaries, stable UUIDs, paragraph order, exact source/display wording and a pinned resolved narration profile per block.
- Reuse existing speech estimation, preview retention, cleanup and provider-neutral synthesis. Publish a complete approved preview group, its native speech items and manifest in one core transaction; a partial generation/cancellation never publishes a partial manifest.
- Persist exact original/display and resolved spoken wording, effective request, speech item/asset IDs and explicit owned caption/cue references. Reopen and retained history remain independent of the profile library/provider.
- Expose bounded source CLI workflows and additive typed headless operations. Do not expand the MCP catalog or installed desktop UI in this slice.
- Adopt project schema47 with deterministic current/history/draft migration from supported predecessors and fail-closed introduced-field validation. Preserve version1 protocol and existing requests.

## Capabilities

### New Capabilities
- `narration-blocks`: bounded planning, native atomic publication, stable ownership and independent sentence identity.

### Modified Capabilities
- `project-persistence`: schema47 manifest adoption across current state and retained generations.
- `channel-style-packs`: retain CP1 component schema46 packages and allow explicit compatible readonly capture from project46/47.
- `speech-generation`: explicitly approved complete retained-preview groups can be consumed atomically without rerunning synthesis; ordinary preview/insertion/regeneration behavior stays compatible.

## Impact

Core model/validation/store/migrations, thin headless transport, separate bridge narration modules/source CLI, canonical versioned fixtures and their Rust/TS consumers, docs and scenario tests. No new provider/dependency, security configuration, deployment or merge. CODEOWNER review remains required separately from agent specification/conformance review. Existing rendering consumes ordinary native audio and captions/cues, without a new renderer.

Public operations/optional metadata are additive. Schema47 requires an explicit migration; older binaries reject it rather than silently dropping manifests. Canonical predecessor inverse evidence must distinguish this deliberate schema transition from unrelated contract drift.

## Non-goals

#208 audition/pronunciation-risk analysis, its private device files and unpublished UTF-16 changes; #210 duration-changing sentence replacement/ripple policy; conflict resolution or integration of #228/#229; guessed word timestamps; new TTS provider/model downloads/paid APIs; durable inference resumption; installed GUI controls or creative-quality claims. Do not modify existing pronunciation resolution or per-request speech behavior. Preserve all originals, backups and preexisting files; no cleanup is authorized.

## Baseline and authorization

Verified GitHub main `ae0a6cbc01726a1b4123303a49ec38783cebf32d` (includes #230/#231), #207 closed, #209 open with no comments/competing PR. Main CI is still running; no complete-green claim. User delegation authorizes scoped spec approval, implementation, checks, push and draft PR; no merge/deploy. Independent reviewer approval of concrete artifacts is required before executable edits under AGENTS.md.
