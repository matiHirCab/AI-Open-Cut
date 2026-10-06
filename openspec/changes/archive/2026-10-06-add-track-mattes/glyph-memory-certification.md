# Active-matte glyph measurement resource certification

This mechanism amendment specifies implementation of the existing shared 1 GiB live-memory requirement. It changes no authored fields, glyph/work quotas, public API, codec, or no-matte behavior. No implementation of this proposal has been applied yet.

## Ownership and entry points

Main requests additional ownership of `fonts/shaping.rs` and `evaluated_scene/text_layout.rs` for private active-only entry points. Existing `shape`, `shape_with_layout`, and `resolve` remain default wrappers selecting the old path. New private admitted variants select the same algorithm with an early cumulative cluster-glyph guard. Render-artifact measurement selects admitted variants only when an active matte graph exists.

The existing exact shaped-text heap walker in `evaluated_scene/mattes.rs` will become `shaped_heap_bytes(&ShapedText) -> Result<u64, CoreError>` and remain the single byte definition for scene and measurement accounting. It includes actual capacities of glyph, line-width and glyph-line vectors, each glyph face/color String, and each optional paint vector and its color Strings. Artifact and renderer already depend inward on evaluated-scene; fonts must never import evaluated-scene or artifact policy.

`measure_evaluated_text_layers_with_budget` retains its default signature/behavior. A private implementation accepts an optional active measurement ledger containing an already-admitted fixed-live base and checked retained/scratch counters. The renderer constructs that ledger only after admitting the original scene, caller clone, graph, actual prepared font payloads, cache reservation, destination/work buffers and external measurement-font transient reserve.

## Required pre-admission and actual adoption

Before even the first font-binding map, document clone, cache-key serialization, shape call, cache-hit clone or measurement result is allocated, admit the associated conservative map/document/key/shaping bound against fixed live plus already-retained cache and results. Every checked arithmetic overflow is INVALID_ARGUMENT.

For each pinned text call, compute the worst per-glyph owned payload from the actual input's longest selected face/color and largest span paint payload, including Vec/header storage. Use the unchanged maximum 16,384 glyphs and 4,096 lines, with geometric Vec growth and old/new allocation overlap allowances. The scratch reservation also includes the actual input document/spans/runs, JSON key serialization growth, styles/paint-range expansions, Unicode bidi/linebreak/segment/cluster maps and line buffers. This is one current-call reservation, released after adoption; it is not charged once per entire scene at maximum glyph count.

Pinned rustybuzz 0.20.1 has two heap buffers (`info`, `pos`), both guarded by `max(initial_len * 64, 16_384)` before resize. The per-call bound must use this larger expansion bound and old+new growth overlap, not just the final 16,384 output limit. The precise multiplier will be justified against the pinned info/position layouts and buffer source before code review; a guess at an opaque constant is insufficient.

The existing shaping code only checks each segment's glyph count before accumulating paragraph clusters. The active variant therefore checks the unchanged cumulative 16,384 count BEFORE each cluster glyph clone/push, including previously emitted paragraphs. Without this early guard, a paragraph with many small independently shaped segments can retain up to segment-count times 16,384 payloads before the final existing check. The default variant remains unchanged; the active guard only rejects candidates that would already exceed the existing successful-output limit.

After each cache/result insertion, measure actual cache capacity, keys, shaped payloads, result-map capacity, item IDs, content, PreparedText paths/runs and cloned style/document payload. Adopt those bytes while scratch remains admitted. Cache and result clones coexist and are counted separately. Reserve HashMap growth overlap before insertion. Do not retroactively rely on a post-allocation measurement to cover allocation.

Before the renderer copies each measured shaped result into the finalized scene, admit that exact actual shaped payload plus allocation-header/growth allowance while the complete measured map and cache/result lifetime are accounted. Cache drops at measurement return; measured map remains live until materialization consumes it. Charge measured retained bytes as a graph resource fact; finalized shaped bytes are counted by the scene's existing walker, avoiding double counting their own payload. Measurements/asset-size map capacities must also be admitted before allocation and remain counted until their drop.

