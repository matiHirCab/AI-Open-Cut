# Typed animation channels (schema 26)

Issue #38 introduces a versioned, closed channel vocabulary. Issue #39 adds cubic Bézier and spring curves. Issue #41 adds typed loops. The `typed_animation_channels_v1` capability advertises the active property subset; `deterministic_animation_curves_v1` advertises the curves; `animation_loops_v1` advertises loop support. `contracts/animation-channels-v1.json` is the canonical channel and limit catalog. Existing `set_keyframes` requests and projects without loops keep their previous meaning and render output.

## Active channels

| Channel | Target | Scalar bounds | Unit |
| --- | --- | --- | --- |
| `transform.position_x`, `transform.position_y` | Visual item, group or component instance without `transform2d` | −1,000,000 to 1,000,000 | Composition pixels, origin at top left; positive X right and positive Y down |
| `transform.scale_x`, `transform.scale_y` | Visual item, group or component instance without `transform2d` | Greater than 0 through 100 | Absolute independent scale factor |
| `transform.opacity` | Visual item, group or component instance without `transform2d` | 0 through 1 | Straight opacity |
| `audio.gain_db` | Media item with audio | −96 through +12 | Decibels multiplied with existing volume as `10^(gainDb/20)` |

Visual targets are media with visual content, text, solid color, rectangle, shape, SVG, and grid. Groups and component instances also accept these five visual channels, composed outside descendants before child stagger. Audio-only media, captions, repeaters, and transitions reject visual channels. Visual channels and `transform2d` cannot coexist on an item; either edit order returns `INVALID_ARGUMENT`. Audio gain requires a media asset with audio. A channel using a missing or incompatible target returns a typed error; inactive catalog names return `INVALID_ARGUMENT` before commit. Crop, source, path, graphic, effect, rotation, skew, anchor, and pan names are cataloged for later milestones and are not writable yet.

Every catalog entry declares its value tag, activation state, and prospective target kind. Inactive transform channels target future visual behavior; crop targets visual media; source timing targets media sources; graphic and effect channels target typed scoped graphic or effect references; pan targets media with audio. Their numeric bounds are explicitly `deferred` in the catalog and will be defined with their renderer milestones. The 4,096-point and 32-stop collection caps are already named; they do not make inactive channels writable.

## Keyframes and edits

`timeline_set_animation_channels` replaces an item's entire channel collection at `expectedRevision`. The headless edit operation is `set_animation_channels` with `itemId` and `animationChannels`; it also works within `timeline_batch_edit` and resolves creation aliases. An empty array clears channels. The call is one atomic revision and undo step. A stale revision, locked track, missing item, invalid value, or failed batch leaves state and history unchanged. Legacy position, scale, opacity, and volume keyframes cannot coexist with their overlapping typed channels.

Each channel has a unique `property`, optional typed target reference, and ordered `keyframes` with `{timeMs, value, curve}`. Active channels require `{type:"scalar",value:<finite number>}` and no target reference. Times are integer milliseconds relative to the item's start, strictly increasing and inside `[0,durationMs)`. The first value holds before its keyframe, the last holds afterward, and an exact timestamp uses its keyframe value. A keyframe's curve controls the segment to the next keyframe. The absent channel uses its static property value. Preview and export use the same evaluated scene.

An active channel may also carry `"loop":{"mode":"repeat","iterations":3}` or `"loop":{"mode":"ping_pong","iterations":"infinite"}`. Modes are `repeat` and `ping_pong`; iterations are an integer from 1 through 10,000 or the literal `"infinite"`. The first and last keyframe times bound the loop. Before the first time, the first value holds. Forward repeat samples each half-open cycle and returns to the first value at an exact seam; it requires exactly equal first and last values so the seam has no value jump. Ping-pong travels forward to the last value, then reflects through the same curves to the first; one iteration is a complete round trip. At the turn it samples the last value, and at the round-trip seam it samples the first. A finite loop holds its endpoint after its final cycle. Infinite loops continue until the item's end. Value continuity does not imply matching speed at a seam.

Loop phase uses absolute item-local time. Stored timestamps remain integer milliseconds, while visual time derived through component rates, parent clocks, stagger and signed repeater offsets retains fractional milliseconds through repeat seams, ping-pong turns and finite exhaustion before curve interpolation. Existing integer sampling and independent audio sampling retain their behavior. A range preview beginning in a later cycle therefore matches a frame preview, draft, or export at that same composition timestamp. Each channel has its own loop; unlooped channels and absent channels keep their existing hold and static fallback behavior. A loop with fewer than two keyframes, bad iteration count, unknown field, inactive property, or unequal repeat endpoints fails with non-retryable `INVALID_ARGUMENT`; missing items, locked tracks, batch aliases, and stale revisions keep their established errors and rollback behavior. Loop records cannot contain renderer expressions, paths, SVG, or network resources.

`"hold"` and `"linear"` remain valid string curves. A cubic Bézier curve is `{ "type":"cubic_bezier", "x1":0.25, "y1":0.1, "x2":0.25, "y2":1 }`; all four coordinates must be finite in `[0,1]` and `x1 <= x2`. A spring curve is `{ "type":"spring", "mass":1, "stiffness":170, "damping":26, "initialVelocity":0 }`; mass is `[0.01,100]`, stiffness `[0.01,10000]`, damping `[0.01,1000]`, and initial velocity `[-100,100]`. A parameterized curve needs a following keyframe, so the terminal keyframe uses `"hold"` or `"linear"`. Unknown fields or variants fail closed with `INVALID_ARGUMENT`.

Segment time is normalized from 0 to 1 in item-local milliseconds. Bézier sampling inverts the monotone X polynomial with 40 bisection iterations and evaluates Y. Spring sampling solves the damped oscillator from position 0 and the declared initial velocity; underdamped, critical, and overdamped parameters are supported. Exact keyframe timestamps return stored values. Intermediate spring overshoot is allowed for position and gain, then sampled values are constrained to the property's documented bounds; independent scale uses `0.000001` as its positive interpolation floor. Fixed scalar samples agree within `1e-9` across supported platforms. Frame, range, draft, and export output retain the documented visual/audio tolerances.

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

Schema-21 projects and every retained undo/redo snapshot first migrate through schema 22 with empty channels, then through schema 23 for parameterized curves, schema 24 for markers, and schema 25 for loops under the project lock. A source schema below 25 cannot contain loop fields. Projects without loops retain their output. Failed migration leaves the prior durable generation intact. Older builds reject schema 25 as a future version.

See [inherited animation and timing](inherited-animation-timing.md) for parent targets, ranked delays, signed copy timing, half-open clipping and atomic derived-bounds failures. Parent channels retain static fallback for absent properties. Clear a newly created group's identity `transform2d` before setting channels. Schema 26 adds timing fields; omitted or zero values preserve valid earlier output.
