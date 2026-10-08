# Project audio buses

Schema39 stores exactly four built-in buses in this order: `voiceover`, `music`, `sfx`, `master`. Each stem initially outputs to `master`; master has a null output. A stem may output to another stem if the complete route is acyclic and ends at master in at most four nodes. Bus creation, deletion and DSP settings are outside this capability.

Audio and video tracks in the root or a component may store an explicit `audioBusId`. Absence or null uses the existing audio role: voiceover→voiceover, music→music, sound_effects→sfx, unassigned→master. Assignments preserve roles, ducking, items and settings. Empty, hidden and muted tracks still undergo validation; overlay/caption tracks cannot carry explicit routing.

The MCP tools `audio_bus_set_route` and `audio_track_route` take `projectId` and `expectedRevision`. Track routing also takes `scope` (`root` or `component:<id>`), `trackId`, and a required nullable `busId`; null clears the explicit route. Bus routing takes `busId` and non-null `outputBusId`. Both operations work in `timeline_batch_edit` and draft operation arrays. Existing created-track aliases and `component:@alias` scopes are resolved by the core. Fixed bus IDs do not create result aliases.

Stale revisions return retryable `REVISION_CONFLICT`; missing tracks return `TRACK_NOT_FOUND`, locked tracks `TRACK_LOCKED`, and invalid bus references/graphs/scopes `INVALID_ARGUMENT`. Failed requests and late-invalid batches preserve complete persisted state. Successful operations commit one revision and participate in existing undo, redo and draft ownership.

Opening genuine schema1–38 projects migrates current and retained history together under the existing project lock. It adds default bus records without changing old roles, media or other values. Premature bus fields, malformed current/retained routing and future schema versions are rejected. Structurally unrelated user slot keys named `audioBuses` or `audioBusId` remain user values.

`project_audio_buses_v1` advertises the persisted routing model. This issue preserves the existing evaluated scenes, semantic plans, normalized filter graphs, pixels and decoded audio, including role-based ducking. Bus DSP and explicit bus side-chain ducking belong to subsequent roadmap issues.

Issue65 starts from issue62's verified implementation. Its draft PR targets `main`; while PR157 is unmerged it contains both implementations and requires PR157 first. Subsequent issues branch from the preceding implementation only after all11 exact-head CI checks pass. User merges are reconciled without discarding verified work.
