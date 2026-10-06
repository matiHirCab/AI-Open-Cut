## MODIFIED Requirements

### Requirement: Normative coordinate and compositing semantics
Motion-graphics evaluation MUST use a top-left coordinate origin with positive X rightward and positive Y downward, integer-millisecond half-open time intervals, explicit coordinate units, deterministic bottom-to-top layer ordering, documented transform/mask/effect/matte pipeline, premultiplied alpha and linear-light compositing before output-color conversion. Schema33/34 masks MUST retain mask-models ownership/coordinate rules and mask-rendering raster/sample semantics before effects. Model/animation capability support SHALL NOT substitute for complete mask/matte-rendering readiness. Authored matte references MUST resolve in composition-scoped finite DAGs and bind providers to the same unique composition occurrence/exact temporal requests. Provider coverage MUST sample isolated transparent source planes, independent of destination/background and direct-draw suppression by matteOnly.

#### Scenario: Resolve equal z-index layers
- **WHEN** multiple visual items in one track have the same explicit z-index
- **THEN** evaluation orders them by stable item array order and uses stable item ID only as a final deterministic tie-break for synthesized or otherwise equivalent inputs

#### Scenario: Evaluate an inherited visual
- **WHEN** a visual has crop, clip, active masks, effects, transform, matte, opacity, blend and ancestor transforms
- **THEN** evaluation applies source rasterization, crop/clip, masks, effects, local anchor/scale/skew/rotation/position, nearest-to-outer ancestors, matte, inherited opacity and destination blend in that order

#### Scenario: Distinguish current and future pipeline stages
- **WHEN** current visuals carry absent or authored schema33/34 static/animated masks and schema34 typed track mattes, while evaluated rendering does not activate non-normal blend modes
- **THEN** authored masks and typed mattes execute their approved semantics and absent masks/mattes remain explicit identity stages; destination blending is normal linear premultiplied source-over, and the later blend milestone retains its typed activation requirements

#### Scenario: Keep scope occurrence and paint order independent
- **WHEN** matte providers appear above/below recipients or local IDs repeat in two differently transformed component instances
- **THEN** DAG dependency execution is scoped to each occurrence, coverage is evaluated before recipient drawing and final destination paint order remains canonical

## ADDED Requirements

### Requirement: Single inward owner for matte dependencies
Core model/validation MUST own typed reference semantics and scoped final-candidate DAG validation; evaluated_scene MUST own immutable occurrence/request/clock binding and complete work/live-memory/resource certificates; render_artifact MUST consume those facts for isolated provider planes and pure coverage multiplication. Store/migration/draft owners MUST retain transaction/source-envelope semantics and existing architecture seams. No transport/backend/process owner SHALL construct its own graph, decode persisted records as rendering policy or reset shared certification budgets. New inward sibling modules/consumers MUST be registered in canonical ownership/CODEOWNERS and ADR/architecture checks without a new top-level owner or outward edge.

#### Scenario: Enforce owner and handoff boundaries
- **WHEN** model edits, batch/draft migration, resource preparation, render planning and all four intent routes exercise new matte dependencies
- **THEN** architecture/module/consumer checks establish the declared inward owners and every renderer consumes one canonical evaluated dependency meaning
