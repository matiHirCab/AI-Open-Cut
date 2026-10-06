# Proposed private #53 core interface against committed #52

Issue53 private mechanism plan reconciled from verified #52 `65e89d637d882d56aa98735aae9374de6a0da9fe`; all11CI passed in run37458611299. Explicit active specification approval still precedes implementation. This proposal supersedes the presumed #52 seams in the older external design, without changing its seven equations or budgets.

## Actual predecessor seams

`evaluated_scene/mattes.rs` currently puts `resource_live_bytes` and `font_payload_bytes` in `EvaluatedMatteGraph`. Its `font_payload_admission`, `adopt_font_payload`, and `admit_caller_scene_clone` require that graph. `renderer.rs` selects all bounded lookup/measurement/cache-clone/finalized-clone paths using `scene.mattes.is_some()`. `render_artifact.rs` similarly selects admitted managed-font reads and retained integrity preparation by that predicate. These are actual resource controls, not provider graph semantics.

`MatteTask::AverageCopy` currently stores only samples. `MatteFrameSchedule.direct_draw` is `Vec<MatteTaskId>`; its paint order is correct but it carries no completed owner/mode fact. The pure executor receives no evaluated layers: it cannot safely recover a mode from an AverageCopy or transport DTO. Its AggregateProvider/source-over path must stay mode-independent.

The ordinary artifact branch already draws one isolated source per layer, averages shutter samples before source-over, and assembles Caption into one plane. Preserve that exact branch for normal scenes without mattes. Non-normal scenes need the bounded source scheduling/resource path, not a synthetic matte role graph.

## Resource ownership independent of the optional matte graph

Proposed private types (names may be finalized before parallel code):

```rust
// evaluated_scene/composition_resources.rs; no artifact or transport imports
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct CompositionResourceFacts {
    pub resource_live_bytes: u64,
    pub font_payload_bytes: u64,
}
// EvaluatedScene
pub composition_resources: Option<CompositionResourceFacts>,
pub mattes: Option<EvaluatedMatteGraph>, // graph-only groups/roles/provider order
```

Create the resource option when the validated authored domain contains a non-normal eligible blend OR nondefault matte program. Inspect hidden/unused component definitions too, as #52 does for retained obligations. Do not create it for absent/explicit-normal blend without a matte program. `Some` means the existing bounded composition preparation is required; it does not mean a matte edge/provider request exists. No extra allocation, traversal, map, readiness relocation, or counter is introduced into the `None` branch.

Move the two resource counters out of the graph; do not retain two independently mutable ledgers. Reuse the existing exact scene/graph/text/glyph/style walkers and functions, generalized in the same inward owner. Proposed signatures:

```rust
pub(crate) fn composition_heap_bytes(scene: &EvaluatedScene) -> Result<u64, CoreError>;
pub(crate) fn font_payload_admission(scene: &EvaluatedScene) -> Result<u64, CoreError>;
pub(crate) fn adopt_font_payload(scene: &mut EvaluatedScene, bytes: u64) -> Result<(), CoreError>;
pub(crate) fn admit_caller_scene_clone(scene: &EvaluatedScene) -> Result<u64, CoreError>;
```

`composition_heap_bytes` counts the optional graph only when present, resource-fact header, original scene, every actual outer/nested Vec/String/map capacity, and already-live resource reservation exactly once. Adoption replaces the previous font portion as #52 does. Fixed planning memory includes caller original/finalized scenes, actual configured paths/roots, prepared unique font payloads plus catalog metadata, retained pins/hash scratch, measured/cache/finalized glyph copies, descriptors, and cache reserve before each new clone/allocation. Moving the ledger does not exempt the graph heap when mattes coexist.

Generalize the existing active-only predicates at renderer measurement, ArtifactIo admitted font/lookup, measurement failure latch, retained SHA sidecar preparation and integrity verification to this resource option. Reuse `MeasurementMemory`, the production cache-hit clone helper, admitted filesystem cursor, bounded font reader, streaming digest, and existing private ports unchanged. No second shaping walker, dummy matte graph, or reverse fonts→artifact edge. Blend-only pinned hidden/unused media/font obligations receive the same canonical retained binding treatment and error precedence. Ordinary decoder inputs remain publication-filtered. Windows OS-private bookkeeping remains outside the disclosed application-owned ledger; known libc/content/caller buffers remain included. Existing Mac dependency/cursor proof and CI obligations carry forward.

