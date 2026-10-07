# Versioned animation presets

`scalar_tween@1` retains compiler version1 and `animation_presets_v1`. The initial motion pack adds five version1 entries with compiler version2 and `initial_motion_preset_pack_v1`. All use headless protocol1. Schema31 introduced retained motion-pack clocks; current project schema38 preserves those compiler, channel and provenance contracts.

Use MCP `timeline_apply_animation_preset` or headless `apply_animation_preset` inside the existing `edit` envelope. Inputs are `itemId`, mandatory explicit `presetId`, mandatory `presetVersion: 1`, `parameters`, and optional `collisionPolicy`. The same edit works in `timeline_batch_edit`, including an earlier item's `@alias`; it does not create an ID or accept `resultAlias`.

For `scalar_tween`, parameters contain an explicit `property`, `startMs`, positive `durationMs`, finite `from`/`to`, and optional `curve` defaulting to `linear`. Property endpoints use the existing bounds: position X/Y [-1,000,000, 1,000,000] composition pixels, independent scale X/Y (0,100], opacity [0,1], and audio gain [-96,12] dB. The five visual properties require an existing legacy-transform visual item, group or root component instance without transform2d; audio gain requires media with audio. Other properties remain outside this seed. No implicit static baseline is sampled.

Times are absolute item-local integer milliseconds, not marker expressions or percentages. Each time and the checked end sum must be <= 9,007,199,254,740,991, with both keys inside `[0,item.durationMs)`. A 1,000-ms item therefore accepts duration 999 from start 0 and rejects duration 1,000. Output is one existing targetless scalar channel with two keys, no loop, the effective starting curve and a terminal `hold`. Curves are existing hold, linear, cubic Bézier and spring records with their existing bounds and evaluated semantics.

Collision policy defaults to `reject` for an existing exact channel identity, even with disjoint times or identical output. Explicit `replace` replaces that entire channel including its loop in the same collection position; new identities append. Unrelated channels, static fields and effects remain unchanged. Neither policy removes overlapping legacy keys: clear/convert those explicitly with existing operations, optionally earlier in the same atomic batch. Invalid arguments, unsupported preset versions or collisions return non-retryable `INVALID_ARGUMENT`; missing items/assets, track locks and stale revisions retain their existing codes and retryability.

Resolved `animationChannels` are authoritative. Optional `animationPresetProvenance` records the source ID/version, compiler version and complete effective parameters by property. It is descriptive local project data. Reopen, evaluation and rendering never execute or look up the recorded preset; valid historical identifiers/versions fitting the stored descriptive shape remain usable after retirement. Raw `set_animation_channels` clears all labels on that item, including identical/empty replacements. Other primitive changes clear affected labels; exact core copies and edits leaving local channels unchanged preserve them. Raw component replacements retain only known labels for unchanged scoped identities. Undo/redo restore channels and provenance together.

Standalone edits and batches use the existing optimistic revision, lock, retained-state/candidate safety, undo and transaction paths. Preset application inside draft operation lists and direct component-definition authoring are excluded in this first version. Core rejects draft intents before writes; ordinary drafts can preview committed channels and edit/clear them through existing low-level operations.

The schema31 milestone adopted supported earlier schemas under the project lock in current state, component items and every retained undo/redo snapshot. Current adoption continues through later versioned migrations to schema38 without recompiling or relabelling preset provenance. Existing schema29 scalar sources and channels remain identical; migration infers no labels and preserves media/fonts, revisions/timestamps and render output. The explicit schema30→31 adapter validates closed tagged Pack and untagged scalar source records without recompilation or retagging. Tagged pack source shapes below30, retained clocks below31, premature older provenance fields, malformed retained state and future project schemas fail closed before publication. The recoverable journal publishes one complete generation. Keep a complete pre-migration project/history backup for rollback; schema29 and schema30 builds reject31 and must never strip fields to downgrade it.

`contracts/animation-presets-v1.json` governs the catalog, bounds, fixed expansion, source examples and failures. Rendering uses existing shared channel/evaluated-scene behavior; no preset files, external resources, expressions or second renderer are accepted.


## Initial motion pack

Each pack request uses `presetVersion: 1` and an object with `kind` equal to its explicit `presetId`, `startMs`, `durationMs`, and only the fields below. Endpoints are absolute canvas translations or independent scale/opacity values; there are no sampled baselines or implicit endpoints. Unknown fields, arrays and duplicate JSON fields (including inside blur) are invalid. Scalar parameters remain untagged.

