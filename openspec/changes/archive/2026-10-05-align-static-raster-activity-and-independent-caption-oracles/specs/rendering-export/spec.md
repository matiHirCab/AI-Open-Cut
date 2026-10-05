## ADDED Requirements

### Requirement: Canonical aligned affine raster cadence
Accepted affine Shape content, including canonical SVG/Grid lowering, MUST preserve exact half-open root activity on aligned frame, audiovisual range and export grids when its visible interval has fractional/off-grid endpoints or a local time-varying key/transition expression is evaluated. Source and compositor cadence MUST not allow a later secondary image event to activate an earlier labelled output frame. Root visibility MUST preserve fractional instance clocks without integer rounding. Accepted local time-varying position/scale/opacity expressions MUST consume canonical scene-cadence timestamps; a fullspan activity interval MUST NOT suppress required cadence normalization. Single authored constants MUST retain their unaffected static graph identity. Public Shape/SVG/Grid transition endpoints MUST retain their existing rejection; private defensive transition classification MUST NOT imply new acceptance. Existing animated-ancestor cadence, geometry, source paint, codecs, media/audio clocks, resource limits, errors and unaffected aligned-boundary/fullscene graph identity MUST remain unchanged. Rendering MUST remain observational without revision/project/history changes.

#### Scenario: Raster activity begins between global output samples
- **WHEN** static Shape/SVG/Grid content is visible at713..799ms in a10fps scene and aligned frame700/800, range700..900 or global export is rendered
- **THEN** both700/800 output samples are independently black while exact nonaligned713/726/798 frame samples remain active with authored paint and unchanged strict comparison bounds

#### Scenario: Fractional inherited visibility and unchanged aligned cases
- **WHEN** a valid mapped fractional instance interval or an aligned-boundary/fullscene static raster reaches the affine source path
- **THEN** fractional visibility remains half-open at canonical root samples without timestamp rounding, unaffected static graph identity/paint is preserved, and existing state/error/resource behavior remains unchanged

#### Scenario: Fullspan own animation consumes the scene cadence
- **WHEN** a fullspan legacy-affine Shape has a valid own opacity ramp0@0ms→0.9@900ms, local geometry keys and renders aligned frame700, aligned range samples or export
- **THEN** expressions use the exact canonical scene-grid times, the700ms opacity matches independently authored0.7 pixels with additional exact equality evidence, and authored constant/hold-sequence controls establish applicable geometry values without sharing ramp/clock evaluation

### Requirement: Image Media canonical source cadence
Accepted image Media with the reviewed off-grid interval[713,799) MUST preserve its half-open activity on aligned Frame, Range and Export grids; accepted local opacity/geometry/transition expressions MUST consume canonical grid times when this cadence correction applies. Existing inclusive legacy endpoint policy outside the reviewed interval MUST remain unchanged; this cadence-only correction MUST NOT claim universal image endpoint repair. Where precise root visibility endpoints are off-grid or evaluated local properties are time-varying or accepted transitions exist, source cadence MUST align with scene cadence after the existing source timestamp mapping in both direct nonaffine and affine paths. Classification MUST use canonical image resource kind and preserve fractional clocks without rounding. Existing affine animated-ancestor normalization for every Media kind MUST remain unchanged. Video MUST receive no new cadence reason; source input seek, rate/start mapping, decoding, measurements, audio/gain, paint, errors/order and unaffected static-image/video graph identity MUST remain unchanged. Independent fade controls MUST retain the authored rational phase and separately derive the existing8bit alpha quantization, with matching encoded input-sequence context; they MUST NOT use sampled actual values or the production fade filter as their oracle. Rendering and rejected edits MUST preserve authoritative project/history bytes.

#### Scenario: Image activity and expressions on aligned outputs
- **WHEN** image Media spans[713,799) in a10fps scene, or has fullspan accepted own opacity/geometry/outgoing fade expressions, and Frame/Range/Export is observed
- **THEN** independently authored held-value controls match applicable expressions at canonical grid times with existing strict bounds; specifically for[713,799), inactive700/800 samples are black and existing nonaligned713/726/798 observations remain active

#### Scenario: Preserve video mapping and unsupported transition errors
- **WHEN** video, missing resource/input, unaffected aligned static image, or an attempted Shape/SVG/Grid transition endpoint reaches existing owning behavior
- **THEN** video/source-clock/audio and static graph identity remain unchanged, established errors/order are retained, unsupported endpoints retain INVALID_ARGUMENT without project/history changes, and no new public acceptance or resource fallback occurs