Legacy text/Caption measurement uses the already bounded measurement-font port but must join the same document/content/run/map ledger. A cache hit bypasses shaping scratch, but must pre-admit the clone payload and cache-key/document allocation.

## Meaningful automated controls

1. Tiny real pinned Text plus matte with a near-1 GiB fixed reservation rejects INVALID_ARGUMENT before an injected shaping/measurement admission spy increments, with zero workspace/process/output publication. The same real-font fixture fits and passes normally.
2. Repeated identical pinned text produces a cache hit and separately charges live cache, measured copies and finalized copies. A boundary placed between one and two copies rejects before the second clone.
3. Distinct rich documents with per-glyph paint layers independently compare exact actual face/color/paint-vector capacities against the shared shaped walker; spare capacities and checked overflow are covered.
4. A pathological segmented document exercises the active early cumulative cluster guard while preserving the existing 16,384 limit and default no-matte geometry/output controls.
5. Actual opaque shaping allocator witness on the pinned dependency verifies typical, long/mixed-script and richly painted inputs stay below the admitted transient reservation, including realloc overlap. These witnesses supplement the source-derived bound; they do not replace it.

After implementation and focused controls, the prior native4 pinned-source PASS remains qualified to its frozen input manifest. Full required tests/native/public gates must use the new stable source.

## Exact proposed checked numerical envelope for independent review

Use the additive pinned-source derivation in `rustybuzz-opaque-shape-heap-bound.md`, rather than an umbrella constant. Define Gvec(n,z)=0 when n=0, otherwise3*max(4,n)*z. All arithmetic is checked, and all census loops are borrowing/allocation-free with early failure as soon as the running reservation exceeds remaining live memory.

For one actual admitted face, opaque plan memory is the sum of:

1. GSUB/GPOS eager outer lookup Vecs: Gvec(Lg,128)+Gvec(Lp,128), plus EACH lookup's Gvec(declaredSubtableCount,sizeof(public subtable enum)). Count aliased logical records repeatedly. Public enum layouts are obtained by size_of, not guessed.
2. Static feature maps: Gvec(48,48)+Gvec(48,64)+2*(Gvec(17,16)+Gvec(1,16)). This covers both planner and finished GSUB/GPOS stage arrays, with exactly48 feature pushes source upper bound.
3. Dynamic map lookups: Gvec(2*Qg,16)+Gvec(2*Qp,16). Q is the sum across ALL font feature-index arrays, using the actual default-coordinate variation substitution; factor2 covers the required-feature duplicate.
4. Fixed selected-shaper Box:256bytes. Actual language `und` clone/working allocation:32bytes (no arbitrary-language input).
5. AAT: Gvec(C,3*sizeof(usize))+C*Gvec(1,12)+Gvec(1,16), with exactly one range per actual borrowed morx chain under empty user features.
6. Context SmallVec:65*3*64*sizeof(usize) (outer lookup plus64 recursive levels, context cap64).
7. Reachable static COLRv1 stacks: Gvec(130,24)+Gvec(129,24)+Gvec(193,sizeof(ttf_parser::Transform))+Gvec(129,4). Retain COLR support; do not reject accepted font formats.

Use the maximum opaque bound of the selected font faces, since a single segment's fresh Face/plan is live at a time. All prepared font payloads remain separately counted.

Let N be the document's UTF8 byte count (<=4096), G=16384, L=4096. H is size_of(ShapedGlyph) plus maximum actual selected face/color String and maximum actual span paint Vec payload (headers and color capacities). D is existing exact owning text heap plus header. R=max(64*N,16384).

Current-call shaping scratch is:

`opaquePlan + 120*R + 8*G*H + 2048*(N+1) + 24*(G+L) + 16*D`.

The independent 24G term explicitly counts glyph-line Vec final/growth overlap;24L counts line-width Vec overlap.8GH covers pending cluster glyph payload, emitted output and overlapping copies/reallocation. Each attempt's active cumulative admission constrains all previously emitted and pending paragraph glyphs to the unchangedG before cloning glyph payload.

