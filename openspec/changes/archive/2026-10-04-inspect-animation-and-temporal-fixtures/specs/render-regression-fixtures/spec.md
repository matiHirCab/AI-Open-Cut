## ADDED Requirements

### Requirement: Independent temporal animation recipe
The regression suite MUST maintain a bounded test-only synthetic recipe and fresh-store generator covering spring overshoot, valid Bézier interpolation, finite/infinite repeat and ping-pong phase/exhaustion, hidden-rank stagger, inherited fractional clocks, repeater offsets and retained source clocks. Expected values MUST use independently specified analytic equations and explicit phase tables rather than production sampling. Exact endpoints, both sides of seams and half-open activity MUST be covered. Recipe generation MUST use valid existing typed APIs without public contract/schema changes.

#### Scenario: T1 Verify overshoot and loop seams
- **WHEN** canonical core sampling evaluates the declared spring/Bézier and finite/infinite loops at interiors, endpoints and fractional samples on both sides of100/300/500/900ms seams
- **THEN** values match independent equations/phase tables, spring overshoot is observable, endpoint holds are exact and finite exhaustion differs correctly from infinite continuation

#### Scenario: T2 Verify inherited stagger and retained source time
- **WHEN** the authored nested hierarchy includes a hidden stagger-rank child, fractional inherited rate, retained leaf clock and repeater copies
- **THEN** independently derived source times, including644.75/569.75/494.75 at root713ms, and sampled values match, hidden rank is retained and activity bounds remain half-open

### Requirement: Temporal production render conformance
The temporal recipe MUST run production frame preview, audiovisual range preview, materialized-draft preview/range and final export through the shared canonical evaluation path. Matching global-grid samples MUST agree with independent static geometry/analytic expectations. Encoded outputs MUST be compared to an independently authored full static-geometry sequence using the same intent, exact output sample grid, codec/profile/color conversion and renderer context; reference construction MUST NOT share animation channels, clocks, hierarchy, curve evaluation or sampled actual data. Independent numeric and lossless geometry/color/mass checks MUST also remain required, with per-authored-color lane masks. Existing references MUST be replaced within the40-render ceiling rather than expanding work, and non-frame-aligned range first frames MUST preserve absolute fractional semantics rather than be compared to an unrelated export grid frame. Existing visual/audio/timing tolerances, immutable canonical goldens, native fail-closed dependency rules and configured timeout budgets MUST remain unchanged. Candidate/render observations MUST not commit or mutate revision, project, drafts or history.

#### Scenario: T3 Compare temporal intents and boundary frames
- **WHEN** the two small fixture families render frame300/500/1000/1200ms, range300..1300ms, fractional-first range713..913ms, export0..1300ms and materialized candidate frame1000/range900..1300ms
- **THEN** independent numeric/lossless geometry and codec-matched full-sequence references establish overshoot, phase, finite exhaustion, inherited timing and first-frame continuity across applicable intents, including exact713/813ms fractional samples, with explicit black inactivity checks and no reference recapture or tolerance weakening

#### Scenario: T4 Preserve failure and retained-state semantics
- **WHEN** animation/controller edits succeed, are undone/redone/reopened, or invalid/missing/locked/stale requests and a failing alias batch are submitted
- **THEN** each retained state matches its independent numeric expectation, failures preserve existing stable errors and atomic project/history bytes, and inspection/rendering leaves authoritative state unchanged

### Requirement: Reproducible temporal evidence
Temporal fixture documentation MUST state recipe identity, source-clock derivation, independent formulas/tables, generator and conformance commands, declared tools and unchanged tolerance policy. Required native execution MUST fail rather than skip when configured dependencies are unusable. Existing golden capture/publication behavior MUST remain unchanged.

#### Scenario: T5 Reproduce independent temporal evidence
- **WHEN** a reviewer generates the fixture in a fresh store and runs declared numeric/native checks
- **THEN** the results are reproducible from reviewed analytic data and explicit dependencies, with concrete failures and unavailable evidence retained rather than silent skips or coordinated self-comparison


