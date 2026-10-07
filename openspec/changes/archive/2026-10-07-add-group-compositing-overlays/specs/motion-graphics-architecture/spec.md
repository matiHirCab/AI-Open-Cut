## MODIFIED Requirements

### Requirement: Normative coordinate and compositing semantics
Motion-graphics evaluation MUST use top-left origin with positive X right/Y down, integer-millisecond half-open intervals, explicit units, deterministic bottom-to-top ordering, documented mask/effect/matte/opacity/shutter/blend pipeline, premultiplied alpha and linear-light compositing before output conversion. Active masks MUST retain mask-models coordinate ownership and mask-rendering raster/sample rules before effects. Matte references MUST retain scoped finite DAG/occurrence/exact temporal semantics and isolated transparent provider coverage independent of destination and blend. Schema35 eligible visual occurrences including Caption MUST apply their closed seven-mode blend once after source assembly/shutter averaging against the existing opaque-black-backed current destination; normal/default SHALL remain exact. Model/animation capability support MUST NOT substitute for complete mask/matte/blend readiness.

The sole inward evaluated composition-resource owner MUST publish shared resource admission independently of the optional real matte DAG, together with canonical immutable completed-copy owner/mode facts for pure execution. Existing glyph/font/cursor/live-capacity rules MUST be reused without parallel policy or reverse authored/transport traversal; private owner inventory and architecture evidence MUST be updated together.


Schema37 explicit clip or nonempty effects on Group/ComponentInstance MUST introduce the controlled local isolation boundary below. Existing leaf pipeline scenarios SHALL apply unchanged to uncontrolled hierarchies; controlled children finish local source/masks/effects/relative affine/matte/relative opacity/shutter before aggregate clip/effects and outward owner affine/opacity. Descendant non-normal blend MUST operate against transparent aggregate-local destination; existing normal premultiplied source-over remains exact. Root output keeps its opaque-black backdrop and final clipping. Pure execution MUST consume immutable aggregate/query-frame facts from the existing inward evaluated composition-resource owner, with no reverse authored lookup.

#### Scenario: Resolve equal z-index layers
- **WHEN** multiple visual items in one track share explicit z-index
- **THEN** stable item array order determines paint order and stable item ID remains only the final deterministic tie-break for synthesized/equivalent inputs

#### Scenario: Evaluate an inherited visual
- **WHEN** a visual has supported crop/clip/masks/effects/transforms/matte/opacity/blend and ancestors
- **THEN** source rasterization, crop/clip, masks, effects, local affine, nearest-to-outer ancestors, matte, inherited opacity/transition, owning shutter average and final declared destination blend occur in that order

#### Scenario: Distinguish current and future pipeline stages
- **WHEN** current visuals carry absent or authored schema33/34/35 static/animated masks and schema34/35 typed track mattes, while schema35 activates the seven typed blend selections
- **THEN** authored masks and typed mattes execute their approved semantics and absent masks/mattes remain explicit identity stages; normal destination blending remains exact linear premultiplied source-over, and non-normal blending executes only its approved typed activation after owning source shutter averaging

#### Scenario: Preserve active defaults and complete readiness
- **WHEN** visuals carry active masks/mattes/blend selections or omit those fields
- **THEN** authored stages execute only approved typed semantics, absent mask/matte stages remain identity and absent/normal blend retains exact source-over/opaque-black behavior; incomplete backend support cannot degrade silently

#### Scenario: Keep scope occurrence and paint order independent
- **WHEN** matte providers lie above/below recipients or repeated local IDs occur in differently transformed component instances
- **THEN** scoped dependency execution prepares isolated coverage without blend, then each direct eligible occurrence blends once in unchanged final destination paint order

#### Scenario: Keep Caption source assembly as one blend unit
- **WHEN** one Caption contains multiple lines, glyphs and background-box fragments
- **THEN** existing source-local assembly produces one owning occurrence plane and its declared mode applies once without per-line/glyph/background blending or new group isolation

#### Scenario: Distinguish isolated and root blend backdrops
- **WHEN** controlled children use multiply/screen/add modes beneath positive owner effects and partial opacity
- **THEN** independent transparent-local blend equations differ from opaque root backdrop where expected, and each completed owning child/copy blends once

### Requirement: Canonical group ancestor evaluation
Core MUST evaluate each visual's local affine transform followed by ancestors nearest outward, yielding Mroot ... Mparent Mlocal for column vectors. Group normalized position and anchor MUST resolve against root composition dimensions; groups SHALL have no measured child bounding box. For groups without explicit clip or nonempty effects, ancestor opacity MUST multiply per descendant without isolated group compositing. A descendant's visual interval MUST intersect all ancestor half-open intervals and its visibility MUST include all ancestor item/track visibility. Empty intersections SHALL emit no visual instruction. With zero or absent staggerMs, child timing SHALL remain absolute containing-scope milliseconds. With staggerMs, each direct visual child branch SHALL use the ranked local-clock delay defined by inherited-animation-timing while parent interval and channel sampling remain unshifted. Audio timing/gain/visibility SHALL retain existing behavior independently of visual parents. Uncontrolled groups MUST NOT reorder children or create drawable instructions; controlled groups MUST use the bounded aggregate instructions below.


The legacy matrix/order/distributed-opacity scenarios below MUST retain exact output for the uncontrolled path. For controlled groups, fixed basis still MUST use containing composition dimensions; children MUST preserve relative canonical comparator order inside a contiguous owner block at the group track/zIndex/stackOrder/ID position, even when descendants originally span tracks. No child local z-index MUST escape that new explicit block. Owner interval/visibility MUST apply to the aggregate and descendant relative activity, with no audio behavior change.

