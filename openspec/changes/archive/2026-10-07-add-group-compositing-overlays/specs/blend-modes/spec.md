## MODIFIED Requirements

### Requirement: Seven premultiplied linear destination blend equations
Working pixels MUST be finite premultiplied linear Cs/As and Cd/Ad with existing1e−6 rounding tolerance. Each non-normal source/backdrop pixel MUST first validate using the SAME existing source_over_at f32 comparisons/arithmetic boundaries, THEN clamp alpha to[0,1] and RGB to[0,clamped A] in f32 BEFORE promoting corrected values to f64 for unassociation/blend equations; larger invalidity MUST reject. A fresh pre-correction f64 tolerance check SHALL NOT narrow canonical f32-accepted endpoints. For positive alpha let s=Cs/As and d=Cd/Ad, otherwise canonical straight zero without division or epsilon threshold. Non-normal B MUST be multiply=s*d, screen=s+d−s*d, overlay=2*s*d for d≤0.5 else1−2*(1−s)*(1−d), add=min(1,s+d), darken=min(s,d), lighten=max(s,d), independently per channel. Output MUST be Co=(1−As)*Cd+(1−Ad)*Cs+As*Ad*B and Ao=As+Ad−As*Ad. Non-normal equations MUST use f64 from retained f32 values and convert validated results back to f32. Normal MUST use the exact existing f32 source-over path. Only existing bounded roundoff SHALL clamp; larger nonfinite/nonpremultiplied values reject.


Schema37 controlled aggregate-local child destinations MUST start transparent black, using these exact equations and source-over alpha. Root output MUST retain the opaque-black witness above. Aggregate owners remain normal-only, applying owner opacity once before normal blending into their enclosing destination.

#### Scenario: Match independent translucent and transparent unit equations
- **WHEN** independent oracles exercise all modes with partial/zero/near-positive-zero alpha, saturated add and overlay backdrop below/at/above0.5
- **THEN** numeric results match within1e−6, alpha remains ordinary source-over, source/destination zero-alpha identities hold and no encoded RGB or source-triggered overlay is substituted

#### Scenario: Correct accepted near-alpha roundoff before division
- **WHEN** a source has f32 As≈1e−9 and Cs at canonical f32(As+1e−6_f32), or the next positive representable f32 beyond that boundary
- **THEN** validated RGB clamps to A before unassociation, straight color is1 rather than1001, a tighter independent corrected-f32 oracle distinguishes the accepted output and the next representable step rejects under the same canonical f32 boundary without epsilon-alpha thresholds

#### Scenario: Preserve actual opaque black background
- **WHEN** the first root scene layer uses multiply/screen or a pure unit helper uses transparent backdrop
- **THEN** actual scenes retain initial opaque black, including multiply-to-black, while mathematical transparent-backdrop helper cases SHALL NOT change project/export background semantics

#### Scenario: Distinguish transparent controlled and opaque root destinations
- **WHEN** a positive child uses multiply/screen/add as the first child of a controlled aggregate versus first root layer
- **THEN** independent alpha/color equations prove transparent-local coverage and unchanged multiply-to-black root behavior

### Requirement: One final blend per owning evaluated occurrence
Masks/effects/transforms/mattes/opacity/transition and existing source shutter averaging MUST complete before the owning occurrence blends once against the current destination in canonical paint order. Caption multiline glyphs/background and Text rich paint fragments MUST assemble into one existing owning source plane first; they SHALL NOT blend independently per line/glyph/paint fragment. Repeater copies and component-expanded leaves MUST each retain the owning leaf's selection and draw once; uncontrolled structural ancestors SHALL NOT impose an isolated blend. Blend mode MUST remain static with no new animation target/property.


Explicit schema37 controlled ancestors MUST compose completed descendant copies once into their transparent local destination, then clip/effect and normal-blend the completed owner plane into its enclosing destination after owner gain/affine. Child non-normal work and closure MUST retain these exact equations/order; no child mode MUST be applied again when the aggregate draws. Named provider products continue to ignore destination blend selection.

#### Scenario: Distinguish source averaging from averaging nonlinear blend
- **WHEN** opaque source samples0 and1 over backdrop0.75 use add
- **THEN** source average0.5 blends to1 and forbidden blend-then-average0.875 fails independent temporal evidence

#### Scenario: Blend multiline Caption as one source occurrence
- **WHEN** a multiline Caption contains glyph and translucent background-box fragments over a colored destination
- **THEN** the existing whole Caption source is assembled then blended once, preserving caption layout/resources/positioning and distinguishing fragment/line-wise blending through independent native pixels

#### Scenario: Observe order and inherited stage effects
- **WHEN** noncommuting modes/layers are reordered under masks/effects/mattes/ancestors or propagated through repeats/components
- **THEN** independent all-intent output reflects canonical paint order and premultiplied source-stage gain exactly once on the uncontrolled hierarchy path without implicit group isolation

#### Scenario: Retain one child blend inside explicit isolation
- **WHEN** a controlled group/instance contains non-normal repeated/component children with masks/mattes/shutters and owner effects
- **THEN** independent plates prove each completed copy blends once locally, owner draws normal once outward and private coverage remains independent

