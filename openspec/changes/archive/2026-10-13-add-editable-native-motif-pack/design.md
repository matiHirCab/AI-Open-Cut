## Context

Actual main 2a733383 already supplies native shapes/Bezier paths, dot grids, components and schema45 solid-color slots. The hero recipe is immutable example data in a single read-only motion prompt message; existing clients parse its final JSON suffix. Actual typed component track input excludes repeater items. Speed lines therefore use one ordinary shape path with six literal closed rectangle contours; the core-only repeater variant is not exposed or repaired by this issue. No foundational defect is being repaired here.

## Goals / Non-Goals

Provide five attributable, reusable native motifs with stable roles, literal bounded geometry, palette and ordinary placement, plus independent transactional/native proof alone and around the hero. Preserve existing public/persisted contracts, old prompt parsing, validation ownership and every prior gate.

No new scene/slot/property/alias language, runtime recipe executor, random generator, raster/AI artwork asset, external tracing, pattern engine, team layout, desktop UI, renderer optimization, dependency, migration, credential change, CI weakening or resource/deadline change. No installed desktop/video access, merge, auto-merge or deployment.

## Decisions

1. **Versioned ordinary data, not a factory or scene model.** One `native-motif-pack-v1` canonical fixture owns metadata, ordinary create/populate/place operations and independent expected facts. Each component is 320x240,1000ms. Gallery demonstrates at most two occurrences per motif; around-hero uses one per motif beside the existing two hero cards. Geometry is literal original native code: analytic primitives and explicitly authored Bezier coordinates, with no external source image, generated artwork asset, silhouette extraction or tracing. Alternatives of opaque images/SVG plates or a new parametric engine would lose ordinary editability or expand scope.

   | Motif | Native roles and exact reference construction | Palette bindings |
   | --- | --- | --- |
   | dot_backdrop | `dots`:280x200 dot viewport at(20,20),40px spacing,4px radius;48 inclusive centres, clipped at viewport boundaries; alpha0.5 | existing `grid.dot.paint.color` |
   | sticker_contour | `paper`:original closed curved path within[20,300]x[20,220],12px solid outline; `accent`:64x64 ellipse at(128,88) | paper fill, paper stroke, accent fill |
   | sun_rays | `rays`:eight disjoint closed path contours at45-degree intervals about(160,120),54–96px radial extent; `sun_disc`:64x64 ellipse at(128,88) | ray fill, disc fill |
   | fireball | `flame_outer` and `flame_core`:two original closed paths, outer within[20,300]x[20,220], core within[90,250]x[80,195], ordered outer then core | outer/core fill |
   | speed_lines | `speed_lines_shape`:one path with six disjoint closed 180x6 rectangle contours at(20+12i,50+24i),i0..5,alpha0.75; six visible lines, one visual leaf | one shape fill shared by all six contours |

   Counts are fixed, without seed or offered count variation. Reference motifs expand to 8 visual leaves per one-of-each set,16 for paired gallery; adding one set to the unchanged14-leaf hero example yields22. These are reference-example caps, not new core limits or promises about arbitrary edits. Every native vector/control point remains stored normally and editable through existing operations.

2. **Use current slots and placement.** Nine existing solid-color bindings expose the motif paints; exact hex6 strings retain casing and paint alpha. Defaults/overrides and omission versus explicit replacement retain existing semantics. Reference placement uses ordinary Transform2D and fixed canvas 1920x1080 at30fps. Supported scalar bounds remain those of core: pixel positions±1,000,000, normalized positions±100, anchors[0,1], positive scales≤100, rotation±36,000 degrees, skew±80 and opacity[0,1]. Existing composed geometry/resource admission still applies; a legal scalar does not promise every combination renders. Tests cover palette black/white boundaries, edge clipping, independent occurrences and invalid core-native parameters; no bridge-side validation duplicates these rules.

