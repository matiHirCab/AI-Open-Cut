# visual-effects Specification

## Purpose

Bounded authored leaf effects and deterministic local composition for visual animation.

## Requirements

### Requirement: Bounded authored effect stacks
Core MUST accept an optional `effects` array on visual media, text, solid-color, rectangle, shape, SVG, grid, Group and ComponentInstance items through existing visual-property edits. Omission in stored model data MUST mean an empty stack; existing edit omission MUST preserve the current stack, explicit [] MUST clear it and null MUST reject. Each stack MUST contain at most16 closed tagged records with unique nonempty IDs of at most128 UTF-8 bytes. Supported records MUST remain gaussian_blur with radiusPx, glow with radiusPx/intensity/typed RGBA color, color_tint with typed RGBA color, vignette with amount, color_adjustment with required exposureStops/contrast/saturation, screen_flash and particle_overlay with the closed fields below. Radii MUST be finite[0,128], intensity/amount/color components[0,1]; color-adjustment exposureStops MUST be finite[-8,8], contrast/saturation finite[0,2]. Other effect/item kinds MUST return nonretryable INVALID_ARGUMENT; missing references and stale revisions MUST preserve existing stable errors. Effects MUST introduce no paths/URLs/SVG/scripts/resource references. Existing standalone and alias batch array order, stable effect ID targets and public undo/redo/reopen behavior MUST have automated conformance evidence; the fifth tagged variant requires schema36; the two overlay variants and nonempty group/instance stacks require schema37. Existing five leaf algorithms remain unchanged.


Group/instance effect animation targets MUST remain unsupported, including otherwise compatible older glow/vignette properties; existing leaf channels remain active. New overlay properties and effect.particle_amount MUST remain unsupported. Group/instance masks, matte/matteOnly, motionBlur, non-normal blend and repeater-owner effects MUST remain unsupported; their eligibility SHALL NOT be inferred from effects support.

#### Scenario: Edit bounded effects transactionally
- **WHEN** an eligible item receives valid ordered stacks through standalone or alias-aware batch edits
- **THEN** IDs/order persist in one atomic undoable revision and actual native pixels before/reorder/undo/redo/reopen reproduce independently expected generations through public APIs/protocols

#### Scenario: Reject invalid stacks and references
- **WHEN** edits have duplicate IDs, excessive stack, unsupported target/type, unknown field, null/out-of-bound value, missing item or stale revision, including a later batch/draft operation
- **THEN** existing typed errors retain meaning and complete current/history/draft/revision/resource inventories remain unchanged

#### Scenario: Preserve omission clear and stable target identity
- **WHEN** existing effects are omitted, cleared with[], reversed with identical IDs or targeted by root/component effect channels
- **THEN** omission/clear retain existing meanings, reordering preserves identity/targets rather than old indices and independently sampled public native output proves the resulting order/clock

#### Scenario: Keep aggregate and overlay target activation finite
- **WHEN** new controls are submitted with group effect channels, owner masks/matte/shutter/non-normal blend or overlay animation properties
- **THEN** existing typed unsupported-input failures preserve all state; old supported leaf channels retain exact behavior