### Requirement: Bounded fail-closed blend execution and normal identity
Existing source/mask/matte/effect/shutter/destination limits and1073741824peak-live-byte reservation including67108864cache reserve MUST remain. New inclusive non-normal work MUST be268435456units/output frame, charging32 checked units per conservative certified destination pixel visit per actual non-normal direct owning occurrence after shutter average. Hidden/unused model validation and ordinary provider/resource checks MUST precede cache/readiness/output. Non-normal blending MUST use fixed pixel scratch, with any extra buffer charged to shared live memory; no full straight-color image or unbounded destination cache SHALL be admitted. Budget/overflow failure MUST be INVALID_ARGUMENT before allocation/decoder/publication; incomplete complete-backend readiness DEPENDENCY_UNAVAILABLE without normal fallback. Absent/explicit-normal blend with no nondefault matte program and no controlled aggregate MUST preserve the exact predecessor bypass, output, counters and acceptance. In schedules requiring bounded composition preparation, normal blending MUST preserve predecessor arithmetic/output and existing source, destination, mask, effect, shutter, matte-work and provider-request charging rules and maxima. Every actually live composition-resource header, DirectDraw descriptor, owner_modes table and associated capacity, clone or growth overlap MUST nevertheless be admitted under the unchanged peak-live-byte limit; normal selection SHALL NOT exempt this storage. Additional metadata may cause an otherwise marginal request to exceed that unchanged memory limit and MUST reject before side effects.

Shared evaluated CompositionResourceFacts MUST own one live/font ledger independently of optional real MatteGraph and activate bounded preparation for non-normal graph-free scenes and nondefault matte programs, including retained hidden/unused obligations. The exact prior all-normal/no-matte/no-controlled-aggregate branch SHALL introduce no new traversal/allocation/counter. The existing inward scene/graph/text/glyph/style/capacity walkers, MeasurementMemory, configured path/font/cache/measured/finalized clone/pinned-integrity guards and bounded lookup ports MUST be reused without duplicate policy or dummy provider graphs. All simultaneously live actual capacities, source/finalized/measurement overlap and descriptors/growth MUST remain admitted under the same1GiB/64MiB/20P certificate. Existing16384pending-plus-emitted glyph admission MUST precede payload clones and reset each fitting attempt; whole-call directory/cursor/traversal admission MUST precede advancement/allocation and fail closed on uncertifiable ports while preserving default fallback and platform ownership controls. Existing source/mask/effect/shutter/65536shared counters MUST NOT reset.

Only actual AggregateProvider materializations MUST charge4096provider requests; ordinary leaves/copies/direct draws SHALL NOT consume that counter. A graph-free schedule MUST retain provider=None and zero synthetic provider requests. Immutable canonical owner_modes and completed DirectDraw/AverageCopy owner facts MUST permit pure index/owner/mode/order/capacity/certificate validation without reverse authored traversal. Every direct mode MUST match its canonical owner fact; every averaged sample MUST share the owner. Provider aggregation MUST remain normal and never be direct-drawable. Original destination MUST be validated without modification before all callbacks, complete-plane retention and admissions finish. All completed direct planes/descriptors/reallocation overlap MUST fit peak memory; final commit MUST have zero fallible checks/callbacks/admissions, relying on proven bounded equation closure and exact old normal arithmetic. Late failure MUST leave destination bytes intact without a second full destination surface.

Pure executor entry ordering MUST first validate original destination dimensions and every canonical premultiplied pixel read-only, before descriptor allocation/admission or any callback; invalid destination yields zero callbacks and unchanged bytes. Only then validate/admit complete descriptors and source execution. Final commit MUST perform no allocation, callback, admission or fallible check.


Controlled aggregates MUST activate the same shared inward resource admission even with all-normal/no-matte children, without dummy providers. Their local destinations/query descriptors/scratch/scene copies and actual growth overlaps MUST be admitted together with all retained buffers under unchanged limits. Non-normal child work MUST retain32units per conservative local destination visit after completed child average; aggregate normal owner drawing MUST retain source-over work/closure. Only actual private AggregateProvider query materializations MUST charge provider requests; ordinary aggregate drawing MUST NOT fabricate providers. Existing normal default scenarios below remain exact when no explicit aggregate controls exist.

#### Scenario: Preserve exact normal and reject bounded overflow
- **WHEN** default/explicit-normal scenes or exact/overflow non-normal certified workloads are rendered
- **THEN** normal bytes/goldens and the no-matte bypass remain exact, normal-with-mattes retains old work/provider charging, exact-limit actual metadata/work admission succeeds when all other existing limits fit, and excess rejects before any degraded/partial output without a normal memory exemption

#### Scenario: Activate shared bounded resources without a provider graph
- **WHEN** non-normal graph-free real pinned Text or ordinary/affine Caption exercises fitting, cached painted glyphs, configured paths, pending glyphs, measured/finalized copies, directory lookup or hidden/unused resource pins near the unchanged memory bound
- **THEN** the same pre-allocation/source-integrity guards pass exact bounds or reject before clone/cursor/workspace/output, no dummy matte graph appears, and default-normal/no-matte output and fallback remain exact

#### Scenario: Preserve provider-only accounting and completed owner facts
- **WHEN**4096ordinary one-pixel occurrences each use16shutter samples within existing limits, or actual provider requests meet4096/exceed4097, or a direct/average owner or mode/capacity fact is forged
- **THEN**65536ordinary leaf visits execute with zero provider requests, provider limits remain exact, all shared source/mask/effect/analysis counters persist, and forged facts reject before callbacks or destination mutation

#### Scenario: Preserve original destination until infallible commit
- **WHEN** original destination data or direct owner/mode/descriptors fail before source callbacks, or a late callback pixel/resource failure follows earlier valid source preparation
- **THEN** invalid original destination/descriptors invoke zero callbacks and every failure leaves original destination bytes unchanged; successful complete retained planes fit admitted peak and all final equations/old-normal arithmetic execute through a finite bounded zero-failure commit without a second destination image

#### Scenario: Retain bounded blend closure through controlled draws
- **WHEN** all-normal/no-matte controlled groups or non-normal local children approach work/capacity limits or a late source callback fails
- **THEN** same inward ledger admits actual local buffers/counters and zero synthetic provider requests, overflow/late failures preserve output and no extra encoding/fallible partial commit occurs