Repository-side phase table uses peaks, not an unjustified sum of unrelated construction owners. Bidi construction occurs before any Segment/Cluster glyph owners: prepared/implicit vectors, nested sequence Vec headers/minimum capacities, bracket pairs (FOUR usize=32bytes), active bracket stack and original classes/levels/paragraphs fit a conservative1024*(N+1) envelope with the explicit pinned unicode-bidi census. During shaping, only completed BidiInfo remains: original classes/levels/paragraphs<=64*(N+1); hard-line Vec growth<=48*(N+1); segment descriptor+actual64byte face ID growth<=384*(N+1); cluster BTree nodes plus descriptor conversion Vec<=512*(N+1); linebreak BTreeSet<=96*(N+1); adjusted/reordered bidi arrays<=64*(N+1); style tuples/cut/grapheme/effective descriptors<=272*(N+1). This conservative total1440*(N+1) fits the separate2048*(N+1) reserve. Output glyph/line payloads use8GH+24(G+L), not a fictional N-only count. Record/growth ceilings must be source-pinned in owning tests. Effective pieces<=originalRuns+2*spans<=768; whole-original-run clone before substring replacement and advanced resolve's second document clone are explicitly covered by16D. Source paint vectors remain accounted as actual copied payloads, not implicit per-byte metadata.

After shaping/cache/result insertion, actual owned payload accounting replaces transient reservation. Clone admission uses three times exact owned payload plus explicit key/style/header copies. Map insertion pre-admits old/new bucket arrays; retained maps use actual capacity-derived bucket upper bounds. Legacy text uses document/path/run/metric and directory-specific bounds rather than the Rustybuzz/G shaping term.

This exact numerical envelope awaits independent approval and owner layout/allocator controls; no implementation has been applied.

## Explicit allocation lifetimes and accounting handoffs

| Phase | Concurrent owned buffers | Admission/adoption |
| --- | --- | --- |
| Before measurement | Original evaluated scene/graph, finalized caller clone, prepared unique font payloads + maps/sidecar, raster-cache reserve, destination/work baseline, external-font read reserve | Existing scene/graph/fixed walker supplies the starting live base. Admit font-binding map buckets before building the map. |
| Current layer key preparation | Previous shaped cache + measured result map, cloned document/spans/runs, serialization key, current font-binding reference | Admit document/key scratch against retained actual bytes before any clone/serialization. |
| Cache miss / fitting attempt | Previous cache/result maps plus document/key, Rustybuzz Face eager lookup/subtables and plan, buffer info/positions, effective styles/paint ranges, Unicode bidi/segments, pending cluster tree, emitted ShapedText (including overlap during vector growth) | Admit the complete one-call envelope before entry. Each fitting attempt drops the rejected ShapedText before the next attempt; early active total-glyph guard resets per attempt. |
| Cache insert | Previous cache/results plus current returned shape and a cache clone, old/new cache buckets during growth; shaping opaque transients have dropped | Pre-admit exact clone bound plus cache bucket growth BEFORE insert. Adopt actual key/cached shaped capacity while the current result is still live. |
| Cache hit | Previous cache/results plus document/key + cloned cached shape; no Rustybuzz/transient shaping buffers | Admit actual clone payload and key/document scratch, then adopt actual measured output. No maximum shape scratch needed. |
| Measure result insert | Previous cache/results plus current shape, cloned style, PreparedText/content and item ID, old/new result buckets during growth | Pre-admit result/map growth; measure actual glyph/style/prepared paths/runs/content capacities after successful measurement and adopt before dropping current scratch. |
| Return from measurement | Shaped cache + font-binding map drop; measured result map persists, original scene + finalized caller clone + faces remain | Release cache/map-only actual bytes. Keep measured map, keys, all shaped/style/prepared/content payloads charged in finalized graph resource facts. |
| Finalized scene shaping copy | Complete measured map persists while each finalized layer receives its own shaped clone | Admit exact measured shaped payload + copy allocation allowance BEFORE cloning. The clone then joins the scene walker; it must not also be charged as a separate resource payload. Measured original remains separately charged. |
| Geometry maps | Measurements and asset-size map buckets/keys coexist with measured map and finalized scene | Admit map bucket growth and each key before allocation; keep map actual capacities reserved until their drop. Their temporary reserve must coexist with all finalized clone payloads. |
| Materialization | Measured map moves into prepare_render_resources, eventually drops/consumes; finalized scene remains | Until consumption completes, measured map resource reserve remains live. Do not subtract it merely because measurement has returned. |