3. **Explicit two-phase transactional construction.** Phase A atomically creates five empty, unused definitions with no slots; their returned IDs resolve exact literal placeholders. Phase B uses those real component IDs, replaces definition tracks/slots and adds the selected gallery or around-hero occurrences in one current-revision batch. Phase B failure rolls back all its updates/placements to the exact pre-B bytes; the documented committed empty setup remains. Undo/redo/reopen exercise both phases separately. This avoids inventing self/forward alias semantics. Scope strings are data, never evaluated or substituted inside unrelated text.

   Actual reference around-hero placement uses scale1: dot(40,-20),sun(900,0),sticker(850,840),fireball(1600,840),speed(180,840). Top dot viewport is y0–200, leaving20px before y220; rays end at y215. Bottom sticker stroke spans approximately y854–1066; other bottom paint remains below y840 and within1080. Hero paint spans y220..<820, so the native PNG must prove both prior card regions byte-identical. Geometry/counts/bindings do not change.

   Independent PNG verification retains strict literal point RGB checks within3. H264 yuv420p range verification retains all prior hero,paint-order,large-fill,gap and exterior point checks within8. Only this new example's thin-feature range points are replaced by fixed fully-covered geometric-interior ROI channel means: radius4 dots use the four central pixel centres (x-1..x,y-1..y); six-pixel strips use four interior rows (y-2..y+1) across four interior columns (x-2..x+1). Unrounded RGB means use the same independently calculated linear-light/8bit-alpha RGB and unchanged8 bounds. ROIs derive from declared geometry and pixel-centre coverage, never artifact fitting, production pixels or broader tolerances. Preserve actual failed media/diagnostics and prove blank,wrong-paint,wrong-order and corrupted-ROI inputs still reject. Prior point failure evidence remains a failure; fresh actual source/default-package acceptance is mandatory.

   The unpublished sticker reference uses a12px centered round outline, preserving original path/paint/alpha/slot/counts. Conservative local paint bounds[14,306]x[14,226] fit320x240. Keep literal width12 assertions and strict PNG/range points; no stroke ROI/tolerance/codec/core change. Retain all failed6px evidence and require fresh actualnative/source/package gates.

   Keep exact prior card PNG equality after the gap; failures retain the actual artifact and first differing pixel coordinates. Previous boundary differences do not prove a chroma/renderer defect, and source grid clipping/counts stay unchanged.

4. **Keep the current prompt contract and hero reader.** Insert the pack's bounded JSON block and explicit end marker before the existing hero guidance/terminal JSON suffix in the same single message. Keep original workflow steps, prompt name/arguments, other prompts and hero source bytes. New readers isolate only the marked motif block; old hero readers still parse the exact trailing hero JSON. A second message or conditional replacement would change an existing observable shape or hide prior data. Retrieval still calls no editor, provider, importer or renderer.

5. **Govern the addition narrowly.** Append only owned `nativeMotifPack` fixture/consumer registry and CODEOWNER rows. Core, typed headless, bridge/Zod, prompt, source/package and native tests consume the same artifact. Add their focused cases to the unchanged complete canonical gate, retaining every old command/assertion. Synchronize only the independently authored exact `CONTRACTS_COMMAND` literal with that approved inventory, preserving the strict equality guard and all previous policy tests. List the package command and both policy scripts as additional governed consumers solely in the new nativeMotifPack category and exact addition descriptor. Add explicit omission/`|| true` rejection tests for the new core and TypeScript motif consumers; exact disposable unions also pin their additive union-only negative-test command. This inventory synchronization does not alter workflows, selectors, tolerances, deadlines, attestation or the default120-minute audit. Own raw MCP/headless catalogs remain byte-identical to main; exact current-addition removal restores the captured ownership predecessor before historical projections. Preserve malformed/missing/partial/unrelated-drift negatives. Disposable exact-head combinations with pending1987f and199c2fa, including their joint overlap, preserve both diagnostic/progress fields and all prior pins; they do not become publishing dependencies.

6. **Attributable evidence.** Core/typed transport tests establish actual existing batch validation, sample geometry/count/order, slots, immutability and history. Source and fresh default packages prove discovery and lifecycle through actual MCP. Existing required default source/package native cases additionally execute the retrieved motifs with actual tools/font, independent literal pixel expectations, blank/wrong-size/order controls, frame500ms and static range[500,600)ms with three frames. Transparent exterior/holes and fractional paints mean the known background shows through; no whole exported-frame alpha promise is added. Each motif appears independently and around unchanged hero records/output. Preserve all older hero/palette/native assertions, references, RGB bounds, resource guards, test deadlines, report schemas and default/private-binary distinctions. Record fresh motif media/provenance in the owned evidence directory; no fake provider or mere file existence counts as native success.

## Risks / Trade-offs

- Phase A can remain after rejected phase B: document it explicitly and prove pre-B rollback; do not claim whole two-phase atomicity.
- Fixed geometry and counts constrain the example: ordinary edits remain possible under existing core rules, without new arbitrary-pattern promises.
- Marker parsing must preserve the old hero terminal suffix and escaped creative request data; test deterministic discovery, original bytes and both readers.
- Non-integer curve/edge coverage needs independent interior/exterior samples and existing codec tolerances, not production-derived expectations or rewritten goldens.
- Extra required native checks consume time: keep scenes/ranges bounded and preserve all existing gates/deadlines/budgets.
- Pending contract owners and main movement require exact semantic unions and independent resolution before normal publication; no force/squash or unmerged-domain import.

## Migration and rollback

No public/project/draft/protocol version or migration changes. Projects persist ordinary definitions/instances, not a recipe interpreter. Failed edits retain canonical codes/retryability and project/history/resource bytes. Rollback removes only new example data/guidance/ownership/evidence; existing native foundations and hero remain.

## Open Questions

None blocking the issue-scoped implementation. Main's previous Windows timeout cause remains separately unconfirmed; final new-head correctness is required without implying a root-cause repair.
