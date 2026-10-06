## Context and reconciled stable baseline

Read full cached GitHub issue52 and CLOSED #20 parent-DAG prerequisite. Issue52 lists #49/#50/#20, not #51, but the authorized stack places #51 before #52. Its actual stable schema33 source implements active masks, fifteen targeted properties and independently reported mask model/animation/rendering support; the exact verified/archived predecessor is5f456618b9335a5f87519c5c4375a49e7f52c2d3, independently matched against all eight catalogs and expanded MCP. Current owners are model.rs/shape/model, timeline.rs, validation.rs and owner submodules, migrations.rs, store.rs, evaluated_scene.rs and extended_visual/certification, render_plan.rs, render_artifact/extended_visual.rs and raster_cache.rs. Existing frame CPU composition is over opaque black; it cannot be sampled as a matte provider because the destination background destroys alpha. New provider planes must use transparent isolated buffers.

## Goals and bounded scope

Support alpha/luma leaf mattes and explicit matteOnly visibility with shared deterministic dependency evaluation. Eligible recipients/providers are the same visual leaf family as #50: Text, SolidColor, Rectangle, Shape, Svg, Grid, and Image/Video Media whose referenced managed asset is visual. Audio, Caption, Group, Repeater, ComponentInstance and TemplateInstance are ineligible for nondefault matte/matteOnly. This avoids introducing group/component-isolation compositing semantics accidentally. A component's eligible local leaves can reference each other, and independent instances render those references in their own occurrence. Parent transforms, repeaters and inherited clocks still apply to eligible leaves. This eligibility decision is concrete and requires parent/independent review against issue acceptance before promotion.

## Closed model and public edits

`MatteReference` is exactly `{sourceId:<canonical item ID>,channel:"alpha"|"luma"}`. Both fields are required; null, unknown keys, positional forms, unsupported channels and raw duplicate fields at observable Serde boundaries reject. sourceId is an existing canonical item ID with 1…128 UTF-8 bytes; URLs, paths and source-kind expressions are never resolved. A reference has no explicit scope string: the owning leaf's root/component scope selects the lookup table. Identical local IDs in another component are independent and cannot satisfy lookup. Cross-scope references therefore return ITEM_NOT_FOUND; a source resolved in the owning scope but of an ineligible type returns INVALID_ARGUMENT.

VisualProperties gains `matte:Option<MatteReference>` default None/omittable and `matteOnly:bool` default false/omittable. Persisted/root/component visual records reject explicit matte:null and matteOnly:null; serialize defaults by omission. Only the presence-sensitive edit field permits matte:null as a clear operation. Existing update_item gains a presence-sensitive optional nullable matte: omission preserves; object replaces; null clears. Optional matteOnly omission preserves; bool sets; null rejects. No new operation is needed. Creation aliases in both itemId and matte.sourceId resolve through the existing sequential root timeline batch map; backward creation aliases work, unresolved/forward aliases retain existing errors. Local component payload references use stable local IDs, not root aliases. Every final component/root candidate is validated once for reference integrity: deleting a referenced provider rejects unless all affected references are cleared/replaced in the same final atomic batch. No automatic reference deletion, global lookup or guessed fallback occurs. Stale revision remains retryable REVISION_CONFLICT; missing target/reference ITEM_NOT_FOUND; malformed/type/cycle/limit failures nonretryable INVALID_ARGUMENT. All failures retain current/history/draft/resources.

## Scoped DAG and graph limits

Each eligible item may have at most one outgoing matte edge target→provider. Self edges and directed cycles reject, including hidden leaves/tracks, unused definitions and temporally disjoint items. Use iterative O(V+E) traversal, canonical stable item-array order, no recursive call stack tied to authored depth. Existing parent/component/repeater graph validators remain separate owned validators. A matte edge does not change parent/stack ordering and the composition instance graph cannot satisfy a local matte reference.

Named inclusive limits:2048 matte edges/composition,4096 edges/project,32 edges on any provider dependency path. Repeated occurrences retain existing4096 expanded visual occurrence limit rather than multiplying authored edge allowance. These edge bounds are intentionally independently reachable under existing4096visuals/composition and4096total component-item caps: one root star2048edges/2049leaves plus one unused local star2048edges/2049leaves reaches4096project edges; an extra independent local edge exceeds only the new project cap. The initially considered4096/16384edge limits were unreachable for valid acyclic one-edge-per-item graphs under existing caps and are not proposed. Enforce these on all retained current/history source generations and applicable drafts, not merely active visible output. Definitions/resources already have their own unchanged caps. Projects with all matte=None and all matteOnly=false incur no new DAG/render rejection.