The active ledger must include the warnings Vec/binding-warning String clones retained by preflight, since they are produced during measurement and survive in RenderPreflight. Use actual capacities after pre-admitted growth.

For each HashMap allocation, derive bucket count conservatively from `next_power_of_two(2*next_len)` (small-map minimum4), charge `(bucketCount * size_of::<(K,V)>()) + bucketCount + alignment/control slack`, and include old+new bucket overlap before insert. Actual retained map accounting uses the corresponding upper bound from its actual usable capacity, not len; this may conservatively exceed the allocator's bucket bytes but must never omit them.

Legacy rich text can emit up to one PreparedTextRun per UTF8 character. Each run owns a resolved font PathBuf, generated item-ID filename, content and color. These lengths are NOT derived from D alone. Before cloning a resolved/default font path, admit the current actual path capacity and the worst emitted-run count times its path/item-ID/color payload, plus vector/header/growth overlap. Item IDs and configured font roots/default path descriptors likewise enter the current-call bounds explicitly.

Legacy font-family lookup currently recursively retains entire ancestor directory Vec listings through `ArtifactIo::list`. A finite document reserve cannot bound external filesystem directory size/depth. The active adapter must therefore select an admitted private lookup/listing port whose implementation admits directory entries and simultaneously retained ancestor descriptors BEFORE growth, or streams the traversal with a bounded/admitted stack. The default `list`/resolver body remains unchanged. A port allocation failure must latch and return an error at measurement finish, not silently fall back to guessed metrics. Its concrete private signature/source allocation proof is a remaining interface item before implementation.

## Concrete admitted font-family lookup port

Add private `ArtifactIo::admitted_font_lookup(root: &Path, normalized_family: &str, remaining_heap: u64) -> io::Result<Option<PathBuf>>`. The default implementation returns Unsupported; RequestScope forwards it. Only the active MatteMeasurementIo resolver uses it. Default `find_font_file` and default port behavior remain unchanged.

The admitted implementation owns its entire allocation contract: all simultaneously retained DFS traversal frames/cursors, ancestor paths, directory-entry working data, normalization/canonicalization strings and returned path must fit remaining_heap. Admission occurs before opening/growing a frame and BEFORE advancing the directory iterator (which can construct a DirEntry/file_name/path). It must pre-reserve supported-platform directory-entry/cursor temporary bounds plus actual ancestor/root path lengths; if the platform/port cannot certify that bound it fails closed before enumeration. It does not collect complete directory listings. An iterative depth-first traversal preserves the original directory iteration and first-match ordering. It retains only a checked frame stack and one current entry; it never turns a resource failure into a font fallback. No authored path/name/depth limit is added; the bound is the existing shared byte budget and supported platform allocation contract.

Returned path actual capacity is checked/adopted; per-run clones then reserve that actual path capacity, generated filename/item ID and color before cloning. The lookup's full peak reservation coexists with the font/glyph/cached/measured fixed base. Injected ports must implement this admitted contract or return Unsupported; post-return inspection alone cannot establish pre-allocation safety. Tests must demonstrate pre-iteration failure, nested-stack accounting, preserved first-match traversal, spare returned path rejection and no silent metric fallback. Runtime Fs proof/layout tests must establish its supported-platform temporary allowance before completion; no guessed path-length or directory-entry-count limit is permitted.


## Appendix: independent pinned opaque source census

# Pinned rustybuzz opaque shaping heap derivation

Read-only derivation, 2026-10-05. No Cargo/build, tracked edits, allocator measurements, or implementation approval performed. Sources are pinned rustybuzz 0.20.1 and its ttf-parser 0.25.1 dependency. The portable source links below identify their published, versioned source.

## Preconditions and accounting boundary

