# SVG Ingestion Specification

## Purpose

Define bounded offline SVG normalization, transactional SVG timeline items, schema migration and deterministic evaluated rendering.

## Requirements

### Requirement: Fail-closed static SVG ingestion
Editor-core MUST accept inline UTF-8 SVG only through the dedicated SVG contract and structurally parse it without network, filesystem, font, entity, or executable resource resolution. The accepted elements MUST be one root svg, nested g, rect, circle, ellipse, line, polygon, polyline, and path. Elements MUST use the SVG namespace or the unqualified form consistently. Only xmlns on the root, root width/height/viewBox/preserveAspectRatio, applicable geometry attributes, fill, fill-rule, stroke, stroke-width, stroke-linecap, stroke-linejoin, stroke-miterlimit, stroke-dasharray, and stroke-dashoffset MUST be accepted. Missing paint MUST default to opaque black fill and no stroke; presentation paint/stroke values MUST inherit through groups. No other element or attribute MUST be silently ignored. XML declarations, comments, and whitespace between elements MUST be inert; other processing instructions, DTDs, entity declarations and non-whitespace text MUST be rejected. Predefined XML escapes and numeric character references MUST be decoded before allowlist checks.

Scripts, event attributes, href/xlink references, URL-valued paints, file/network/data URLs, foreign namespaces, style attributes/elements, text/font content, embedded images, filters, masks, clipping paths, gradients, patterns, use, nested svg, transforms and animation MUST return non-retryable INVALID_ARGUMENT before rasterization or mutation. This is a rejection sanitizer, with no stripping or fallback. Errors MUST identify the unsupported category without echoing the source.

#### Scenario: Accept reference-free static artwork
- **WHEN** input uses supported elements, inherited solid presentation paints and legal geometry
- **THEN** core produces canonical vector content without resource access and preserves document paint order

#### Scenario: Reject active and resource-bearing content
- **WHEN** hostile content uses scripts, namespaced or encoded URLs, handlers, DTD/entity expansion, CSS, font content, unsupported attributes or elements
- **THEN** core returns INVALID_ARGUMENT without resource access, rasterization, source logging, state change or output publication

### Requirement: Explicit SVG geometry and complexity
Root width and height MUST be positive finite unitless or px lengths at most 16384; percentages and other units MUST fail. An omitted viewBox MUST be (0,0,width,height); supplied viewBox MUST contain four finite numbers with positive extents and coordinates within existing vector bounds. preserveAspectRatio MUST be omitted or xMidYMid meet, mapping uniformly into the centered viewport with transparent margins and clipping to the viewport. Coordinates MUST use top-left origin, positive X right and Y down. Geometry MUST use existing vector coordinate and stroke bounds. Rectangles MUST support nonnegative rx/ry only when equal after copying one omitted radius to the other; effective radius MUST clamp to half the smaller dimension. Circle/ellipse radii MUST be positive. Polygon/polyline MUST contain at least three/two points respectively. Paths MUST accept only absolute M/L/H/V/Q/C/Z (including legal repeated argument sets), lower to structured paths, and reject arcs, relative commands and unsupported grammar. Solid colors MUST accept only none, #RGB, #RGBA, #RRGGBB and #RRGGBBAA, with SVG hex alpha semantics; inherited defaults are black fill/no stroke, nonzero fill rule, width 1, butt cap, miter join, miter limit 4, no dash and offset 0. Lines MUST ignore inherited fill and require visible stroke; other paintless geometry MUST fail. Shapes MUST fill before stroking, and open drawable subpaths MUST close only for fill.

Input MUST be at most 1048576 UTF-8 bytes, XML depth at most 32 including the root, element count at most 4096 including groups/root, and attribute count at most 32 per element. Each path MUST contain at most 4096 normalized commands; the document MUST contain at most 65536 normalized commands, counting primitive lowering. Checks MUST precede allocation or expansion beyond each budget. Existing compiled shape, curve subdivision, flattened segment, scene, transformed surface and raster memory limits MUST also apply, including hidden content and component expansion. Non-finite or overflowed numeric intermediates MUST fail with INVALID_ARGUMENT without lowering quality.


A supported drawing command L/H/V/Q/C following Z MUST begin a new normalized subpath at the closed subpath's initial point; the current point MUST reset to that initial point. An explicit M MUST start its own subpath without an extra move. Implicit MoveTo records MUST count against path and document command budgets before insertion. The persisted structured path grammar MUST remain unchanged.

SVG raster surface limits MUST apply to the mapped viewport at the composed sampling density, without phantom padding or an unused source-space child surface. Valid bounded source geometry that fits those limits after mapping MUST NOT fail solely because its standalone source-space surface would be larger. Surface dimensions MUST remain at most 16384 pixels and area at most 16777216 pixels. Compilation MUST preserve source geometry, subdivision and per-shape/scene work bounds, check remaining scene capacity before expansion, and reject non-finite or unrepresentable mapped raster coordinates, stroke widths and dash metrics before artifact preparation, without silent omission or quality reduction.