## Coordinates, alpha and luma

A provider plane is its isolated source with canonical crop/clip→active masks→ordered effects→local and nearest-to-outer ancestor transforms→its own matte→its inherited opacity and transition gain, on transparent premultiplied linear RGBA. It contains no destination background or destination blend. Normal direct drawing uses each individual evaluated copy plane at that copy’s ordinary stack position. The aggregate provider plane is sampled only by matte consumers and SHALL NOT be drawn once per copy. `matteOnly=true` suppresses only direct destination drawing; it does not change the plane or audio. `hidden` leaf/track/ancestor or inactive time produces zero provider coverage. An inactive existing provider is zero, never identity or a missing-reference fallback.

Provider alpha coverage is A. Luma is `0.2126*R+0.7152*G+0.0722*B` of premultiplied linear components, not encoded RGB, not unpremultiplied color and not alpha repeated. Transparent colored paint contributes zero. Recipient transformed premultiplied RGBA is multiplied once by provider coverage at the identical requested output pixel center, after recipient transforms and before recipient inherited opacity/transition gain. Its own completed premultiplied copy samples then undergo existing shutter averaging before that individual copy destination source-over; provider queries occur at each actual recipient shutter sample, not after recipient averaging. Coverage cannot revive crop/clip removal or increase source support. No inversion field or blend mode is introduced. All working values stay finite in[0,1] with existing1e−6 f32 rounding tolerance; larger violations reject rather than clamp away faults.

## Component, repeater and temporal occurrence resolution

Resolve references first in authored composition scopes. At evaluation bind each root/component occurrence using its unique instance path, retained exact composed clock and canvas; two instances of one definition never share provider planes merely because local IDs match. For a given composition occurrence, a provider ID denotes the ordered transparent source-over aggregate of all evaluated occurrences of that authored leaf in that occurrence, including its generated repeater copies and its ordinarily visible base copy. Each copy keeps its existing affine, inherited timing/stagger and visibility. For each evaluated provider copy, first complete its own matte and shutter-average its isolated premultiplied plane; then source-over aggregate those completed copy planes in canonical evaluated paint order. Do not aggregate copies at each shutter time before averaging: source-over is nonlinear in alpha. Root recipients never reference component-local leaves, and a local recipient never sees sibling component instances. A repeated recipient samples that aggregate in composition/output coordinates, not an arbitrarily selected nearest copy. This aggregate rule is an explicit observable choice; asymmetric offset/stagger fixtures must lock it down.

At a requested exact root sample time each provider copy honors its own existing per-leaf MotionBlur and shutter grid once; average each copy’s isolated premultiplied planes, then aggregate copies and extract channel coverage. Each recipient shutter sample queries the provider at that actual root sample time. Nested matte chains may therefore require nested provider shutter samples; never substitute product-of-averages, freeze to frame center, round inherited clocks early or apply a shutter twice to the same provider request. Memoize identical immutable `(composition-occurrence,provider-id,exact requested sample time,render request)` requests. Existing SampleTime/integer shutter rules (up to16 samples) and provider clocks determine keys; floating approximate-time deduplication is forbidden. No new animation channel target/property is added: existing source/target/mask animation drives the planes. Temporal certification includes the transitive provider reachable times throughout each requested interval, including nested shutter extrema and inherited/repeater mappings.

## Shared scene, bounds and publication

EvaluatedScene must own immutable scoped DAG bindings, provider occurrence membership, matteOnly/direct-draw roles and exact temporal requests. All four intents consume them; headless/MCP cannot build independent matte graphs. Offscreen/matteOnly providers remain included in validation, certification, resources and readiness even when not drawn. Separate source planes from opaque destination canvas. Exact bypass requires BOTH every matte=None AND every matteOnly=false. A matteOnly=true unreferenced leaf still suppresses its direct drawing while retaining ordinary audio/timing. For that complete default condition, bypass is exact, with unchanged mask/effect/audio/clock behavior and ordinary revision-scoped cache admission.

