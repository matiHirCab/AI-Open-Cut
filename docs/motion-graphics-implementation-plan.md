# OpenCut motion-graphics implementation plan

Status: proposed

Architecture: accepted in [ADR 0004](adr/0004-motion-graphics-architecture.md); implementation remains milestone-driven

Scope: canonical Rust editor model, headless protocol, agent bridge/MCP, renderer, and the minimum desktop editing surfaces needed to inspect and edit the new model
Baseline reviewed: project schema v6 and agent bridge v0.1.0

## Outcome

Implement an agent-friendly motion-graphics layer over OpenCut's existing revisioned editor core. A complete vertical slice must be able to create a reusable animated rule card, instantiate it several times with different content, synchronize its entrances and sound effects to narration markers, preview the result with audio, and export the same result deterministically.

The implementation must preserve the repository's current strengths:

- all persisted semantics live in `crates/editor-core`;
- every mutation uses optimistic revisions and remains undoable;
- compound edits validate on a clone and commit atomically;
- headless and MCP layers translate contracts rather than owning editor rules;
- still preview, range preview, and export evaluate the same scene and audio plan;
- old projects and their undo/redo history migrate deterministically.

## Current baseline and gap

| Requested capability | Current baseline | Required end state |
| --- | --- | --- |
| Nested compositions/components | Flat tracks and flat items | Groups, parent transforms, component definitions, typed slots, nested instances, safe duplication |
| Vector graphics | Solid color and filled rectangle items | Paths, SVG, grids, repeaters, fills, strokes, gradients, dashes, corner radii, glow, path animation |
| Rich typography | Font, color, outline, shadow, wrapping, alignment, padding, anchor | Shaped text, weights, tracking, line height, auto-fit, max bounds, layered strokes, blur shadows, styled spans |
| Animation | Position, uniform scale, opacity, volume; five easings | Rotation, independent scale, crop/effect/path channels, Bézier and spring curves, loops, stagger, presets, motion blur |
| Masks/compositing | Alpha and fade/crossfade | Masks, mattes, clipping, blend modes, ordered effects, vignette, glow, blur, tint, flashes, particles |
| Tracks/stacking | Track create/update/index plus array-order rendering | Documented bottom-to-top rules, explicit item z-index, stable tie-breaking, track reorder alias, groups |
| Atomic construction | `timeline_batch_edit` with aliases and rollback | Preserve it; add high-level component/template operations that are themselves atomic and usable inside batches |
| Narration timing | Transcription words exist; speech result has duration only | Provider-neutral speech alignment, named markers, marker-relative timing, automatic marker creation |
| Designed audio events | Item gain/fades, roles, basic narration ducking | Semantic event library, marker placement, buses, dB gain, compression, limiting, normalization, analysis |
| Audiovisual preview | `preview_render_range` already renders MP4 with optional audio | Presets, audio by default for review, cache, immutable revision metadata, render/export parity tests |

## Locked architectural decisions

[ADR 0004](adr/0004-motion-graphics-architecture.md) records these decisions as normative constraints before any public-model change. The summary below remains the roadmap view.

1. **Keep root tracks additive.** Do not replace `Project.tracks` immediately. Add reusable `components` beside the root tracks, and let `CompositionInstance` items reference component definitions. This avoids a needless root-project rewrite while still allowing arbitrary nesting.
2. **Introduce one evaluated scene representation.** Add a renderer-neutral `RenderPlan`/`EvaluatedScene` between the persisted model and FFmpeg command generation. Flatten parent transforms, component instances, timing references, repeaters, masks, effects, and z-order into immutable per-frame/per-range instructions. Every renderer entry point consumes this representation.
3. **Use a hybrid renderer initially.** Keep FFmpeg for media decode, audio processing, final composition, and encode. Rasterize complex vector and shaped-text layers through a deterministic Rust graphics backend, then feed those layers into the existing filter graph. Hide this behind a compositor interface so a GPU backend can replace it later without changing project files or MCP contracts.
4. **Make ordering and effect order normative.** Specify coordinate spaces, alpha mode, color space, transform order, mask order, effect order, and stable layer ordering in the core documentation and golden tests.
5. **Compile presets to primitives.** Persist the resolved keyframes/effects plus optional preset provenance. Rendering must never depend on an MCP-only preset implementation.
6. **Version contracts additively.** Bump the project schema once per independently shippable model milestone, migrate current state and retained history under the project lock, and reject future versions.

## Target canonical model

Names are illustrative; settle exact Rust names in the ADR and contract fixtures.

### Common visual properties

Every visual item should share:

```rust
struct VisualProperties {
    transform: Transform2D,
    parent_id: Option<String>,
    z_index: i32,
    blend_mode: BlendMode,
    clip: Option<Clip>,
    masks: Vec<Mask>,
    effects: Vec<Effect>,
    animation: AnimationSet,
    hidden: bool,
}

struct Transform2D {
    position: Position2D,      // x, y, and pixels|normalized units
    anchor: AnchorPoint,
    scale_x: f64,
    scale_y: f64,
    rotation_deg: f64,
    skew_x_deg: f64,
    skew_y_deg: f64,
    opacity: f64,
}
```