### Requirement: Canonical effect composition
Effects MUST operate in declared order on the leaf source or explicitly controlled aggregate's local premultiplied linear-light raster after crop/clip and graphic fill/stroke, before local and ancestor affine transforms, preserving the verified baseline's existing mask/matte/opacity/blend stages. Gaussian blur MUST retain separable normalized Gaussian kernels, sigma=radiusPx, support ceil(3*sigma), transparent extension and zero-radius identity. Glow MUST retain blurred input alpha times intensity/color alpha and original-over-colored-halo composition. Tint MUST mix straight linear RGB toward declared color using color alpha, preserving source alpha. Vignette MUST multiply RGB by `1-amount*clamp((u*u+v*v)/2,0,1)`, u/v ranging−1…1 across unstroked local bounds, preserving alpha. Color adjustment MUST unpremultiply when alpha>0, compute E=RGB*2^exposureStops, C=0.18+contrast*(E-0.18), L=0.2126*C.r+0.7152*C.g+0.0722*C.b and S=L+saturation*(C-L), clamp S to[0,1] only after all three stages, and premultiply by unchanged alpha; alpha0 MUST produce RGB0. Identity(0,1,1) MUST preserve source output without extra quantization. It MUST charge three pixel-pass units per expanded pixel, retaining the aggregate limit. Unstroked anchors MUST remain fixed when blur/glow expand support. Existing16effects/item,4096expanded occurrences and268435456cumulative pixel-pass units/scene sample(each horizontal/vertical tap one pass), stricter surface and shared memory limits MUST remain; excess MUST fail INVALID_ARGUMENT before persistence/output publication without clipping work to hide excess. Existing declared order MUST have independent analytic/native evidence, not merely production-helper inequality.


Screen flash and particles MUST follow the exact bounded equations in the added requirements. For controlled aggregates, descendant completed planes MUST compose against transparent local black in canonical order, spatial clip MUST precede owner effects, and the fixed anchor/outward affine and owner opacity MUST follow owner effects once. Leaf stages remain unchanged. Explicitly activating even an identity aggregate effect creates isolation and SHALL NOT promise equality with the formerly distributed overlapping-child opacity path.

#### Scenario: Preserve ordered effects and anchors
- **WHEN** asymmetric content uses two differently ordered effects and a noncentral rotated anchor
- **THEN** the outputs reflect declared order and expanded support without moving the authored anchor

#### Scenario: Prove independent nonexpanding effect order
- **WHEN** opaque red asymmetric shape bounds32x24 uses vignette amount.8 and green tint alpha.5 in opposite order with quarter-turn/noncentral anchor and owner opacity.5
- **THEN** independent local(4.5,5.5)→world(32.5,16.5) evidence produces prepared-PAM RGB(114,137,0) versus(114,114,0) within1byte on the existing opaque scene; a complete independently authored plate through the existing conversion boundary predicts final PNG within1byte, preserves source alpha/anchor and distinguishes the declared noncommutativity

#### Scenario: Fail bounded work without side effects
- **WHEN** hidden/retained/expanded/component content exceeds an existing effect/raster/certification budget
- **THEN** owning preflight rejects before excessive allocation or state/artifact publication, with named exact/overflow automation and unchanged limits

#### Scenario: Preserve identity and encoded-color interpretation
- **WHEN** zero blur/vignette/tint-alpha identities or tint encoded green.5 are exercised
- **THEN** identities preserve existing source output and independent color conversion matches existing linear tint semantics; identity-only fixtures SHALL NOT substitute for the noncommuting witness

#### Scenario: Distinguish aggregate effects and clip order
- **WHEN** overlapping differently colored children use owner opacity.5, positive Gaussian/glow and composition-bounds clipping
- **THEN** independent local premultiplied plates distinguish aggregate-versus-per-child effects, once-per-aggregate opacity and a halo beyond pre-effect clipping; anchor basis remains fixed

### Requirement: Deterministic parameterized color-control evidence
The fifth leaf effect MUST retain full closed required fields and stable ID semantics, unchanged eligibility/count/UTF-8 limits, and expose no paths/scripts/URLs/resources. Canonical fixtures MUST own endpoint/overflow/unknown/missing-field cases, linear equations and independent source/output witnesses. Existing Gaussian/glow/tint/vignette algorithms SHALL remain unchanged; active color controls MUST have meaningful identity, negative-intermediate, saturation, exposure, alpha and order evidence rather than merely helper inequality. No new animation-channel property is activated; existing incompatible effect targets MUST fail atomically.

#### Scenario: Validate all parameter endpoints and failures
- **WHEN** a color-control record uses finite inclusive endpoints, identity, missing/unknown field, nonfinite/overflow value, duplicate ID, excessive stack or unsupported target
- **THEN** owning validation accepts only the approved closed subset and rejects invalid standalone/later-batch/draft operations without any authoritative/history/draft/resource publication

