## Context

Issue #42 follows typed channels (#38), component local clocks (#24), repeaters (#31), and loops (#41). Active visual channels currently reject group and component-instance targets. Group children use absolute containing-scope time; component children use affine mapped local time. Repeaters generate additional visual occurrences with identical source clocks. The living repeater and group requirements explicitly prohibit time shifts, so this change must update them rather than layer an undocumented renderer effect.

## Goals / Non-Goals

**Goals:** One editor-core owned temporal and transform composition for animated group and component parents, deterministic child stagger, and time-shifted repeater copies; additive typed editing; strict bounded evaluation; atomic schema migration; parity across frame, range, draft and export.

**Non-Goals:** Activate deferred channels, animate audio parents, repeat audio, change unrelated media time stretching, add expressions or external resources, alter static hierarchy ordering, or define presets/motion blur.

## Decisions

1. **Closed persisted fields and edit surface.** Add optional `staggerMs` to group and component-instance items and optional `timeOffsetMs` to the repeater descriptor. Use zero as the missing-field behavior. `staggerMs` is a nonnegative integer in [0, 60000]; `timeOffsetMs` is a signed integer in [-60000, 60000]. Extend AddGroup and AddComponentInstance creation, typed update_item for group/instance stagger, component_instance_update for timing replacement, component duplication, repeater create/descriptor replacement, and set_animation_channels. update_item retains its closed typed field rules; no new transport command is required. A new versioned project schema makes source-version validation unambiguous. Alternative considered: polymorphic time-expression or floating offset fields; rejected because they admit ambiguous rounding and exceed this milestone's closed vocabulary.

2. **Occurrence clock composition.** A group delays each direct visual child's entire occurrence branch by `rank * staggerMs` in its containing-scope milliseconds. A component instance applies the same rule to direct top-level visual children in the definition's local milliseconds after its affine clock mapping. Ranks are zero-based in existing canonical track/z-index/stack-order/ID order across the containing scope; hidden or clipped children retain their ranks. Direct audio-only, caption and transition items do not take part. The delayed branch's item activity, media source sampling, channel phase, loop phase, and nested controllers use the shifted clock; parent visibility and half-open interval are evaluated on the unshifted parent clock. Composition uses finite affine clocks without intermediate integer rounding, and intersected output remains half-open. Alternative considered: delay channel phase alone; rejected because it leaves static entrances and media sampling inconsistent with child timing.

3. **Repeater copy clocks.** Additional copy `i` evaluates the complete source subtree at containing-scope time `t - i * timeOffsetMs`, including nested component and group clocks; ordinary source uses `i=0` and is unchanged. Positive offsets delay copies, negative offsets advance them. Copy activity must intersect the unshifted repeater and ancestor intervals; source activity is tested on shifted time. Existing transform and opacity powers, occurrence order, identities, visual-only closure, and copy bounds remain intact. Alternative considered: shift persisted source start values or materialize copies; rejected because source editing, history, and lazy expansion would diverge.

4. **Parent animation.** Activate only the already implemented visual scalar properties (position X/Y, scale X/Y, opacity) for groups and component instances without `transform2d`. A parent samples its channel at its own item-local time before child clock offsets. Matrix and opacity inheritance use the existing outer-to-inner hierarchy and typed curve/loop sampler; no new transform property is activated. Alternative considered: add a separate parent animation model; rejected because it would duplicate channel validation and curve semantics.

5. **Preflight, failure, and compatibility.** Core validates complete retained scopes, hidden content, referenced closures, rank multiplication, shifted clock bounds, composed matrices, and expanded fact budgets before generated materialization or artifact I/O. Missing references keep their established precedence; malformed values, unsafe derived clocks, and complexity excess return non-retryable `INVALID_ARGUMENT`. A stale revision remains retryable `REVISION_CONFLICT`. Old projects use zero defaults and render identically. Additive public fields and capability reporting are governed by canonical fixtures and all declared consumers. Alternative considered: relying on renderer clipping or transport validation; rejected because it breaks atomicity and cross-intent equality.

## Risks / Trade-offs

- **Large nested offsets can move source time beyond valid intervals** → checked affine composition and bounded retained-domain preflight reject non-finite or unsafe derived values before work.
- **Hidden siblings can affect rank** → rank includes hidden direct visual children and is documented, making later visibility edits non-reordering.
- **Repeater expansion can multiply timing work** → retain existing 256-copy and aggregate occurrence/fact limits and project complete shifted closures before cloning.
- **Migration changes the persisted schema** → migrate current state and every retained undo/redo snapshot under the project lock with fault-injection and reopen tests; older binaries fail closed.

## Migration Plan

Advance schema 25 to 26. For each supported source snapshot, reject fields that are forbidden by its source version, then add zero/default-omitted timing fields and update the label in one recoverable project/history transaction. Preserve IDs, revisions, media ownership and bytes, channels, and rendered output for missing fields. A failure leaves the prior generation authoritative; reopening a valid schema-26 project performs no rewrite. Rollback means reopening the old durable generation after interrupted publication, not a downgrade of schema 26.

## Open Questions

None for implementation after the proposal and delta requirements are explicitly approved.


## Implementation details verified against the approved decisions

- EvaluatedScene records outer-to-inner inherited stages with shared typed channel facts and each controller's own affine clock. Render planning compiles inverses with bounded scratch registers 4–7; the existing parameterized curve compiler retains registers 0–3 and its clamping/loop semantics. Dynamic geometry samples an output-canvas region and uses conservative inherited scale bounds for retained raster preflight, without generating frame facts.
- A separate audio clock follows existing component retiming without visual branch delays. Occurrence interval paths keep external ancestors and repeater activity unshifted while translating source-subtree intervals; this allows shifted sources to enter a clip without escaping it.
- Animated shape/media sampling normalizes its input cadence to the scene FPS so coordinate maps and opacity share the same composition timestamp. Existing zero-offset, static paths retain their prior filter graph.
- Group creation keeps its historical identity transform2d default. Aliased batches clear that field before setting active channels, preserving old simple-operation behavior and transform compatibility rules.
