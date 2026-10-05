## ADDED Requirements

### Requirement: Deterministic painted path coverage
Core MUST rasterize each sampled mask path using canonical filled contour/fillRule/implicit closure, adaptive tolerance 0.25/rho and existing antialiased coverage semantics. Move-only/no drawable fill MUST yield zero coverage. Provably no-drawable paths MUST require no local raster/distance/kernel allocation while retaining owner-domain inversion/combination semantics; nonempty candidate geometry whose raster has no seeds MUST follow empty-distance behavior. Density MUST be max(1,ownerDensity*maximum singular value of sampled mask affine linear part). Sampled geometric bounds plus full signed-expansion and three-sigma feather support and two grid pixels MUST determine outward-rounded local grid bounds before allocation. Inverse matrices and raster-coordinate conversions MUST be finite/invertible and have displacement at most 0.25 grid pixels. Original-coordinate Paint MUST be evaluated in floating premultiplied linear RGB; alpha coverage MUST be geometric coverage times paint alpha and luma MUST be geometric coverage times0.2126R+0.7152G+0.0722B using premultiplied linear channels. Transparent paint MUST contribute zero. No encoded RGBA8 color roundtrip, stroke mask, raw resource or unsupported source SHALL be introduced.

#### Scenario: Distinguish geometry from colored paint
- **WHEN** thin/subpixel geometry uses dark colored paint or alpha 0.25, including original-coordinate gradients
- **THEN** geometry support remains independently defined and alpha/luma match independent numeric/color oracles, without erasing low-alpha/dark support

#### Scenario: Preserve empty paths and precise coordinates
- **WHEN** move-only geometry, noncentral sampled anchors or large unsafe raster-coordinate conversions are evaluated
- **THEN** empty geometry yields zero coverage, safe anchors retain sampled unexpanded geometric meaning and unsafe conversions fail with INVALID_ARGUMENT before side effects

### Requirement: Bounded alpha-aware signed expansion
At zero expansion core MUST return original geometric coverage C exactly. At nonzero expansion core MUST compute exact squared Euclidean grid-center distance fields D_I to C>0 and D_O to C<1 with deterministic separable O(P) lower-envelope transforms, without per-pixel radius/segment searches. Partial0<C<1 MUST use d=C−0.5; C=1 MUST use d=sqrt(D_O)−0.5; C=0 MUST use d=0.5−sqrt(D_I). Expanded geometric coverage MUST be clamp(0.5+d+expansionPx*rho,0,1). Empty support MUST remain empty at every expansion. Required transparent padded exterior MUST participate in distance computation; absence of seeds SHALL NOT create float infinity arithmetic or fabricated support. Expansion MUST operate on geometry before original-coordinate paint/channel extraction, never threshold painted alpha/luma. Full outside support MUST be included before allocation rather than clipped to hide excess work.

#### Scenario: Prove exact distance reconstruction and zero identity
- **WHEN** small independent brute-force grids include binary edges, slanted antialiasing, partial coverage below0.5, holes and both expansion signs
- **THEN** separable distances/reconstructed coverage match the approved oracle and zero expansion is exact identity

#### Scenario: Preserve thin support and padded boundaries
- **WHEN** thin partial coverage expands/contracts near finite raster edges or a wholly empty path expands
- **THEN** all nonzero geometric samples seed support, exterior/padding match the same infinite-transparent-domain rule within required finite support, and empty geometry remains zero without radius-search work

### Requirement: Canonical feather and ordered mask composition
Feather MUST apply a normalized separable Gaussian of sigma=featherPx*rho with support ceil(3*sigma), horizontal then vertical deterministic accumulation, transparent extension and no boundary renormalization; zero MUST bypass exactly. Core MUST use top-left(0,0) of the exact post-crop/clip pre-effect source raster as owner-local basis, logical extent W/ownerDensity,H/ownerDensity for normalized positions and local centers((x+0.5)/ownerDensity,(y+0.5)/ownerDensity). Existing vector padded analytic origin SHALL NOT be added again to mask coordinates. Core MUST inverse-bilinearly sample scalar mask coverage at those owner-local raster pixel centers, apply mask Transform2D opacity once, then invert as1−B within owner source domain. Declared add/subtract/intersect/exclude MUST use A+B−A*B, A*(1−B), A*B and A+B−2*A*B with initial0 for first add/exclude and1 for subtract/intersect. Empty stack MUST bypass new processing with identity1. Combined coverage MUST multiply all four source premultiplied components once after crop/clip and before existing effects and owner/ancestor transforms. Existing owner anchor, effect support, inherited opacity and transition gain MUST remain unchanged. Scalar coverage MUST be finite in[0,1] with at most existing 1e−6 f32 roundoff; larger violations MUST fail INVALID_ARGUMENT.

#### Scenario: Prove kernel transform inversion and operation order
- **WHEN** independent impulse/step kernels, asymmetric bilinear masks, nonidentity affine/opacity, inversion and all four operations are sampled
- **THEN** approved numeric oracles match within 1e−6, changing noncommuting operation order changes the expected coverage and source/effect/ancestor order remains canonical