Compatibility defaults map the current transform to pixel coordinates, top-left anchor, equal X/Y scale, zero rotation/skew, and the existing opacity. Move text anchoring from `TextStyle` to the common transform with a one-version compatibility alias.

Normative transform order: source bounds/crop, local anchor translation, local scale/skew/rotation, local position, then ancestor transforms from nearest parent outward. Opacity multiplies through ancestors.

### Groups, components, and slots

```rust
struct ComponentDefinition {
    id: String,
    name: String,
    width: u32,
    height: u32,
    duration_ms: u64,
    tracks: Vec<Track>,
    markers: Vec<Marker>,
    slots: Vec<TemplateSlot>,
}

enum TemplateSlotKind {
    Text, Color, Number, Boolean, Asset, RichText, Enum, Duration,
}

struct ComponentInstanceItem {
    component_id: String,
    slot_values: Map<String, SlotValue>,
    start_ms: u64,
    duration_ms: u64,
    time_scale: f64,
    visual: VisualProperties,
}
```

- A `GroupItem` is a transform/effect node. Children point to it through `parent_id`; the group must not also maintain a second child list.
- Parent references are scoped to one root/component timeline, must form a DAG, and have a configured maximum depth.
- Component references must also form a DAG. Reject direct and indirect recursion with a stable validation error.
- Component-local item times are relative to the component. An instance maps them to root time using `start_ms`, trim, and `time_scale`.
- Slot bindings point to stable item properties, not arbitrary JSON paths. Validate type, required/default value, allowed range, and target existence when defining the component.
- Duplicating a component instance creates a new instance ID while retaining the definition reference; callers may replace slot values in the same operation.

### Shapes, SVG, grids, and repeaters

Add visual item variants for:

- `ShapeItem`: rectangle, rounded rectangle, ellipse, line, polygon, star, or Bézier path;
- `SvgItem`: sanitized inline SVG or a content-addressed SVG asset;
- `GridItem`: procedural rectangular, isometric, dot, or diagonal grid;
- `RepeaterItem`: references a shape, group, or component and applies transform/opacity/time offsets per copy.

Use reusable paint types:

```rust
enum Paint { Solid(Color), LinearGradient(...), RadialGradient(...) }
struct Stroke { paint: Paint, width: f64, dash: Vec<f64>, line_cap: ..., line_join: ... }
```

Paths use a validated command representation rather than accepting arbitrary renderer expressions. SVG ingestion must reject scripts, event handlers, external URLs, fonts outside configured roots, and unbounded node/path complexity. Repeater expansion must enforce a copy limit and be lazy in `EvaluatedScene` to avoid project-file explosions.

### Rich text

Replace the single-style text payload with a backwards-compatible `RichTextDocument`:

- UTF-8 text plus style spans indexed by grapheme boundaries;
- font family/path, weight, style, size, OpenType features, fill;
- ordered strokes, each with paint and width;
- one or more shadows with X/Y offset, blur, spread, and color;
- horizontal and vertical alignment;
- explicit anchor in the common transform;
- tracking, line height, paragraph spacing;
- maximum width and height;
- wrap mode and `none|shrink|fit_width|fit_box` auto-fit;
- optional background paint, padding, and corner radius.

Use a deterministic shaping/layout backend. Font resolution must record the actual font asset/hash used so reopening a project cannot silently change line breaks. Auto-fit is a bounded deterministic search and returns the resolved size in renderer diagnostics.

### Animation and time references

Expand animation into typed channels:

- transform: position X/Y, scale X/Y, rotation, skew, opacity, anchor where useful;
- media: crop rectangle, source position, playback rate;
- graphics: path points/trim, fill/stroke colors, stroke width, gradient stops;
- effects: blur/glow/tint/vignette/particle parameters;
- audio: gain, pan, and supported effect parameters.

Each keyframe has a typed value and a time expression:

```rust
enum TimeExpression {
    Milliseconds(u64),
    Marker { name: String, offset_ms: i64 },
}

enum Curve {
    Hold,
    Linear,
    CubicBezier { x1: f64, y1: f64, x2: f64, y2: f64 },
    Spring { mass: f64, stiffness: f64, damping: f64, initial_velocity: f64 },
}
```

Add repeat modes (`count`, `forever`, `ping_pong`) and group/component stagger settings. Presets such as `impact_slam`, `slide_left`, `scan`, `pulse`, and `radar_expand` are pure expansion functions with versioned preset IDs. Store preset version and resolved primitives so old projects do not change when a preset evolves.

Motion blur uses a documented shutter angle and sample count. The evaluator samples the fully inherited transform, including spring motion and parent movement.

### Masks, mattes, and effects

Define the layer pipeline precisely:

