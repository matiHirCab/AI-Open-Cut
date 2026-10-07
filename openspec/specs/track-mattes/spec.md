# track-mattes Specification

## Purpose
Define approved composition-scoped track mattes, isolated linear coverage, deterministic sampling and bounded execution.

## Requirements

### Requirement: Closed scoped matte references and visibility
Core MUST accept optional matte references exactly `{sourceId,channel}` with required canonical1…128UTF-8-byte item ID and alpha/luma channel, strict nested object/enum/unknown-field rules and raw duplicate-field rejection where keys are observable. Existing mask-model eligible visual leaves MUST be the only nondefault matte/matteOnly targets and providers. Eligible stored/project/component DTOs MUST expose optional NON-NULL MatteReference and optional boolean matteOnly. Ineligible stored/project/component DTOs MUST expose matte as optional-never (absence only; inline standard JSON Schema {"not":{}}) and matteOnly as optional literal false, rejecting explicit null/object/true. Shared update-item DTOs MUST expose optional nullable matte and optional boolean matteOnly; core final-candidate eligibility MUST accept null-clear/false/default no-op on ineligible targets and reject nondefault object/true. Media asset eligibility and missing asset/reference errors MUST remain canonical core decisions, not transport lookups. Missing visual-record matte MUST mean None and matteOnly MUST default false; persisted/component visual records MUST reject explicit matte:null/matteOnly:null and serialize defaults by omission. Null clearing SHALL be permitted only in the existing presence-sensitive update-item edit field. Existing update_item omission MUST preserve; matte object MUST replace; matte null MUST clear; matteOnly boolean MUST set and null MUST reject. Component payloads MUST accept equivalent eligible local leaves. No paths/URLs/backend expressions/source-kind lookup SHALL resolve a provider.

#### Scenario: Author through standalone and creation-alias batches
- **WHEN** a client creates provider and recipient then sets matte/matteOnly through standalone edits or backward creation aliases in one timeline batch
- **THEN** fields resolve within owning root scope, persist exactly in one revision and undo/redo/reopen/drafts preserve the correct generation

#### Scenario: Preserve defaults and eligibility failures
- **WHEN** omission/clear/default fields, unsupported item types, malformed/null fields or missing references are submitted
- **THEN** omission/clear are deterministic, unsupported/malformed input returns INVALID_ARGUMENT and missing targets/references ITEM_NOT_FOUND without publication or history/resource changes

### Requirement: Composition-scoped finite matte dependency DAG
Core MUST resolve every target→provider edge only within the target's root or component-definition item scope. Self references, directed cycles, missing references and ineligible providers MUST reject. Canonical iterative O(V+E) validation MUST include hidden tracks/leaves, unused definitions and temporally disjoint items. Named inclusive bounds MUST be2048edges/composition,4096edges/project and32edges on a provider dependency path, with checked arithmetic and unchanged existing visual/item/component caps. Matte reference/cycle/deletion/aggregate validation MUST execute once on the final atomic candidate after sequential structural/ID/alias resolution. Final atomic candidate references MUST remain valid after provider deletion/replacement; no automatic clearing or cross-scope fallback SHALL occur.

#### Scenario: Reject hidden scoped cycles and dangling edges
- **WHEN** hidden/unused/disjoint leaves form self/cyclic dependencies, a provider is removed while still referenced or another scope has an identical ID
- **THEN** cycles/ineligible resolved sources return INVALID_ARGUMENT, unresolved scoped sources ITEM_NOT_FOUND and every authoritative/resource byte remains unchanged

#### Scenario: Accept exact bounds and atomic reference replacement
- **WHEN** graphs use exactly each named limit or one additional otherwise-valid edge/path, or a batch removes a provider and clears all references in its final candidate in either delete-then-clear or clear-then-delete order
- **THEN** exact limits and both final-valid operation orders succeed independently of transient dangling references, unresolved forward aliases still reject, overflow rejects before publication and parent/paint-order semantics remain unchanged

### Requirement: Isolated premultiplied provider coverage
Provider planes MUST use transparent premultiplied linear RGBA with canonical crop/clip, active masks, ordered effects, transforms, provider matte, provider inherited opacity/transition gain and existing temporal averaging, excluding destination background/blend. Alpha MUST be A and luma `0.2126R+0.7152G+0.0722B` of those premultiplied linear components. Recipient RGBA MUST multiply coverage exactly once at matching world positions of the certified requested output or local query-frame pixel centers after its transforms and before its own inherited opacity/transition. Each recipient copy MUST then shutter-average its completed premultiplied samples before individual-copy destination source-over; provider coverage MUST be queried at each actual recipient shutter sample, never substituted after recipient averaging. Hidden/inactive providers MUST yield zero. matteOnly MUST suppress direct destination drawing alone, preserve provider coverage/audio/timing and never mean hidden. No inversion/blend selection SHALL be introduced.