#### Scenario: Preserve independent color and alpha semantics
- **WHEN** encoded source colors, partial/zero alpha or negative intermediate contrast values use exposure/contrast/saturation
- **THEN** independent floating expectations prove linear conversion, operation order, final-only clamping, alpha preservation and transparent black with existing lossless1byte bounds

#### Scenario: Preserve stable targets and observable order
- **WHEN** color adjustment is placed before/after tint or Gaussian and existing targeted effects retain IDs, or a targeted older effect is retyped to incompatible color adjustment
- **THEN** native output proves declared order and unchanged target identity, while incompatible channel edits/retyping reject before any mutation

### Requirement: Bounded coverage-preserving screen flash
The closed screen_flash record MUST contain id, type, startMs integer[0,60000], durationMs integer[1,60000], intensity finite[0,1] and existing typed RGBA color. Time MUST use the original owner-local canonical SampleTime, not range-relative time. Inside [startMs,startMs+durationMs), strength MUST equal intensity*color.alpha*(1-(t-startMs)/durationMs); elsewhere it MUST be zero. Exact integer/Split comparisons and relative progress MUST precede floating conversion. For premultiplied linear p and alpha A, each channel MUST become p[c]+(A-p[c])*linear(color[c])*strength; A MUST remain unchanged and A0 MUST retain RGB0. Zero intensity/color alpha/inactive intervals MUST be identity. Whole-screen flashes require full-coverage source; empty transparent aggregates MUST remain transparent. Work MUST charge3P per expanded pixel, including identity, against the unchanged cumulative effect bound.

#### Scenario: Prove envelope color and coverage independently
- **WHEN** positive partial-alpha flash renders at start, interior, immediately before exclusive end, exact end and outside, including high integer/fractional owner clocks
- **THEN** independent linear-light expectations prove the half-open decay and preserved alpha; nonzero-origin ranges and repeated/retimed instances use original owner time

#### Scenario: Reject malformed flash atomically
- **WHEN** required fields are missing, unknown or outside integral/finite inclusive bounds in current/history/draft or standalone/later batch
- **THEN** canonical typed validation rejects without changing any generation/resource/destination

### Requirement: Bounded deterministic particle overlay
The closed particle_overlay record MUST contain id, type, count integer[0,256], seed integer[0,4294967295], radiusPx finite[0,16], speedPxPerSecond finite[0,1024], lifetimeMs integer[1,60000] and typed RGBA color. Seed/count MUST never round, clamp or use randomness. Automated structural integer-lane/index-order evidence MUST prove deterministic emission ordering; uniformly colored native overlap plates MUST prove coverage/composition rather than claim to distinguish reversed index iteration. For each index i in ascending [0,count), lane l in0..2 MUST use h=mix32(seed XOR wrapping32((i+1)*0x9e3779b9) XOR wrapping32((l+1)*0x85ebca6b)); mix32 MUST XOR right16, wrapping multiply0x7feb352d, XOR right15, wrapping multiply0x846ca68b, XOR right16. U MUST equal h/4294967296. The fixed emission domain MUST be original unstroked leaf bounds or aggregate composition bounds (group containing-scope dimensions, instance definition dimensions), independent of generated support. A zero-width or zero-height original leaf domain MUST make particle execution identity before any modulo, while preserving certified record/work admission; positive aggregate domains MUST retain empty-owner emission. Base x/y MUST equal domain origin+U0/U1*domain dimensions. Phase MUST equal ((owner-local time modulo lifetimeMs)+U2*lifetimeMs) modulo lifetimeMs; integer/Split whole clocks MUST be reduced modulo before f64 conversion. y MUST wrap domain-relative baseY+speedPxPerSecond*phase/1000 modulo domain height; x MUST remain baseX. Owner-local centers MUST map to raster coordinates by subtracting the raster signed local origin and multiplying by its certified density d; raster radius MUST equal radiusPx*d. Coverage MUST use equal-weight4x4 samples per raster pixel at (x+(a+.5)/4,y+(b+.5)/4), a,b0..3; distance-squared<=raster-radius-squared contributes1/16. Aggregate rasters MUST use d=1; existing leaf certified density MUST remain authoritative. Circle scan boxes MUST cover floor(center-rasterRadius)-1 through ceil(center+rasterRadius)+1 exclusively, intersected with admitted raster support, bounded by ceil(2*rasterRadius)+3 pixels per axis. Typed color MUST convert to linear and premultiply by color alpha*coverage, then source-over in particle-index order after the prior effect result. Count/radius/color-alpha0 MUST be identity. Empty eligible aggregates MUST still emit particles in their fixed positive composition domain. Support MUST include domain expanded by radiusPx in owner coordinates (ceil(radiusPx*d) raster padding) without moving the anchor. Work MUST charge4P for result/source handling plus count*20*(ceil(2*radiusPx*d)+3)^2 units for bounded circle coverage/source-over, including identity, with checked arithmetic; per-particle execution MUST scan only this bounded box rather than the complete surface. Existing268435456 cumulative effect units, source/surface/shared-memory limits MUST remain.