Keep #49/#51 surface16384axis/16777216pixels,4096expanded visual occurrences,268435456destination pixel visits/frame,1073741824peak live bytes/request including67108864cache reservation and all effect/mask caps. New matte work cap268435456 units/output frame across all recipients/providers and shutter samples; provider request cap4096 uncached provider materializations/output frame. Charge provider source work under existing source/mask/effect/affine budgets for every actual provider sample, plus4P for EACH evaluated base/repeater copy source-over into its provider aggregate and5P per coverage extraction+recipient RGBA multiply, where P is touched/requested plane pixel count conservatively certified before execution. Checked arithmetic rejects overflow. Nested shutter multiplicity must be computed/short-circuited when the bound is exceeded before constructing exponential request trees. Memoization is an optional reduction only when keys match exactly and all mandatory source checks still pass.

Live bytes include every simultaneous16-byte RGBA provider/recipient pixel,4-byte scalar coverage, shutter accumulation, cache pin/reservation and compiler/decoder/effect/mask buffers. Compute last-use/reference counts and deterministic provider-first evaluation; do not assume all planes are free or only one provider is live. Count provider-only decoded resources normally; validate managed assets/fonts and full typed graph before cache hits. Request preflight completes before raster allocation/FFmpeg/artifact publication. Oversized valid descriptions return INVALID_ARGUMENT; unavailable complete backend support DEPENDENCY_UNAVAILABLE, without fallback to identity matte. A provider may be requested before its normal paint position, but final destination draw order remains unchanged.

## Migration, drafts, cache and contracts

Schema34 is proposed from verified/archived schema33 #51 commit `5f456618b9335a5f87519c5c4375a49e7f52c2d3`. Missing matte/matteOnly migrate to None/false under project lock for current and all retained undo/redo snapshots in one existing transaction. Preserve IDs/revisions/timestamps/history order and existing assets/provenance/fonts; no read rewrite after adoption. Raw source envelopes<34 reject presence of matte/matteOnly at scoped item fields, even null/false, before new defaults; detect authored operations in retained drafts too, using actual current/matching retained source bases. Future versions reject. Applicable legacy drafts replay on their own retained bases during staged adoption; malformed/dangling/cyclic candidates reject whole migration without writes. Valid schema34 unavailable-base drafts remain unchanged but cannot preview/apply/commit; current read/edit/discard remain usable. Do not replay them onto current state. Retain full existing journal before/after-publication recovery guarantees.

Canonical proposed `track-mattes-v1.json` owns representation, scoped DAG limits, isolated-plane formulas, operation examples, raw rejection fixtures and numeric/native oracles; register every Rust/headless/MCP/bridge consumer and CODEOWNER. Keep headless/MCP parsed-object strictness distinct from raw observable duplicate-key rejection. `matte_models_v1` is editor/schema/DAG availability; `track_mattes_v1` is true only when complete frame/range/draft/export backend readiness covers matte providers, active masks and all existing visual families. Do not report readiness from fixture decoding alone. Reporter status schema literal transitions33→34 and any new capability literals are named reviewed changes. MCP digest may change only through approved fields/capabilities/exact structural literal locations; compare a named predecessor projection to the actual verified #51 digest on promotion, never broad key deletion. Older strict schema33 response clients require an updated response schema; additive request compatibility is qualified accordingly. Revision/evaluated snapshot and cache identities change normally after authored edits; compare render-semantic oracles, not raw entire plan hashes.

## Independent evidence and acceptance mapping

Analytic owner tests independently compute scalar alpha/luma and premultiplied source-over, isolated red(alpha0.5→luma0.1063), green0.5→0.3576, dark opaque and transparent colored sources; shared ancestor opacity, provider/recipient opacity and destination black/colored background must distinguish stage mistakes. Numerically exercise A→B→C chains, source paint order independence, and matteOnly/provider visible equality with changed direct draw only. Include overlapping-copy nonlinear witness: two same-color copies have shutter alpha samples (0.5,0) and (0,0.5); averaging each copy gives0.25 each and aggregated alpha0.4375, whereas the forbidden aggregate-before-average gives0.5. Normal direct drawing must draw the two completed individual copies once, never the aggregate twice. Check all scoped missing/cycle/self/path-depth/aggregate edge boundaries including hidden/unused graphs; no helper-only public acceptance claim.