SVG coordinate conversion MUST retain each mapped raster-space point in f64, convert it to f32 and back to f64, and reject non-finite original or converted values or Euclidean displacement hypot(dx,dy) greater than 0.25 raster pixels. The 0.25 boundary MUST be inclusive. This check MUST run after viewport mapping and sampling density, before adding points to the backend path or computing its bounds, including offscreen and move-only contours. Evaluation and raster preparation MUST share this checked conversion. Excessive conversion loss MUST return non-retryable INVALID_ARGUMENT before artifact preparation. Exactly representable large points within existing backend bounds MUST remain accepted. The conversion threshold MUST be documented separately from the unchanged curve-flattening tolerance and MUST NOT be represented as a combined 0.25-pixel error guarantee. Existing integer-bound, stroke, dash, component-transform, work and allocation checks MUST remain enforced, and standalone shape rendering MUST remain unchanged.

#### Scenario: Preserve viewport and drawing semantics
- **WHEN** valid artwork uses nonzero viewBox origin, unequal viewport aspect ratio, open curves, alpha colors and overlapping siblings
- **THEN** independently checked geometry and pixels preserve centered meet scaling, transparent margins, clipping, fill/stroke semantics and document order

#### Scenario: Enforce every boundary before expensive work
- **WHEN** valid fixtures hit each inclusive budget or counterexamples exceed one budget, use non-finite numbers or invalid grammar
- **THEN** valid boundaries succeed and counterexamples return INVALID_ARGUMENT before excess allocation, rasterization or writes

#### Scenario: Continue supported drawing after closepath
- **WHEN** legal absolute L/H/V/Q/C commands follow Z, with nonzero subpath starts, repeated argument sets or a later explicit M
- **THEN** normalized geometry starts each implicit continuation at the closed subpath's initial point and explicit moves introduce no duplicate move

#### Scenario: Count implicit continuation commands before expansion
- **WHEN** inserted MoveTo records bring a path or document to its inclusive command limit, or would exceed that limit
- **THEN** boundary inputs succeed and excess input returns INVALID_ARGUMENT before excess command allocation, with existing revision and batch rollback guarantees

#### Scenario: Render large source geometry through a small viewport
- **WHEN** a 100x100 SVG uses viewBox 0 0 10000 10000 and a matching rectangle, or equivalent mapped artwork uses nonzero origins, clipping, curves and strokes
- **THEN** rendering succeeds with independently correct viewport pixels and no unused source-space raster allocation

#### Scenario: Enforce actual mapped surface and precision budgets
- **WHEN** viewport mapping and composed magnification reach or exceed a surface, subdivision, scene-work or memory limit, including hidden or expanded component content, or mapped raster values become unrepresentable
- **THEN** inclusive valid bounds succeed and invalid values fail with INVALID_ARGUMENT before excess allocation or artifact publication without weakening standalone shape limits

Mapped SVG raster representability MUST include outward-rounded backend integer bounds, representable bound dimensions and coordinate arithmetic, plus conservative stroke expansion for caps and miters. Dash conversion and construction MUST succeed without non-finite intermediates or fallback to an undashed stroke. Unrepresentable geometry MUST return non-retryable INVALID_ARGUMENT before artifact preparation or backend execution. Legitimate empty coverage from offscreen geometry or degenerate fills MUST remain valid when numeric bounds are representable. Validation and raster preparation MUST use consistent coordinate conversion rules; geometry MUST NOT be individually clamped or silently dropped.

Transform-dependent SVG limits MUST use complete component occurrence transforms, including group ancestry and existing keyframe scale bounds and clock semantics. Referenced definitions MUST NOT additionally undergo isolated-root raster sizing. Hidden occurrences MUST undergo the same bounded validation. Definitions unreachable from project instances MUST be validated as virtual roots with identity outer transforms, including their nested instances. All stored normalized documents MUST remain validated. Remaining work and memory capacity MUST be enforced during occurrence expansion without counting intermediate compilation twice; existing cycle, depth, occurrence, subdivision, surface and aggregate limits MUST remain enforced.

#### Scenario: Reject finite but backend-unrepresentable mapping
- **WHEN** a 100x100 SVG uses viewBox 0 0 0.0001 0.0001 and a rectangle spanning -5000 to 5000 on both axes, or fill/stroke bounds overflow backend numeric conversion
- **THEN** evaluation returns non-retryable INVALID_ARGUMENT before artifact preparation, rather than successfully publishing blank or incomplete output