#### Scenario: Prove hash clocks coverage and stacking independently
- **WHEN** seeds0,1,u32MAX, radius endpoints and overlapping particles render under integer/Split/fractional, nested-instance and repeater clocks
- **THEN** independently pinned integer lane vectors and full analytic coverage/linear plates prove determinism, wrapping, overlap coverage/composition, original time and exact high-clock reduction

#### Scenario: Distinguish particle stack order and identities
- **WHEN** particle→positive Gaussian is compared to Gaussian→particle, and count0/radius0/alpha0/changed seed controls render empty or populated aggregates
- **THEN** complete native plates distinguish positive order/seed effects while each identity preserves its input and empty aggregates emit positive particles

#### Scenario: Reject finite bounds and excessive sampled work
- **WHEN** missing/unknown/nonfinite/fractional count/seed, excessive values or cumulative particle/effect/sample work occurs in hidden/retained/component/draft content
- **THEN** owning preflight rejects before allocation/publication with unchanged limits and exact boundary controls

#### Scenario: Retain density ownership and degenerate identity
- **WHEN** a graphic leaf has density>1, fractional local bounds and positive particles, or zero-width/height unstroked bounds
- **THEN** independent mapped-center/4x4raster coverage proves scaled radii and conservative certified work/support; degenerate domains remain identity without NaN/modulo-zero

### Requirement: Explicit bounded composition clip
Only Group and ComponentInstance MUST accept optional stored non-null closed clip:{type:"composition_bounds"}. Stored omission MUST disable clipping; edit omission MUST preserve and edit null MUST clear, with a clear no-op on unsupported items allowed consistently with existing optional visual clears. Non-null clip on ineligible targets, unknown fields/tags and stored null MUST reject INVALID_ARGUMENT. Group basis MUST be the containing composition canvas; instance basis MUST be referenced definition dimensions. Spatial clipping MUST apply to the completed transparent aggregate before owner effects at local pixel centers inside [0,width)x[0,height), with exact half-open hard coverage, not transformed output bounds. Domain/anchor MUST never derive from child union. No implicit clipping MUST apply to uncontrolled instances. Clip/effects MUST persist through existing standalone/alias batch/component edits, undo/redo/draft/reopen.

#### Scenario: Author preserve clear and restore spatial clip
- **WHEN** root/nested groups and unequal-size instances receive clip through existing standalone/batch/component edits then omission/null/undo/redo/reopen
- **THEN** stored/edit semantics, missing/stale errors and independent native clip/anchor output match; temporal interval clipping remains separate

#### Scenario: Reject unsupported or malformed clipping atomically
- **WHEN** stored null or non-null clip on leaves/repeaters or an unknown rect/path/executable/resource field appears later in a batch/draft/history
- **THEN** typed failure preserves every current/history/draft/revision/resource byte
