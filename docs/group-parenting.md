# Groups and parent transforms

Schema 10 adds non-drawing `group` items and optional `parent: {"scope":"root","id":"..."}` on visual items. Headless status advertises `group_parenting`. Groups live on overlay tracks, default to visible identity Transform2D and zero zIndex, and participate in normal item ordering without drawing pixels or producing audio.

MCP exposes `add_group` and `item_set_parent`, with the usual `projectId` and `expectedRevision` arguments. The same operations work in an ordered `timeline_batch_edit`:

```json
[
  {"operation":"add_group","trackId":"overlay-id","startMs":0,"durationMs":1000,"resultAlias":"group"},
  {"operation":"item_set_parent","itemId":"existing-visual-id","parent":{"scope":"root","id":"@group"}}
]
```

Creation accepts an optional complete `transform2d` and parent. Parenting requires an object or explicit `null` to detach and preserves local properties without world-position compensation. Both item and parent aliases resolve against earlier creations. The batch commits and undoes once.

Root is the only composition scope. Parents may be on different or locked tracks; the child's track must be unlocked. Parent IDs contain at most 128 ASCII letters, digits, underscores, or hyphens. Missing parents return `ITEM_NOT_FOUND`; non-group targets, cycles, cross-scope edges, and graph overflows return `INVALID_ARGUMENT`. Hidden/inactive nodes are validated. Limits are 32 ancestor edges (root depth zero) and 4096 visual/group nodes. Audio-only media and transitions cannot be parented.

Transforms apply locally, then through ancestors nearest outward. Group anchors and normalized positions use composition dimensions, never child bounds. [Transform2D coordinate and numeric rules](transform2d.md) apply, including composed raster limits of 16,384 pixels per dimension and 16,777,216 pixels in area. Animated legacy children retain root-time keyframes; position/scale endpoints bound sampling within the composition, using non-overlapping tiles of at most 4096×4096 pixels. Movement distance does not change the per-object geometry limits. Legacy text retains its styled-box anchor before ancestor transforms. Opacity multiplies per descendant without isolated compositing. Flat track/zIndex/stackOrder/ID ordering is preserved.

Child time remains absolute root milliseconds. Visual activity intersects every ancestor's half-open interval and item/track visibility. Source time and media audio remain unchanged. Frame, range, draft, and export share evaluated facts and complete-backend readiness checks.

Groups accept static transforms, visibility, timing, moves, reordering, and node-only duplication. Splitting, keyframes, audio, legacy transform updates, and transition endpoint use return `INVALID_ARGUMENT`. Duplicating a group never clones or reparents children. Visual child moves, splits, and duplicates retain their parent.

Deleting a referenced group returns `INVALID_ARGUMENT`: detach or reparent children first. Track deletion requires an empty track (`VALIDATION_FAILED` otherwise). Revision conflicts, locked targets, and failed batches preserve state and history.

Schemas 1–9 and retained undo/redo snapshots migrate atomically under the project lock. Existing items remain unparented and preserve IDs, timing, transforms, media, and provenance. Schema-10 grouped content requires a group-aware reader; older binaries must reject it. Unknown future schemas fail closed. Rollback requires a compatible reader or pre-upgrade backup. Simple requests retain their meaning; protocol major and stable errors remain unchanged. The runtime catalog is `contracts/group-parent-v1.json`; other motion-graphics roadmap vocabulary stays fixture-only.

Both transition endpoints are checked during current-state, history, draft, mutation, and renderer validation. An endpoint resolving to a group returns `INVALID_ARGUMENT`, including hidden records, without rewriting invalid files.
