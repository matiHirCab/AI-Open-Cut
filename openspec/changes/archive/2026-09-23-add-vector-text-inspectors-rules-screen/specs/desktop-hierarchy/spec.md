## ADDED Requirements

### Requirement: Structured vector and text inspection
Desktop MUST show selected shape geometry/paint/stroke, procedural-grid descriptor and text document/style/layout with the existing scoped identity. Root controls MUST expose rectangle/rounded-rectangle/ellipse dimensions, rounded radii, solid fill color, stroke color/width; grid spacing/angle/line color/width; and text content, font size/base color, selected run bold/italic/color, tracking, line height, bounds/fit and existing stroke/shadow layer fields. Font identity and unsupported geometry/paint fields MUST remain visible read-only. Component-local occurrences MUST remain entirely read-only. Controls MUST label units according to existing local-pixel, degree and text-layout contracts; they MUST NOT expose raw SVG, executable expressions, paths or network resources.

#### Scenario: I1 Inspect supported and read-only content
- **WHEN** selection moves among root shapes, grids, text and repeated component occurrences
- **THEN** values reflect the selected authoritative item and scope, only applicable root controls are editable, and unsupported values are preserved

### Requirement: Transactional vector and text inspector edits
Desktop MUST submit existing typed core operations with the displayed expected revision. Apply MUST preserve fields not edited, including geometry variant, gradients, font bindings, style layers, layout and document runs; explicit plain-content replacement MUST follow existing simple-text replacement semantics. Parse failures MUST not call core mutation; core MUST own all semantic validation and complexity limits. Success MUST reload the authoritative snapshot and reset the draft. Selection, refresh and history changes MUST discard stale drafts. Failures MUST preserve project/history and show parsing feedback or unchanged core code/message/retryability without automatic retry or speculative publication. Reset MUST restore displayed authoritative values without mutation.

#### Scenario: I2 Apply and preserve unrelated fields
- **WHEN** users edit a supported vector/grid property or text style/run property and apply
- **THEN** one core transaction changes only the intended fields, preserves other authored values and reloads the committed revision

#### Scenario: I3 Reject malformed invalid or stale edits
- **WHEN** parsing fails, core rejects non-finite/out-of-range values, the item is missing or locked, or the displayed revision is stale
- **THEN** no partial edit commits, the applicable failure remains visible and conflicts offer explicit refresh

#### Scenario: I4 Restore history and discard stale drafts
- **WHEN** an inspector edit is undone, redone or reopened, or the user changes selection, refreshes or resets a draft
- **THEN** displayed values match the applicable authoritative state and stale local input is never submitted against a different item or revision