1. source decode or vector/text rasterization;
2. crop and local clipping;
3. item masks in declared order;
4. ordered item effects;
5. local and parent transforms;
6. track matte application;
7. opacity;
8. blend into the destination in stable z-order.

Support alpha/luma masks with add, subtract, intersect, and exclude operations; inversion, feather, expansion, transform, and path animation. A track matte references a sibling visual item in the same composition scope and must not create a dependency cycle.

Initial blend modes: normal, multiply, screen, overlay, add, darken, lighten. Initial effects: Gaussian/directional blur, glow, color tint, vignette, exposure/contrast/saturation, screen flash, and a bounded deterministic particle overlay. Effects are an ordered list; changing order is observable and undoable.

Perform compositing in linear light with premultiplied alpha, then convert to the configured output color space. Record and test this rule so preview and export cannot diverge.

### Markers, speech alignment, and semantic audio

```rust
struct Marker {
    id: String,
    name: String,
    time_ms: u64,
    kind: MarkerKind,
    source: Option<MarkerSource>,
}

struct SpeechAlignment {
    sentences: Vec<TimedText>,
    words: Vec<TimedText>,
    phonemes: Vec<TimedText>,
    quality: AlignmentQuality,
}
```

- Extend the provider-neutral speech contract so providers declare sentence/word/phoneme alignment support.
- Persist returned alignment in generated-asset provenance.
- For providers without word alignment, support a separate provider-neutral aligner; mark estimated/forced timings with their quality instead of presenting them as native.
- `tts_generate_and_insert` and speech commit accept a marker policy: none, sentences, selected words, or all words. Generated marker names must be unique and deterministic.
- Marker-relative keyframes and audio events resolve inside the relevant root/component scope. Missing or ambiguous marker names are validation failures.

Add a project-level semantic sound library:

```rust
struct SoundEventDefinition {
    event: String,
    variants: Vec<AssetId>,
    default_gain_db: f64,
    bus: AudioBusId,
}

struct AudioEventItem {
    event: String,
    at: TimeExpression,
    gain_db: f64,
    variant_seed: u64,
}
```

Variant choice is deterministic from the saved seed. Missing event definitions fail before commit. Provide buses for voiceover, music, sound effects, and master, with dB gain, pan, EQ, compression, side-chain ducking, true-peak limiting, and loudness normalization. Store normalization targets (for example integrated LUFS and true peak), not one-off filter strings.

## Public API plan

Keep low-level operations available, but make scene construction concise. Tool names below are the intended agent-facing surface.

### Composition and ordering

- `group_create`, `group_set_parent`, `group_ungroup`
- `component_create`, `component_define_slots`, `composition_instantiate_template`
- `component_duplicate_instance` with slot overrides
- `track_create`, `track_reorder` (clear alias over the existing indexed update)
- `item_set_z_index`, `item_reparent`

### Graphics and text

- `timeline_add_shape`
- `timeline_add_svg`
- `timeline_add_grid`
- `timeline_add_repeater`
- extend `timeline_add_text` and `timeline_update_item` with rich text while preserving simple text inputs

### Animation and compositing

- extend `timeline_set_keyframes` with typed channels, marker times, Bézier/spring curves, and loop settings
- `timeline_apply_animation_preset`
- `timeline_set_masks`, `timeline_set_matte`, `timeline_set_effects`, `timeline_set_blend_mode`

### Timing and audio

- `marker_create`, `marker_update`, `marker_delete`, `marker_list`
- extend speech tools with `alignment`, `markerPolicy`, and alignment capability reporting
- `sound_event_register`, `timeline_add_audio_event`
- `audio_set_bus`, `audio_analyze`, `audio_normalize`

### Atomic workflows and preview

- Keep `timeline_batch_edit` as the canonical generic transaction and add `timeline_apply_batch` as a documented compatibility alias only if the requested name materially helps clients.
- Every new mutation is a valid batch operation and supports `resultAlias` for later references.
- `composition_instantiate_template` is one core operation even if it creates many resolved nodes; it returns the instance ID and created aliases in one revision.
- Keep `preview_render_range`; add `resolutionPreset: 540p|720p|project`, retain custom dimensions, default `includeAudio` to true for audiovisual review, and key cache entries by project ID, revision, range, dimensions, FPS, and audio option.
- Jobs return artifact metadata/resource links by default and never inline binary data unless explicitly requested.

## Deterministic stacking contract

Make the following rule public and cover it with tests:

1. Root tracks are ordered bottom-to-top by their array/index order.
2. Within a track and overlapping time, lower `z_index` renders first.
3. Equal z-index values use stable item creation order; persist a `stack_order` value so IDs are not treated as visual intent.
4. Group children retain their track/z ordering, then the flattened group is composited at the group's parent position.
5. A component instance is one root layer; its internal tracks follow the same rules recursively.
6. Mattes are dependency inputs, not visible ordering exceptions; hidden matte sources remain available only when explicitly configured as matte-only.
7. Audio track order never changes summing semantics; audio buses and explicit routing do.