## Immutable completed direct-owner facts

Validated static `BlendMode` is stored in each `EvaluatedVisualLayer`, including Caption and every expanded leaf occurrence. Structural ancestors cannot provide it. Source cache facts exclude a destination-only selection only when certified source semantics are identical; final semantic/composite facts include mode/order/background.

Generalize the existing schedule type in its evaluated owner rather than rebuilding authored graphs in artifact code:

```rust
pub(crate) struct DirectDraw {
    pub task: CompositionTaskId,
    pub layer_index: usize,
    pub blend_mode: BlendMode,
    pub destination_visits: u64, // conservative certified transformed footprint
}
pub(crate) enum CompositionTask {
    LeafSample { layer_index: usize, at_ms: u64,
                 provider: Option<(CompositionTaskId, MatteChannel)>,
                 source_live_bytes: u64 },
    AverageCopy { layer_index: usize, samples: Vec<CompositionTaskId> },
    AggregateProvider { copies: Vec<CompositionTaskId> },
}
pub(crate) struct CompositionFrameSchedule {
    pub canvas: (u32, u32),
    pub tasks: Vec<CompositionTask>,
    pub direct_draw: Vec<DirectDraw>,
    pub owner_modes: Vec<BlendMode>, // canonical evaluated layer-index authority
    pub last_uses: Vec<usize>,
    pub certificate: CompositionFrameCertificate,
}
```

A single-sample direct task links to its LeafSample owner. AverageCopy records the same immutable owner and every sample must link to that owner. The compiler publishes one canonical immutable owner_modes table keyed by actual evaluated layer_index. Every task/direct owner index must be in range; DirectDraw.blend_mode must equal owner_modes[layer_index]. AverageCopy and every sample must link to the same owner index. Table actual capacity, clone overlap and growth are charged once in descriptor admission. No reverse authored graph/layer lookup occurs in pure execution. DirectDraw mode is copied by the evaluated schedule compiler from that owner after validation; it is not supplied by a rendering request. The pure executor checks owner linkage, legal task ordering, duplicate direct IDs, closed mode, and last-use index before callbacks. Direct order is canonical layer/copy paint order, unchanged from #52. Hidden/noncontributing/matteOnly copies have no direct entry. No AggregateProvider is direct-drawable. An ordinarily visible provider copy has its own direct mode, while provider coverage uses its unchanged averaged source plane.

The compiler accepts optional graph roles: without a graph, all ordinary eligible published occurrences use provider=None, contribute under existing visibility semantics, and only copy/shutter tasks are built. No participant IDs or matte graph are fabricated. With a graph, the old scoped role/memo semantics remain. Callback remains `(layer_index, root_u64) -> LeafSamplePlane`; no mode is applied there. It executes crop→masks→effects→affine, then executor own matte→gain→own sample average. Destination blend occurs exactly once on the completed direct plane. AggregateProvider always uses existing normal source-over; it never reads DirectDraw mode or backdrop.

## Certificates and unchanged counters

Retain certificate `provider_requests`, `matte_work_units`, `fixed_live_bytes`, `descriptor_bytes`, `peak_live_bytes`; add `non_normal_blend_work_units`. Exactly AggregateProvider materializations count toward 4096. LeafSample, AverageCopy, direct draws, temporal function classes, and ordinary sampling do not consume that counter. Do not introduce a generalized 4096 task/sample/request limit. Existing expanded occurrence limit4096 and source pixel-work268435456 still apply independently.

