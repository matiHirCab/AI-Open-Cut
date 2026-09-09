## MODIFIED Requirements

### Requirement: Shared complete repeater rendering
Frame preview, audiovisual range preview, materialized-draft preview, and final export MUST consume the same editor-core-expanded repeater occurrences from `EvaluatedScene`, including occurrence identity, interval, transform, opacity, hierarchy, component clock/bindings, and numeric source-subtree order. Renderer and artifact layers MUST NOT inspect persisted repeater descriptors, reconstruct copy semantics, omit unsupported copies, or acquire remote resources. Equivalent render requests for one revision MUST be deterministic and remain within SSIM >=0.99, aligned float-PCM RMS <=0.0001, and timing within one output frame. Repeated sources that contain audio MUST remain rejected because this milestone only permits shape, group, and component-instance sources whose repeated evaluated closure is visual-only; a group or component closure containing audio MUST fail canonical validation with `INVALID_ARGUMENT`.

#### Scenario: Match all render intents
- **WHEN** root and nested repeaters copy transformed shapes, groups that actually contain resolved component descendants, and retimed component instances at boundary opacity values
- **THEN** frame, range, draft, and export consume equal complete semantic occurrence facts and their outputs meet the documented visual/audio/timing tolerances

#### Scenario: Preserve visible order with large component subtrees
- **WHEN** a repeated component contains at least eleven overlapping equal-z visual siblings whose source order determines occlusion
- **THEN** every render intent displays the same numerically ordered topmost occurrence without lexicographic reordering

#### Scenario: Reject audio-bearing or unsupported closures
- **WHEN** a group or component-instance source transitively contains audible media or another unsupported instruction for repeater expansion
- **THEN** editor-core rejects the repeater with `INVALID_ARGUMENT` before scene publication, artifact preparation, graphics rasterization, FFmpeg execution, or output publication

#### Scenario: Fail rather than degrade
- **WHEN** copy expansion exceeds a canonical budget or no local backend can render the complete repeated scene
- **THEN** evaluation returns `INVALID_ARGUMENT` for complexity or readiness returns `DEPENDENCY_UNAVAILABLE`, and no partial or copy-reduced artifact is published
