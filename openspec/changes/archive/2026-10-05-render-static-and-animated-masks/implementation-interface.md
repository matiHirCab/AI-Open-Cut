# Proposed issue51 implementation interface

Status: external planning only, not implementation authorization. Base is verified issue50 commit 62c30eb224d36e0b0a73c88362e4423555a084f7. Reconcile this interface against promoted approved artifacts before coding. No repository edits accompany this plan.

## Existing APIs and constraints

* `render_artifact/extended_visual.rs::Raster` is private to that file: width/height plus `Vec<[f32;4]>` linear premultiplied pixels, bounded `empty`, source decode, crop, bilinear, padding and Gaussian effect methods. Keep it private; pass its mutable pixel slice to the mask helper. No RGBA8 conversion, new public Raster type or duplicate source buffer is needed.
* `render_artifact/shapes.rs::paint_at(&Paint, VectorPoint)->[f64;4]` is already `pub(super)` and returns floating premultiplied linear paint using canonical gradient search. Pure mask raster can call it as a sibling. Existing `coverage` is private and fills two tiny-skia pixmaps for fill/stroke. Its exact contour/fillRule/AA behavior is reusable; its whole `rasterize` returns encoded PAM and is unsuitable for mask paint.
* `evaluated_scene/shapes.rs::Contour` is crate-private with `points` and `closed`. `EvaluatedShape::new` compiles and derives analytic bounds at tolerance0.25/density; `segments()` returns charged work. Private `compile_contours` and `with_budget` are the real contour owners. Ordinary shape raster padding is not the mask support padding. A narrow dedicated compiler wrapper is needed, rather than constructing a stroked/painted `EvaluatedShape` and adopting its incidental padding.
* `evaluated_scene/extended_visual.rs::sample` currently returns `(sampled_layer,crop,effects)`, with `SampleTime::local` retaining inherited clocks. `prepare` rasterizes each source, determines owner affine, applies crop, then effects, then inverse owner/ancestor affine sampling and opacity/transition. Insert mask multiplication immediately after crop and before effects; preserve owner affine calculations, original shape bounds and existing effect support.
* `evaluated_scene::EvaluatedAffine` contains matrix/inverse and opacity. `transform_matrices_logical` owns canonical transform order. Mask anchor differs: analytic unexpanded path bounds, including bounds origin, rather than padded raster dimensions. Main will expose a narrow evaluated mask affine builder using existing matrix utilities; pure raster will consume its certified inverse, not reinterpret Transform2D.
* `extended_certification` owns left-first coupled interval certification and shared65536nodes; `extended_visual::certify_composition_memory` owns current1GiB aggregate memory/cache accounting. Extend these owning functions. Do not place authored/domain validation in render_artifact or transport.

## Disjoint source ownership after approval

Main owns model/animation enums/guards, schema33 migration/store, validation/extended_visual, evaluated_scene/extended_visual and extended_certification, new evaluated_scene/masks.rs, evaluated_scene/shapes.rs compiler wrapper, render_artifact.rs module declaration, render_artifact/shapes.rs shared fill helper, render_artifact/extended_visual.rs integration, all other existing core tests.

Research owns only new `render_artifact/masks.rs` and its new sibling test module `render_artifact/masks/tests.rs`. These contain scalar fill/EDT/channel/Gaussian/bilinear/combination/application and independent numeric tests. Research does not edit model, evaluated_scene, existing shapes/extended_visual files or native integration files without separate reservation. Canonical catalog/transport/docs helper retains its existing public-layer ownership. Reviewer remains read-only.

Main publishes evaluated structs and two shared helper signatures first, then sends a stable interface to research. Until those exist, research may prepare independent oracle data/tests outside the repository. No concurrent edits to shared files.

## Main-owned evaluated facts (proposed exact internal surface)