#### Scenario: Evaluate nested transforms independently
- **WHEN** asymmetric visuals use nested translation, noncentral anchors, independent scales, skew, rotation, and opacity
- **THEN** evaluated corners agree with an independent matrix oracle within 1e-9 pixels and opacity equals the product of local and ancestor opacity

#### Scenario: Apply visibility and intervals
- **WHEN** a child overlaps only part of ancestor intervals or any ancestor or ancestor track is hidden
- **THEN** visuals use the interval intersection or disappear while existing media audio behavior remains unchanged

#### Scenario: Preserve ordering and legacy behavior
- **WHEN** parented visuals span tracks or unparented legacy scenes are evaluated repeatedly
- **THEN** existing track/zIndex/stackOrder/ID order remains intact, legacy scenes remain equivalent, and evaluation never mutates project or history

#### Scenario: Inherit animated ancestors and staggered clocks
- **WHEN** a supported parent channel animates a nested group and its staggered children
- **THEN** each child uses the sampled parent matrix and opacity with its ranked local clock, while existing stacking and parent interval clipping remain stable

#### Scenario: Preserve defaults and activate explicit group blocks
- **WHEN** otherwise identical cross-track groups omit controls versus author clip/nonempty effects
- **THEN** uncontrolled hierarchy remains byte/pixel equivalent; only controlled hierarchy creates the explicitly placed contiguous aggregate block with independently expected order and exactly-once owner opacity

### Requirement: Hierarchical instance transforms and stacking
Core MUST flatten uncontrolled instances and publish controlled instances as private owned aggregates in EvaluatedScene without persisted references, paths or backend expressions. Local coordinates MUST use definition dimensions; instance position MUST resolve in the containing composition and instance normalized anchor against referenced definition dimensions. Matrices MUST compose outward through local groups, instance transforms and outer groups, multiplying opacity per descendant for uncontrolled instances without implicit fit scaling or isolation; controlled instances MUST apply their owner opacity once after local aggregation. Instance content MUST occupy one contiguous block at its parent track/zIndex/stackOrder/ID position, recursively using the same comparator within each definition. Local z-index MUST NOT escape the instance block. For uncontrolled instances only root output clipping SHALL apply; component dimensions SHALL NOT introduce an implicit clipping mask. Explicit composition_bounds MUST clip controlled instances locally before owner effects. Existing flat scene/group rules SHALL remain unchanged outside instance boundaries.


Existing coordinate/product-opacity scenarios below MUST retain their exact uncontrolled behavior; explicit controlled instance anchor basis MUST remain definition dimensions even for signed/expanded/empty aggregate bounds. Nested controlled instances/groups MUST form recursively completed local planes with stable scoped occurrence identity and no duplicated owner opacity.

#### Scenario: Resolve nested coordinates and order
- **WHEN** instances with different composition sizes, noncentral anchors, skew/rotation and groups overlap root items and other occurrences
- **THEN** corners match an independent matrix oracle within 1e-9 pixels, opacity is the ancestor product and exact visual order preserves each occurrence block

#### Scenario: Preserve shared evaluation and overflow behavior
- **WHEN** the same revision is evaluated repeatedly or composed geometry exceeds existing limits
- **THEN** valid scenes are equal without project/history mutation and overflow uses existing typed failure before raster allocation or artifact preparation

#### Scenario: Distinguish nested isolated instance stages
- **WHEN** two occurrences of one definition use different clocks, noncentral anchors, nested controls and offcanvas descendants
- **THEN** independent local/world matrices and planes prove separate membership, fixed anchors, clip-before-effects, signed support and nested once-only opacity

## ADDED Requirements

### Requirement: Certified controlled aggregate evaluation
Each explicit clip/nonempty effect aggregate MUST resolve closed immutable scope/owner occurrence, local composition basis, retained original clock, relative descendants, outward affine/opacity, ordered effects, visibility/interval and signed local support before pure execution. Aggregate/query descriptor count MUST be included in existing4096 expanded visual occurrence admission; actual vector/map/key/ID/scene-copy capacities and every simultaneously live source/aggregate/shutter/query/output/cache buffer MUST be certified under unchanged1073741824 peak bytes including67108864 cache reserve. Static/sampled local support MUST be derived before world output culling, with checked finite affine bounds and unchanged16384 dimensions/16777216 area. No inverse of singular owner affine MUST be needed to recover child locals. Zero determinant/outward opacity0 MUST follow existing transparent output semantics without division; local work still MUST be certified. Temporal/nested multiplicity MUST be bounded before enumeration, including hidden/unused/retained/component content. Positive particle support MUST survive empty children. matteOnly leaves MUST be omitted from aggregate drawing and hence aggregate blur, while retaining separate named coverage/audio. Failures MUST be sanitized stable INVALID_ARGUMENT or DEPENDENCY_UNAVAILABLE, never partial output.

#### Scenario: Retain signed offcanvas support before culling
- **WHEN** an offcanvas child or nested aggregate is transformed back onscreen or its positive blur brings recipient pixels into output
- **THEN** complete independent plates prove no early world clipping, fixed anchors and certified signed local/query domains

#### Scenario: Reject exact count work capacity and geometry overflow
- **WHEN** aggregate descriptors, local support, nested shutters or actual overlapping live capacities meet each limit or exceed by one
- **THEN** exact bounds pass and overflow rejects before unsafe allocation/resource/output/state publication without dropping sources or resetting predecessor budgets
