## ADDED Requirements

### Requirement: Strict core-owned vector vocabulary
Core MUST expose serializable vector primitives with closed camelCase fields and closed `type` tags, reject missing required fields and unknown fields or variants, and validate every numeric value as finite. Pure validation MUST return existing non-retryable INVALID_ARGUMENT on semantic failure without I/O or mutation. Points MUST use local pixel coordinates with top-left origin, positive X right and positive Y down; every coordinate MUST be in [-1000000, 1000000]. Resource strings, SVG markup, renderer expressions, and implicit coercion MUST NOT be accepted.

#### Scenario: Round-trip typed primitives
- **WHEN** every supported primitive is serialized, parsed, validated, and serialized again
- **THEN** its typed value and canonical field names are preserved without resource resolution or mutation

#### Scenario: Reject malformed and unsafe values
- **WHEN** input contains an unknown field/tag, absent required field, wrong type, raw SVG/path/URL/expression string, non-finite number, or coordinate outside the inclusive limit
- **THEN** strict decoding rejects malformed structure or core validation returns INVALID_ARGUMENT, with no side effects

### Requirement: Explicit colors and gradient paints
Color MUST be `{r,g,b,a}` with finite components in [0,1], unassociated sRGB RGB channels and linear alpha. Paint MUST be `solid` with `color`, `linearGradient` with `start`, `end`, and `stops`, or `radialGradient` with `center`, `radius`, and `stops`. Linear endpoints MUST differ and radial radius MUST be in (0,1000000]. Each gradient MUST have 2 through 64 stops `{offset,color}`, ordered by strictly increasing finite offsets in [0,1], with first offset 0 and last offset 1. Semantics MUST specify pad extension and interpolation after converting RGB to linear light and premultiplying alpha; fully transparent interpolated colors resolve to zero RGB on unpremultiplication. Radial gradients MUST be circular with focal point at center. Validation MUST preserve supplied stop order and MUST NOT sort, clamp, or repair inputs.

#### Scenario: Accept all paint variants and boundaries
- **WHEN** solid, linear, and radial paints use opaque/transparent colors, legal coordinates, and 2 or 64 ordered endpoint-inclusive stops
- **THEN** validation succeeds and round-trip preserves all supplied values and order

#### Scenario: Reject ambiguous gradients
- **WHEN** colors exceed bounds, endpoints coincide, radius is nonpositive, stops duplicate/decrease offsets, endpoints are absent, or stop count is below 2 or above 64
- **THEN** core rejects the paint with INVALID_ARGUMENT without normalization

### Requirement: Bounded strokes and corner radii
Stroke MUST require `paint`, `width`, `dash`, `dashOffset`, `lineCap`, `lineJoin`, and `miterLimit`. Width MUST be in (0,16384]; dash MUST contain zero or an even number of at most 64 positive finite entries each at most 1000000; dashOffset MUST be in [-1000000,1000000]. Caps MUST be `butt`, `round`, or `square`; joins MUST be `miter`, `round`, or `bevel`; miterLimit MUST be in [1,1000], defining the ratio of miter length to half-width, with bevel fallback above the ratio. Strokes MUST be centered, with dash lengths in local pixels, positive offset advancing into the pattern and phase restarting per subpath. Corner radii MUST require `topLeft`, `topRight`, `bottomRight`, and `bottomLeft`, each a scalar in [0,16384]. For a positive bounded rectangle, the effective radii MUST use one common scale factor min(1, width/topSum, width/bottomSum, height/leftSum, height/rightSum), ignoring zero denominators; source radii MUST remain unchanged.

#### Scenario: Accept stroke and radius boundaries
- **WHEN** all cap/join variants, solid and dashed strokes, positive/negative offsets, and zero/maximum corner radii are validated
- **THEN** legal values succeed and common-factor radius resolution preserves proportions and source values

#### Scenario: Reject invalid stroke collections
- **WHEN** width is zero/oversized, dash count is odd or above 64, a dash is nonpositive, miterLimit is outside bounds, a radius is negative, or rectangle dimensions are nonpositive/non-finite/above 16384
- **THEN** core returns INVALID_ARGUMENT before geometric work

### Requirement: Structured bounded Bezier paths
Path MUST require `fillRule` (`nonzero` or `evenodd`) and `commands` with 1 through 4096 commands. Commands MUST be `moveTo` and `lineTo` with `to`, `quadraticTo` with `control` and `to`, `cubicTo` with `control1`, `control2`, and `to`, or `close` without parameters; all points MUST satisfy the coordinate contract. Every subpath MUST start with moveTo. Drawing or closing before moveTo and repeated close MUST be rejected. Close MUST require at least one drawing command since the current moveTo, connect to that subpath's start, and require a new moveTo before further drawing. Multiple subpaths and move-only paths MUST be accepted; move-only paths produce no geometry. Filling MUST implicitly close open drawable subpaths while stroking MUST close only on explicit close. No string parser, relative commands, arcs, tessellation, or renderer-specific syntax SHALL be introduced in this change.

#### Scenario: Accept open and closed paths
- **WHEN** paths contain move-only, line, quadratic, cubic, open, explicitly closed, or multiple legal subpaths with either fill rule
- **THEN** core validates them and serialization preserves command order and explicit closure

#### Scenario: Reject path grammar and complexity violations
- **WHEN** paths are empty, exceed 4096 commands, draw before moveTo, close without drawing, repeat close, draw after close without moveTo, or contain invalid control points
- **THEN** core returns INVALID_ARGUMENT without traversal beyond the rejected collection limit or resource work

### Requirement: Governed parity and deferred activation
A version-1 vector catalog MUST define exact identifiers, named limits, activation status `core_primitives_only`, and valid/invalid examples. Rust production types and mirrored strict TypeScript schemas MUST consume the same fixtures and agree on decoding, bounds, and semantic acceptance; core MUST remain the runtime domain authority. Existing wire operations, MCP surface and capabilities, persisted schema, legacy color strings, EvaluatedScene, and render output MUST remain unchanged. Documentation MUST identify ShapeItem and agent/render activation as subsequent issue #28 work. No new reference, revision, batch-alias, history, or migration behavior SHALL be introduced by these reference-free primitives.

#### Scenario: Verify shared fixture evidence
- **WHEN** both language suites validate the catalog including malformed wrappers, all variants, inclusive limits, overflow, and wrong-type fixtures
- **THEN** they agree on every expected outcome and fail on catalog or identifier drift

#### Scenario: Preserve existing workflows
- **WHEN** legacy projects are opened, edited with standalone/batch operations, subjected to stale revisions and failed batches, undone/redone, reopened, and previewed/exported
- **THEN** existing regression suites retain their established state, errors, atomicity, schema, and render behavior without advertising shape support
