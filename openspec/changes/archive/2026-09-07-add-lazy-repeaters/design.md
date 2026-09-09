## Context

Issue #31 follows the implemented component evaluation and shape-rendering milestones. Core currently persists schema 16 and evaluates root/component hierarchies into a flat renderer-neutral scene with a 65,536 occurrence preflight and downstream layer/resource/work budgets. Shapes, groups, and component instances already have strict identity, timing, hierarchy, transform, transaction, migration, and render semantics. The missing capability is a compact controller that adds repeated visual occurrences without creating hundreds of persisted timeline records.

This change crosses editor-core, persistence, headless, agent-bridge, contract catalogs, desktop exhaustive consumers, and render verification. ADR 0003 remains unchanged: validation and expansion stay in editor-core and outer layers only translate typed data or execute evaluated instructions.

## Goals / Non-Goals

**Goals:** Persist one strict repeater descriptor; reference shapes, groups, or component instances within one composition scope; create 1-256 additional visual copies; apply deterministic per-copy affine and opacity offsets; preflight nested work; support standalone/batch aliases and reversible persistence; render identical evaluated semantics across all intents.

**Non-Goals:** Repeater time offsets or stagger, inherited animation changes, audio repetition, media/text/SVG/grid/caption/transition sources, arbitrary expressions, persisted expansion, cross-scope references, desktop authoring controls, backend-specific types, remote resources, or schema downgrade. Time offsets remain MG-M3-05 (#42).

## Decisions

### Persist a reference descriptor rather than expanded items

Add `TimelineItem::Repeater(RepeaterItem)` with common identity/start/duration/hidden/z-index/stack-order fields and required `repeater: RepeaterDescriptor`. The descriptor contains closed `source {scope,id}`, integer `copies`, closed `transformOffset`, and `opacityOffset`. `copies` counts additional occurrences; the source continues its ordinary evaluation. This prevents surprising source suppression and lets multiple repeaters reference one source independently.

Persisted expansion was rejected because it would inflate project/history files, expose generated IDs to editing, complicate undo and aliases, and lose lazy evaluation. Treating repeater as a ShapeGeometry variant was rejected because groups and components expand to occurrence subtrees rather than one vector geometry.

Repeaters occupy overlay tracks and use timing, hidden, z-index, and stack order. Their common transform remains identity and they cannot be parented or animated in this milestone; placement is expressed solely by the per-copy offset. This avoids two competing transform frames while preserving timeline ordering. Complete descriptor replacement through `update_item` mirrors grid editing and keeps nested patch semantics strict.

### Use a dedicated affine offset with an explicit origin

`transformOffset` reuses Transform2D position units and numeric bounds but has only position, scaleX/Y, rotationDeg, and skewX/Y. It has no anchor or opacity. Position is normalized against the source scope canvas. The affine is built around the source-parent coordinate origin, and copy `i` receives `O^i` before its source local transform. Copy opacity is `sourceInheritedOpacity * clamp(1 + i * opacityOffset, 0, 1)`.

Reusing complete Transform2D was rejected because anchor semantics for group/component subtrees are ambiguous and its opacity would duplicate the required opacity offset. Additive scale deltas were rejected because zero/negative crossings create singular transforms; positive multiplicative scale factors compose cleanly and existing inverse/surface preflight remains applicable.

### Keep source timing and define the repeater as an ordering block

The source continues to render at its ordinary canonical position. Generated copies share the source's local clock and are active only in the intersection of the source occurrence, ancestors, and repeater half-open interval. There is no time offset. At evaluation, the repeater's canonical track/z-index/stack-order position becomes a generated block: copy indices ascend, and each copied group/component subtree preserves its existing internal order. Stable occurrence IDs derive from scope, repeater ID, copy index, and source occurrence path.

Inserting copies adjacent to the source was rejected because it makes repeater track ordering meaningless and prevents users from placing the generated block independently. Allowing time offsets now was rejected because it belongs to animation milestone #42 and would require new clock-overlap rules.

### Validate one combined occurrence dependency graph

Core validates source existence/kind/scope for root, local definitions, hidden content, history, and drafts. It rejects a repeater whose selected group/component expansion closure reaches the repeater, plus any cycle or depth violation formed by parent, component, and repeater edges. Source deletion and ungroup fail until references are removed or retargeted. Group/component sources must have visual-only transitive closures in this milestone so audio does not get duplicated accidentally.

Only allowing preceding siblings was considered as a simple acyclicity rule, but rejected because stable IDs already support cross-track forward references and order should not control graph validity. Silently skipping recursive branches was rejected because preview/export would become data-dependent and non-canonical.

### Preflight lazy multiplication before allocation

Per-repeater copies are limited to 256 inclusive. Core walks expansion closures with checked arithmetic, validates derived affine powers/opacities/intervals, and accounts for ordinary plus generated occurrences before allocating output collections or render surfaces. The existing 65,536 occurrence cap and all layer, media, audio, vector, surface, and memory caps remain authoritative. Hidden/unused descriptors and definitions validate even when they emit no pixels. Repeaters remain persisted as one record in projects, history, drafts, and mutation responses.

A larger copy cap with only downstream layer checks was rejected because group/component closures can multiply traversal work before leaf-layer counts are known. Streaming without preflight was rejected because it could partially allocate or execute before discovering overflow.

### Synchronize additive contracts and readiness

Add canonical `repeaters-v1.json`, register governed consumers, update operation/MCP/capability catalogs, and mirror the exact strict representation in Rust and TypeScript. Public identifiers are `add_repeater`, `timeline_add_repeater`, `repeater_items`, and `repeater_rendering`. Editing capability is independent from renderer readiness; rendering is advertised only when the configured local backend accepts the complete evaluated scene. Protocol 1 envelopes/errors retain meaning, but documentation warns that the schema-17 item variant requires an updated decoder.

A protocol major bump was rejected because the new operation and capability identifiers are additive for clients that do not decode repeater-bearing projects. Claiming transparent compatibility for old project decoders was rejected because closed item unions can fail on the new variant.

### Preserve ownership at the evaluated-scene seam

Model/validation/timeline/migration/evaluation changes stay in editor-core. Evaluation emits ordinary flat visual facts with generated occurrence IDs; no persisted RepeaterItem crosses into render planning. Headless deserializes/serializes and translates core errors. Agent-bridge mirrors schemas and registers workflows without duplicating semantic validation. Renderers therefore need no repeater algorithm and all preview/export entry points inherit one implementation.

A renderer-side loop was rejected because each backend could diverge on order, transforms, opacity, hierarchy, and limits. A new dependency or ADR edge is not expected; any discovered edge returns to design approval before implementation.

## Risks / Trade-offs

- Nested group/component repeaters can multiply work rapidly -> validate closures and checked counts before allocation, retain all existing downstream budgets, and cover exact boundaries.
- Matrix powers can overflow or become non-invertible -> bounded positive scale factors plus per-derived-matrix finite/inverse checks fail closed.
- Cross-track sources make deletion and ordering less obvious -> explicit reference protection, deterministic generated blocks, inspection fixtures, and client documentation.
- New item variants touch exhaustive matches -> audit core, assets, drafts, definitions, desktop, headless, bridge, catalogs, and test compile-time/fixture parity.
- Schema 17 cannot be opened by older binaries -> forward-only source gating, atomic current/history migration, future-version rejection, and documented rollback from preserved pre-upgrade data.
- Restricting group/component closures to visual-only content limits initial use -> reject explicitly now; audio/time repetition can be specified later without silently changing current projects.

## Migration Plan

After explicit approval, add schema 17. Validate every source snapshot under its declared version before relabeling; reject any repeater record below 17, including hidden/unused definitions and retained generations. Existing schema 1-16 migrations run unchanged; 16-to-17 only advances schemaVersion. Current state and all undo/redo snapshots publish atomically under the existing lock/recovery protocol. Preserve revisions, IDs, ordering, assets, provenance, drafts, and render output. Unknown future versions keep existing fail-closed behavior.

Operational rollback uses a preserved pre-upgrade project/history generation with the earlier binary; no automatic downgrade or removal of repeaters is attempted.

## Verification Plan

Map every scenario to automated fixture-driven tests. Cover strict duplicate-preserving decoding, all numeric/reference/source-kind failures, root/local graph cycles, exact copy/occurrence/layer limits, affine/origin/opacity oracles, intervals and ordering, visual-only closure checks, standalone/batch aliases and rollback, supported lifecycle, undo/redo/reopen/drafts, mixed-history migration and fault recovery, and frame/range/draft/export semantic/pixel parity. Use independent expected matrices/identities/pixels rather than only comparing shared code paths.

Run direct strict OpenSpec validation during the active change. After implementation run Rust format, workspace strict Clippy and tests; bridge typecheck, lint, unit, contract, integration and packaged smoke suites; hermetic Python worker tests; and renderer golden/parity gates. Record exact evidence and treat unavailable required checks as blocking. Use openspec-verify-change, obtain the designated contract review, archive the verified change, then run the archive-only `moon run root:openspec-validate` policy gate and `git diff --check`.

## Open Questions

The proposal selects additional-copy counting, a 256 per-repeater limit, source-parent affine composition, additive clamped opacity, generated-block ordering, visual-only group/component closures, and no time offset. These choices require explicit user/reviewer approval with the other artifacts before implementation. Any requested alternative must update the specs, design, and tasks before code changes.
