## ADDED Requirements

### Requirement: Accepted visual requested-origin animation fidelity
Frame and range previews, including materialized draft candidates, MUST sample accepted animated visual content at the exact requested root origin and existing output cadence using canonical retained/inherited/loop clocks once. SolidColor's accepted legacy/typed visual animation and Caption's accepted inherited parent animation MUST satisfy this requirement; Caption-owned visual channels MUST remain unsupported with their existing error. Supported accepted-input cases MUST NOT acquire missing-measurement/raster/resource errors or silently disappear because requested-origin preparation omits their source kind. Source activity, opacity and transitions MUST share canonical sample time, while aligned requests and export retain their canonical global grid, with the verified direct synthetic SolidColor/legacy Rectangle placement and inclusive-seam defects corrected; unaffected static paths, codec profile, audio/media decode/gain/placement and public contracts retain their existing semantics. Equivalent independently authored reference geometry/opacity/activity MUST match within existing documented visual/audio tolerances; correction MUST NOT widen timestamps, tolerances, resource limits or reviewed golden references.

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