Actual core caller `crates/editor-core/src/fonts/shaping.rs:285–314` constructs a fresh Face per segment, sets horizontal LTR/RTL, language `und`, default font variation coordinates, and calls `rustybuzz::shape(&face, &[], buffer)`. This derivation depends on those concrete facts. Empty user features is especially important for AAT ranges. Face-owned eager lookups, planner, finished plan, glyph buffers and recursive auxiliary scratch can coexist; add their reservations rather than reuse one term by assumption. Core font bytes, segmentation/cluster maps, output Glyph storage/cache and outer measured/final layout overlap remain separate core-owner accounting. This note bounds requested Rust heap payload/capacities, not allocator implementation overhead or stack pages.

For a growable Vec with at most n emitted records and the pinned standard-library geometric growth, conservative payload reservation `G(n,z)=3*max(4,n)*z` bounds old+new allocation overlap and a final capacity plus stable-sort scratch (capacity <=2n, scratch <=n). Sum G separately across simultaneously retained vectors. Zero-record never-touched vectors need no allocation. Validate the pinned growth/sort assumptions and layout ceilings in owning tests; this is not a portable promise for an arbitrary replacement allocator/library. Checked arithmetic must reject overflow before opaque construction.

## 1. Face construction allocates before shape-plan construction

`rustybuzz/src/hb/ot_layout_common.rs:18–136`:
- `PositioningTable::new` collects ALL borrowed GPOS lookups into Vec<PositioningLookup>.
- Every PositioningLookup collects parsed PositioningSubtable enum values into its own Vec.
- GSUB SubstitutionTable/SubstLookup do the same for SubstitutionSubtable.
- Records are borrowing wrappers; subtable enum members do not recursively own font-table data.

Let Lg/Lp be all borrowed GSUB/GPOS lookup counts and sg_i/sp_i the declared borrowed subtable counts for each lookup. Count source records even if parse later drops invalid subtables. A safe eager bound is:

`G(Lg,128)+G(Lp,128)+sum_i G(sg_i,size_of::<ttf_parser::gsub::SubstitutionSubtable>())+sum_i G(sp_i,size_of::<ttf_parser::gpos::PositioningSubtable>())`.

128 is a source-layout ceiling for each private lookup header: Vec header (3 machine words), fixed set digest (three u64 words: set_digest.rs mask_t=u64 and nested three-pattern combiner), props, optional reverse bool, and alignment; this is <=56 bytes on64-bit and <=48 bytes on32-bit, so128 dominates without depending on field ordering; do not treat 128 as the subtable-enum size. Public enum `size_of` avoids guessed 32/64-bit layouts (`ttf-parser/src/tables/gsub.rs:246`, `gpos.rs:961`). Borrowed table iteration can perform this census before rustybuzz Face::from_slice; ttf-parser::Face parse is borrowing. Count repeated encoded records separately even if they point at shared offsets.

## 2. OT planner and finished map

`rustybuzz/src/hb/ot_shape.rs:74–187` pushes at most 26 common horizontal feature_infos, before/after one selected shaper. Additional selected-shaper pushes (including overrides): Arabic15, Indic20 (17-array +locl+ccmp+disable liga), Khmer13 (9-array+locl+ccmp+clig+disable liga), Myanmar10, USE22 (6 explicit +7 basic+4 topographical+5 other), Hangul4, default/Thai/Hebrew0. Thus MAX_FEATURE_PUSHES=48. Dedup can only reduce this; mapped features <=48.

GSUB pause maximum is Indic15 + common initial1 + compile final1 =17; GPOS compile final1. Arabic <=11 selected pauses, USE8, Khmer3, Myanmar7; none exceeds Indic. Source: respective collect_features/override_features, `ot_map.rs:314–345`. Reserve BOTH builder stages and compiled stages (18 combined records each).

Private source-layout ceilings on 32/64-bit targets from `ot_map.rs` struct declarations: feature_info <=48 bytes, feature_map <=64, lookup_map <=16, stage_info/StageMap <=16. Static OT allocations therefore have conservative additive term:

`G(48,48)+G(48,64)+2*(G(17,16)+G(1,16))`.