## Milestones

Effort ranges are engineering estimates, not calendar commitments. They assume one engineer familiar with the repository; renderer and speech/audio work can run in parallel after Milestone 1.

### Milestone 0 — contracts, evaluator seam, and benchmarks (1–2 engineer-weeks)

- Write the ADR and JSON contract fixtures for transforms, layers, components, markers, effects, and curves.
- Add a renderer-neutral evaluated-scene module and route current media/text/rectangle rendering through it without changing output.
- Capture baseline golden frames, a short A/V fixture, filter-graph snapshots, render duration, and peak memory.
- Add feature/capability version reporting so clients can detect `sceneGraph`, `richText`, `vectorShapes`, and related levels.

Exit gate: existing projects render pixel-equivalently, existing MCP tests pass unchanged, and the evaluator can describe the current flat scene.

### Milestone 1 — scene graph, explicit stacking, groups, and components (3–5 engineer-weeks)

- Schema migration: common visual properties, persisted stack order/z-index, groups, component definitions, instances, and slots.
- Core validation for scope, parent/component DAGs, duration mapping, slot targets/types, and depth/item limits.
- Add create/update/duplicate/reparent/reorder operations and batch alias resolution.
- Flatten nested instances and inherited transforms into the evaluated scene.
- Add minimal desktop tree/timeline visualization and inspector controls for parent and z-index.

Exit gate: build one rule-card component with at least six child layers, instantiate it three times with different text/number/icon slots, move the parent once, and verify all children move in still preview, range preview, export, undo, redo, and reopen.

### Milestone 2 — vector graphics and rich typography (4–6 engineer-weeks)

- Add shape, SVG, grid, and repeater models and MCP operations.
- Add paint/stroke/gradient/dash/corner-radius support and sanitized SVG ingestion.
- Integrate deterministic text shaping, font recording, span styles, layered strokes/shadows, tracking, line height, bounds, and auto-fit.
- Implement raster caches keyed by content/style/font hash/scale and invalidate only affected layers.
- Add minimal vector and rich-text inspector controls.

Exit gate: reproduce the static rules-screen frame without generated image panels: outlined cards, accent bars, corner brackets, diagonal grid, concentric circles, and layered `EVERY./SINGLE./ONE.` text all render consistently at project, 720p, and 540p sizes.

### Milestone 3 — animation engine and presets (4–6 engineer-weeks)

- Add typed channels, independent scale, rotation, crop, Bézier/spring curves, loops, and marker time expressions.
- Implement parent-aware evaluation, repeaters with time offsets, group/component stagger, and motion-blur sampling.
- Add versioned preset expansion for `impact_slam`, `slide_left`, `scan`, `pulse`, and `radar_expand`.
- Update split/trim/duplicate logic for every channel and marker-relative keyframe. Define whether split resolves or preserves marker references and test it.

Exit gate: one `impact_slam` operation produces overshoot, shake, flash, and motion blur; a looping grid and expanding radar rings remain seamless across a range boundary; split/undo/reopen preserve identical evaluated values.

### Milestone 4 — masks, mattes, blend modes, and effects (4–7 engineer-weeks)

- Implement the normative layer pipeline, mask/matte DAG validation, blend modes, and ordered effects.
- Add animated mask paths/feather/expansion and reusable clipping on groups/components.
- Implement vignette, blur, glow, tint, flash, and bounded particle overlay.
- Add color/alpha conformance fixtures and preview/export parity tests.

Exit gate: implement a framed creature reveal using a wipe or matte, flash, glow, tint, and particles, with no externally precomposited graphics; sampled frames from range preview and final export match within the documented tolerance.

### Milestone 5 — narration alignment, markers, and designed audio (4–6 engineer-weeks)

- Extend speech provider contracts and generated provenance with alignment and quality.
- Add marker CRUD, marker-relative timing, marker policies on speech commit, and an optional alignment adapter for providers lacking timestamps.
- Add semantic sound libraries/events and deterministic variant selection.
- Add audio buses, dB controls, pan, EQ, compression, side-chain ducking, two-pass loudness normalization, true-peak limiting, waveform/peak/LUFS analysis.
- Preserve current simple volume/fade and role-based ducking as migrated defaults.

Exit gate: speech generation creates named `EVERY`, `SINGLE`, `ONE`, rule, `Starting with`, and `Venusaur` markers; visual presets and audio events bind to those names; the exported mix meets the configured LUFS and true-peak tolerances and narration remains intelligible.

### Milestone 6 — audiovisual review, hardening, and release (2–4 engineer-weeks)

- Add preview presets, render caching, stale-revision protection, cancellation cleanup, and artifact retention limits.
- Add an agent workflow prompt that constructs the complete reference scene mostly through one batch plus one template instantiation per card.
- Complete desktop selection, hierarchy, markers, masks/effects, audio-bus meters, and preview controls needed to inspect agent-created projects.
- Run compatibility, security, stress, packaging, and cross-platform verification; publish schemas, examples, and migration notes.