#### Scenario: Preserve representable clipping and empty coverage
- **WHEN** accepted boundary values, negative coordinates, viewport-crossing fills or strokes, caps, miters, dashes, offscreen geometry or degenerate fills remain numerically representable
- **THEN** evaluation succeeds and independent expected pixels or empty coverage match existing clipping and paint semantics

#### Scenario: Compose cancelling component scales before sizing
- **WHEN** a 100x100 SVG has local scale 100 inside a component instantiated at scale 0.01, including equivalent nested group and component ancestry using legacy transforms or Transform2D
- **THEN** the occurrence renders like the equivalent identity-scale scene without an isolated component surface rejection

#### Scenario: Validate hidden and unreachable occurrence graphs
- **WHEN** hidden occurrences, unreachable definitions with nested instances, or multiple differently scaled instances contain SVGs
- **THEN** each occurrence uses its complete applicable transform, unreachable definitions use identity outer transforms, and oversized or excessive graphs fail before exceeding existing budgets

#### Scenario: Preserve rendering lifecycle and failure atomicity
- **WHEN** headless or MCP operations render equivalent corrected scenes through frame, range, materialized draft and export before and after undo, redo and reopen, or encounter invalid evaluated geometry
- **THEN** existing independent pixel, semantic, audio and timing tolerances hold; failures preserve revision, state, retained history and published artifacts; stale edit revisions retain REVISION_CONFLICT precedence and schema 15 and protocol 1 remain unchanged

#### Scenario: Reject diagonal geometry collapsed by float conversion
- **WHEN** a 100x100 SVG with viewBox 0 0 .001 .001 contains a red polygon with points -5000,-5000 5000,5000 5000,5000.0001 -5000,-4999.9999
- **THEN** rendering rejects it with non-retryable INVALID_ARGUMENT before artifact preparation rather than successfully publishing blank output

#### Scenario: Accept equivalent representable diagonal geometry
- **WHEN** the same viewport uses polygon points -.005,-.005 .005,.005 .005,.0051 -.005,-.0049
- **THEN** rendering succeeds with an independently correct red diagonal band, including red pixel 50,55 within existing native encoding tolerances

#### Scenario: Enforce inclusive Euclidean conversion error
- **WHEN** positive or negative mapped points have exact conversion, displacement exactly 0.25, immediately greater displacement, combined X/Y error, non-finite values, or move-only contours
- **THEN** finite displacement at most 0.25 succeeds subject to existing checks, and non-finite or excessive displacement fails even when integer bounds remain representable or the backend would discard the contour

#### Scenario: Measure conversion error in raster-space units
- **WHEN** viewport mapping or complete component sampling scale changes coordinate conversion error
- **THEN** acceptance uses final raster-space displacement rather than source-space displacement, while exactly representable large coordinates within existing backend limits remain accepted

#### Scenario: Preserve public state and artifacts on precision rejection
- **WHEN** headless or MCP rendering or frame, range, materialized draft or export evaluation encounters excessive coordinate conversion error
- **THEN** the operation returns non-retryable INVALID_ARGUMENT without changing revision, authoritative state, retained history, drafts or published artifacts; existing stale-revision precedence, aliases, schema 15 and protocol 1 remain unchanged

### Requirement: Transactional SVG timeline operations
The core add_svg edit and MCP timeline_add_svg MUST accept trackId, startMs, durationMs, svg source string, optional complete transform2d, and optional parent, with projectId/expectedRevision in the existing envelope. New items MUST be type svg, default to identity Transform2D, visible state and zero zIndex on unlocked overlay tracks, and use existing safe integer timing and half-open intervals. The operation MUST return one created item ID through existing edit results. Atomic batches MUST support resultAlias and earlier aliases in trackId and parent.id; later operations MUST resolve the created item alias. One SVG document MUST remain one timeline item and one undoable operation.

Existing move, trim, split, duplicate, delete, visibility, transform, position/scale/opacity keyframes, parenting and stacking operations MUST apply with their existing restrictions. Component-local SVG items and materialized drafts MUST use the same normalized validation. SVG-source and geometry/paint patches, audio edits and transition-endpoint use MUST return INVALID_ARGUMENT. Missing track/parent/item, locked track and stale revisions MUST retain TRACK_NOT_FOUND, ITEM_NOT_FOUND, TRACK_LOCKED and retryable REVISION_CONFLICT. A stale revision MUST fail before SVG semantic processing. Later batch failure MUST preserve current state, retained history, drafts, revision and files, and publish no partial aliases.

#### Scenario: Create and reference SVG atomically
- **WHEN** a batch creates an overlay/group, adds SVG using earlier aliases and edits the SVG through its result alias
- **THEN** it commits once with one history step and the existing deterministic ID/alias result conventions

#### Scenario: Reject failed edits without partial publication
- **WHEN** SVG input is invalid, a reference is missing, a track is locked, an alias is unresolved, a revision is stale or a later batch edit fails
- **THEN** the specified existing typed error applies and no partial authoritative state or output is published