```rust
// evaluated_scene/masks.rs; pub(crate), not persisted/public/Serde.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct MaskOwnerBasis {
    pub size: (u32,u32),       // exact post-crop/clip source raster W,H
    pub density: f64,         // source raster pixels per local project pixel
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MaskGrid {
    pub contours: Vec<Contour>,
    pub fill_rule: FillRule,
    pub analytic_bounds: [f64;4], // sampled, unexpanded, before affine/padding
    pub origin: (f64,f64),     // outward aligned local geometry coordinates
    pub size: (u32,u32),
    pub density: f64,          // rho, no integer rounding
    pub segments: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct EvaluatedMask {
    pub id: String,
    pub grid: Option<MaskGrid>, // None iff analytically provable no drawable fill
    pub paint: Paint,
    pub channel: MaskChannel,
    pub operation: MaskOperation,
    pub inverted: bool,
    pub inverse: [f64;6],      // owner-local project coordinates -> path coordinates
    pub opacity: f64,          // sampled mask transform opacity, applied once
    pub expansion_grid: f64,   // sampled expansionPx*rho
    pub feather_grid: f64,     // sampled featherPx*rho
    pub certified_scratch_bytes: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct EvaluatedMaskStack {
    pub owner: MaskOwnerBasis,
    pub masks: Vec<EvaluatedMask>, // authored order, sampled once
    pub work_units: u64,
    pub accumulated_bytes: u64, //4*W*H
    pub retained_fact_bytes: u64, //64*sumS + separately checked fact/ID/Paint/Vec storage
    pub peak_scratch_bytes: u64, // current pixel/line scratch + explicit kernel bytes
}
```

Constructors live only in evaluated_scene. Research consumes these immutable facts; it checks slice lengths/finiteness/certified allocations defensively but does not choose clocks, authoring defaults, applicability or work limits. `grid=None` is semantically zero before inversion: no local fill/EDT/feather allocation, but opacity/inversion/ordered combination still execute within owner domain. A nonempty compiled path whose sampled fill has no seeds remains a real grid with the approved no-seed EDT behavior.

Main-owned signatures:

```rust
// shapes.rs wrapper around existing private Compiler; no ordinary-padding adoption.
pub(super) fn compile_mask_path(path: &VectorPath, density: f64,
    remaining_scene_segments: usize) -> Result<CompiledMaskPath,CoreError>;
// CompiledMaskPath carries contours, fill_rule, analytic_bounds and segments,
// plus explicit drawable_fill rather than nonfinite/empty sentinel bounds.

// masks.rs: same SampleTime built by shared source clock owner, never floor/f64 clock remake.
pub(super) fn sample_masks(authored: &[Mask], channels: &[AnimationChannel],
    time: crate::animation::SampleTime) -> Result<Vec<Mask>,CoreError>;
pub(super) fn certify_sampled_masks(sampled: &[Mask], owner: MaskOwnerBasis,
    frame_budget: &mut MaskFrameBudget, scene_segments: &mut usize)
    -> Result<EvaluatedMaskStack,CoreError>;
```

`MaskFrameBudget` is main-owned checked aggregate over all occurrences/shutter samples in one output frame. It charges final4*Powner multiplication once for nonempty owner stacks, each mask's5*Powner sampling/combination and all approved local coverage/paint/EDT/kernel work. No helper resets budgets per mask. Geometry segment accounting shares existing scene cap at corresponding sampled scenes; mask shutter compile work additionally charges the per-output-frame mask budget. Continuous publication certification bounds the same formulas through the existing shared node budget.

## Research-owned pure application API

```rust
// render_artifact/masks.rs
pub(super) fn apply_stack(
    pixels: &mut [[f32;4]],
    stack: &EvaluatedMaskStack,
) -> Result<(),CoreError>;
```

Empty stack returns immediately without allocation or touching pixels. Nonempty stack allocates exactly one4*Powner accumulation map, seeds0 for first add/exclude or1 for first subtract/intersect, and processes one current local grid at a time. It inverse-samples/combines directly into that map; do not allocate a second owner-sized per-mask map. Multiply all four source components once only after the stack is combined. A scalar local grid can be represented privately as width,height,Vec<f32>; its bilinear convention subtracts0.5 and uses transparent extension exactly like existing Raster. The helper exposes no process/file/cache capability and stores no timeline-sized results.

Main-owned reusable fill API in shapes.rs:

```rust
pub(super) fn fill_contours(
    contours: &[Contour], fill_rule: FillRule,
    origin: (f64,f64), density: f64, size: (u32,u32),
) -> Result<Vec<f32>,CoreError>;
```

This shares/refactors the current tiny-skia fill path builder and AA alpha/255 behavior without changing ordinary stroke/source rasterization. Main retains edits to that file; research calls it and existing `paint_at`. Allocation of the Pixmap and scalar output is included in mask scratch, with checked reserve and pre-certified dimensions. Raster-coordinate conversion displacement<=0.25gridpixel must be checked before casting contour coordinates, never silently saturate.

## Coordinate and sampling handshake