Named leaf providers MUST retain full own source crop/masks/effects/transitive matte/world transforms/inherited opacity/transition/visibility/own shutter, but MUST exclude ancestor aggregate spatial clip/effects; controlled aggregates MUST NOT become providers/recipients. The provider product MUST be independent of direct aggregate drawing. Common controlled-owner opacity MUST remain once in provider coverage and once in recipient drawing at its own aggregate boundary; no shared-ancestor factor MUST be removed. A controlled recipient MUST apply coverage in its relative-local plane using corresponding world sample positions before descendant shutter averaging and aggregate owner clip/effects/opacity. matteOnly MUST also suppress aggregate direct contribution and aggregate blur source, while retaining private coverage/audio. No aggregate dependency back into named providers MUST arise.

#### Scenario: Distinguish colored alpha luma and stage order
- **WHEN** differently colored translucent providers matte translucent recipients beneath common ancestors and over different destination backgrounds
- **THEN** independent alpha/luma and premultiplied equations match, transparent color contributes zero, background does not enter coverage and source/recipient opacity is applied once each

#### Scenario: Use matte-only and inactive providers
- **WHEN** a provider toggles matteOnly or is inactive/hidden
- **THEN** matteOnly changes its direct drawing without changing recipient coverage/audio, while inactive/hidden coverage is zero without guessing identity fallback

#### Scenario: Separate private providers from aggregate drawing
- **WHEN** common-owner opacity, ancestor clip/positive blur, matteOnly or cross-isolation alpha/luma provider references affect controlled recipients
- **THEN** independent coverage/recipient plates prove full provider ancestry, excluded ancestor controls, retained common opacity, direct-source suppression and paint-order-independent acyclic execution

### Requirement: Deterministic occurrence and sampled matte semantics
References MUST bind to a unique owning composition occurrence and its retained exact clock/canvas. Each evaluated provider copy MUST first complete its own matte and shutter-average its isolated premultiplied plane; a provider ID MUST then designate canonical transparent source-over aggregation of those completed base/repeater copy planes within that occurrence, retaining transforms/stagger/visibility; repeated recipients MUST sample that aggregate in shared world coordinates represented by the certified requested query frame. Independent component instances MUST retain isolated provider membership. Normal direct provider drawing MUST draw each individual copy plane once at its own paint position; the aggregate SHALL be used only by matte consumers and SHALL NOT be drawn per copy. Copies SHALL NOT be source-over aggregated before their individual shutter averaging. At each recipient shutter sample, providers MUST honor their own existing shutter grid once, including transitive provider dependencies, with exact-key memoization only; no approximate time deduplication, center freezing or early inherited-time rounding SHALL occur. Existing animation properties MUST remain effective without adding matte animation targets.


Each query MUST carry an immutable finite origin/two affine axes/dimensions mapping its pixel centers into world coordinates; providers MUST raster/sample their own source in that requested frame directly, including outside final outputcanvas, without resampling an already output-clipped plane. Default uncontrolled frame MUST be the exact prior output grid. Memo identity MUST include exact query basis/domain, scoped occurrence, exact clock and all prior key facts; different domains MUST never alias. Provider full-world shutters MUST remain independent from controlled direct-drawing boundary freezes.

#### Scenario: Bind two component occurrences and staggered copies
- **WHEN** one definition is instantiated twice with different clocks/ancestors and its local source/recipient have staggered repeater copies
- **THEN** asymmetric native/numeric evidence matches declared aggregate/membership semantics and neither sibling instance nor nearest-copy guessing supplies coverage

#### Scenario: Distinguish overlapping copy averaging order
- **WHEN** two overlapping same-color copies have shutter alpha samples (0.5,0) and (0,0.5)
- **THEN** averaged copy alpha0.25 each produces aggregate0.4375, rejects forbidden aggregate-before-average0.5, and ordinary direct drawing draws each completed copy once

#### Scenario: Preserve nested shutter and exact clock sampling
- **WHEN** provider/recipient/mask animation and distinct provider/recipient motion blur sample fractional inherited or large integer clocks
- **THEN** exact source requests and temporal averages match independent sampled equations, with no product-of-averages substitution or silently discarded samples

#### Scenario: Query certified offcanvas provider domains
- **WHEN** aggregate Gaussian brings a matted offcanvas recipient onscreen, or one provider is queried in two unequal rotated/local frames with fractional shutters
- **THEN** independent native plates prove retained offcanvas coverage, no double filtering, exact query key isolation and unchanged private full-world sample grids

