# Conditional #52 implementation ownership and interface

Planning only against verified #51 `5f456618b9335a5f87519c5c4375a49e7f52c2d3`; no implementation is authorized by this document. Root must approve the promoted specification before code, and the canonical track-mattes catalog must precede declarations. All interface definitions below are internal and introduce no wire fields beyond approved matte/matteOnly.

## File reservations

Main core owner:
- new `src/model/matte.rs`, existing `model.rs`: closed MatteReference/MatteChannel, VisualProperties defaults, presence-sensitive UpdateItem fields.
- new `src/validation/matte.rs`, existing `validation.rs`, `timeline.rs`: scoped leaf/reference/cycle/depth/edge validation, canonical eligibility/errors, deferred matte-only final-candidate validation in atomic batches.
- `migrations.rs`, `store.rs`, `error.rs` if necessary: schema34 raw/source-matched guards, retained current/history/draft staged adoption and recovery, existing typed errors.
- new `src/evaluated_scene/mattes.rs`, existing `evaluated_scene.rs`, `extended_visual.rs`, `extended_certification.rs`: authored-scope/instance-path occurrence binding, immutable evaluated roles, exact provider-request schedule, transitive temporal/source/analysis/work/memory certificates.
- existing `src/render_artifact/extended_visual.rs`, renderer/planning seams only if needed: separate transformed transparent pre-gain source callback for matte execution, default branch remains unchanged.
- main-owned new public `tests/track_mattes.rs` plus existing owner private tests: model/DAG/persistence/fault/public-native integration, default bypass/source-cache evidence.

Research pure/provider owner ONLY:
- new `src/render_artifact/mattes.rs` and `src/render_artifact/mattes/tests.rs`: transparent plane primitives, execution of already-bound/certified tasks, exact immutable task result reuse and actual capacity/lifetime accounting, independent analytic/numerical tests.
- No authoring validators, source rasterizers, model declarations, scene binding, schema/store changes or main files.

Public owner: canonical track-mattes catalog and eight marker34 projections, ownership/CODEOWNERS, headless reporter/protocol, bridge schemas/contracts/workflows/docs. Catalog creation and exact enum/fixture handshake must precede model changes. Root owns OpenSpec approval/tasks/final verification/archive/publication. Reviewer remains independent read-only.

## Exact model handshake

`MatteReference { source_id: String, channel: MatteChannel }`, enum Alpha/Luma serialized alpha/luma. Both fields required, closed/duplicate-strict. Stored `VisualProperties.matte: Option<MatteReference>` defaults None and omits None, `matte_only: bool` defaults false and omits false. UpdateItem `matte: Option<Option<MatteReference>>` uses a present-field deserializer so absent=None, null=Some(None), object=Some(Some(reference)); `matte_only: Option<bool>` uses existing present/non-null pattern. Boxing may follow existing Rust representation style if strict Clippy needs it; wire remains exact. Source IDs are literal scoped canonical item IDs, never URLs/path expressions; lookup never resolves filesystem/network content. Root batch aliases apply to sourceId and itemId; component-local payload IDs retain their own namespace.

## Bound scene facts (main publishes first)

Introduce `EvaluatedScene.mattes: Option<EvaluatedMatteGraph>`; None iff every matte absent and every matteOnly false. For that complete default, no new graph/maps/planes/accounting allocations and the existing renderer path stays exact.

Proposed internal types in evaluated_scene/mattes.rs:

```rust
pub(crate) struct CompositionOccurrenceId(pub usize);
pub(crate) struct MatteGroupId(pub usize);
pub(crate) struct MatteTaskId(pub usize);
pub(crate) struct MatteProviderGroup {
    pub composition: CompositionOccurrenceId,
    pub authored_item_id: String,
    pub members: Vec<usize>, // evaluated layer indices, canonical paint order
    pub provider: Option<(MatteGroupId, MatteChannel)>,
}
pub(crate) struct EvaluatedMatteRole {
    pub group: Option<MatteGroupId>,
    pub provider: Option<(MatteGroupId, MatteChannel)>,
    pub matte_only: bool,
    pub contributes: bool, // effective hidden/track/ancestor visibility
}
pub(crate) struct EvaluatedMatteGraph {
    pub groups: Vec<MatteProviderGroup>,
    pub roles: Vec<EvaluatedMatteRole>, // one entry per evaluated visual layer
    pub provider_first: Vec<MatteGroupId>,
    // Interned typed composition occurrence paths; actual capacities charged.
}
```