Owner-local mask coordinates are the canonical post-crop/clip source raster basis, with top-left domain0,0 and extent(W/density,H/density) in local project pixels. Owner pixel center is `((x+.5)/density,(y+.5)/density)`. Normalized mask position resolves using that domain's local extent; no owner/ancestor world affine, transition gain or inherited opacity enters mask affine. Source shape analytic bounds/origin remain the existing owner-anchor/effect facts; they are not substituted for mask analytic bounds or added again to mask coordinates. Root has resolved this exact zero-origin post-crop raster basis; the promoted authoritative design/spec must record it before approval. Vector-source padded analytic origin is not added to mask coordinates.

Apply certified inverse to that center to obtain original path-local coordinates q. Scalar grid center coordinates are `((q.x-origin.x)*rho,(q.y-origin.y)*rho)` under the existing half-pixel bilinear convention. Geometry and paint use original q-domain; padding/expansion never stretch gradients. Anchor is sampled analytic unexpanded path bounds: `(left+ax*(right-left),top+ay*(bottom-top))`. Matrix construction uses existing transform order with that anchor and owner-relative position, independent of rho. Rho is max(1,ownerDensity*maxSingularValue(maskLinear)); expansion and sigma are local quantities multiplied by rho only once. No dimensions are rounded to an integer density.

## Scratch, failure and numerical contract

For each current Pgrid reserve64*Pgrid +16*(Wgrid+Hgrid) pixel/line scratch plus additive kernel bytes (4*K for normalized f32 taps), not hidden inside64*Pgrid. Reserve64*sumS over ALL retained MaskGrid contour vectors PLUS checked retained metadata bytes: EvaluatedMask/MaskGrid/stack headers, ID UTF8 storage, Paint headers/gradient-stop heap storage and Vec headers/capacities. Charge grid=None/S0 facts and64-stop paints even though they allocate no contour/pixel grid. Compute metadata through checked capacity*size_of accounting or a conservative explicit authored-fact bound;64*sumS is not a blanket allowance for that metadata. Reserve this total plus4*Powner accumulator, atop the existing simultaneous1GiB source/effect/transformed/temporal/final/cache ceiling. The immutable full stack retains all contours even though pixel processing is sequential. Across scene/shutter lifetimes charge every concurrently retained fact stack or drop it before constructing the next; do not mistake sequential pixel processing for released contour storage. All pixmaps/EDT/feather buffers must fit64*Pgrid, with actual allocation peaks checked against the certificate. EDT uses absence for no seeds, integer/rational envelope boundaries and stable lower-index ties, no float infinity. Gaussian normalized taps, horizontal left-to-right then vertical top-to-bottom, transparent extension and no edge renormalization are fixed. Scalar coverage validates finite[0,1], clamps only<=1e-6roundoff. Source components remain premultiplied linear, with no intermediate byte encoding. All invalid facts/allocations return nonretryable INVALID_ARGUMENT; stable external media/resource failures retain their current owners and codes.

No-mask path must return to original prepare/size/memory/work logic exactly. Do not add mandatory mask scratch for empty stacks or make the conditional empty-effects4K certificate stricter. Nonempty masks intentionally change oldschema32 output after fullschema33 migration/renderability validation; metadata migration must never discard them.

## Implementation handshake and tests

1. After root approves promoted51 artifacts, main adds failing typed/wire/domain tests and publishes the evaluated types/compiler/fill signatures. Send research concrete commit-independent paths and examples; reconcile any naming changes before integration.
2. Research adds red independent scalar/brute-force distance/Gaussian/operation tests in its reserved new files, then pure implementation. Main concurrently implements clocks/channels/schema/store/continuous certificates, without touching research files.
3. Main inserts one apply_stack call after crop/before effects, using sampled certified stack and current Raster pixel slice. Same code executes every intent and shutter sample. Test source→mask→effect ordering independently, noncentral/nonuniform affine and mask opacity once.
4. Research oracles do not call production distance/kernel/combination helpers to derive expected values. Tiny-skia fill edge expectations are explicitly byte/255 where appropriate; analytic/AA/empty/subpixel/colored-alpha-dark-luma/gradient-extension cases remain independent. Main owns combined native/public/migration/fault/budget tests and no-mask exact parity.
5. Required full suites, genuine opt-in native evidence, independent review and archive lifecycle remain mandatory. No51implementation or fixture expectation edits are authorized by this external plan.

## Promotion reconciliation

Living #50 Active mask authoring contract must be fully MODIFIED32→33 preserving its three scenarios, not left alongside contradictory ADDED33 text. All seven active current catalog markers advance33; mask-model inactive annotation becomes a link to mask-rendering authority. Existing mask schema32 activation threshold remains32; new channels activate33. Animation catalog mask property/target additions and MCP exact predecessor projection are separate from the other marker-only amendments. Use actual verified50 pins/digest, not guessed external values.