### Requirement: Conservative local animated geometry envelope
Production local animated-geometry measurement MUST include the full canonical curve envelope of each ordered same-property position/scale key segment, including valid spring overshoot and stored endpoints. It MUST reuse existing canonical curve_bounds and sampled-scale clamp[0.000001,100.0] (clamping raw extrema rather than rejecting an otherwise valid sampled scale), preserve legacy monotonic/hold behavior, and conservatively cover retained source clocks, loops and inherited phase without resampling or changing evaluation. Existing Cartesian transform bounds, canvas clipping, finite checks and dimension/area/resource limits MUST remain enforced before canvas clipping. This correction MUST NOT change easing, clocks, public contracts, persisted schema or preview request ownership.

#### Scenario: T6 Retain position and scale overshoot pixels
- **WHEN** valid spring position or scale animation, including paired legacy and scalar-axis properties with unlooped or retained source clocks, has an interior extremum beyond stored endpoint values
- **THEN** independently expected transformed geometry fits the production measured envelope and actual frame/range/materialized-draft/export output retains the overshoot, including the reviewed300ms spring lane

#### Scenario: T7 Preserve bounded measurement and endpoint behavior
- **WHEN** hold/monotonic keys, exact spring endpoints or a curve envelope exceeding existing finite/raster resource limits is measured
- **THEN** existing endpoint/hold sampling remains unchanged, conservative bounds preserve valid prior behavior and unsafe bounds fail through existing stable errors before raster allocation/publication


### Requirement: Exact requested origins for supported animated visuals
Production non-frame-aligned frame/range origins MUST use the existing canonical bounded CPU sampling path for supported animated Rectangle, Shape, Media and existing shaped/PAM-capable Text sources with local position/scale/opacity keys, animated ancestors or transitions, and for static Rectangle sources with explicit Transform2D to preserve half-open activity. Frame MUST sample its requested milliseconds; Range MUST sample the canonical existing startMs+floor(n*1000/fps) grid, preserving inherited fractional and retained source clocks, loop phase, half-open activity, opacity and transitions exactly once. The same private eligibility MUST govern preflight and resource preparation, without altering codec/fidelity selection, public contracts or schema. Existing aligned/export/other plain-static paths, media source mapping, audio decode/gain, safety limits/errors and side-effect ordering MUST remain unchanged. Caption and unshaped legacy Text MUST remain excluded without introducing a missing raster-binding failure; their existing nonaligned expression limitations MUST be explicitly documented as remaining and MUST NOT be converted into a new rejection or a universal correction claim.

Canonical sampled-source measurement MUST retain Rectangle intrinsic width/height independently of affine eligibility and MUST obtain existing Media measurements using the same intent-aware sampling eligibility as preflight/preparation. Nonaffine sampled media MUST retain measured size while affine computation remains affine-only. Required Media/Text measurements MUST NOT be weakened or replaced with canvas-size fallbacks. Private preflight wiring MAY carry the existing render intent; public APIs, codecs, media/audio clocks and measurement-error ordering MUST remain unchanged.

#### Scenario: T8 Sample the exact requested visual clock once
- **WHEN** a supported animated direct-expression or existing CPU-prepared source, or a static affine Rectangle cell is requested at Frame713 or Range713..913ms at10fps
- **THEN** actual713/813ms geometry, retained/inherited loop phase, activity, opacity and relevant transition/legacy properties match independent expected times without700ms quantization or duplicate phase application

#### Scenario: T9 Preserve path eligibility and safety
- **WHEN** supported-source nonaligned sampling is preflighted, or aligned/export/plain-static/Caption/unshaped-Text requests and media/audio regression cases execute
- **THEN** preflight and preparation agree, unsafe existing work fails before side effects, unchanged paths retain their prior behavior and media/audio timing, and the Caption and unshaped-Text limitations are honestly tracked without changing its error behavior


#### Scenario: T10 Measure eligible intrinsic sources before sampling
- **WHEN** an actual normalized opacity-only Rectangle or legacy Position/Scale/Opacity/transition Media enters nonaligned canonical sampling without requiring affine computation
- **THEN** Rectangle source dimensions and existing probed Media dimensions are available consistently before sample preflight and preparation,5x5 Rectangle source pixels remain5x5, failed measurements preserve existing errors without output/workspace side effects, and unchanged measurement/decode/audio paths retain their prior semantics
