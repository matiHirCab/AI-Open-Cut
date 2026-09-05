# Deterministic stacking

Issue #19 activates the `stacking` capability and the runtime catalog `contracts/stacking-v1.json`. The headless protocol remains version 1. Project schema 9 adds flattened `zIndex` and `stackOrder` to every timeline item.

Visual layers render bottom-to-top by track array position, signed z-index, then item array position. Persisted `stackOrder` equals that item position. Stable item ID is only a final tie-break for equivalent synthesized order inputs. Z-index never moves an item above another track. Hidden items retain their position and ordering fields. Audio routing, gain, ducking, and transition endpoints retain their existing semantics.

`zIndex` is an integer from -2147483648 through 2147483647. New items start at zero. `stackOrder` is an unsigned 32-bit ordinal maintained by core, not a freely writable sort key. Creation, deletion, split, duplication, and moves maintain consecutive ordinals. Split, duplication, and moves preserve the source z-index.

## Operations

All three MCP tools require `projectId` and `expectedRevision`:

| Tool / edit operation | Fields | Behavior |
| --- | --- | --- |
| `item_set_z_index` | `itemId`, `zIndex` | Change visual priority inside the owning track. Supports visual media, text, solids, rectangles, and captions. |
| `item_reorder` | `itemId`, `index` | Move any item kind to a final zero-based array index within its existing track; preserve timing and z-index. |
| `track_reorder` | `trackId`, `index` | Move a track to a final zero-based array index, using existing indexed `update_track` semantics. |

Indices range from zero through collection length minus one. Since z-index sorts before stack order, reordering changes visual priority among items with equal z-index. Setting z-index on audio-only media or transition records returns `INVALID_ARGUMENT`. Missing references, locked tracks, invalid collection indices, and stale revisions retain `ITEM_NOT_FOUND`, `TRACK_NOT_FOUND`, `TRACK_LOCKED`, `VALIDATION_FAILED`, and retryable `REVISION_CONFLICT` respectively. Malformed wire payloads fail typed decoding; no raw expressions or resource locators are accepted.

Headless callers use the existing `edit` envelope, for example:

```json
{"operation":"edit","projectId":"project-id","expectedRevision":3,"edit":{"operation":"item_set_z_index","itemId":"item-id","zIndex":10}}
```

The same edit objects work inside MCP `timeline_batch_edit` and headless `edit_batch`. `itemId` and `trackId` can reference earlier creation aliases such as `@title` or `@overlay`. Reorder operations do not create IDs and cannot receive result aliases. A batch commits once or rolls back entirely. Changed IDs include the primary target and items whose ordinal changed, without duplicates; creation aliases continue to identify the new entity. Undo, redo, reopening, and materialized draft previews preserve ordering.

## Migration and rendering

Opening schemas 1–8 migrates current state and every retained undo/redo snapshot to schema 9 under the project lock. Each snapshot receives zero z-index and ordinals from its own arrays, preserving legacy pixels, timing, transforms, media, provenance, and revisions. The complete envelope is validated before asset or generation publication. Interrupted publication recovers one authoritative generation. Schema 0, future versions, malformed current-schema ordering, and invalid history fail closed. Older binaries cannot open schema 9; rollback requires a complete pre-upgrade project/history generation.

Every render intent consumes the same ordered `EvaluatedScene`. Regression checks compare exact ordering and independent occlusion expectations, require visual SSIM of at least 0.99 and aligned float-PCM RMS error at most 0.0001, and allow at most one output frame of timing difference. Existing evaluation complexity and complete-backend readiness limits remain in force.
