# Typed animation channels (schema 22)

Issue #38 introduces a versioned, closed channel vocabulary. The `typed_animation_channels_v1` capability advertises the active subset. `contracts/animation-channels-v1.json` is the canonical channel and limit catalog. Existing `set_keyframes` requests and projects without `animationChannels` keep their previous meaning and render output.

## Active channels

| Channel | Target | Scalar bounds | Unit |
| --- | --- | --- | --- |
| `transform.position_x`, `transform.position_y` | Visual item with legacy transform | −1,000,000 to 1,000,000 | Composition pixels, origin at top left; positive X right and positive Y down |
| `transform.scale_x`, `transform.scale_y` | Visual item with legacy transform | Greater than 0 through 100 | Absolute independent scale factor |
| `transform.opacity` | Visual item with legacy transform | 0 through 1 | Straight opacity |
| `audio.gain_db` | Media item with audio | −96 through +12 | Decibels multiplied with existing volume as `10^(gainDb/20)` |

Visual targets are media with visual content, text, solid color, rectangle, shape, SVG, and grid. Audio-only media, captions, groups, component instances, repeaters, and transitions reject visual channels. Visual channels and `transform2d` cannot coexist on an item; either edit order returns `INVALID_ARGUMENT`. Audio gain requires a media asset with audio. A channel using a missing or incompatible target returns a typed error; inactive catalog names return `INVALID_ARGUMENT` before commit. Crop, source, path, graphic, effect, rotation, skew, anchor, and pan names are cataloged for later milestones and are not writable yet.

## Keyframes and edits

`timeline_set_animation_channels` replaces an item's entire channel collection at `expectedRevision`. The headless edit operation is `set_animation_channels` with `itemId` and `animationChannels`; it also works within `timeline_batch_edit` and resolves creation aliases. An empty array clears channels. The call is one atomic revision and undo step. A stale revision, locked track, missing item, invalid value, or failed batch leaves state and history unchanged. Legacy position, scale, opacity, and volume keyframes cannot coexist with their overlapping typed channels.

Each channel has a unique `property`, optional typed target reference, and ordered `keyframes` with `{timeMs, value, curve}`. Active channels require `{type:"scalar",value:<finite number>}` and no target reference. Times are integer milliseconds relative to the item's start, strictly increasing and inside `[0,durationMs)`. The first value holds before its keyframe, the last holds afterward, and an exact timestamp uses its keyframe value. `hold` and `linear` are the only curves in this milestone; linear interpolation occurs in the channel's declared unit, including decibels. The absent channel uses its static property value. Preview and export sample the same evaluated scene.

Limits are 64 channels per item, 1,000 keyframes per channel, 4,096 path points, and 32 gradient stops. The latter two limits reserve bounded value shapes for inactive catalog entries. No channel accepts raw FFmpeg expressions, executable SVG, file paths, or network resources.

```json
{
  "operation": "set_animation_channels",
  "itemId": "rectangle-id",
  "animationChannels": [{
    "property": "transform.scale_y",
    "keyframes": [
      {"timeMs": 0, "value": {"type": "scalar", "value": 1}, "curve": "linear"},
      {"timeMs": 500, "value": {"type": "scalar", "value": 1.2}, "curve": "hold"}
    ]
  }]
}
```

Schema-21 projects and every retained undo/redo snapshot migrate to schema 22 with empty channels under the project lock. Failed migration leaves the prior durable generation intact. Older builds reject schema 22 as a future version.
