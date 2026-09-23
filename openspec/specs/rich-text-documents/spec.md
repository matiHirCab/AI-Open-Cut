# rich-text-documents Specification

## Purpose

Define canonical stored rich-text documents and compatible, reversible editing behavior.

## Requirements

### Requirement: Canonical bounded text documents
Core MUST represent each schema-18 text item with a required closed RichTextDocument `{runs:[{text,bold?,italic?,color?}]}` and retain `text` equal to ordered run concatenation. Documents MUST have 1-256 runs and concatenated text of 1-4096 UTF-8 bytes, preserve Unicode/whitespace/newlines without normalization, and obey existing rich-text run type/color and text-item style/project bounds. Optional run flags MUST be Boolean and colors six-digit hex; null, unknown fields, malformed Unicode and mismatched projections MUST fail with non-retryable INVALID_ARGUMENT. Existing slot-value limits MUST remain unchanged. Core MUST validate hidden and unused component content. Run boundaries MUST NOT imply line breaks or expose string offsets; absent overrides inherit existing item defaults. Documents MUST NOT introduce paths, links, executable markup or expressions; text content MUST remain literal.

#### Scenario: Preserve ordered Unicode runs
- **WHEN** a valid document contains whitespace, newlines, supplementary characters, combining marks and distinct run styles
- **THEN** storage preserves each run exactly and text is their exact concatenation

#### Scenario: Reject malformed and excessive documents
- **WHEN** root or hidden/unused component text contains zero/257 runs, empty/4097-byte aggregate text, null or unknown fields, malformed Unicode, invalid styles, non-finite item values or a mismatched text projection
- **THEN** core rejects the candidate with INVALID_ARGUMENT without mutation

#### Scenario: Accept inclusive limits and literal text
- **WHEN** a document has 256 runs and exactly 4096 UTF-8 bytes including markup-like literal text
- **THEN** it is accepted when other existing domain constraints hold and no markup is executed or external resource is resolved

### Requirement: Compatible reversible document edits
Existing add_text and update_item operations MUST accept document inputs in standalone, batch and durable draft workflows. Creation MUST accept exactly one of text/document; an update MUST accept at most one. Simple text MUST produce one unstyled run and document input MUST recompute the compatibility text. Simultaneous fields or explicit null MUST fail with INVALID_ARGUMENT. An update omitting both MUST preserve content; replacing text MUST clear run overrides while retaining other item style defaults. Non-text document targets MUST fail with INVALID_ARGUMENT. Missing references, locked/incompatible tracks and stale revisions MUST retain existing stable errors, including ITEM_NOT_FOUND and REVISION_CONFLICT, with no state/history changes. Batches MUST preserve aliases, one-revision commit and rollback. Split, duplicate, component copies, move and trim MUST preserve document content under existing timing/identity rules. Undo/redo and reopen MUST preserve exact documents; draft previews MUST NOT commit changes.

#### Scenario: Use legacy and document inputs
- **WHEN** a caller creates plain text, replaces it with multiple styled runs, edits only item style, then replaces simple text
- **THEN** projections remain synchronized, style-only edits retain runs, and the final document contains exactly one unstyled run

#### Scenario: Reject conflicting or invalid targets
- **WHEN** a caller supplies both fields, null, a document on a non-text item, missing IDs, a locked/incompatible track or a stale revision
- **THEN** the established typed failure leaves project revision and history unchanged

#### Scenario: Resolve aliases and roll back a batch
- **WHEN** a batch creates document text with an alias and updates that alias before either completing or encountering an invalid later operation
- **THEN** success commits one revision/undo step with the alias mapping and failure commits none of the operations

#### Scenario: Preserve documents through lifecycle operations
- **WHEN** document text is copied into a component, split, duplicated, moved, trimmed, undone/redone and reopened, or edited in a retained draft preview
- **THEN** each resulting item retains its exact applicable runs with existing identity/timing/history semantics and draft preview leaves authoritative state unchanged

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

### Requirement: Additive revisioned font-size editing
The existing `update_item` operation MUST accept optional integer `fontSize` in [1,1000] on text items in standalone, aliased batch and durable draft workflows. Omission MUST preserve size; a supplied value MUST change only the text item's stored font size and retain its ID, document/runs/spans, color, style, font binding, transform, timing and other authored fields. Core MUST reject null, fractions, values outside the range and non-text targets with non-retryable INVALID_ARGUMENT; missing items, locked tracks and stale revisions MUST retain their existing typed errors and rollback behavior. Successful edits MUST remain one revision/undo step, with exact undo/redo/reopen behavior. Old requests MUST remain valid. Canonical headless/MCP catalogs, Rust/TypeScript consumers, capability reporting and parity evidence MUST agree with the additive field; no persisted schema migration is required.

#### Scenario: Resize without replacing text
- **WHEN** a standalone or aliased batch edit updates `fontSize` on an existing styled text item
- **THEN** its ID, document, font binding and unrelated fields remain unchanged while evaluation uses the new size, including after undo, redo and reopen

#### Scenario: Reject invalid size and targets atomically
- **WHEN** `fontSize` is null, fractional, zero, greater than 1000, targets a non-text or missing item, or follows an earlier valid operation in a batch that later fails
- **THEN** structural or core validation returns the established typed error and project revision, history and authoritative files remain unchanged

#### Scenario: Preserve compatibility and stale-revision behavior
- **WHEN** an old update request omits `fontSize`, a durable draft previews size changes, or an edit uses a stale revision
- **THEN** old behavior remains valid, draft preview leaves authoritative state unchanged, and stale edits return retryable REVISION_CONFLICT without mutation
