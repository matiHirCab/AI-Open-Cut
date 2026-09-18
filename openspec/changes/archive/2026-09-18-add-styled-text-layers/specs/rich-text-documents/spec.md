## ADDED Requirements

### Requirement: Grapheme indexed style spans
Schema-20 RichTextDocument MUST retain its existing runs and optionally accept `spans`, a closed array of at most 256 `{start,end,style}` records. Offsets MUST be zero-based Unicode 16.0 extended grapheme indices over concatenated, unnormalized runs, using UAX #29 default extended grapheme boundaries across run boundaries. Ranges MUST be half-open, nonempty, sorted by start, nonoverlapping and within the document's grapheme count. Adjacent ranges and an empty spans array MUST be valid. Style MUST be a nonempty closed object containing optional Boolean `bold`, Boolean `italic`, six-digit hex `color` and `paintLayers`. Explicit null, fractional/negative/unsafe offsets, unknown fields and invalid ranges MUST fail with non-retryable INVALID_ARGUMENT before mutation. Existing run/content limits and existing rich-text slot limits MUST remain unchanged; all root, hidden, unused and component documents MUST be validated. Span overrides MUST take precedence over run overrides, then item defaults. Paint-only changes MUST NOT divide shaping context; bold/italic changes MUST use already-bound faces. A shaping cluster crossing paint boundaries MUST use the effective paint at its lowest logical source offset, without splitting glyphs or changing advances. Existing documents without spans MUST preserve existing behavior even when runs split a grapheme.

#### Scenario: Address complete Unicode graphemes
- **WHEN** a span selects combining marks, supplementary emoji, a flag, a ZWJ sequence or CRLF, including a grapheme split across runs
- **THEN** each extended grapheme counts once, complete selected graphemes receive the override, and stored text is unchanged

#### Scenario: Reject invalid span boundaries and structure
- **WHEN** spans overlap, are unsorted, empty, out of bounds, exceed 256 entries or contain malformed offsets/styles in any document location
- **THEN** INVALID_ARGUMENT preserves state, revision and history

#### Scenario: Preserve shaping and explicit false overrides
- **WHEN** paint changes cross a ligature or a span explicitly disables run bold/italic
- **THEN** paint follows the lowest logical cluster offset without changing glyph placement and explicit false selects the bound non-bold/non-italic face

### Requirement: Bounded ordered text paint stacks
TextStyle and span style MUST optionally accept `paintLayers`, an ordered array of zero to 16 closed tagged records: fill `{kind:"fill",color,opacity}`, stroke `{kind:"stroke",color,opacity,widthPx}` or shadow `{kind:"shadow",color,opacity,offsetXPx,offsetYPx,blurSigmaPx}`. Colors MUST use #RRGGBB; all numbers MUST be finite; opacity MUST be in [0,1], stroke width in (0,200], offsets in [-4096,4096] and blur sigma in [0,64], inclusively except zero stroke width. Fractional dimensions MUST be supported. Invalid layers MUST fail with INVALID_ARGUMENT. A span stack MUST replace the item stack; omitted span stacks MUST inherit it. An explicit empty stack MUST paint no glyph ink but preserve layout/background. An omitted item stack MUST retain the existing shadow/outline/fill rendering path exactly, with effective run/span colors for fill. Explicit stacks MUST replace legacy glyph paints without suppressing background, padding, alignment, opacity or transforms. They MUST paint first-to-last using source-over, with each layer covering all applicable glyphs before the next layer. Mixed effective stacks MUST paint contiguous logical style segments in ascending source order, with each segment's stack in array order. No layer MUST accept paths, expressions, markup or network resources.

#### Scenario: Compose multiple ordered layers
- **WHEN** a text item uses two shadows, two differently sized strokes and a fill, and one span replaces its stack
- **THEN** array order and segment order determine compositing, with no implicit reordering by layer kind

#### Scenario: Distinguish inheritance from empty paint
- **WHEN** item/span stacks are omitted, explicitly empty or explicitly supplied alongside legacy outline/shadow settings
- **THEN** omission inherits, empty stacks hide glyph ink, supplied stacks replace legacy paints, and background/layout remain effective

#### Scenario: Enforce inclusive paint limits
- **WHEN** each numeric or collection limit is met exactly or exceeded, or inputs contain NaN/infinity, null, invalid colors, resource references or unknown fields
- **THEN** valid boundaries succeed and invalid inputs fail atomically with INVALID_ARGUMENT

### Requirement: Reversible indexed text editing
Existing add_text and update_item operations MUST accept spans and layers through their document/style inputs as standalone operations, timeline_batch_edit operations with aliases, and durable draft edits. Document replacement MUST replace its spans; simple-text replacement MUST clear spans and run overrides while retaining item style. Style-only edits MUST preserve the document. Text and rich-text slot substitutions MUST replace the document and its spans using existing precedence; spans MUST be validated against the substituted content. Copy, split, duplicate, component/repeater evaluation, trim, move, undo/redo and reopen MUST preserve applicable exact document/style data and pinned fonts. Existing revision checks, missing-reference errors, locks, atomic rollback and one-revision batch semantics MUST remain unchanged. Failed operations and draft previews MUST NOT alter authoritative state/history or publish new font ownership.

#### Scenario: Create and update through an alias
- **WHEN** a batch adds styled text under an alias then updates that alias before succeeding or failing on a later operation
- **THEN** success commits one revision/undo step and failure commits nothing

#### Scenario: Preserve lifecycle and replacement semantics
- **WHEN** styled text is copied, split, moved, trimmed, instantiated, overridden through a slot, undone/redone, edited in a draft and reopened
- **THEN** exact applicable styles and font bindings survive, replacements have no stale spans, and preview changes no authoritative state

#### Scenario: Fail on stale revision or missing target
- **WHEN** standalone, batch or draft commit targets a missing item or supplies a stale revision
- **THEN** ITEM_NOT_FOUND or REVISION_CONFLICT retains existing precedence and retryability without side effects
