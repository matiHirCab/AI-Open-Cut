## ADDED Requirements

### Requirement: Closed opt-in text layout
Schema-21 TextStyle MUST optionally accept a closed `layout` object with optional `trackingPx` (default 0, finite [0,1000]), `lineHeightPx` (finite [1,4320]), `bounds` (closed object with optional finite `widthPx` in [1,7680] and `heightPx` in [1,4320], at least one required), `wrap` (`none|word|cluster`, default `word`), `verticalAlignment` (`top|center|bottom`, default `top`), `fit` (`none|shrink|fit_width|fit_box`, default `none`) and `backgroundCornerRadiusPx` (finite [0,2160], default 0). Fractional pixel values MUST be supported. Explicit null and unknown fields/enums MUST return non-retryable INVALID_ARGUMENT. Absent layout MUST preserve the exact legacy behavior. With layout present, legacy wrapWidthPx MUST supply width only when bounds.widthPx is absent; lineSpacingPx MUST apply only when lineHeightPx is absent. Existing background color, opacity, padding and horizontal alignment MUST remain effective. Non-top vertical alignment MUST require a height. No new field MUST accept expressions, paths, markup or network resources. Core MUST validate root, hidden, unused component and draft text.

#### Scenario: Opt in and inherit existing styles
- **WHEN** legacy text omits layout or new text supplies a partial layout object
- **THEN** the former retains exact legacy behavior and the latter uses documented defaults and fallback fields with existing background and paint styles

#### Scenario: Validate all boundaries and locations
- **WHEN** any text location supplies exact numeric limits, fractions, exceeded limits, null, non-finite numbers, unknown fields or unsupported enums
- **THEN** valid inputs succeed and invalid inputs fail atomically with INVALID_ARGUMENT

### Requirement: Deterministic cluster-safe box layout
Core MUST use pinned faces and the stored shaping profile, preserving Unicode, bidi context, mandatory separators, run/span precedence and glyph clusters. Layout coordinates MUST be text-local project pixels with x right and y down before item/group transforms. Bounds MUST denote the outer padding box; available content dimensions MUST subtract padding and MUST be positive. Tracking MUST add fixed pixel spacing between adjacent visual shaping clusters on each line, never inside a cluster or after the last cluster. `none` wrapping MUST honor only mandatory breaks; `cluster` MUST break at complete shaping clusters; `word` MUST choose the last pinned Unicode line-break opportunity that fits, falling back to cluster breaks for an oversized word. A cluster wider than the content width MUST remain intact and report overflow. Absent width MUST disable soft wrapping. Each line's metric height MUST use the maximum selected-face ascent minus descent plus line gap; explicit lineHeightPx MUST set the baseline step and line box height, otherwise the legacy metric-plus-spacing rule MUST apply. The first baseline MUST use the maximum ascent; N lines MUST occupy N line boxes, including mandatory empty lines. Horizontal alignment MUST position lines within content width, and vertical alignment MUST position the line-box block within content height. Unbounded axes MUST use intrinsic line-box extents. Alignment offsets MUST be zero on an overflowing axis. Background MUST cover the padding box behind all glyph layers with its corner radius clamped to half the smaller box dimension. Background rounding MUST NOT clip glyph ink. Fit and overflow MUST use line advances/line boxes, excluding glyph ink overhang, strokes and shadows; raster allocation MUST include all ink/effect extents. Content MUST NOT be truncated or clipped.

#### Scenario: Track and wrap multilingual clusters
- **WHEN** fixtures contain ligatures, combining marks, emoji, mixed bidi runs, oversized words/clusters and every supported mandatory separator in each wrap mode
- **THEN** pinned cluster boundaries and line breaks follow the specified mode and tracking never splits a cluster or creates trailing spacing

#### Scenario: Position and paint a bounded text block
- **WHEN** text uses fractional bounds, asymmetric padding, each horizontal/vertical alignment, explicit line height, rounded backgrounds and overflowing ink effects
- **THEN** line boxes determine alignment and overflow, the background covers the padding box, and all glyph/effect ink remains visible with existing transforms and paint ordering

#### Scenario: Reject unusable padding boxes
- **WHEN** padding consumes an entire bounded axis or vertical alignment requires an absent height
- **THEN** INVALID_ARGUMENT leaves state and history unchanged