These ceilings need compile-time/owner tests if represented as MaxSourceLayout constants. They are actual field-layout proofs, not a byte constant selected from a budget. Finished plan shaper data Box is also live: Arabic 8*u32+bool, Khmer9*u32, Hangul4*u32, USE u32 plus Option<Arabic>, Indic largest: 17*u32 + 5*(Range<usize>+bool) + fixed IndicConfig/bool. A 256-byte source-layout ceiling covers that largest fixed Indic Box on 32/64-bit targets (5*24 +68 + <=32 + alignment); no nested Vec or Box exists in these data structs. Sources `ot_shaper_indic.rs:276,396,438`, Arabic274, Khmer79, USE160, Hangul47. If the implementation chooses512, document its excess rather than call512 the measured size.

### Dynamic lookup-index admission, including required features and variations

`ot_map.rs:485–608` pushes each feature lookup index into Vec BEFORE per-stage sorting and dedup. Distinct face lookup count is insufficient. Let Qg/Qp be the sum of lookup-index lengths of ALL feature records in each table, using the same default-coordinate chosen FeatureVariation substitute as the actual Face, when present. Required feature can admit the same feature list a second time: conservative temporary record bound is 2*Qg and 2*Qp. Reserve `G(2*Qg,16)+G(2*Qp,16)`.

Compute default-coordinate variation_index with borrowed `table.variations.find_index(ttf_face.variation_coordinates())`, then for EACH table.features index choose `find_substitute(index, variation_index)` or its original Feature, and sum `.lookup_indices.len()` checked. Public borrowed APIs: `ttf-parser/src/ggg/feature_variations.rs:28–48`; actual selection `rustybuzz/src/hb/ot_map.rs:493–505,588–608`. Do not use empty coords if the actual face holds default coordinate array populated for variable axes. No arrays need be collected for the census. The required-feature double bound is intentionally conservative; counting only base Feature lengths ignores potentially longer substitutes.

## 3. AAT nested chain_flags

`aat_layout.rs:497–511` populates builder ONLY from plan.user_features; actual user features is empty. In `aat_map.rs:170–251`, feature_events therefore receives exactly ONE sentinel; active/current feature Vecs remain empty and no features are cloned. Compile flags executes exactly ONE range. `aat_layout_morx_table.rs:28–52` outer chain_flags resizes to actual borrowed morx chain count C, and each chain appends ONE `{flags:u32,cluster_first:u32,cluster_last:u32}` record (12 bytes). Later resize at76 uses same C.

Add `G(C,3*sizeof(usize))+C*G(1,12)+G(1,16)` for nested outer Vec headers, per-chain one-range minimum capacities, and the sentinel feature event (<=16-byte layout). This term depends on borrowed morx chain records, not arbitrary font bytes or number of font feature entries. C=0 with no morx may avoid this construction; charging sentinel unconditionally is safe. This proof is invalid if nonempty user features become supported.

## 4. Nested matching SmallVecs during lookup application

`ot_layout.rs:14–15` sets MAX_NESTING_LEVEL=64, MAX_CONTEXT_LENGTH=64. `ot_layout_gsubgpos.rs:57–65` rejects count>64 BEFORE match_positions.resize. All six context paths and `ot/layout/GSUB/ligature.rs:32` own one SmallVec<[usize;4]> per matching application; helpers pass it by reference. `recurse` at1072–1112 permits 64 nested calls beneath the outer lookup, so conservative live count is 65, not merely64. Add `65*3*64*sizeof(usize)` (=99840 on64-bit), assuming pinned SmallVec geometric growth <=64 for requests<=64. No need arbitrary256KiB if this explicitly derived term is used. Inner recursive call may retain outer context positions until return. Verify no separate simultaneous second owner is introduced in future versions.

## 5. Reachable COLR glyph-extents paint stacks

Important additional path: `face.rs:274–308` constructs `hb_paint_extents_context_t` when COLRv1 glyph extents lack a clip box; glyph-extents is reachable during shaping/positioning. `paint_extents.rs:128–146` owns FOUR Vecs: clips, groups, transforms, composite_modes. Do not omit them because they are outside ShapePlan.

`ttf-parser/src/tables/colr.rs:1830–1870` uses fixed recursion_stack[64], admission BEFORE parsing a node. At most three transform pushes per centered transform paint format; maximum transform count1+3*64=193. PaintComposite pushes two groups/modes before popping (1769–1795). ClipBox on nested PaintColrGlyph and PaintGlyph clips may add one per relevant level; conservative per-stack count1+2*64=129 covers clips/groups/modes. All formats read fallible parameters before push; child parse errors are ignored and balanced pops still execute. A top-level clip adds at most one additional clip: use130 clips conservatively. Add:

`G(130,24)+G(129,24)+G(193,sizeof::<ttf_parser::Transform>())+G(129,4)`.

hb_bounds_t = status enum plus four f32 => <=24; Transform public size is24; CompositeMode source enum has no payload and <=4-byte layout. This is <34KiB and derived from library recursion, not a new font-depth policy. RecursionStack itself is stack memory, not this heap term. Repeated sibling paints reuse stacks; they do not accumulate depth.

## 6. Other allocation sites / qualifications

Complete rustybuzz src text census found no HashMap/BTreeMap owner. Its remaining runtime owners are glyph buffer info/pos (main separately proved120*max(64*N,16384)), plan.user_features (empty), Language String (`und`, three UTF8 bytes), script/language tag SmallVecs (inline capacities3; default und and script tags <=3), and shape_wasm outline Vecs. wasm-shaper is OPTIONAL, not the current dependency default; explicitly pin feature configuration or include that separate call path if enabled. rustybuzz face paint reads borrowed color/image data, not PNG decoding. Include actual Language allocation/cloning using own capacity ledger or conservatively source-derived fixed multiple of3, not a general arbitrary-language bound.

## Required evidence before claiming final guard proof

No runtime allocator/layout measurements were performed in this research task. Owning implementation must: use borrowing census BEFORE eager rustybuzz Face construction; checked-add every term to existing opaque buffers/retained glyph/cache layouts; assert public info/pos/enum/Transform sizes and private source-layout ceilings by independent source-pinned tests; include high lookup/subtable counts, feature substitute longer than base, required-feature duplicate admission, AAT many-chain one-range, deep context and COLR stack evidence; freeze/pin exact dependency feature configuration. Growth/sort/SmallVec assumptions need explicit pinned-version evidence or a deliberately larger source-derived reservation. This note is not global conformance approval and does not authorize font policy limits.

## Source links / exact audit status