Public standalone and creation-alias batch conformance sets/clears matte/visibility, reorders provider+target stack, undo/redo/reopen and drafts; failed later operation/current/history/legacy draft candidates preserve complete byte inventories and resources. Native mandatory configured tools/font tests use asymmetric animated source+recipient, active painted masks/effects and noncentral ancestors, repeaters with stagger and two component occurrences. Frame/range/draft/export share analytic expected pixels at multiple times and native SSIM≥0.99, decoded PCM RMS≤0.0001/alignment≤1 output frame; same-intent identical lossless render exact. Audio remains unchanged under matteOnly for video/audio-bearing sources, and rendered duration/time-grid unchanged. Source inclusion readiness, corrupted managed resource before cache hit and all bound failures publish no artifact. Native test runs must be actual opt-in execution with required flags, not reported passing after a conditional early return. Requirements in issue52 are covered by these model/DAG/public/persistence/render/contract scenarios; final reviewer traceability table must cite actual test names and execution evidence after implementation.


## Exact predecessor and canonical transition

`predecessor-catalog-digests.json` pins the actual stable #51 eight complete
semantic catalogs with the same recursive localeCompare/JSON.stringify codec as
existing governance tests. The expanded MCP surface was independently expanded
and hashed as `803bf5954ebd4cb47be98dd87b4994e6d261eae20693199c0f569f535452f170`.
The exact verified/archived predecessor commit5f456618b9335a5f87519c5c4375a49e7f52c2d3 was independently checked using git show: all eight complete semantic digests and raw catalog bytes match, and expanded MCP matches. Any future predecessor change requires explicit reconciliation and independent review before promotion.

All eight active catalogs (six animation catalogs, mask-models-v1 and
mask-rendering-v1) advance only top-level projectSchemaVersion33→34. No existing
mask property, target, status, stored algorithm/order/timing annotation, numeric
limit, fixture or capability changes. Verify each exact34 marker, restore only that
named top-level field to33 and reproduce its complete pinned #51 digest. Retain
existing #51→#50→#49 projections/pins by composing the new marker-only projection
first. mask-rendering-v1 has a genuine #51 predecessor, not a fabricated #50 one.

The MCP transition adds only matte_models_v1 and track_mattes_v1 capability entries,
matte/matteOnly fields in the existing update-item and flattened visual DTO schemas,
and exactly two existing schema literal paths33→34:
`toolDefinitions.editor_get_status.outputSchema.properties.projectSchemaVersion.const`
and `$defs.ProjectGetStateOutputPropertiesProjectProperties.schemaVersion.const`.
The exact existing sibling schema paths are enumerated in
`predecessor-mcp-field-locations.json` (replace the final masks path component with
matte/matteOnly; masks itself is unchanged). The new canonical reference is local,
closed and required-field strict; only genuinely new reachable definitions may be
added. Build the final projection manifest from that explicit list before declaring
catalog changes: verify additions' exact schemas and cardinalities, remove only
those named properties/definitions/capabilities, restore the two literals, and
require the exact pinned expanded #51 digest. No inferred matching-name/key deletion,
reordering, description loss or weakening of old schema assertions is permitted.
All protocol major/tool/prompt/resource counts and unrelated annotations stay unchanged.

## Preserve actual #51 resource and ownership protections

Provider sampling reuses immutable source/clock/contour/paint facts, the post-crop
mask owner top-left zero basis with extent W/density,H/density, and existing
mask→effect→affine order. Source PAM/raster caching remains separate from mask/matte
final composition; revisions still invalidate revision-scoped admission. Reused
source bytes cannot bypass managed media/font integrity or candidate certification.

Every transitive provider sample charges existing268435456 mask-work units/frame,
65536 per-path/1048576 scene segments and shared65536 left-first continuous-analysis
nodes without resetting budgets per edge/request. Keep the actual mask reservation:
4P_owner accumulator;64P+16(W+H) current scalar/EDT scratch;4K Gaussian taps;
64ΣS concurrently retained contour allowance PLUS actual stack/fact/ID/Paint/gradient
and outer/nested Vec capacities, including no-grid/S0 masks and shared authored
program storage; and ADDITIVE opaque fill2048*(S+C+64)+32*(W+H) separate from the
8P simultaneous Pixmap/result allowance. Shared pure runtime verifies actual emitted
L≤S+C, reserves opaque+pixel storage before fill and adopts actual returned capacity.
The existing TLS allocator/math witnesses remain governed tests. New matte provider,
request/cache-key/hash-map/topological/occurrence descriptors and their actual
capacities, closure/sample planning scratch and every simultaneous16P plane,
4P coverage and shutter accumulation are additional live memory; none may be hidden
inside mask scratch or ignored because a source plane is memoized.

