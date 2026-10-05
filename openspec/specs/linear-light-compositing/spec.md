# Linear-Light Compositing Specification

## Purpose

Define the canonical linear premultiplied scene pipeline, bounded resource certification, and native fractional-clock safety for every existing visual source.

## Requirements

### Requirement: Linear premultiplied working representation
Every current evaluated visual source MUST enter scene composition as a finite floating-point premultiplied linear-light RGBA raster. The current decoded local SDR RGBA boundary MUST interpret normalized RGB as sRGB using `L(s)=s/12.92` for `s<=0.04045`, otherwise `((s+0.055)/1.055)^2.4`; alpha MUST remain linear coverage in [0,1]. Working RGB MUST equal straight linear RGB multiplied by alpha; alpha zero MUST imply RGB zero regardless of hidden source RGB. Source rasterizer paint semantics and decode orientation MUST remain those of their existing owning contracts. Bilinear sampling, ordered effects, opacity, and temporal averaging MUST operate on that floating-point representation without intermediate straight-color or 8-bit encoding round trips. Final scene output MUST unpremultiply only at positive alpha, apply `S(L)=12.92*L` for `L<=0.0031308`, otherwise `1.055*L^(1/2.4)-0.055`, clamp to [0,1], and quantize by nearest integer to the existing 8-bit SDR encoding boundary. Finite working components MUST satisfy alpha in [0,1] and RGB in [0,alpha], allowing absolute f32 roundoff at most `1e-6`; larger violations MUST fail with `INVALID_ARGUMENT`, and valid roundoff MUST be clamped to the legal premultiplied interval before final conversion. Numeric unit oracles MUST use absolute working-component tolerance at most `1e-6`; uncompressed output-byte assertions MUST allow at most one byte for quantization. Current exports MUST retain the existing opaque black composition background. This milestone MUST NOT claim HDR, arbitrary profile conversion, or new authored output-color controls.

#### Scenario: Decode translucent and hidden colors
- **WHEN** local RGBA has fractional alpha or nonzero RGB under alpha zero
- **THEN** independent transfer-function tests match the premultiplied working values and the transparent pixel contributes no color during interpolation or blending

#### Scenario: Preserve floating-point resampling and temporal averaging
- **WHEN** a transformed transparent-edge source or motion-blurred visual mixes samples of different coverage and color
- **THEN** interpolation and averaging mix linear premultiplied RGBA, and final conversion matches an independent numeric oracle without colored transparent fringes

#### Scenario: Convert only the completed scene
- **WHEN** half-covered white is composited over opaque black
- **THEN** final uncompressed RGB is approximately 188 per channel for exact alpha 0.5, with one-byte tolerance for quantized source alpha, rather than the encoded-space midpoint 128

### Requirement: Explicit current layer pipeline
Every existing visual MUST follow source rasterization/decode, crop and representable local clipping, declared masks, ordered local effects, local anchor/scale/skew/rotation/position, nearest-to-outer ancestor affine transforms, matte, inherited opacity and transition gain, then destination blend in evaluated bottom-to-top order. Existing crop MUST retain its normalized crop/remap behavior; existing SVG clipping and fill/stroke remain source-local. This milestone MUST represent absent masks and mattes as identity and absent blend selection as normal source-over; it MUST NOT add authored mask, matte, clip, blend, or effect kinds. Opacity/transition gain MUST multiply all four working components exactly once. Source-over MUST use `Co=Cs+Cd*(1-As)` and `Ao=As+Ad*(1-As)` on premultiplied linear components. The raster support extension and unstroked anchor semantics of existing effects MUST remain effective.

#### Scenario: Observe crop effects and affine ordering
- **WHEN** asymmetric cropped content with ordered blur/tint effects is rotated around a noncentral anchor under an ancestor transform
- **THEN** independent stage-order evidence shows crop before effects and effects before affine sampling, without shifting the authored anchor or applying opacity twice

#### Scenario: Blend mixed current sources in evaluated order
- **WHEN** text, decoded media, shape/SVG/grid, rectangle, solid-color, and Caption sources overlap with fractional coverage
- **THEN** every source participates in the same linear source-over semantics in evaluated paint order, including sources without effects or Transform2D