Exit gate: the end-to-end reference project can be created from an empty project, reviewed as a 540p A/V range, adjusted by marker/slot name, exported, reopened, undone/redone, and rendered on every supported platform.

## Workstream and dependency order

```text
Contracts + EvaluatedScene
          |
          v
Scene graph + stacking + components
       /          |             \
      v           v              v
Vector/text   Animation       Markers/speech contract
      \           |              /
       \          v             /
        +---- Compositing ------+
                    |
                    v
             Audio events/mix
                    |
                    v
          Preview/release hardening
```

After Milestone 1, vector/text, animation primitives, and speech alignment contracts can proceed in parallel, but masks/compositing should not ship before the evaluated-scene and stacking contracts are stable.

## Migration and compatibility plan

- Use incremental schema versions rather than one large migration. Suggested boundaries: v7 common visuals/stacking; v8 components; v9 graphics/rich text/effects; v10 markers/alignment/audio buses.
- Migrate every current item to explicit default `VisualProperties` and `stack_order` based on current track/item order, preserving current pixels.
- Preserve simple text request fields as sugar that constructs a one-span rich-text document. Continue returning compatibility fields for one deprecation cycle if consumers depend on them.
- Preserve current `scale` as an alias that sets both axes. Reject requests that provide both `scale` and conflicting `scaleX`/`scaleY`.
- Preserve existing track indexed update; `track_reorder` calls the same core operation.
- Preserve `timeline_batch_edit`; do not create a second transaction engine.
- Migrate both `project.json` and every undo/redo snapshot atomically under the existing project lock. Add fixtures for v1, v6, and each new intermediate version.
- Unknown future versions remain a hard error.

## Validation and safety limits

Set explicit configurable limits and stable error codes for:

- maximum group/component nesting depth;
- maximum component instances and expanded render nodes per frame;
- maximum repeater copies and particles;
- maximum SVG bytes, nodes, path commands, gradient stops, and text spans;
- component/matte/parent cycles;
- invalid or ambiguous marker references;
- missing font, asset, component, slot, sound event, or matte target;
- non-finite transform, curve, effect, and audio values;
- render-plan size and preview duration/resolution.

No SVG or template input may execute scripts, read arbitrary paths, resolve network resources, or inject raw FFmpeg expressions.

## Test strategy

### Core and migration

- serialization and validation for every new type;
- parent/component/matte cycle rejection and depth limits;
- slot type/default/override behavior;
- stable z-order and transform inheritance;
- keyframe interpolation, spring determinism, marker offsets, loops, stagger, split, trim, duplicate;
- schema and history migrations, undo/redo, atomic rollback, alias resolution.

### Renderer

- golden PNGs for shape, text, transform, mask, blend, and effect cases;
- short temporal fixtures that sample first/middle/last and overshoot frames;
- alpha/color conformance cases;
- preview-frame, preview-range, and export parity at the same timestamp;
- cache invalidation tests that prove edits do not reuse stale layers;
- malformed/oversized SVG and resource-limit tests.

### Audio

- deterministic event variant and marker placement;
- ducking attack/release and bus routing;
- measured gain, pan, EQ/compression behavior;
- LUFS and true-peak tolerance after normalization/limiting;
- A/V sync across preview and export.

### Contracts and end to end

- Rust/TypeScript/provider contract fixtures must agree;
- every low-level and high-level operation through headless and MCP;
- one full reference-scene fixture demonstrating all ten capability groups;
- packaged fake-provider smoke test and real-provider opt-in test;
- Windows, macOS, and Linux renderer feature detection and fallback behavior.

The repository-wide completion gate remains `cargo fmt --check --all`, strict workspace Clippy, workspace Rust tests, TypeScript typecheck/lint/unit/integration/smoke tests, and relevant hermetic Python provider tests.

## Performance and observability

- Establish benchmark scenes before implementation: static rules screen, animated typography, masked reveal, and mixed-audio sequence.
- Report scene-evaluation, rasterization, filter-graph construction, decode, composite, encode, and audio-analysis timings separately.
- Cache component flattening, shaped text, static vector rasterization, and range-preview artifacts by content hash and revision.
- Stream renderer progress by frames/time and include warnings for fallbacks, missing optional features, auto-fit results, and alignment quality.
- Add counters for expanded nodes, raster cache hit rate, motion-blur samples, audio passes, and peak working-set memory.

Do not set a release performance target until baseline measurements exist. Milestone 0 should turn the measurements into platform-specific budgets; each later milestone must stay within them or document an approved regression.

## Release slicing

Keep incomplete model features capability-gated. A sensible external release sequence is:

1. **Composition alpha:** explicit z-order, groups, components, slots, shapes, and static rich text.
2. **Motion beta:** advanced curves, presets, loops/stagger, marker timing, and A/V range-preview workflow.
3. **Compositing beta:** masks, mattes, blend modes, effects, and motion blur.
4. **Audio beta:** speech alignment, semantic events, buses, normalization, limiting, and analysis.
5. **Motion-graphics stable:** complete reference-scene fixture, migrations, desktop inspection, performance budgets, and cross-platform packaging.

