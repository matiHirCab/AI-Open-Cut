# Timeline audio events

Protocol1 clients can place a named sound with `timeline_add_audio_event` and inspect it as a media item with `audioEvent` provenance. Schema41 and `timeline_audio_events_v1` identify support.

```json
{"projectId":"project-1","expectedRevision":4,"scope":"root","trackId":"audio","event":"impact","at":{"type":"marker","markerName":"impact","offsetMs":-50},"durationMs":300,"gainDb":-3,"variantSeed":1}
```

Scope is root or component:<id>; names resolve only in that composition. Absolute timing uses `{"type":"milliseconds","valueMs":250}`. Marker expressions remain live through marker moves. Missing markers use ITEM_NOT_FOUND, ambiguous/unsafe intervals INVALID_ARGUMENT. Offsets, times and seeds must be JavaScript-safe. Component intervals cannot exceed their local duration.

Placement requires an unlocked audio track and registered definition. Unknown event is INVALID_ARGUMENT; missing/locked track uses TRACK_NOT_FOUND/TRACK_LOCKED. Stale expected revisions retain retryable REVISION_CONFLICT before domain validation. Existing batch aliases address preceding track/component/definition creations and placement result IDs; unknown/forward aliases use VALIDATION_FAILED. Existing draft rules apply; no separate draft alias engine is introduced.

Gain defaults0; each captured definition and placement gain is finite[-120,24]dB. Effective base volume is existing audio.volume multiplied by `10^((defaultGainDb+gainDb)/20)`, without clamping. Existing mute/fade/volume automation remain modifiers. Duration defaults the selected asset's known positive duration or can truncate from source0. No arbitrary paths, filters or expressions are accepted. Audio-bearing video variants produce sound only.

Seed defaults the definition's saved seed and selects ordered index modulo count. Placement snapshots asset/hash/default gain/bus/seed/index: later definition replacement affects future placements only. Existing moves, duplicates, split, undo/redo and reopen retain the snapshot. Captured bus is routing intent; issue66 supplies bus DSP. Existing role ducking remains unchanged.

Version2 drafts additionally retain audioEventAssetIds managed roots. A stale draft preserves its previously selected asset even after definition replacement; update/rebase recaptures against the new base. Same-base replay checks exact roots. Current/component/history/draft references share existing integrity/deletion/GC policy. Old media and drafts omit these optional fields.

Schema1..40 current and undo/redo adopt41 atomically. Source40 populated sound libraries and39+ buses/routes survive exactly. Pre41 event metadata and unknown future documents fail before rewrite. Complete contracts and original native RGB/PCM checks remain required.

Issue64 branch starts at verified issue63 head337329198fcd40173a1b401e39ae7782610cbb94. PR base is main and cumulative required merge order is157→158→159→issue64 successor. No merge or deployment is authorized by this implementation.