Core owns typed model plus composition validation, final edit candidate validation,
migration/source-envelope/staged draft transactions, evaluated DAG/budgets and pure
provider compositing. Intended inward siblings are validation/matte.rs,
evaluated_scene/mattes.rs and render_artifact/mattes.rs; use existing owner entry
points and ADR0003 edges. Backend planning/artifact/process orchestration preserves
its existing seams; adapters consume typed core behavior, never build a second DAG.
Canonical ownership, ADR annotations and architecture tests register every new
consumer/module without a new top-level owner or outward dependency.

Standalone operations validate their complete resulting candidate. Atomic batch
structural/ID/alias resolution still executes sequentially, so forward aliases reject,
but matte reference/deletion/cycle/aggregate checks run ONCE on the final candidate.
In particular `[delete(provider), update(recipient,matte:null)]` must succeed just as
the opposite ordering does; an intermediate dangling edge cannot veto a final-valid
batch. Keep other historical operation/ID/revision semantics and complete rollback.
The current timeline::apply_operation validates per-operation visual candidates;
the implementation must explicitly integrate deferred matte validation for batches
rather than accidentally depending on a favorable operation order.

Canonical existing f32 premultiplied tolerance/arithmetic is used before promoting
any numeric computation: validate alpha[-1e-6,1+1e-6], RGB[-1e-6,A+1e-6], then clamp
alpha[0,1] and RGB[0,A] in that same f32 boundary. Larger/nonfinite violations reject;
never invent an alpha epsilon threshold or unassociate color to calculate luma.
The original default source-over/no-matte path remains unchanged.


## Exact stored/component applicability and public field shapes

The public schema mirrors the closed core model without looking up assets or
resolving references in adapters. Stored/project/component visual DTOs on eligible
Text/SolidColor/Rectangle/Shape/Svg/Grid/Media variants expose matte as OPTIONAL,
NON-NULL closed MatteReference and matteOnly as optional boolean; absent defaults
remain None/false. Media DTOs stay structurally generic: only core asset ownership
can establish Image/Video eligibility or retain ASSET_NOT_FOUND for missing assets.
Nonvisual Media asset use of nondefault mattes rejects in core.

Ineligible Caption/Group/Repeater/ComponentInstance/TemplateInstance/Transition/Audio
visual DTOs expose matte as optional NEVER (absence only) and matteOnly as optional
literal false. Explicit matte:null is rejected in stored/component DTOs, including
ineligible variants. No ineligible shape can encode an object reference or true.
For catalog JSON Schema, the absent-only matte property uses the valid inline
standard JSON Schema {"not":{}}; do not add an unreachable named never definition. Parsed
runtime schemas enforce the equivalent optional-never behavior. Structural schema
consumers must handle this exact approved boolean form rather than relax schema
strictness generally. This is a representation change only at the named new fields.

Shared standalone/batch/draft update-item schemas remain optional nullable matte
(object replaces, null clears, omission preserves) and optional boolean matteOnly
(null rejects). Core final-candidate eligibility permits false/null-clear/default
no-op on an ineligible item, but rejects object matte or true; the shared edit schema
cannot select item eligibility before owning core resolves the target. No adapter
adds a second project/asset/eligibility validator.

`mcp-addition-whitelist.json` explicitly enumerates all58 approved new property
paths derived from the29 exact verified predecessor sibling locations, with each
path's stored/edit applicability and required shape. The two exact structural
schemaVersion/projectSchemaVersion literal paths and two new capability entries
remain the only other surface changes. Before projection verify every exact schema,
optional/non-null/never/literal-false distinction and cardinality, then remove only
these58 properties and their genuinely new reachable reference definitions. A
malformed/missing addition, newly permitted ineligible true/object/null, unrelated
never-schema drift, or unlisted path must fail parity. Retain exact803bf... predecessor
reproduction and composed older proofs.

