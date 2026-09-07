## ADDED Requirements

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

#### Scenario: Preserve viewport and drawing semantics
- **WHEN** valid artwork uses nonzero viewBox origin, unequal viewport aspect ratio, open curves, alpha colors and overlapping siblings
- **THEN** independently checked geometry and pixels preserve centered meet scaling, transparent margins, clipping, fill/stroke semantics and document order

#### Scenario: Enforce every boundary before expensive work
- **WHEN** valid fixtures hit each inclusive budget or counterexamples exceed one budget, use non-finite numbers or invalid grammar
- **THEN** valid boundaries succeed and counterexamples return INVALID_ARGUMENT before excess allocation, rasterization or writes

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

#### Scenario: Compare all render intents and lifecycle states
- **WHEN** overlapping SVG artwork in transformed and retimed components is previewed/exported before and after undo/redo/reopen
- **THEN** independent visual/geometry expectations and the shared semantic, pixel, audio and timing tolerances hold

#### Scenario: Advertise only implemented support
- **WHEN** clients inspect capabilities with a ready or unavailable renderer and exercise canonical accepted/rejected requests
- **THEN** editing/rendering capabilities reflect actual readiness, both languages agree on fixtures, and existing calls retain their previous contracts
