# Semantic sound-event definitions

Schema 40 projects have a project-owned `soundDefinitions` library. `sound_event_register` creates a named definition or replaces its content at the same index. Each closed record contains `event`, ordered `variantAssetIds`, `defaultGainDb`, `busId`, and `variantSeed`.

Names are case-sensitive ASCII identifiers beginning with a letter, at most 128 characters. A project has at most 512 definitions with 1–32 distinct content-addressed audio variants each. Audio assets and videos containing audio are eligible; variants must already belong to the project and have valid SHA256 and positive size metadata. Registering does not copy media or place an audible timeline item.

Gain is finite and within [-120, 24] dB. The required seed is an integer from 0 through 9007199254740991. Core resolution uses the saved seed or an explicit safe seed modulo the declared variant count. Variant ordering, gain and the existing built-in bus are returned deterministically. Audio roles, ducking, rendering and marker timing are unchanged by registration.

Standalone MCP input includes `projectId` and `expectedRevision`. Batches use `operation: "sound_event_register"`; `resultAlias` resolves to the event name for subsequent operations. Existing batch alias rules apply. Registration is available through existing draft creation, update, preview, rebase and commit. Every successful standalone or batch commit increments revision once and supports undo, redo and reopen.

Invalid names, bounds, duplicate variants, missing buses and ineligible assets return `INVALID_ARGUMENT`; well-formed missing variant IDs return `ASSET_NOT_FOUND`. Stale revisions return `REVISION_CONFLICT`. Retained dangling references or damaged media return `ASSET_INTEGRITY_FAILED`. Registered variants and pending draft registrations block deletion with `ASSET_IN_USE`; replaced variants remain retained by undo history until the existing history collector releases them.

Every supported legacy schema migrates current state and undo/redo snapshots atomically to an empty library. Schema 39 bus routes are preserved. Premature structural fields, missing current arrays and future versions are rejected. Failed mutations do not adopt a migration; dynamic user keys named `soundDefinitions` remain values. The protocol remains version 1 and advertises `semantic_sound_event_definitions_v1`.

This issue's draft PR targets `main` and starts from verified issue65 head 321dc3583df2d95380efb613153b0135bb32268d. While predecessor PR 157 and 158 remain unmerged, scope is cumulative 62+65+63 and merge order is 157→158→the issue63 PR. Only the user may merge; subsequent branches start from verified implementation heads.
