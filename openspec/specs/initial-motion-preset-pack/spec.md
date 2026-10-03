# Initial Motion Preset Pack Specification

## Purpose

Define the five bounded versioned motion presets and their deterministic compilation into editable existing animation primitives.

## Requirements

### Requirement: Closed initial motion pack semantics
Core MUST support exactly impact_slam@1, slide_left@1, scan@1, pulse@1 and radar_expand@1 with strict tagged parameter shapes, exact ordered values and phase rules below. Pack parameters and nested motionBlur records MUST be object-only and reject duplicate fields/unknown fields without losing original JSON entries. Records MUST contain matching kind/startMs/durationMs and only their listed fields; scan/pulse/radar MUST accept only the additional optional iterations field. Existing scalar-tween parameters MUST remain valid and unchanged. All endpoints MUST be explicit finite absolute values within canonical primitive bounds; fixed curves and iteration defaults MUST materialize deterministically. Let t(q)=startMs+floor(q*durationMs/8), checked with safe integer arithmetic; every key including t(8) MUST be strictly inside item duration, with each value and sum<=MAX_SAFE_INTEGER. Collapsed phases MUST fail, never clamp/retime. The definitions below MUST be enforced in core and copied into canonical fixtures/docs before consumer updates.

| kind | required endpoint/config fields | exact ordered channels and values |
| --- | --- | --- |
| impact_slam | centerX, centerY, shakeAmplitudePx, scaleFrom, scaleOvershoot, scaleTo, opacityFrom, opacityTo, flashOpacity, motionBlur | position_x at q=[0,4,5,6,7,8]: [centerX,centerX,centerX+A,centerX-A/2,centerX+A/4,centerX]; position_y at same q: [centerY,centerY,centerY-A,centerY+A/2,centerY-A/4,centerY]; scale_x then scale_y at q=[0,4,6,8]: [scaleFrom,scaleOvershoot,scaleTo,scaleTo]; opacity at q=[0,4,5,6,8]: [opacityFrom,opacityTo,flashOpacity,opacityTo,opacityTo]. Assign supplied enabled MotionBlur atomically. |
| slide_left | positionFromX, positionToX | position_x at q=[0,8]: [positionFromX,positionToX]; no loop |
| scan | positionFromX, positionToX | position_x at q=[0,4,8]: [positionFromX,positionToX,positionFromX]; repeat loop |
| pulse | scaleFrom, scalePeak | scale_x then scale_y at q=[0,4,8]: [scaleFrom,scalePeak,scaleFrom]; repeat loop |
| radar_expand | scaleFrom, scaleTo, opacityPeak | scale_x then scale_y at q=[0,6,8]: [scaleFrom,scaleTo,scaleFrom]; opacity at q=[0,2,6,8]: [0,opacityPeak,0,0]; repeat loop. Reset occurs during zero opacity. |

All nonterminal keys use existing linear, terminal hold. Before start/after final finite loop completion, use existing channel hold semantics. Slam and slide have no loop. scan/pulse/radar additionally accept optional iterations, exactly existing positive count1..10000 or string "infinite". Materialize defaults: scan/radar infinite, pulse1. Always emit existing repeat loop with effective iterations; first/last values match. Existing loop sampling, inherited time, item clipping and iteration semantics are retained.

Minima: slam8ms, radar4ms, scan/pulse2ms, slide1ms. Existing bounds apply to every input and derived key: positions +/-1,000,000, scales (0,100], opacity [0,1]. Require A>0, scaleOvershoot>scaleTo>0, scaleFrom>0; flashOpacity<opacityTo; slide fromX>toX; scan distinct endpoints; pulse peak>from; radar to>from and opacityPeak>0. motionBlur is the existing strict record; require finite shutterAngleDeg in (0,360], sampleCount2..16 and unchanged raster-work budgets. Any derived position exceeding bounds fails before publication.

