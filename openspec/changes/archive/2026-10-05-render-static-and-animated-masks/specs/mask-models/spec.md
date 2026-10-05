## RENAMED Requirements

- FROM: `### Requirement: Explicit inactive rendering interpretation`
- TO: `### Requirement: Explicit mask rendering activation boundary`

- FROM: `### Requirement: Strict-lint-compatible native mask identity witnesses`
- TO: `### Requirement: Strict-lint-compatible activated mask conformance witnesses`

## MODIFIED Requirements

### Requirement: Explicit mask rendering activation boundary
Masks MUST retain owner-local painted coverage and existing model ownership/limits. Pixel positions MUST use local project pixels in the top-left(0,0) basis of the exact owner's post-crop/clip pre-effect source raster. For raster width W,height H and ownerDensity raster pixels per local project pixel, normalized positions MUST use logical extent W/ownerDensity,H/ownerDensity and owner pixel centers MUST be((x+0.5)/ownerDensity,(y+0.5)/ownerDensity). Existing padded vector analytic origins MUST remain only in existing owner-anchor/effect facts and SHALL NOT be added again to mask coordinates; anchor MUST use sampled untransformed, unexpanded path geometry bounds. Alpha MUST mean painted alpha and luma MUST mean Rec.709 weighted premultiplied linear RGB; transparent paint contributes zero. Transform opacity MUST multiply coverage once; inversion MUST map B→1−B within owner raster domain without reviving crop/clip removal or extending source support. Add/subtract/intersect/exclude MUST use A+B−A*B,A*(1−B),A*B,A+B−2*A*B with initial0 for first add/exclude and1 for subtract/intersect and empty-stack identity1. Positive expansion MUST grow geometric support, negative contract it and zero preserve exact coverage; feather MUST mean local Gaussian sigma. Executable order MUST be geometric fill→approved signed expansion→original-coordinate paint/channel extraction→feather→mask affine/opacity→inversion→declared combination before owner effects. Schema33 MUST activate these semantics through mask-rendering; metadata/model capability alone SHALL NOT promise renderer readiness. Existing32 static metadata MUST persist on adoption and intentionally acquire mask output rather than remain visually inactive.

#### Scenario: Distinguish model and rendering support after activation
- **WHEN** an eligible visual has static or animated nonempty painted alpha/luma masks
- **THEN** stored metadata remains observable/undoable and a ready schema 33 renderer executes approved masks, while model-only capability reporting does not claim render readiness

#### Scenario: Preserve deferred source and topology scope
- **WHEN** clients request source-layer masks, raw resources or animation of inversion/operation/channel/path topology
- **THEN** existing closed source/model and channel validation reject them without activating mattes/DAG or unsupported concepts

### Requirement: Strict-lint-compatible activated mask conformance witnesses
Native mask conformance MUST retain supported fixed-width complete-array RGB and float32 PCM iteration compatible with pinned Rust and strict CI Clippy, without warning suppression. Existing length checks, independent visible-color/nonzero-audio thresholds and exact same-input draft/committed/reopened pixel/audio/timing comparisons MUST remain meaningful. Schema33 MUST replace prior model-only masked-versus-unmasked rendering identity assertions with independently expected activated-mask appearance; it SHALL NOT retain contradictory identity promises, disable native assertions or weaken thresholds. Absent/empty-mask identity MUST remain exact. Native tests MUST actually execute with required renderer dependencies and preserve independent wrong-result controls.

#### Scenario: Preserve independent witnesses under strict Clippy
- **WHEN** required schema33 native mask conformance executes with actual FFmpeg and FFprobe and strict workspace Clippy checks its test source
- **THEN** supported typed RGB/PCM groups preserve ordered interpretation and thresholds, same-input draft/committed/reopened outputs agree, authored activated masks match independent expected appearance, absent masks remain exact identity, and no lint is suppressed
