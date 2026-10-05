## MODIFIED Requirements

### Requirement: Hybrid renderer boundary
The render architecture SHALL retain FFmpeg for media decode, audio processing, and encoding, with editor-core owning canonical floating-point linear scene composition, SHALL place deterministic complex-vector and shaped-text rasterization behind a replaceable graphics interface, and MUST keep backend-specific expressions and types out of persisted and public contracts. Backend selection MUST follow a deterministic local priority shared by frame preview, audiovisual range preview, draft preview, and final export, and MUST limit failover to a locally available substitute that supports the complete evaluated scene and preserves the same `EvaluatedScene` semantics and documented output tolerance. A backend MUST NOT omit, approximate, downgrade, reorder, or remotely acquire resources for unsupported instructions. When no conforming backend is ready for the complete scene, readiness or rendering MUST fail with `DEPENDENCY_UNAVAILABLE` before graphics rasterization, FFmpeg execution, or artifact publication, and MUST NOT publish a partial or degraded artifact.

#### Scenario: Replace a graphics backend
- **WHEN** a conforming graphics implementation replaces the initial deterministic Rust backend
- **THEN** project files, public operations, evaluated-scene semantics, ordering, and preview/export tolerance contracts remain unchanged

#### Scenario: Reject unsafe renderer input
- **WHEN** input attempts to supply a raw FFmpeg expression, executable SVG content, arbitrary path, network resource, non-finite value, or content exceeding an explicit complexity limit
- **THEN** the canonical owning layer rejects it before it reaches a renderer backend

#### Scenario: Fail over to a conforming local backend
- **WHEN** the preferred graphics backend is unavailable and the next locally configured backend supports every instruction in the complete evaluated scene
- **THEN** the renderer selects that backend by deterministic priority for preview and export while preserving the same scene semantics and documented output tolerance

#### Scenario: Reject degraded fallback
- **WHEN** no locally available backend can execute every instruction in the complete evaluated scene without omission, approximation, downgrade, reordering, or remote resource acquisition
- **THEN** readiness or rendering fails with `DEPENDENCY_UNAVAILABLE` before graphics rasterization, FFmpeg execution, or artifact publication and no partial or degraded artifact is published

### Requirement: Normative coordinate and compositing semantics
Motion-graphics evaluation MUST use a top-left coordinate origin with positive X rightward and positive Y downward, integer-millisecond half-open time intervals, explicit coordinate units, deterministic bottom-to-top layer ordering, the documented transform/mask/effect pipeline, premultiplied alpha, and linear-light compositing before output-color conversion.

#### Scenario: Resolve equal z-index layers
- **WHEN** multiple visual items in one track have the same explicit z-index
- **THEN** evaluation orders them by stable item array order and uses stable item ID only as a final deterministic tie-break for synthesized or otherwise equivalent order inputs

#### Scenario: Evaluate an inherited visual
- **WHEN** a visual has local crop, clip, masks, effects, transform, matte, opacity, blend mode, and ancestor transforms
- **THEN** evaluation applies source rasterization, crop and clip, declared masks, declared effects, local anchor/scale/skew/rotation/position, nearest-to-outer ancestor transforms, matte, inherited opacity, and destination blend in that order

#### Scenario: Distinguish current and future pipeline stages
- **WHEN** current evaluated visuals do not represent authored masks, track mattes or non-normal blend modes
- **THEN** mask/matte stages are explicit identity stages and destination blending is normal linear premultiplied source-over; future milestones must add typed semantics before activating those features