Descriptor admission counts actual new DirectDraw capacity and expanded AverageCopy header, nested sample vectors, last_uses, memo maps, release slots, temporary realloc overlap, and fixed executor metadata. Recompute the current conservative128-byte slot allowance against actual type sizes; do not silently assume it still fits after additions. Admit before task/member/map growth, then adopt actual capacities. Schedule compiler uses the existing shared `SampledFrameBudget` for every actual uncached leaf sample, preserving source/effect/mask/ordinary segment and per-time occurrence rules. Continuous certification joins existing shared65536 nodes and exact collision policy; blend destination bounds use that owner, not a reset per provider/copy.

`non_normal_blend_work_units = checked_sum(32 * destination_visits)` only for non-normal DirectDraw entries, at the completed occurrence after shutter averaging. Bound268435456 inclusive. Certified footprint includes transparent interior and does not drop an excessive offscreen source to evade admission. Actual touched work cannot exceed certificate. MatteOnly/provider-only tasks incur no destination blend work, but retain all source/provider work. Ordinary normal entries do not charge this new counter.

All pixel/support/mode/owner/capacity/work/callback failures precede destination mutation. Before commit validate the ORIGINAL destination without clamping or modifying it, validate ALL completed direct planes, modes and descriptors together, finish every callback and admit all blend visits. Keep every simultaneously retained completed direct plane plus descriptor/reallocation overlap in the peak-live ledger; no early direct commit is permitted. The pure blend helper validates/clamps canonical f32 operands before f64 equations; normal calls the original f32 source-over. Prove closure for canonical validated/clamped f32 operands: each corrected component is within its alpha, straight colors lie in[0,1], separable B lies in[0,1], and each nonnegative coefficient sum is ordinary source-over alpha≤1; bounded f64 arithmetic remains finite and final f32 conversion stays canonical. Preserve original normal f32 result/clamp semantics after identical prevalidation. Final commit loop has ZERO fallible checks and performs no callback/admission/descriptor validation, requiring no second full destination surface. A late invalid callback or forged direct descriptor leaves caller bytes identical. The opaque-black initializer stays unchanged. Avoid destination-dependent precompute allocations.

## File ownership and interface constraints

Main: model/enum/schema35, eligibility/edit/raw guards/staged migrations/recovery, evaluated layer mode, `evaluated_scene/composition_resources.rs`, existing mattes schedule/certification owner, renderer/resource predicate integration, extended artifact callback integration, core/domain/resource/native lifecycle tests. Existing graph-only helpers can stay in mattes.rs while exact resource walkers are moved/reexported once; no duplicate policy owner.

Pure sibling: a new `render_artifact/blend.rs` plus tests for seven-mode pixel operation; only the narrow direct-commit integration in existing artifact mattes.rs after main publishes completed DirectDraw and certificate types. Parent must reserve that existing file explicitly if delegated; do not allow simultaneous main/pure edits. Provider/average/last-use code remains unchanged except typed descriptor linkage/accounting. Public sibling owns canonical-first catalogs/schema projections/headless/bridge/docs. No transport walks occur inside pure execution.

Publish exact structs/constants/signatures, empty-support contract, shared byte accounting, and ownership hashes before parallel implementation. Canonical closed mode fixtures must precede enum declarations. Separate approval.md authorizes implementation through tasks.md; this interface alone is not approval.

## Meaningful bounds and regression witnesses