## Definition of done for the initiative

The initiative is complete only when a clean project can be built through documented MCP calls into this scene:

- a moving diagonal grid and decorative vector frame;
- three or more instances of one reusable rule-card component with different slot content;
- layered outlined typography driven by a versioned impact preset;
- parented, staggered card entrances plus looping scan/radar animation;
- a masked hero reveal with blend/effect stack and particles;
- narration-generated named markers used by both visuals and semantic sound events;
- bus-routed audio with ducking, normalization, limiting, and reported LUFS/true peak;
- a cached 540p/720p audiovisual range preview whose sampled output matches final export;
- deterministic reopen, duplicate-with-overrides, undo, redo, migration, and cross-platform rendering.

That fixture should remain in the repository as the primary integration and regression test for OpenCut's motion-graphics API.

## GitHub execution backlog

The implementation is tracked in `matiHirCab/AI-Open-Cut` through seven milestones, seven epics, and seventy-one leaf issues. Stable `MG-*` keys are the idempotency boundary for backlog automation: a reconciliation run must update an exact key match rather than create a duplicate.

### Repository organization

- Milestones: `MG M0 — Foundations` through `MG M6 — Preview & Release`, with no due dates.
- Common labels: `enhancement` and `initiative:motion-graphics`.
- Epic label: `type:epic`.
- Area labels: `area:core`, `area:renderer`, `area:agent-api`, `area:desktop`, `area:audio-speech`, and `area:quality`.
- Initial assignee: `matiHirCab`.
- Hierarchy: every leaf is a native GitHub sub-issue and an ordered checklist entry in its milestone epic.

Each leaf issue contains Problem, Intended outcome, Implementation requirements, Public contracts, Acceptance criteria, Dependencies, and Verification sections. Dependencies use issue links rather than symbolic keys once the backlog is published. Every public model change remains additive or includes a deterministic migration for current state and retained undo/redo history.

### Issue registry

#### M0 — Foundations