## MODIFIED Requirements

### Requirement: Accepted visual requested-origin animation fidelity
Frame and range previews, including materialized draft candidates, MUST sample accepted animated visual content at the exact requested root origin and existing output cadence using canonical retained/inherited/loop clocks once. SolidColor's accepted legacy/typed visual animation and Caption's accepted inherited parent animation MUST satisfy this requirement; Caption-owned visual channels MUST remain unsupported with their existing error. Supported accepted-input cases MUST NOT acquire missing-measurement/raster/resource errors or silently disappear because requested-origin preparation omits their source kind. Source activity, opacity and transitions MUST share canonical sample time, while aligned requests and export retain their canonical global grid, with the verified direct synthetic SolidColor/legacy Rectangle placement/inclusive-seam defects and separately defined affine Shape/SVG/Grid and image Media cadence defects corrected; unaffected static paths, codec profile, audio/media decode/gain/placement and public contracts retain their existing semantics. Equivalent independently authored reference geometry/opacity/activity MUST match within existing documented visual/audio tolerances; correction MUST NOT widen timestamps, tolerances, resource limits or reviewed golden references.

#### Scenario: Match nonaligned SolidColor samples to independent controls
- **WHEN** an accepted animated SolidColor is sampled at nonaligned frame713 and range713/813 root times, including an inherited/retained loop and a materialized draft candidate
- **THEN** visible color/opacity/geometry/activity match independently fixed controls at each exact canonical time, preserve finite phase and source mapping, and corresponding aligned/export samples retain their established results

#### Scenario: Preserve accepted inherited Caption at requested samples
- **WHEN** a valid Caption inherits an animated parent opacity or transform and is previewed at nonaligned frame/range origins with its existing font/resource configuration
- **THEN** it remains visible with canonical inherited values/activity matching independent constant-parent controls, resource preparation succeeds under existing policy, own Caption channels remain rejected, and unrelated aligned/export/audio behavior is unchanged

#### Scenario: Preserve nonaligned plain static half-open activity
- **WHEN** a valid ordinary Rectangle, SolidColor, Caption, Shape, SVG, Grid, pinned shaped Text or image Media with no animation/parent/effects and its existing source-specific default transforms has span[713,799) at10fps and is sampled at Frame713/726/798 or Range713 first sample
- **THEN** it is visible with its independently fixed source paint/placement, is inactive at aligned700/800, and trim/Undo/Redo/reopen preserve the intended interval without converting normal Caption font/bottomcenter/escaping/normal text expansion configuration to affine semantics

#### Scenario: Reject genuine resource failures before output effects
- **WHEN** requested-origin preparation exceeds existing finite/resource limits or an existing required media/font resource is invalid, or the same accepted scene is observed repeatedly through frame/range/draft/export
- **THEN** established typed failure and preflight ordering occur before output inspection/workspace/render effects, repeatable valid output preserves project/history/draft bytes, and no fallback measurement, new unsupported error or clock reset is used

#### Scenario: Preserve bounded synthetic global-grid phase and split seams
- **WHEN** direct synthetic SolidColor or legacy Rectangle retained finite repeat/pingpong content is split at700/713/800ms or left-trimmed and observed through aligned frame/range, global export or materialized draft output
- **THEN** checked exact global-grid frame counts generate every required source cell (including700/800 for713..813 at10fps), canonical finite exhaustion and inherited fractional phase remain intact, visibility is half-open with no adjacent-half double alpha, independent unsplit/static controls match within unchanged tolerances, and checked10/24/30/120fps boundary/overflow cases retain existing safety without media/audio/Caption or loop-helper changes

#### Scenario: Preserve reviewed fullscene identity and established endpoint failures
- **WHEN** direct root nonaffine SolidColor or legacy Rectangle has no instance/ancestors/stages and mapped span[0,sceneDuration), an aligned interior Frame or valid Range/Export actually reaches its direct legacy branch after existing sampled/effect/blur routing, or a partial/clipped/inherited/endpoint case is requested
- **THEN** only eligible fullscene consumers retain exact historical graph identity; raw root nonaffine SolidColor intrinsic dimensions are finalized only when needed before sampled preflight, affine/component measurements stay unchanged, all other cases preserve corrected bounded/half-open sampling, and established900ms success/1000ms nonretryable FFMPEG_FAILED publish failure without artifact/1001ms VALIDATION_FAILED and unchanged authoritative/error-directory bytes remain intact without Debug/comparator/golden changes
