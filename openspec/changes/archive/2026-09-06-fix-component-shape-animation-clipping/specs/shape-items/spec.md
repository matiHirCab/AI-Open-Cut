## MODIFIED Requirements

### Requirement: Canonical shape evaluation and bounded raster work
Core MUST evaluate shapes into process-local drawing instructions carrying resolved geometry, paint, stroke, transforms, visibility, half-open timing and canonical stacking. Root and component-local shapes MUST obey existing parent scopes, instance clocks, inherited opacity and expansion limits. Hidden and unused persisted content MUST still validate. Shapes MUST introduce no media references. Shape rendering MUST preserve linear-light premultiplied gradient interpolation, pad extension, fill rules, centered strokes, caps/joins/dashes and miter fallback from vector-primitives.

Each shape MUST compile to at most 8192 path commands and at most 65536 flattened segments, with a maximum curve subdivision depth of 16 and target deviation at most 0.25 output pixels. The authored VectorPath limit MUST remain 4096 commands; the compiled limit accommodates closure of maximum-sized polygons and stars. A scene sample MUST contain at most 1048576 flattened segments; checks MUST precede allocation beyond these limits. Failure to meet tolerance within the bounds MUST return INVALID_ARGUMENT without degraded geometry. Existing evaluated geometry, surface-memory and visual-layer limits MUST also apply before rasterization.

Shape local normalized coordinates MUST resolve against the owning component's canvas, while root-space animation sampling bounds MUST clip against the requested output viewport after all ancestor and instance transforms. Nested sampling MUST preserve that distinction at every level. Adding constant position or scale keyframes MUST NOT change visible geometry. Existing finite geometry and resource bounds MUST be validated before clipping; viewport exclusion MUST NOT bypass rejection. Rendering MUST remain read-only with unchanged project/history/revision contracts and existing typed failures.

#### Scenario: Evaluate nested and repeated shapes
- **WHEN** root and component-local shapes appear under transformed parents and repeated time-scaled instances
- **THEN** evaluation preserves occurrence identity, local clocks, canonical order, visibility and opacity without changing audio or consulting persisted state downstream

#### Scenario: Bound expensive geometry
- **WHEN** geometry exceeds command, subdivision, segment, scene or existing surface limits, including after transforms and instance expansion
- **THEN** core returns INVALID_ARGUMENT before raster allocation, writes or render execution

#### Scenario: Render exact vector semantics
- **WHEN** fixtures exercise gradient transparency, both fill rules, open and closed curves, dash phase, every cap/join and miter fallback
- **THEN** independently checked pixels and geometry reflect the existing vector contract rather than backend defaults

#### Scenario: Preserve translated shapes with constant animation
- **WHEN** a 10x10 red shape in a 40x40 component translated to (100,20) on a 240x120 output receives constant scale keys of 1
- **THEN** its image remains identical to the static version, including visible red geometry at the translated position, across frame, range and export rendering

#### Scenario: Compose nested and retimed animation before clipping
- **WHEN** shape position and scale animate inside translated, scaled, nested or retimed component instances
- **THEN** each sampled root timestamp resolves the established local clock and composed geometry, with bounds clipped only to the requested output viewport and independently expected pixels preserved

#### Scenario: Preserve visible portions at output boundaries
- **WHEN** an animated component shape is partially or fully outside the output viewport
- **THEN** its visible portion remains at the correct root position, fully offscreen geometry produces no visible pixels, and invalid oversized or non-finite geometry still fails with INVALID_ARGUMENT before allocation, writes or backend execution

#### Scenario: Separate local units from requested output dimensions
- **WHEN** a component uses normalized positions and animated shapes are rendered at output dimensions different from both component dimensions and project settings
- **THEN** normalized positions still resolve against the component canvas, animation bounds clip against the requested output dimensions, and matching frame/range/export samples retain the same visible placement