### Requirement: Preflight matte work and live memory
Complete evaluated DAG/provider resource/time certification MUST precede raster allocation, decoder execution and artifact publication. Existing surface, occurrence, mask/effect/destination-work and1073741824peak-live-byte limits including67108864cache reserve MUST remain. New inclusive limits MUST be268435456matte units and4096uncached provider materializations/output frame, summed across every dependency and shutter request. Existing provider source/mask/effect/affine work MUST be charged per actual provider sample, plus4P for EACH evaluated base/repeater copy source-over into its provider aggregate and5P/channel-and-recipient multiplication. Each uncached provider materialization MUST consume a request unit, including identical-key recomputation after eviction or absent memoization; only a live exact-key memo hit MAY avoid a new unit, while canonical source/resource integrity and all mandatory certification checks MUST still precede that hit. All actual source/sample work and descriptor/live-memory charges MUST remain. Nested multiplicity MUST reject before an exponential request tree is materialized. Live bytes MUST include all simultaneously live16byte RGBA planes,4byte scalar coverage, accumulators, resources/cache, DAG/occurrence/request/memoization descriptors and actual outer/nested capacities plus all existing buffers. Existing mask reservations MUST retain separate4P_owner accumulation,64P+16(W+H) scalar/EDT scratch,4K Gaussian taps,64ΣS concurrent contours plus actual retained fact/ID/Paint/Vec capacities including S0/no-grid masks, and ADDITIVE2048*(S+C+64)+32*(W+H) opaque shared-fill transient storage separate from8P Pixmap/result storage. Existing mask/effect/source work, shared geometry and65536 left-first analysis budgets SHALL NOT reset per matte provider/request. Checked preflight MUST bound nested shutter multiplication before enumeration; memo tables/plans and pure runtime MUST enforce certified request/memory/work limits even after source-cache hits. Hidden/offscreen/matteOnly providers SHALL NOT evade model/resource validation or temporal certification. Active-matte text preflight MUST admit all simultaneously live original/finalized scenes, prepared font bytes, shaping/Unicode/document scratch, glyph face/color/paint payloads, cache/measured/finalized clones, map/key/warning descriptors and legacy resolved-font path/run/listing transients BEFORE their allocation or growth. Actual outer/nested capacities MUST be adopted while their source/copy/transient overlaps remain admitted. The unchanged16384 final glyph limit MUST also bound active pending-plus-emitted cluster glyph payload before cloning; each fitting attempt resets that count. Private active font-directory lookup MUST enforce an admitted whole-call allocation contract before iterator advancement or traversal growth and fail closed when the platform/port cannot certify it, without adding authored limits or changing the no-matte route.


Additional controlled query-frame domains, actual descriptor/key/vector/plane capacities and simultaneously live aggregate/query/provider/coverage/shutter buffers MUST be admitted before growth/allocation by the same inward resource owner. Every query-frame materialization MUST charge the existing request unit and actual provider/matte/effect/source/sample work; unchanged4P per provider copy aggregation and5P per coverage extraction/recipient multiplication MUST apply using requested-domain P. Exact memo reuse MUST remain a live full-key hit only, with all mandatory validation/admission first. Domain enlargement and nested multiplicity MUST be checked before world culling or enumeration; no cache hit or hidden ancestor MUST bypass limits. All predecessor font/glyph/mask/EDT/opaque-fill/cache reservations above MUST remain unchanged.

#### Scenario: Reject work memory and nested multiplicity overflow
- **WHEN** valid transformed/provider chains or shutter multiplication exceed any certified work/request/live-memory bound
- **THEN** INVALID_ARGUMENT occurs before raster/FFmpeg/publication, exact boundaries succeed and no provider or sample is dropped to fit

#### Scenario: Preserve exact absence and complete readiness
- **WHEN** a project has every matte=None and every matteOnly=false, or a backend cannot execute every required matte provider and existing stage
- **THEN** that complete default condition uses the exact existing bypass; matteOnly=true without references still suppresses direct drawing while incomplete support yields DEPENDENCY_UNAVAILABLE before output, with no identity/degraded fallback

#### Scenario: Preserve existing mask reservations under dependency reuse
- **WHEN** provider chains reuse masked sources with high-segment tiny grids, large Gaussian kernels, no-grid64-stop masks, spare Vec capacities or near-exhausted shared analysis/segment counters
- **THEN** every simultaneous actual buffer and repeated sample remains charged, existing additive opaque-fill and capacity controls pass, and excess rejects atomically before raster/decoder/publication without resetting predecessor budgets

#### Scenario: Reject invalid premultiplied provider data
- **WHEN** typed runtime planes contain nonfinite/larger-than-canonicalf32-tolerance values or legal near-zero-alpha boundary roundoff
- **THEN** invalid values fail without recipient mutation, legal values clamp in the same existingf32 boundary to alpha[0,1]/RGB[0,A] before alpha/luma sampling and no alpha epsilon cutoff changes coverage

#### Scenario: Certify aggregate query memory and multiplicity
- **WHEN** nested local matte queries, enlarged offcanvas domains, cache eviction or spare-capacity overlaps approach existing limits
- **THEN** exact boundaries and additive descriptor/plane charges pass; overflow rejects before unsafe enumeration/allocation/FFmpeg/publication without narrowing coverage or resetting work