| Key | Deliverable | GitHub issue |
| --- | --- | --- |
| MG-M0 | Establish motion-graphics contracts and rendering foundations | [#3](https://github.com/matiHirCab/AI-Open-Cut/issues/3) |
| MG-M0-01 | Record the motion-graphics architecture ADR | [#10](https://github.com/matiHirCab/AI-Open-Cut/issues/10) |
| MG-M0-02 | Add canonical motion-graphics contract fixtures | [#11](https://github.com/matiHirCab/AI-Open-Cut/issues/11) |
| MG-M0-03 | Implement the renderer-neutral EvaluatedScene model | [#12](https://github.com/matiHirCab/AI-Open-Cut/issues/12) |
| MG-M0-04 | Route all render entry points through EvaluatedScene | [#13](https://github.com/matiHirCab/AI-Open-Cut/issues/13) |
| MG-M0-05 | Establish golden visual and audiovisual fixtures | [#14](https://github.com/matiHirCab/AI-Open-Cut/issues/14) |
| MG-M0-06 | Add renderer benchmarks and stage-level observability | [#15](https://github.com/matiHirCab/AI-Open-Cut/issues/15) |
| MG-M0-07 | Add contract and render-parity CI gates | [#16](https://github.com/matiHirCab/AI-Open-Cut/issues/16) |

#### M1 — Scene Graph & Components

| Key | Deliverable | GitHub issue |
| --- | --- | --- |
| MG-M1 | Add deterministic scene graphs and reusable components | [#4](https://github.com/matiHirCab/AI-Open-Cut/issues/4) |
| MG-M1-01 | Migrate projects to common VisualProperties | [#17](https://github.com/matiHirCab/AI-Open-Cut/issues/17) |
| MG-M1-02 | Implement Transform2D and documented coordinate semantics | [#18](https://github.com/matiHirCab/AI-Open-Cut/issues/18) |
| MG-M1-03 | Implement deterministic stacking and explicit z-index | [#19](https://github.com/matiHirCab/AI-Open-Cut/issues/19) |
| MG-M1-04 | Add GroupItem and parent-DAG validation | [#20](https://github.com/matiHirCab/AI-Open-Cut/issues/20) |
| MG-M1-05 | Expose group and parenting operations through headless and MCP | [#21](https://github.com/matiHirCab/AI-Open-Cut/issues/21) |
| MG-M1-06 | Add component definitions and nested composition validation | [#22](https://github.com/matiHirCab/AI-Open-Cut/issues/22) |
| MG-M1-07 | Implement typed template slots and bindings | [#23](https://github.com/matiHirCab/AI-Open-Cut/issues/23) |
| MG-M1-08 | Evaluate component instances and local time mapping | [#24](https://github.com/matiHirCab/AI-Open-Cut/issues/24) |
| MG-M1-09 | Add atomic component lifecycle and instantiation APIs | [#25](https://github.com/matiHirCab/AI-Open-Cut/issues/25) |
| MG-M1-10 | Add hierarchy inspection and the reusable rule-card fixture | [#26](https://github.com/matiHirCab/AI-Open-Cut/issues/26) |

#### M2 — Vector Graphics & Typography

| Key | Deliverable | GitHub issue |
| --- | --- | --- |
| MG-M2 | Add native vector graphics and production typography | [#5](https://github.com/matiHirCab/AI-Open-Cut/issues/5) |
| MG-M2-01 | Add vector paint, stroke, gradient, and path primitives | [#27](https://github.com/matiHirCab/AI-Open-Cut/issues/27) |
| MG-M2-02 | Add ShapeItem rendering and timeline_add_shape | [#28](https://github.com/matiHirCab/AI-Open-Cut/issues/28) |
| MG-M2-03 | Add secure SVG ingestion and timeline_add_svg | [#29](https://github.com/matiHirCab/AI-Open-Cut/issues/29) |
| MG-M2-04 | Add procedural grids and timeline_add_grid | [#30](https://github.com/matiHirCab/AI-Open-Cut/issues/30) |
| MG-M2-05 | Add lazy repeaters and timeline_add_repeater | [#31](https://github.com/matiHirCab/AI-Open-Cut/issues/31) |
| MG-M2-06 | Introduce RichTextDocument with compatibility migration | [#32](https://github.com/matiHirCab/AI-Open-Cut/issues/32) |
| MG-M2-07 | Add content-addressed font resolution and deterministic shaping | [#33](https://github.com/matiHirCab/AI-Open-Cut/issues/33) |
| MG-M2-08 | Add styled spans, layered strokes, and blurred shadows | [#34](https://github.com/matiHirCab/AI-Open-Cut/issues/34) |
| MG-M2-09 | Add advanced text layout and deterministic auto-fit | [#35](https://github.com/matiHirCab/AI-Open-Cut/issues/35) |
| MG-M2-10 | Add text and vector raster caches | [#36](https://github.com/matiHirCab/AI-Open-Cut/issues/36) |
| MG-M2-11 | Add vector/text inspectors and the static rules-screen fixture | [#37](https://github.com/matiHirCab/AI-Open-Cut/issues/37) |

#### M3 — Animation

| Key | Deliverable | GitHub issue |
| --- | --- | --- |
| MG-M3 | Add professional animation controls and presets | [#6](https://github.com/matiHirCab/AI-Open-Cut/issues/6) |
| MG-M3-01 | Add typed animation channels and keyframe values | [#38](https://github.com/matiHirCab/AI-Open-Cut/issues/38) |
| MG-M3-02 | Add cubic Bézier and deterministic spring curves | [#39](https://github.com/matiHirCab/AI-Open-Cut/issues/39) |
| MG-M3-03 | Add marker CRUD and marker-relative time expressions | [#40](https://github.com/matiHirCab/AI-Open-Cut/issues/40) |
| MG-M3-04 | Add finite, infinite, and ping-pong animation loops | [#41](https://github.com/matiHirCab/AI-Open-Cut/issues/41) |
| MG-M3-05 | Add inherited animation, stagger, and repeater time offsets | [#42](https://github.com/matiHirCab/AI-Open-Cut/issues/42) |
| MG-M3-06 | Render rotation, crop, path, gradient, and effect animation | [#43](https://github.com/matiHirCab/AI-Open-Cut/issues/43) |
| MG-M3-07 | Add motion-blur sampling | [#44](https://github.com/matiHirCab/AI-Open-Cut/issues/44) |
| MG-M3-08 | Add a versioned animation-preset compiler | [#45](https://github.com/matiHirCab/AI-Open-Cut/issues/45) |
| MG-M3-09 | Ship the initial motion preset pack | [#46](https://github.com/matiHirCab/AI-Open-Cut/issues/46) |
| MG-M3-10 | Preserve animation semantics through split, trim, and duplicate | [#47](https://github.com/matiHirCab/AI-Open-Cut/issues/47) |
| MG-M3-11 | Add animation inspection and temporal regression fixtures | [#48](https://github.com/matiHirCab/AI-Open-Cut/issues/48) |

#### M4 — Compositing

| Key | Deliverable | GitHub issue |
| --- | --- | --- |
| MG-M4 | Add masks, mattes, blend modes, and effects | [#7](https://github.com/matiHirCab/AI-Open-Cut/issues/7) |
| MG-M4-01 | Implement the normative linear-light compositing pipeline | [#49](https://github.com/matiHirCab/AI-Open-Cut/issues/49) |
| MG-M4-02 | Add mask models and validation | [#50](https://github.com/matiHirCab/AI-Open-Cut/issues/50) |
| MG-M4-03 | Render static and animated masks | [#51](https://github.com/matiHirCab/AI-Open-Cut/issues/51) |
| MG-M4-04 | Add track mattes and dependency-DAG validation | [#52](https://github.com/matiHirCab/AI-Open-Cut/issues/52) |
| MG-M4-05 | Add the initial blend-mode set | [#53](https://github.com/matiHirCab/AI-Open-Cut/issues/53) |
| MG-M4-06 | Add ordered effect stacks | [#54](https://github.com/matiHirCab/AI-Open-Cut/issues/54) |
| MG-M4-07 | Implement blur, glow, tint, vignette, and color controls | [#55](https://github.com/matiHirCab/AI-Open-Cut/issues/55) |
| MG-M4-08 | Implement screen flashes, particles, and group clipping | [#56](https://github.com/matiHirCab/AI-Open-Cut/issues/56) |
| MG-M4-09 | Expose compositing APIs and desktop controls | [#57](https://github.com/matiHirCab/AI-Open-Cut/issues/57) |
| MG-M4-10 | Add the masked hero-reveal parity fixture | [#58](https://github.com/matiHirCab/AI-Open-Cut/issues/58) |

#### M5 — Narration & Audio

| Key | Deliverable | GitHub issue |
| --- | --- | --- |
| MG-M5 | Add narration alignment and designed audio events | [#8](https://github.com/matiHirCab/AI-Open-Cut/issues/8) |
| MG-M5-01 | Extend speech-provider contracts with alignment capabilities | [#59](https://github.com/matiHirCab/AI-Open-Cut/issues/59) |
| MG-M5-02 | Persist speech alignment and quality provenance | [#60](https://github.com/matiHirCab/AI-Open-Cut/issues/60) |
| MG-M5-03 | Add a provider-neutral forced-alignment service | [#61](https://github.com/matiHirCab/AI-Open-Cut/issues/61) |
| MG-M5-04 | Generate timeline markers from speech alignment | [#62](https://github.com/matiHirCab/AI-Open-Cut/issues/62) |
| MG-M5-05 | Add semantic sound-event definitions | [#63](https://github.com/matiHirCab/AI-Open-Cut/issues/63) |
| MG-M5-06 | Add timeline audio events and marker placement APIs | [#64](https://github.com/matiHirCab/AI-Open-Cut/issues/64) |
| MG-M5-07 | Add project audio buses and schema migration | [#65](https://github.com/matiHirCab/AI-Open-Cut/issues/65) |
| MG-M5-08 | Render bus gain, pan, EQ, and compression | [#66](https://github.com/matiHirCab/AI-Open-Cut/issues/66) |
| MG-M5-09 | Upgrade narration side-chain ducking | [#67](https://github.com/matiHirCab/AI-Open-Cut/issues/67) |
| MG-M5-10 | Add waveform, peak, true-peak, and LUFS analysis | [#68](https://github.com/matiHirCab/AI-Open-Cut/issues/68) |
| MG-M5-11 | Add two-pass loudness normalization and limiting | [#69](https://github.com/matiHirCab/AI-Open-Cut/issues/69) |
| MG-M5-12 | Add marker/audio inspection and the narration-driven fixture | [#70](https://github.com/matiHirCab/AI-Open-Cut/issues/70) |

#### M6 — Preview & Release

| Key | Deliverable | GitHub issue |
| --- | --- | --- |
| MG-M6 | Harden audiovisual review and ship motion graphics | [#9](https://github.com/matiHirCab/AI-Open-Cut/issues/9) |
| MG-M6-01 | Add 540p, 720p, and project preview presets | [#71](https://github.com/matiHirCab/AI-Open-Cut/issues/71) |
| MG-M6-02 | Add revision-keyed audiovisual preview caching | [#72](https://github.com/matiHirCab/AI-Open-Cut/issues/72) |
| MG-M6-03 | Harden preview cancellation, stale revisions, and retention | [#73](https://github.com/matiHirCab/AI-Open-Cut/issues/73) |
| MG-M6-04 | Return artifact metadata and resource links by default | [#74](https://github.com/matiHirCab/AI-Open-Cut/issues/74) |
| MG-M6-05 | Add the complete motion-graphics agent workflow prompt | [#75](https://github.com/matiHirCab/AI-Open-Cut/issues/75) |
| MG-M6-06 | Consolidate desktop review surfaces | [#76](https://github.com/matiHirCab/AI-Open-Cut/issues/76) |
| MG-M6-07 | Add the complete reference-scene end-to-end fixture | [#77](https://github.com/matiHirCab/AI-Open-Cut/issues/77) |
| MG-M6-08 | Add cross-platform renderer detection and packaging | [#78](https://github.com/matiHirCab/AI-Open-Cut/issues/78) |
| MG-M6-09 | Add security, stress, and performance release gates | [#79](https://github.com/matiHirCab/AI-Open-Cut/issues/79) |
| MG-M6-10 | Publish API, migration, example, and release documentation | [#80](https://github.com/matiHirCab/AI-Open-Cut/issues/80) |