#### Scenario: Preserve absent-stage defaults
- **WHEN** an existing project omits masks, mattes, effects and blend selection
- **THEN** it remains valid without migration and those stages act as identities with normal source-over

### Requirement: Canonical bounded composition and fail-closed publication
Editor-core MUST certify finite geometry/color/alpha and canonical work limits before destination inspection, rasterization, decoder/encoder execution, workspace creation or artifact publication. Composition MUST retain existing stricter source/effect/geometry limits, bound every working surface to positive dimensions no greater than 16384 per axis and 16777216 pixels, bound expanded visual occurrences to 4096, and bound cumulative transformed destination pixel work to 268435456 pixel visits per output frame, counting each layer temporal/shutter sample separately. Peak live editor-owned composition raster and byte-buffer memory MUST be certified at no more than 1073741824 bytes per render request, including the final canvas, per-layer transformed/temporal-average rasters, local/effect scratch rasters, decoded/serialized/streaming RGBA byte buffers, and a conservative 67108864-byte reservation for the existing shared raster cache. Raster accounting MUST use 16 bytes per f32 RGBA pixel and 4 bytes per uncompressed RGBA byte pixel, plus bounded headers/temporary vector storage. Source rasters MUST be processed one layer at a time or retained only within this certified byte budget; an arbitrary number of cached decoded sources MUST NOT accumulate. Concurrent render requests MUST retain their existing worker admission limits. Checked arithmetic MUST reject overflow. Composition MUST stream output frames without retaining the entire requested sequence in memory. Invalid or excessive facts MUST return non-retryable `INVALID_ARGUMENT`; missing resources, unsafe paths, dependency failures and runtime process failures MUST preserve their existing stable typed errors. Rejection or runtime failure MUST publish no partial artifact or project/history change.

#### Scenario: Reject invalid or excessive composition before side effects
- **WHEN** a sample has a non-finite fact, excessive raster dimensions/occurrences/pixel work/live byte memory, or overflowing accounting
- **THEN** core returns `INVALID_ARGUMENT` before destination/workspace/process actions and preserves existing project, revision and retained history

#### Scenario: Reject missing bindings and clean runtime failures
- **WHEN** a resource is missing or a source decoder or output encoder fails
- **THEN** existing resource/render errors remain stable and temporary output is cleaned without artifact publication or project mutation

#### Scenario: Reject aggregate memory excess
- **WHEN** individually valid source/output surfaces and effect/temporal scratch buffers together exceed the certified live-byte ceiling
- **THEN** certification returns `INVALID_ARGUMENT` before workspace, decode, allocation or publication, even though no individual surface exceeds its pixel limit

### Requirement: Certified fractional native media clocks
For an active video source sample whose nonnegative mapped local media clock has a represented fractional millisecond, render preflight MUST certify the native source-clock conversion before any output/workspace creation or source raster/decode process. Certification MUST use compensated addition for the authored whole sourceInMs and mapped local clock, account for conversion to native seconds and its floating-point comparison precision, and conservatively bound the total absolute arithmetic error to at most 0.000001 milliseconds. An unsafe or non-finite fractional clock MUST fail with non-retryable INVALID_ARGUMENT rather than quantizing or dropping the fraction. This render-only numerical certification MUST NOT add an authored time restriction, modify persisted schema/history, or reject an exact whole-millisecond source sample solely because it exceeds the fractional precision envelope; existing backend whole-clock compatibility and failures remain authoritative. Image sources retain their zero-time decode rule.

#### Scenario: Reject unsafe large fractional source offset before resources
- **WHEN** a Java-safe whole sourceInMs near 2^53 is combined with a represented local fractional clock such as 0.25 milliseconds
- **THEN** preflight returns non-retryable INVALID_ARGUMENT before workspace/output creation or any raster/decode process, and leaves the authored project and history unchanged

#### Scenario: Preserve realistic fractions and whole-clock compatibility
- **WHEN** ordinary source offsets and nested instance clocks represent fractional source times within the certified error envelope, or a source sample has an exact whole-millisecond mapped clock
- **THEN** realistic fractional samples retain their represented source cadence within the declared arithmetic tolerance, while exact whole samples retain existing backend compatibility without a new persisted limit