- [Actual core caller](../../../crates/editor-core/src/fonts/shaping.rs)
- [Eager Face lookup storage](https://docs.rs/crate/rustybuzz/0.20.1/source/src/hb/ot_layout_common.rs)
- [OT map layout/admission](https://docs.rs/crate/rustybuzz/0.20.1/source/src/hb/ot_map.rs)
- [Shared feature collection](https://docs.rs/crate/rustybuzz/0.20.1/source/src/hb/ot_shape.rs)
- [AAT empty-feature range construction](https://docs.rs/crate/rustybuzz/0.20.1/source/src/hb/aat_map.rs)
- [Nested context admission](https://docs.rs/crate/rustybuzz/0.20.1/source/src/hb/ot_layout_gsubgpos.rs)
- [Paint heap owner](https://docs.rs/crate/rustybuzz/0.20.1/source/src/hb/paint_extents.rs)
- [Pinned COLR64 recursion cap](https://docs.rs/crate/ttf-parser/0.25.1/source/src/tables/colr.rs)

The bound deliberately uses target `size_of` for public GSUB/GPOS subtable enums: exact compiler sizes were NOT measured because builds are prohibited in this research task. The private128/48/64/16/256 ceilings are justified field-derived upper bounds, not claimed ABI sizes. Static source census and cardinality proof are complete for the actual no-wasm/horizontal/default-coordinate/und/empty-user-feature call. Remaining qualification is pinned library growth behavior plus independent actual peak/layout witnesses and integration into the main owner’s existing glyph/font/cache/measurement ledger. Root’s separate nonopaque audit `root-repository-shaping-audit.md` remains required; this note does not cover directory traversal, bidi, style expansion, or core glyph maps.


## Appendix: independent repository allocation census

# Root read-only repository-side allocation audit

Scope: current no-code proposal, not numerical approval. Sources remain current working tree and pinned unicode-bidi0.3.18.

`model/styled_text.rs:100 effective_runs`: each original run creates two endpoint cuts plus at most2 per span, but each global span boundary belongs in at most one original run interior. Thus total returned nonempty pieces ≤ original run count +2*span count ≤768. Clone of an entire original run precedes replacement with substring; count that temporary original text allocation as well as returned pieces. Span typography colors validated hex; source style payload capacities still count exactly. `paint_ranges:136` clones at most256 paint vectors, each ≤16 layers. `checked_span_boundaries`/`grapheme_boundaries` temporarily concatenate full text and collect ≤N+1 usize positions; both calls can overlap only with already-returned effective runs, not with each other's temporary boundary vectors.

`fonts/shaping.rs` styles ≤effective piece count. Segment descriptors ≤character count≤N; face hashes64B each. Cluster BTreeMap has at mostN unique input byte offsets, independently of expanded glyph count; descriptors/btree nodes depend on N, owned glyph payload on admitted cumulative G. `clusters.into_values().collect` coexists with moving map nodes; count descriptor vector and nodes simultaneously. Hard line ranges≤N+1. Linebreak opportunity BTreeSet≤N+1, per paragraph. Result glyph-line integers depend on G, not N! These usize vector capacities/reallocations must explicitly be in8GH or a separate G term. Line widths depend on L. Bidi adjusted levels clones ENTIRE text levels per line (≤N), even when line is short; levels and reorder indices≤N per line, dropped each line.

Pinned `unicode-bidi0.3.18/lib.rs:495`: base original classes≤N, paragraphs≤N, flags≤N, processing class clone≤N, levels≤N. `prepare.rs isolating_run_sequences` moves each level-run Range exactly once into final nested arrays; logical sum of ranges≤N. Outer sequence arrays and stack nested Vec headers≤N and ranges≤N with minimum Vec capacity4 for many one-range sequences (64B per singleton on64bit), and old/new growth overlap. Explicit stack capped embedding level125 but ≤N source inputs. Implicit weak indices, neutral indices, bracket pairs and bracket stack all bounded by source positions≤N; only one sequence at a time. No N-squared owning clone found in these paths. `1024*(N+L+1)` may be defendable only with an enumerated conservative per-input-byte descriptor/growth table; an umbrella declaration alone is not proof.

`evaluated_scene/text_layout.rs:resolve`: a second entire document clone coexists with artifact's document/key during advanced shape fitting. Each failed attempt drops shaped payload before next; cumulative early G count must reset each attempt. Work quota remains after each candidate, unchanged.

`render_artifact.rs:measure_rich_text`: up to N prepared run segments, EACH owns resolved font PathBuf, formatted itemID-dependent file path, optional run color and content. Default/resolved path length does not derive from text heap D and can exceed1024 bytes. Must pre-admit based on actual path/ID/color lengths before per-run clones or add dynamic ledger seam. Prepared text map header/bucket growth and warnings clone lifetime also required.

`find_font_file:1231`: recursively retains each ancestor's directory Vec while descending. Listing is arbitrary filesystem size and unbounded by authored text; cannot cover using16D. Active-only admitted streaming/list port is necessary, including path normalization/canonicalization transient and directory stack. Default ordering/first-match behavior should remain equivalent; default path unchanged.

`text::measure → evaluated_scene/text_bounds::measure`: per-glyph borrowed ttf_parser face and glyph bbox, no newly owned glyph clone; style cloned once at return. Existing glyph rendering allocations happen later and keep existing source/canvas budgets.


## Exact private implementation ownership

Main owns additional private active-entry edits in fonts/shaping.rs and evaluated_scene/text_layout.rs, with no reverse dependency. Main may add render_artifact/text_measurement_memory.rs and its tests as artifact-owned policy/code using existing inward evaluated-scene/fonts edges. Existing evaluated_scene/mattes.rs supplies the single shared shaped/style heap walkers. Renderer/artifact/request-scope are already owned. Main may expose a cfg(test)-only sibling observation mode from existing render_artifact/shapes/fill_allocation_tests.rs; it preserves fill's128-request layout assertions and does not add a second global allocator, runtime unsafe code or font-to-artifact dependency. Actual shaping memory witnesses reside inside artifact-owned tests. Public/catalog ownership inventory updates remain with the public owner after exact approval.