Impact targets must be compatible supported visual leaves for both legacy visual channels and MotionBlur: visual media/text/solid/rectangle/shape/SVG/grid, with no transform2d. Other pack entries retain existing scalar compatibility, including eligible root group/component instances and excluding captions/transitions/repeaters/audio-only media. No definition is authored.

The endpoint inequalities, minima, compatibility, loop/blur settings and derived key bounds above MUST be enforced with non-retryable INVALID_ARGUMENT before publication. Compilation MUST accept no arbitrary expressions, resource paths, executable content, implicit baselines, new geometry or transport-owned expansion.

#### Scenario: Independent pack expansion
- **WHEN** each pack entry is applied with valid explicit endpoints/configuration to a compatible existing item
- **THEN** independent canonical fixed oracles match every generated key/channel/loop, slam blur and complete effective source record

#### Scenario: Phase minima and odd durations
- **WHEN** each entry is applied at its minimum duration and an odd valid duration
- **THEN** integer floor phase times and derived shake/reset/flash values match exact independent oracles without repeated key times

#### Scenario: Invalid shape and derived bounds
- **WHEN** a record has wrong kind, positional array, duplicate top-level/nested field, unknown/missing field, nonfinite/out-of-range endpoint, wrong inequalities, too-short/unsafe timing, out-of-range derived shake, invalid iterations or disabled/out-of-budget slam blur
- **THEN** core returns nonretryable INVALID_ARGUMENT with no project/history/alias/draft/resource change

#### Scenario: Seamless repeated scan and radar
- **WHEN** repeat sampling crosses cycle boundaries, final finite iteration or fractional inherited times
- **THEN** scan endpoints match, radar resets during zero opacity, pulse returns to its initial scale, finite/infinite semantics match existing repeat channels, and sampling remains bounded

### Requirement: Compatible targets and atomic pack application
The pack MUST use existing legacy visual target compatibility and existing project safety checks. It MUST preserve static transforms/effects/geometry, unrelated channels, provenance and loops. Default reject and explicit whole-channel replace MUST apply to all output identities atomically. Slam MUST also assign its supplied enabled MotionBlur; any present MotionBlur field MUST collide under reject, while replace MUST replace it atomically with all channels. Other pack entries MUST preserve existing blur. Missing references, track locks, revision conflicts, aliases and batch limits MUST retain canonical codes and retryability. Pack operations MUST support earlier creation aliases, one revision/undo entry, deterministic reopen and exact undo/redo; preset draft intents and direct component-definition authoring MUST remain excluded.

#### Scenario: Partial collision has no prefix
- **WHEN** only the second or third generated identity collides, or final candidate scene safety fails after an aliased creation and valid earlier edit
- **THEN** reject fails atomically and preserves project/history/draft/resource bytes without returning committed aliases

#### Scenario: Replace a subset in order
- **WHEN** replace applies to a mix of existing and absent pack identities among unrelated channels
- **THEN** colliding channels retain their collection positions, absent channels append in compiler order, unrelated state remains equal, and undo restores all old channels/loops/source records

#### Scenario: Existing failures and history
- **WHEN** pack requests encounter stale revisions, missing item/asset, locked track, forward alias, illegal resultAlias, out-of-budget blur or incompatible targets
- **THEN** established codes/retryability and byte-preserving rollback match the scalar compiler; valid standalone/aliased batches undo, redo and reopen deterministically

### Requirement: Shared primitive rendering for the pack
Frame preview, audiovisual range preview, ordinary draft preview and export MUST evaluate saved channels using the existing EvaluatedScene and interpolation, independent of catalog and provenance. Manual primitive equivalents MUST preserve documented visual/audio tolerance; no new evaluator, golden thresholds or expressions SHALL be introduced.

#### Scenario: Render fixed primitive equivalents
- **WHEN** each preset and independently authored equivalent channels are rendered at first/interior/terminal keys, held boundaries and fractional inherited times
- **THEN** frame/range/draft/export outputs agree within existing tolerances and project/history bytes remain unchanged, including after valid source retirement
