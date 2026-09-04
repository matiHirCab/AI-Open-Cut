## ADDED Requirements

### Requirement: Canonical Transform2D affine evaluation
Core MUST resolve top-left coordinates with positive X right and positive Y down, normalized position against composition dimensions, and normalized anchor against post-crop unscaled source dimensions. It MUST apply anchor translation, independent scale, X shear, Y shear, clockwise rotation, then position. For column vectors the matrix SHALL be T(position) R(rotation) Ky(skewY) Kx(skewX) S(scaleX,scaleY) T(-anchor*sourceSize). Text SHALL use its measured styled raster box and active Transform2D captions SHALL use a measured text box with 12 pixels of inset on each side, media the existing fitted/cropped box, and solids/rectangles their declared box. EvaluatedScene MUST own these typed facts without paths, persisted references, or backend expressions. This milestone activates static Transform2D only; the historical schema-v7 milestone remains isolated.

#### Scenario: Compare independent coordinate oracles
- **WHEN** asymmetric source corners are transformed with noncentral anchor, independent scale, both skews, rotation, and pixel or normalized position
- **THEN** evaluation agrees with the normative matrix and equivalent pixel/normalized positions agree exactly within floating-point numerical tolerance of 1e-9 pixels

#### Scenario: Preserve immutable deterministic evaluation
- **WHEN** the same project revision is evaluated repeatedly across output intents
- **THEN** affine facts are equal and project state, revision, and history are unchanged

### Requirement: Bounded affine evaluation
Core MUST reject non-finite derived matrices or coordinates and pre-clipping transformed bounds wider or taller than 16384 pixels or larger than 16777216 pixels in area with INVALID_ARGUMENT after bounded read-only font measurement but before raster/output allocation, resource writes, or backend execution. Existing scene limits and missing-asset precedence MUST remain unchanged.

#### Scenario: Enforce geometry bounds before allocation
- **WHEN** transformed output is exactly a dimension or area limit, exceeds it, or produces non-finite derived values
- **THEN** valid boundary geometry succeeds and invalid geometry fails before render work or artifacts

#### Scenario: Resolve missing assets first
- **WHEN** a transformed media item references a missing asset even with excessive geometry
- **THEN** evaluation returns ASSET_NOT_FOUND before geometry processing or filesystem work

### Requirement: Read-only text measurement precedes affine finalization
Core MUST preflight references, values, timing, and collection limits before read-only path-safe font selection and measurement. Only typed dimensions and layout facts SHALL enter pure affine finalization; paths and font bytes MUST remain outside the scene. Rendering MUST reuse the selected font and measured layout. Affine validation MUST complete before workspace creation, text writes, raster allocation, or execution.

#### Scenario: Resolve font-dependent geometry
- **WHEN** identical text is measured using two configured fonts with different metrics
- **THEN** each finalized affine anchor uses its selected font's measured source box, shared across preview and export

#### Scenario: Reject measured geometry overflow
- **WHEN** a measured text box exceeds the approved transformed bounds
- **THEN** core returns INVALID_ARGUMENT with no workspace, text write, raster buffer, process call, or artifact publication