#### Scenario: Preserve clipping and source alpha
- **WHEN** a cropped/clipped transparent source uses inverted or expanded masks before glow/blur
- **THEN** masking never revives source-removed pixels or expands owner source support, while subsequent existing effects retain their declared support and alpha behavior

### Requirement: Certified mask work and shared live memory
Each output frame MUST have at most 268435456 mask work units summed across all masks, expanded occurrences and actual shutter samples. For local P=W*H and S compiled segments, coverage MUST charge4P+4S*H*(1+ceil(log2(max(S,1)))); Paint MUST charge P*(1+ceil(log2(N))) for N=1 on solid or gradient stop count; nonzero expansion MUST charge24P; nonzero feather MUST charge2P*(2ceil(3*featherPx*rho)+1); inverse sampling/combination MUST charge5P_owner per mask and final source multiplication4P_owner once per owning sample. Every arithmetic operation MUST be checked. Existing16384 dimension/16777216 pixel/4096 occurrence/65536 per-path segment/1048576 shared scene segment limits MUST apply, counting mask and existing contours in the corresponding sampled scene without altering existing ordinary-geometry budget scope. Every actual mask shutter compilation MUST additionally count toward mask work for that output frame. Existing effect and destination-work budgets MUST retain their meaning; mask budget SHALL NOT replace or relax them.

Masks MUST process pixels sequentially with one4P_owner accumulated map and conservative current-mask pixel/line scratch reservation64P+16(W+H) bytes. Nonzero feather MUST additionally reserve4K bytes for K=2*ceil(3*featherPx*rho)+1 normalized f32 Gaussian taps, never hidden within64P. Core MUST separately charge SUM64S bytes over ALL concurrently retained compiled contours PLUS checked fact headers, IDs, Paint/gradient-stop heap storage and Vec descriptor/capacity storage, including no-grid/S=0 facts and every retained mask stack/scene/shutter sample. Sequential pixel processing SHALL NOT imply release of retained facts; facts MUST either be dropped before constructing the next stack or counted for their simultaneous lifetime. Actual buffers MUST fit certified reservations. All mask/source/effect/temporal/output/cache live memory together MUST fit existing 1073741824 bytes/request including existing 67108864 cache reservation. Range/export MUST stream rather than retain sequence-sized mask rasters. Existing65536-node continuous candidate-certification budget MUST be shared with mask interval bounds in canonical left-first order, including hidden/unused/retained/repeated/shutter content; unresolved intervals MUST reject. Actual requested samples MUST be preflighted before destination inspection, workspace/raster/decode/process actions, and runtime allocations MUST defensively enforce bounds. No truncation, clipping, downgrade or skipped mask SHALL hide excessive work.

#### Scenario: Reject EDT kernel and aggregate memory excess
- **WHEN** otherwise typed inputs exceed distance/coverage/kernel/sample work, shared geometry, live memory or checked arithmetic bounds
- **THEN** core returns nonretryable INVALID_ARGUMENT before publication or render side effects and preserves authoritative state/history/drafts/resources/artifacts

#### Scenario: Reject unsafe curve interiors and singular transforms
- **WHEN** valid endpoints have an unsafe inverse/skew/scale/support interior or continuous certification cannot prove safety within its shared budget
- **THEN** final edit/draft candidates fail atomically and actual render preflight also fails before any output/process side effects

### Requirement: Complete shared sampled masks and readiness
Frame preview, audiovisual range preview, materialized draft preview and final export MUST consume the same immutable evaluated masks at the same source/shutter clocks. Sampled mask facts MUST be renderer-neutral typed geometry/paint/scalars, without persisted paths/backend expressions. mask_animation_v1 MUST identify editor channel support independently of rendering dependencies; mask_rendering_v1 MUST be emitted only when a conforming complete path/paint/shared-scene renderer is ready. Existing mask_models_v1 MUST remain model detection, never substitute for rendering readiness. Missing conforming backend MUST return existing DEPENDENCY_UNAVAILABLE, runtime process/resource errors MUST retain their existing stable semantics and failures MUST publish no partial artifacts. Masks absent/empty MUST preserve existing exact evaluated geometry/clocks/effects/resources and lossless output across all intents.

#### Scenario: Compare every intent at inherited and shutter samples
- **WHEN** painted masks animate through retained/fractional component/repeater clocks, curve/loop boundaries and shutter sub-samples at nonzero requested origins
- **THEN** all four intents expose identical sampled mask semantics, satisfy independent pixel expectations and existing visual/audio/timing tolerance, and reopen retains authored state

#### Scenario: Keep unsupported readiness and no-mask compatibility honest
- **WHEN** rendering dependencies are missing or a project has absent/empty masks
- **THEN** editor animation/model capabilities remain accurate, unavailable mask rendering is not claimed, and no-mask rendering/failure/budget behavior remains unchanged