#### Scenario: Retain standard lifecycle behavior
- **WHEN** an SVG is transformed, animated, moved, split, duplicated, hidden, parented, reordered, deleted, undone, redone and reopened, including component and draft workflows
- **THEN** the normalized content and expected visual/lifecycle state remain deterministic under existing operation rules

### Requirement: Canonical persisted SVG and migration
Schema 15 SVG items MUST persist a version-1 normalized document containing width, height and document-ordered reference-free shape records, each with geometry, nullable fill and stroke, plus a viewport mapping derived from the accepted viewBox. Persisted content MUST contain no XML source, resource identifier, font or executable syntax. Current state, hidden/unused component content, durable drafts and retained history MUST undergo the same normalized document validation; caller-supplied persisted values MUST NOT bypass bounds. SVG MUST introduce no managed asset references or files.

Opening supported schemas 1 through 14 MUST migrate current state and every retained undo/redo snapshot to 15 under the project lock in one recoverable transaction. The 14-to-15 step MUST change only the version; older schemas containing SVG MUST fail before relabeling. Unknown future versions and invalid current/history documents MUST fail closed without partial rewrite using existing persistence error contracts. Existing operations, colors, IDs, provenance, history and rendered content MUST remain unchanged. Older binaries MUST reject schema 15; downgrade MUST require restoring a pre-migration backup.

#### Scenario: Migrate legacy current state and history
- **WHEN** supported nonempty legacy state with retained undo and redo is opened
- **THEN** all snapshots migrate atomically with unchanged content and render semantics, including after interrupted transaction recovery

#### Scenario: Reject forged or future persisted content
- **WHEN** normalized SVG exceeds bounds or carries unknown fields/resources, appears in a legacy schema, or any snapshot has an unsupported future version
- **THEN** opening fails through existing validation/persistence errors without publishing a partial migration

### Requirement: Shared evaluated SVG rendering and public evidence
Frame, audiovisual range, materialized draft preview and final export MUST consume identical evaluated SVG semantics for equivalent immutable state and output settings. SVG MUST compose its viewport mapping, document order and local geometry with existing parent/component clocks, Transform2D, inherited opacity, timing and canonical stacking. Raster adapters MUST consume evaluated geometry without reparsing source or inspecting persisted state, and MUST use the established bounded shape raster path. Invalid evaluated content MUST fail before artifact preparation or backend execution. No system fonts or resource-dependent fallback MUST be consulted.

Equivalent normalized plans MUST compare exactly; decoded visual SSIM MUST be at least 0.99, aligned float-PCM RMS error at most 0.0001 and timing deviation at most one output frame. Tests MUST include independent expected pixels/geometry and nonempty synthetic audio. Protocol 1 MUST add svg_items when editing is supported and svg_rendering only when the complete rendering subsystem is ready, preserving existing capability names. Canonical versioned SVG, headless and MCP fixtures and every governed consumer MUST agree on schema, bounds, errors and aliases, with existing clients unchanged.


SVG internal fill, stroke and document-order sibling composition MUST preserve premultiplied linear-light floating-point precision until one final image encoding. Item and inherited opacity MUST apply once after document composition. Implementations MUST account for the full live working set before rasterization, conservatively at 44 bytes per viewport pixel for the selected accumulator, reusable coverage and output buffers, using checked arithmetic and preserving the existing aggregate memory ceiling. Standalone shape output MUST remain unchanged.

#### Scenario: Compare all render intents and lifecycle states
- **WHEN** overlapping SVG artwork in transformed and retimed components is previewed/exported before and after undo/redo/reopen
- **THEN** independent visual/geometry expectations and the shared semantic, pixel, audio and timing tolerances hold

#### Scenario: Advertise only implemented support
- **WHEN** clients inspect capabilities with a ready or unavailable renderer and exercise canonical accepted/rejected requests
- **THEN** editing/rendering capabilities reflect actual readiness, both languages agree on fixtures, and existing calls retain their previous contracts

#### Scenario: Preserve repeated translucent composition
- **WHEN** 500 identical overlapping rectangles use #ff000001, or supported artwork mixes translucent fills, strokes and ordered colors
- **THEN** encoded pixels agree within one channel value with an independent premultiplied linear-light oracle, including alpha 1 - (254/255)^500 for the repeated rectangles, with no per-sibling byte quantization

#### Scenario: Verify corrected rendering through public operations and lifecycle
- **WHEN** canonical corrected SVG requests use standalone operations and alias-bearing batches, then immutable frame/range/draft/export rendering exercises transformed and retimed components, item opacity and undo/redo/reopen
- **THEN** independent expected pixels and existing semantic, audio and timing tolerances hold with schema 15 and protocol 1 unchanged; invalid requests and stale revisions preserve existing typed errors and atomic state/history behavior