### Requirement: Bounded deterministic font fitting
`none` MUST use the authored fontSize. `shrink` MUST require at least one effective bound and choose the largest fitting integer size in [1,authored fontSize] against all present bounds. `fit_width` MUST require an effective width and choose the largest fitting integer size in [1,1000] against width only. `fit_box` MUST require both bounds and choose the largest fitting integer size in [1,1000] against both. Each candidate MUST be fully shaped and wrapped before measurement; tracking, explicit line height, padding and effects MUST remain fixed pixels. Fit MUST use inclusive comparisons with no platform-dependent epsilon. If no candidate fits, core MUST use size 1, retain all content and report overflow. Height overflow in fit_width MUST still be reported. Evaluation MUST perform at most 1000 candidates per effective text, at most 16384 glyphs and 4096 lines per candidate and at most 16777216 cumulative candidate glyphs per evaluated scene. Existing expanded-scene and raster limits MUST remain. Excess work, non-finite geometry or invalid required bounds MUST fail with INVALID_ARGUMENT before destination inspection or artifact allocation. Fitting MUST NOT mutate authored fontSize, font ownership, revision or history.

#### Scenario: Resolve each fit mode and exact ties
- **WHEN** independently measured fixtures exercise shrinking, enlargement, wrapping transitions, exact bound equality and each mode
- **THEN** the selected integer size is the largest admissible fit and authored state remains unchanged

#### Scenario: Report unavoidable overflow
- **WHEN** fixed tracking, line height, padding or a wide cluster prevents even size 1 from fitting
- **THEN** evaluation uses size 1 with overflow and preserves the complete text

#### Scenario: Bound fitting work and reject missing bounds
- **WHEN** fitting reaches or exceeds candidate/glyph/line/scene limits or lacks mode-required bounds
- **THEN** inclusive limits succeed and invalid requests fail before output side effects

### Requirement: Compatible reversible layout editing and migration
Existing add_text/update_item style inputs MUST expose layout in standalone, timeline_batch_edit alias and durable draft workflows with existing style replacement semantics. Omitted style MUST retain layout; replacing style without layout MUST reset to legacy layout. Layout on a non-text target MUST fail with INVALID_ARGUMENT. Copies, splits, moves, trims, components, repeaters, text/rich-text slot substitutions, undo/redo and reopen MUST preserve applicable style and pinned fonts; substitutions MUST recompute fit from effective content. Missing references and stale revisions MUST retain existing stable errors and precedence, including ITEM_NOT_FOUND and REVISION_CONFLICT. Failed batches MUST roll back all edits and font ownership, and success MUST commit one revision/undo step. Draft preview MUST leave authoritative state/history unchanged. Migration to schema 21 MUST transform current state and all retained undo/redo snapshots under the project lock atomically, preserving provenance, managed media/fonts and omitted layout. Unknown future schema versions MUST fail closed. Canonical style, schema-version, headless, MCP and capability fixtures and governed consumers MUST agree and advertise advanced layout support additively without changing existing operation names or error retryability.

#### Scenario: Edit through aliases and lifecycle operations
- **WHEN** a batch creates text under an alias and updates layout, then text undergoes lifecycle operations, substitutions, draft preview, undo/redo and reopen
- **THEN** successful edits retain exact applicable layout/fonts, substitutions recompute fit and authoritative mutations follow existing one-revision semantics

#### Scenario: Reject stale missing and incompatible edits
- **WHEN** standalone, batch or draft edits target a missing item, non-text item, locked track, stale revision or fail after an earlier valid edit
- **THEN** existing typed errors and precedence apply with no partial state/history/font ownership changes

#### Scenario: Migrate history atomically and preserve legacy pixels
- **WHEN** a schema-20 project with retained history, hidden/component text and drafts is opened, including injected migration write failures
- **THEN** success preserves legacy glyph plans and decoded lossless pixels through undo/redo/reopen, and failures expose no partially migrated authoritative snapshots

#### Scenario: Keep public contracts compatible
- **WHEN** legacy and layout-enabled requests traverse native headless and MCP clients, or a future persisted version is opened
- **THEN** old requests remain valid, new capability/schema fixtures match every governed consumer and unsupported persisted versions are rejected