Bind authored scoped references before occurrence expansion. Assign composition IDs while traversing root/component instances, including repeated component-instance copies; do not infer scopes by parsing generated item IDs. A repeater of a local leaf adds members to that leaf's same composition group. A repeated component instance creates distinct child composition occurrences. Group identity is (composition occurrence, authored source ID), not local ID alone and not repeated recipient copy index. Existing/inactive/hidden providers can have zero contributing members and yield transparent zero, never missing/identity fallback. Provider-only managed assets/fonts remain included in integrity/readiness/certification even if direct drawing is suppressed.

## Certified exact frame schedule (main owns expansion)

At each output frame, main builds a provider-first immutable task schedule using exact `(group, root_at_ms: u64)` request keys. Every externally requested root time and canonical shutter midpoint is integer u64; inherited fractions remain in each existing SampleTime/local clock, never converted into approximate memo keys. If future APIs accept noninteger root requests, this key must be extended explicitly rather than silently rounded.

Main computes/certifies nested shutter cardinality/work/request/descriptor bytes with checked arithmetic before enumerating tasks. Abort at4096 uncached provider requests,268435456 matte units, shared source/mask/effect/destination limits or1GiB. Memoize completed copy requests `(layer_index, root_at_ms)` and provider requests `(group, root_at_ms)` exactly; mandatory source and resource validation precede cache admission. A bounded schedule is allowed to reuse immutable results; weighted duplicate shutter entries still contribute their full authored averaging weight. No product-of-averages/center freeze or second shutter applies to a completed copy task.

```rust
pub(crate) enum MatteTask {
    LeafSample {
        layer_index: usize, at_ms: u64,
        provider: Option<(MatteTaskId, MatteChannel)>,
        source_live_bytes: u64, // certified source/mask/effect/transform transient
    },
    AverageCopy { samples: Vec<MatteTaskId> },
    AggregateProvider { copies: Vec<MatteTaskId> },
}
pub(crate) struct MatteFrameSchedule {
    pub canvas: (u32,u32),
    pub tasks: Vec<MatteTask>, // all dependencies precede consumer tasks
    pub direct_draw: Vec<MatteTaskId>, // completed individual copies in paint order
    pub last_uses: Vec<usize>,
    pub certificate: MatteFrameCertificate,
}
```

`AverageCopy` preserves ordered duplicate sample references and averages isolated premultiplied components. `AggregateProvider` source-overs completed averaged copies once in canonical order. Direct draw references each completed copy once, skips matteOnly roles, and never draws a provider aggregate per copy. Unreferenced matteOnly leaves still suppress direct drawing without changing audio/duration. The schedule includes all ordinary destination visuals, including unchanged Caption (which cannot author matte/matteOnly); callback support stays with its existing source owner.

The certificate names fixed cache reservation, existing destination/shutter buffers, actual graph/task/key/Vec capacities, per-task source-live reserve, all live16P planes and4P scalar coverage, shared compiler/mask/effect allocations and source buffers. Last-use execution frees results deterministically. Preflight computes conservative maxima; pure runtime checks typed schedule indices/order/actual capacities/reservations and rejects forged undersized certificates rather than claiming constructor safety alone. Request count/source/mask/segment/continuous-node budgets are shared across the entire output frame, not reset for each group/task/callback. Coverage extraction can multiply directly into a recipient plane, avoiding an unnecessary allocated coverage vector; charge5P work and count any actual4P vector if one exists.

## Transparent plane/callback handshake (research publishes after facts)

Research defines or consumes a single internal plane type, avoiding an8bit/sRGB round trip:

```rust
pub(super) struct LinearPlane {
    pub left: usize, pub top: usize,
    pub width: usize, pub height: usize,
    pub pixels: Vec<[f32;4]>, // linear premultiplied RGBA on transparent support
}
pub(super) struct LeafSamplePlane {
    pub plane: LinearPlane,
    pub gain: f32, // local/inherited opacity and transition product, once
}
pub(super) fn compose_frame(
    schedule: &MatteFrameSchedule,
    destination: &mut [[f32;4]], // main's existing opaque-black canvas
    sample: &mut dyn FnMut(usize,u64) -> Result<LeafSamplePlane,CoreError>,
) -> Result<(),CoreError>;
```

Main's callback completes source raster→crop/clip→active masks→ordered effects→local/nearest-to-outer ancestor geometric transforms using existing density/anchor/padding/bilinear conventions, but returns pixels BEFORE the scalar gain. Pure LeafSample execution samples the completed provider plane at identical output pixel centers, applies alpha or premultiplied-linear luma once, then the gain once. Hidden/inactive source yields zero transparent plane. Source support cannot grow through matte application. Pure averaging is after each sample's own matte and gain; provider aggregation is after each copy's averaging. Source callbacks share main's actual frame budgets and existing source PAM/cache integrity seams.

Sample reserve includes the simultaneously returned transformed plane so callback allocations are covered before execution. After callback, executor releases transient reservation and adopts checked actual returned Vec capacity without a live-byte gap/double-hidden allocation. It retains other task planes until the certified last use. Any optional scalar coverage or intermediate average/aggregate allocation is separately charged. Existing f32 epsilon validation/clamp is reused exactly; no unpremultiplication for luma or invented alpha threshold.

Default rendering must not route through this callback/scheduler when graph=None. Main may expose existing Raster internals only to inward artifact siblings or convert by moving pixels into LinearPlane; no duplicate pixel buffer merely to cross the module boundary. Destination serialization remains in the existing final compositor.

## Batch/staged ownership integration

Add an explicit private matte-validation mode to the existing timeline operation owner, or a dedicated batch entry that forwards to one internal implementation. Standalone candidate validation includes full matte graph; atomic-batch operations retain all existing per-operation ID/alias/shape/mask/parent rules but defer ONLY matte reference/deletion/cycle/aggregate graph checks until the complete final candidate. Do not use global/thread-local switches or disable existing visual validation. This makes delete-provider then clear-recipient equivalent to reverse ordering without changing sequential alias semantics. Store/draft replay batches use the same final-candidate owner.

Raw pre34 field detection is narrowly scoped to actual visual items and authored draft operations. Staged edit/draft classifier includes matte-bearing and matteOnly-bearing operations and relevant existing-program replacements; source-matched draft checks use retained own bases. Guard null/false field presence before defaults. Preserve nonmatte missing-draft/revision/recovery precedence and all existing publication fault guarantees.

## Focused handshakes before broad implementation

1. Canonical public owner signals track-mattes surface and schema34 fixture readiness.
2. Main publishes model names and immutable facts/schedule signatures (first small core commit is not performed by agents; files simply coexist).
3. Research authors semantic red plane/executor tests through the stable callback, then implements only its two files.
4. Main authors model/DAG/public/staged red tests and wires transparent source callback + certification.
5. Integrate exact0.4375 copy-average-before-aggregate witness, nested provider shutters/occurrences, sourceGain/matte/background controls, no-matte empty allocation bypass and all-intent native/asymmetric/audio/cache/fault tests.
6. Parent coordinates stable Cargo/binary slots and complete required gates. Native/instrumented default-headless mutations wait unfiltered workspace completion; all actual opt-in evidence remains separate.

## Exact retained certificate and resource integrity ports

Main additionally owns the continuous request/collision certificate in the existing evaluated/extended-certification boundary, optional active-matte SceneResourceBindings integrity sidecar, and bounded ArtifactIo fingerprint port in render_artifact.rs/FileSystemArtifactIo. Follow continuous-resource-certification.md; preserve existing ownership edges and exact complete-default bypass. These mechanisms await explicit manifested review/parent approval.