| Preset | Explicit fields | Output and defaults |
| --- | --- | --- |
| `impact_slam` | `centerX`, `centerY`, `shakeAmplitudePx`, `scaleFrom`, `scaleOvershoot`, `scaleTo`, `opacityFrom`, `opacityTo`, `flashOpacity`, `motionBlur` | X/Y shake, X/Y scale overshoot, opacity dip and return, supplied enabled blur; no loop |
| `slide_left` | `positionFromX`, `positionToX` | X translation from greater to lesser value; no loop |
| `scan` | `positionFromX`, `positionToX`, optional `iterations` | Sweep to a distinct endpoint and return; default infinite repeat |
| `pulse` | `scaleFrom`, `scalePeak`, optional `iterations` | X/Y scale up and return; default one repeat cycle |
| `radar_expand` | `scaleFrom`, `scaleTo`, `opacityPeak`, optional `iterations` | X/Y expansion with fade, then reset while invisible; default infinite repeat |

For phase q, the key time is `startMs + floor(q * durationMs / 8)` using checked integer arithmetic. All keys, including the end, must lie strictly inside the item's duration. Minimum durations are8ms for slam,1ms for slide,2ms for scan/pulse and4ms for radar. Collapsed phases and unsafe sums fail. Nonterminal curves are linear; the terminal curve is hold. Existing clipping, inherited clocks, repeat seams and held boundaries remain in force.

| Preset / channel | Phases q | Values in order |
| --- | --- | --- |
| slam X | 0,4,5,6,7,8 | centerX, centerX, centerX+A, centerX-A/2, centerX+A/4, centerX |
| slam Y | 0,4,5,6,7,8 | centerY, centerY, centerY-A, centerY+A/2, centerY-A/4, centerY |
| slam scale X then Y | 0,4,6,8 | scaleFrom, scaleOvershoot, scaleTo, scaleTo |
| slam opacity | 0,4,5,6,8 | opacityFrom, opacityTo, flashOpacity, opacityTo, opacityTo |
| slide X | 0,8 | positionFromX, positionToX |
| scan X | 0,4,8 | positionFromX, positionToX, positionFromX |
| pulse scale X then Y | 0,4,8 | scaleFrom, scalePeak, scaleFrom |
| radar scale X then Y | 0,6,8 | scaleFrom, scaleTo, scaleFrom |
| radar opacity | 0,2,6,8 | 0, opacityPeak,0,0 |

A is `shakeAmplitudePx` and must be positive. All supplied and derived translations stay within +/-1,000,000; scales stay in (0,100] and opacities in [0,1]. Slam requires `scaleOvershoot > scaleTo` and `flashOpacity < opacityTo`; its flash is an opacity dip, without color tint. Pulse requires `scalePeak > scaleFrom`; radar requires `scaleTo > scaleFrom` and positive `opacityPeak`. Iterations are integers1..10000 or `"infinite"`, always materialized in saved repeat parameters. `motionBlur` is `{shutterAngleDeg, sampleCount}` with angle in (0,360] and integer sample count2..16; existing raster-work limits still apply.

Slam requires a supported legacy-transform visual leaf (visual media, text, solid, rectangle, shape, SVG or grid). Other entries also support eligible root groups/component instances. Audio-only items, captions, transitions, repeaters and Transform2D remain incompatible; no preset creates geometry or authors a component definition.

Collision rejection covers every generated identity atomically. For slam it also covers any existing blur field, including disabled blur. Explicit replace retains matching channel positions, appends absent identities in compiler order and replaces slam blur; unrelated channels/effects and non-slam blur remain unchanged. Overlapping legacy keys still reject. A pack edit produces one revision/undo entry, including an aliased creation batch; an invalid final scene publishes no prefix.

Every generated property gets the complete effective descriptive source parameters. Historical IDs/positive versions remain readable without catalog lookup, expansion equality or requiring every sibling channel to survive. A low-level blur change clears impact-shaped source labels, including retired IDs. Raw component removal of blur does the same; unchanged channel and blur bytes retain known impact labels. Scalar/unrelated labels follow their existing rules. Raw channel setters clear all labels; accepted copies, local timing edits, undo/redo and reopening preserve authoritative primitives under existing owners. No automatic split/trim retiming is introduced.

`contracts/initial-motion-preset-pack-v1.json` supplies independent fixed channels, phase/sampling/source oracles and failure cases. Frame, audiovisual range, ordinary draft and export evaluate those saved primitives through the existing renderer.

Split, trim and duplicate preserve original tagged pack provenance and exact primitive keys while composing schema31 source clocks. Newly compiled properties start without inherited clocks; unrelated clocks remain intact. Undo/redo and reopen restore attribution together with the retained animation.