## Concrete proposed catalog review input

The external `proposed-track-mattes-v1.json` supplies concrete field/limit/independent
numeric/graph/raw cases for this specification review. It is not yet a checked-in
canonical contract and does not authorize implementation. Before canonical authoring,
independently reconcile every value with the approved design,58 exact public paths,
source guards and numeric formulas; preserve existing eight authority pins. Only after
explicit approval may the same reviewed semantics be authored as contracts/track-mattes-v1.json
with all governed consumers/CODEOWNERS registered before declarations change.


Provider-request admission is operational, not merely a unique-key count: each
uncached provider materialization consumes one of4096 units/output frame, including
recomputation of an identical exact key after eviction or when memoization is
omitted. Only a live exact-key memo hit consumes no new request unit; canonical source/resource integrity and all mandatory certification checks still precede the hit. Every actual
recomputed source/mask/effect/affine sample still charges its owning work counters;
every descriptor and memo/live plane remains charged. Optional memoization cannot
permit unbounded repeated materialization under a previously seen key.

## Implementation handoff

The concrete inward interfaces and exclusive file reservations are in core-implementation-interface.md. The pure provider owner consumes only the main-owned pre-certified task schedule and cannot resolve a second authored DAG. Root owns canonical catalog creation before model declarations, all approval/tasks and final publication.

## Verified correction reconciliation and bounded predecessor proof

The exact corrected51 predecessor is5f456618b9335a5f87519c5c4375a49e7f52c2d3, all11CIchecks passed run37373695134attempt2 after two hosted-runner cancellations. All eight catalog bytes, raw MCP/headless catalogs and ownership bytes equal original3ff6db67; no pin or public shape changes. Preserve its three newly synchronized governance/linear oracle requirements when later syncing52. Historic original approval remains in approval.md/artifact-sha256.json; this reconciliation requires independent approval before resumed implementation.

The single existing5000ms deterministic MCP test must retain all five complete current34→33→32→31→pre-linear digest assertions; current34 and oldcurrent33(803bf...) are both asserted in that same test, not moved into separate tests to fit the deadline. Compute the strict compact schema33 predecessor once, independently expand it and assert803bf..., then feed that same compact source to the existing exact33→32 helper. Reuse checked compact32 for32 and31, and preserve current module-first/fresh-second/full-byte comparison/current serialized reuse. No helper, source mutation, timeout extension, proof splitting or broad key deletion change is authorized. New dedicated negative/projection tests remain additional coverage.

## Native verification boundary

Use native-boundary-verification.md for actual prepared-PAM and independent final-codec checks. This preserves task5.1/5.2 semantics, all numerical tolerances and the unchanged production encoding route.

## Exact continuous certification and pinned-source admission

continuous-resource-certification.md specifies exact integer shift/clamp/collision analysis under the existing shared analysis/memory budgets and the private active-matte pinned-media streaming admission port. Approval is required before those mechanism edits; existing wire/default/numerical policies remain unchanged.

## Active glyph measurement resource mechanism

See glyph-memory-certification.md for the exact additive pinned Rustybuzz/table/context/COLR bound, repository allocation-phase ledger, cumulative active-only unchanged glyph limit and concrete private admitted directory lookup contract. Original and finalized scenes, font payloads, shaping cache, measured results, glyph/style/paint capacities and geometry maps coexist and must be admitted before allocation/copy. Defaults retain their existing path and ordering. Approval of the exact mechanism artifacts is required before implementation; allocator/layout/near-budget controls and fresh affected conformance remain required before completion.

## Approved mandatory native CI mechanism

See mandatory-native-ci-plan.md and the repository-validation delta for the approved additive exact command sequence after existing font-resolution conformance: the standalone core native matte suite, default headless build, and dedicated MCP artifact witness. Both existing required native flags and explicit tools/font are inherited. The workflow and authoritative policy constant preserve every prior command/environment/step/dependency/timeout/report/cache-restore/aggregate constraint. Additive regressions reject omission, alteration, success masking and instrumented-build substitution. No domain, public catalog, numerical oracle or resource limit changes accompany this CI correction. Source/policy conformance and actual local native results remain distinct from pending exact-head remote execution.
