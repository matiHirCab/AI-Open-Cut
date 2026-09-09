## ADDED Requirements

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
