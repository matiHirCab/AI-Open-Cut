## Context

Unapproved issue46 proposal based on unmerged PR134 at67d05dbf. The roadmap at docs/motion-graphics-implementation-plan.md:357/:502 calls for slam overshoot/shake/flash/motion blur and looping scan/radar motion. Current code supports primitive scalar channels, repeat loops and per-leaf MotionBlur. Source parameters are a closed scalar record; adding pack shapes needs schema30. Core remains the owner; transports only decode and forward.

## Goals / Non-Goals

Goals: the five requested presets with precise editable creative output, safe bounds, atomic replacement, complete descriptive source data and independent render/lifecycle fixtures. Non-goals are in proposal.md. Existing-item ring/grid construction belongs to callers. No implementation is authorized until the exact artifacts are approved.

## Decisions

### Explicit pack shapes and phase times

Preserve old untagged scalar parameters exactly. Every pack record has mandatory `kind` equal to its live presetId, startMs and durationMs, plus only the fields below. Saved historical IDs/versions need not match the live catalog. Endpoint values are absolute existing canvas translations, per-axis scale multipliers and opacity. No endpoint/curve/baseline/canvas-size defaults. Let t(q) = startMs + floor(q*durationMs/8), calculated with safe checked integer arithmetic; end=t(8). Every authored key including end must be strictly inside item duration. Start/end/sum obey MAX_SAFE_INTEGER. Reject collapsed times; no clamping/retiming.

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

Rationale: explicit values make expansions independent of mutable baselines and tests independently calculable; ordinary channels and blur remain editable. Alternatives: spring slam needs additional artistic parameters/curve oracles; tint/effect flash needs effect-target/ownership contracts; geometry creation needs creator aliases. This proposal uses a visible opacity flash and existing caller-authored shapes. Exact flash interpretation is an approval point, not presumed equivalence to every possible roadmap artwork.

### Atomic compilation and collision rules

Pure compiler produces an ordered vector and optional blur assignment without filesystem/network/randomness. Compile all output before mutation. Default reject fails if any generated channel identity already exists, regardless of time, or slam's motionBlur field is present (even disabled). Explicit replace overwrites complete matching channels in existing positions and replaces slam blur; absent identities append in compiler order. Unrelated channels, effects, source records and non-slam blur remain unchanged. Both policies reject legacy collisions. Validate the complete candidate through existing reference, scene/raster, locking, revision, alias and journal owners; publish none of an invalid output/batch. This proposal retains issue45's byte-preserving failure requirement: parent must resolve the legacy-load migration finding before implementation.

Alternatives: preserving an existing blur while claiming the supplied slam blur would violate effective parameters; silent replace would overwrite authored motion. Entire multi-output candidate is one edit/undo entry.

### Descriptive per-property source lifecycle

Record the complete effective pack parameters (including iterations/defaults or enabled motionBlur) in each generated property's existing source slot. Persisted looped shapes must explicitly carry iterations even when the request omitted it; absence must fail instead of defaulting descriptive state. Use compilerVersion1 for scalar and2 for pack; versions remain positive u32. Validate source map key membership in the parameter shape's generated property set and existence of that targetless channel, without catalog lookup, expansion equality or requiring every sibling to exist. Old scalar sources still match parameters.property. Editing channels follows existing clear/preserve rules. Changing/removing MotionBlur through an accepted low-level edit additionally clears all known impact-shaped source records on that item, including historical IDs with that shape; changing an unrelated static field preserves them. Raw component replacement preserves an impact source only if both its channel bytes and associated blur bytes are unchanged. Exact copies retain sources; invalid duration/split continues to fail; accepted unchanged local primitives retain existing local semantics.

Alternative: a new item-wide application log would broaden persistence and ambiguity. Per-property records retain existing lifecycle and bounds. Provenance remains descriptive, not an integrity signature; a saved source's blur need not equal current runtime blur during read/retirement validation.

### Contract and schema compatibility

Expand the existing operation's request union additively; report initial_motion_preset_pack_v1 alongside animation_presets_v1. Keep scalar output/parameters unchanged. Maximum per application5 channels/25 keys (slam6+6+4+4+5); other entries <=3 channels/10 keys. Existing64 channels/sources per item,1000 keys/channel,100 batch operations and retained/scene/raster budgets remain unchanged. Manually update canonical fixtures and MCP structural digest, governed declarations/consumers and CODEOWNERS; owner is @matiHirCab.

Schema30 is needed for tagged descriptive sources. Migrate current/components/all undo/redo under lock and recoverable journal; preserve old scalar sources/channels/revisions/timestamps/media/fonts/output, infer no labels, and perform complete validation before publication. Reject malformed, premature tagged shapes below30, and future schema generations before writes. Alternative: reducing each pack source to scalar endpoints loses shake/flash/loop/blur intent; silently dropping provenance is incompatible. A complete pre-migration project/history backup is the rollback mechanism; schema29 builds must reject30.

## Risks / Trade-offs

- Artistic names are underspecified: approve exact opacity flash and positional shake plus phase table before code.
- Enabled slam blur adds work: preserve existing finite/raster/sampling budgets, inherited safety and exact midpoint fix; test boundary overflow and rejected whole candidates.
- Repeat loop seams can show a radar reset: reset only after opacity reaches0; assert boundary/interior/fractional inherited visual equality with independent primitives.
- Multi-property/blur replacement overwrites authored motion: explicit reject/replace and exact undo are mandatory.
- Existing split semantics are not automatic phase-preserving retiming: preserve documented accepted/failing outcomes; broader milestone split guarantees require a separately approved contract change.
- Unmerged prerequisite/reproduced failed-edit migration gap: resolve with parent, rebase and rerun affected checks before implementation.

## Migration Plan

After exact design approval and issue45 resolution: fixtures/migration before consumers, implement through tasks, run all required conformance/native suites, obtain CODEOWNER acceptance, verify/sync/archive, then final protected checks. Parent scope confirmation precedes publication. No merge/deploy authorization. Roll back using complete backup and old build; never strip fields to downgrade.

## Open Questions

Approve the exact five parameter shapes, opacity flash/shake definition, enabled blur collision semantics, loop defaults/limits and invisible radar reset, schema30, complete descriptive sources and blur-sensitive lifecycle. User continuation authorized planning, not these concrete decisions.