1. Actual blend-only schedule with4096 ordinary one-pixel occurrences and no graph must accept provider_requests=0. Repeat with16 weighted shutter samples per occurrence (65536 leaf visits) where existing source/work/memory caps fit. Count actual tasks/callbacks/weights; ensure a wrongly generalized4096 task cap would fail. Keep expanded occurrence4097 rejection separately.
2. Preserve actual4096 AggregateProvider boundary/excess4097 and deep duration1 exact-collision weighted samples. A mode-only revision must not change these counters or introduce graph edges.
3. Reachable blend work boundary:128 non-normal256×256 direct footprints gives8388608 visits×32=268435456; source visits remain below old268M, stored planes fit shared1GiB. Add one one-pixel direct draw for+32 excess before workspace/process/output. Owner arithmetic additionally checks exact+1unit forged certificate and overflow; do not claim a public one-unit step when public visits have32-unit granularity.
4. Near-budget blend-only real pinned Text/Caption must exercise the SAME cache-hit clone refusal, configured-path before-clone guard, bounded lookup/admitted read, cumulative glyph limit, measured→finalized overlap and fitting real-font output. Use allocator/read/shape/process spies, not only the option predicate. Normal/no-matte control retains byte-identical old native/golden/counter path.
5. Blend-only pinned media hidden/inactive/unused valid warm/fresh plus same-header/size corruption must fail integrity before cache/workspace/output. Actual graph-free scene asserted; no dummy role graph satisfies it.
6. Direct linkage forgery (AverageCopy samples from another owner, AggregateProvider as direct, mismatched mode/owner, duplicate direct entry, late invalid pixel) rejects before destination mutation. Separate repeated copies inherit the leaf mode and preserve order; mode does not affect provider coverage.
7. Actual add shutter-average1-versus.875 witness, Caption multiline whole-plane glyph/background witness, seven modes on all four intents, nonlinear reorder→undo/redo→reopen, and normal/source-PAM cache controls remain the external design’s required automation.

## Adopted design/delta constraints

- Design checked-memory/evaluated/cache sections: replace implicit matte-only ledger activation with the separate optional resource fact above; specify retained graph-free bindings and actual glyph/font/cache/cursor/clone lifetime accounting by the existing #52 owners.
- Design occurrence semantics: add completed immutable DirectDraw→LeafSample/AverageCopy owner linkage and provider-independent aggregate path; require typed schedule/capacity validation and precommit failure preservation.
- Blend-modes delta bounded-composition requirement/scenarios: explicit provider4096-only versus ordinary4096 occurrences/16 shutter visits, no synthetic graph/generalized sampling cap, new checked blend work counter and exact reachable boundary/+one visit control.
- Linear-light/rendering-export deltas: require graph-free non-normal Text/Caption/media to use the same bounded preparation/integrity path; preserve absent/explicit-normal no-matte branch exactly, opaque black, source-local Caption assembly, and all four intent lowering.
- Architecture delta: identify sole evaluated resource owner and inward reuse, static owner mode facts, no renderer/transport reverse authored traversal, no second shaping/heap policy, pure executor receives completed facts only.
- Tasks4.1/4.5/4.6/4.7 and5.5: add the concrete owner/descriptor/cache/resource witnesses above. Retain all old requirements/scenarios when full MODIFIED blocks are reconciled; these additions do not authorize deleting #52 glyph, native codec-boundary, platform, source-envelope or failure controls.

Actual predecessor digests/CI success and complete living requirement text are reconciled in the active design/deltas and verified52-catalog-pins.json. Implementation is authorized only by separate approval.md; no #53 execution evidence is implied.

The pure executor’s FIRST operation validates original destination dimensions and every pixel read-only, before descriptor allocation/admission or any callback. Invalid original destination must yield zero callbacks and unchanged bytes. Only afterward validate/admit descriptors and run sources; successful commit performs no allocation, callback, admission or fallible check.

## Normal identity and actual metadata admission

Absent/explicit-normal blend with no nondefault matte program MUST preserve the exact predecessor bypass, output, counters and acceptance. In schedules requiring bounded composition preparation, normal blending MUST preserve predecessor arithmetic/output and existing source, destination, mask, effect, shutter, matte-work and provider-request charging rules and maxima. Every actually live composition-resource header, DirectDraw descriptor, owner_modes table and associated capacity, clone or growth overlap MUST nevertheless be admitted under the unchanged peak-live-byte limit; normal selection SHALL NOT exempt this storage. Additional metadata may cause an otherwise marginal request to exceed that unchanged memory limit and MUST reject before side effects.

Final implementation name reconciliation: the existing private MatteFrameSchedule/MatteTask names are retained for the single bounded scheduler/executor. DirectDraw and owner_modes implement the approved completed owning facts without adding a parallel CompositionFrameSchedule. CompositionResourceFacts lives in composition_resources.rs independently of the optional real EvaluatedMatteGraph. New per-output-frame blend work is exhaustively certified for all requested frames before materialization, while the existing continuous source/mask/effect owner retains its shared analysis quota.
