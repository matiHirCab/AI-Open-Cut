## ADDED Requirements

### Requirement: Shared bounded styled text rasterization
Frame, range, materialized draft and final export MUST consume the same evaluated span styles, pinned glyph positions and paint stacks. Stroke width MUST be total centered width in text-local project pixels, with round joins. A shadow MUST use the union of its segment's filled glyph coverage, translated in local coordinates (+x right, +y down), then blurred with a normalized separable Gaussian kernel of radius ceil(3*sigma), transparent outside the mask; zero sigma MUST bypass blur. Shadow color/opacity MUST be applied once to the blurred coverage. Shadow masks MUST NOT include strokes, backgrounds or previously painted layers. Painting MUST precede the existing item/group Transform2D and opacity application. Bounds MUST include stroke half-width, signed offsets and full finite blur support before allocation. All numeric results MUST be finite. Existing per-raster limits of 16384 per dimension and 16777216 pixels MUST remain; aggregate intermediate mask pixel work per effective text MUST not exceed 268435456, computed as raster pixels times the sum of one pass per fill/stroke/zero-blur shadow and three passes per nonzero-blur shadow over evaluated paint segments. Limits MUST be checked before destination inspection or artifact allocation, with INVALID_ARGUMENT on overflow. Existing expanded-scene limits MUST also apply.

#### Scenario: Verify stroke and shadow geometry independently
- **WHEN** fixtures render fractional strokes, negative offsets, sigma zero and nonzero, transparent layers and overlapping glyphs under scaling/rotation
- **THEN** coverage, compositing, padded bounds and alpha falloff match independent analytical or reviewed reference expectations without clipped blur tails or per-glyph shadow darkening

#### Scenario: Reject excessive paint work before output
- **WHEN** raster dimensions, pixel area or aggregate layer work meets the limit or exceeds it, including component/repeater-expanded text
- **THEN** inclusive limits pass and excessive candidates return INVALID_ARGUMENT before destination probing, output writes or allocation

### Requirement: Styled text render intent parity
Equivalent styled-text requests MUST produce identical evaluated glyph/style plans across frame, range, draft and export, visual SSIM at least 0.99, aligned decoded float-PCM RMS error at most 0.0001 and timing alignment within one output frame. Reopen with removed original fonts MUST retain the same output through managed bindings. With new fields absent, schema-19 and migrated schema-20 text MUST retain exact existing glyph plans and decoded lossless raster output. Existing keyframes, wrapping, mandatory separators, backgrounds, anchoring, stacking, component slots, repeaters and half-open timing MUST remain effective. Failures MUST retain existing integrity, dependency and path error behavior without silently substituting fonts or dropping layers.

#### Scenario: Compare every render intent and legacy fallback
- **WHEN** reviewed multilingual fixtures with spans/layers, animated root text, transformed components, independent slots, repeaters and audio render through all intents before/after reopen
- **THEN** shared plans and decoded outputs meet the specified tolerances, and equivalent fixtures without new fields match the legacy path exactly

#### Scenario: Preserve typed rendering failures
- **WHEN** required managed faces are missing/damaged or a render input violates resource confinement
- **THEN** the established typed error occurs before artifact publication without fallback
