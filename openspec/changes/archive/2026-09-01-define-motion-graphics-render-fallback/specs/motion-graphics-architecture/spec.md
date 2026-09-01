## MODIFIED Requirements

### Requirement: Hybrid renderer boundary
The render architecture SHALL retain FFmpeg for media decode, audio processing, final composition, and encoding, SHALL place deterministic complex-vector and shaped-text rasterization behind a replaceable graphics interface, and MUST keep backend-specific expressions and types out of persisted and public contracts. Backend selection MUST follow a deterministic local priority shared by frame preview, audiovisual range preview, draft preview, and final export, and MUST limit failover to a locally available substitute that supports the complete evaluated scene and preserves the same `EvaluatedScene` semantics and documented output tolerance. A backend MUST NOT omit, approximate, downgrade, reorder, or remotely acquire resources for unsupported instructions. When no conforming backend is ready for the complete scene, readiness or rendering MUST fail with `DEPENDENCY_UNAVAILABLE` before graphics rasterization, FFmpeg execution, or artifact publication, and MUST NOT publish a partial or degraded artifact.

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
