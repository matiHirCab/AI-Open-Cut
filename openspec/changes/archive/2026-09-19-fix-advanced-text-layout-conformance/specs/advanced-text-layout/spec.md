## ADDED Requirements

### Requirement: Stable logical layout box placement

Advanced text layout SHALL preserve its fractional logical padding-box dimensions and local origin independently of integer raster dimensions, glyph overhang and paint margins. Anchors MUST refer to the logical box. Raster offsets MUST be composed before item and ancestor transforms for static and animated rendering, including component and repeater occurrences. Visible ink outside the logical box MUST remain visible. Layout-absent geometry MUST remain unchanged.

#### Scenario: G1 Negative shadow does not move the box
- **WHEN** a 100 by 70 logical box at (50,40) receives a shadow with horizontal offset -20 pixels
- **THEN** its background remains at (50,40) with the same logical dimensions while the shadow extends outside it
- **AND** strokes and glyph overhang likewise expand the raster without moving the box

#### Scenario: G2 Fractional anchored transformed occurrences
- **WHEN** fractional bounds are rendered with each supported anchor, rotation, scaling, animation, and component or repeater parent transforms
- **THEN** the logical box follows the composed transforms independently of effect margins and integer raster rounding
- **AND** overflowing ink remains visible and layout-absent placement matches legacy expectations exactly

### Requirement: Canonical fractional layout width

Advanced layout SHALL use the same logical-order cluster-width measurement for wrap comparisons, accepted line widths, diagnostics and fitting. Tracking SHALL be added before each subsequent cluster advance, with no trailing tracking; reverting to a word-break opportunity MUST restore its corresponding width. Bidi reordering SHALL control glyph placement without changing the canonical measured width. Comparisons MUST remain exact without an epsilon, and layout-absent arithmetic MUST remain unchanged.

#### Scenario: W1 Reported dimensions round trip into bounds
- **WHEN** MMMMM at font size 30 and tracking 0.1 is measured and its reported dimensions are reused as bounds
- **THEN** it remains one line and shrink retains size 30

#### Scenario: W2 Wrapping and exact boundary agreement
- **WHEN** word and cluster wrapping measure text including bidi runs and mixed faces at exact reported widths and narrowly smaller bounds
- **THEN** wrap decisions and diagnostics use the same canonical widths, including after word-break backtracking
- **AND** fitting selects the largest admissible integer using the existing fit-mode semantics and retains all content when size one overflows

### Requirement: Retained draft layout integrity before publication

Retained drafts SHALL receive canonical text-style validation on schema-20 migration and schema-21 reopen, including text creation, text-style updates and text within component creation or update payloads. Invalid layouts MUST return INVALID_ARGUMENT before authoritative migration or font publication and leave project, retained history, drafts and font files unchanged. Validation MUST preserve existing document and paint validation without replaying stale drafts or changing revision-conflict behavior.

#### Scenario: D1 Invalid retained standalone styles
- **WHEN** a retained draft contains negative tracking or invalid bounds and padding in a text creation or update operation during schema-20 migration or schema-21 reopen
- **THEN** opening fails with INVALID_ARGUMENT and project, history, draft and font bytes remain unchanged

#### Scenario: D2 Nested component styles receive identical validation
- **WHEN** invalid layouts occur on text items inside retained component creation or update tracks
- **THEN** opening fails with INVALID_ARGUMENT before authoritative publication with the same byte-preservation guarantees

#### Scenario: D3 Valid and stale drafts remain compatible
- **WHEN** valid retained drafts include legacy text, advanced text or stale base revisions
- **THEN** migration and reopen preserve their established compatibility without replaying the operations
- **AND** subsequent mutations retain the existing revision-conflict checks and invalid document or paint payloads still fail validation

### Requirement: Inclusive shared candidate glyph accounting

Expanded-scene layout preflight SHALL share the existing inclusive ceiling of 16,777,216 candidate glyphs across all evaluated occurrences. Exactly the ceiling SHALL be permitted by this budget and exceeding it MUST fail through the existing typed work-limit error. All existing per-text candidate, per-candidate glyph and line limits SHALL remain unchanged. Newline-only candidates SHALL retain their existing glyph-accounting semantics; no new line-work rejection rule or runtime budget override SHALL be introduced.

#### Scenario: B1 Actual ceiling boundaries
- **WHEN** cumulative candidate glyph accounting reaches exactly 16,777,216 and then attempts to add one further glyph
- **THEN** the equality is accepted and the additional glyph is rejected without overflow in the accounting itself

#### Scenario: B2 Expanded occurrences share preflight accounting
- **WHEN** separately valid component or repeater occurrences collectively exceed the shared candidate-glyph budget
- **THEN** preflight rejects the expanded scene rather than resetting the budget for each occurrence

#### Scenario: B3 Newline-only accounting remains compatible
- **WHEN** newline-only text is considered at multiple candidate sizes within existing per-candidate line limits
- **THEN** candidate glyph usage reflects the existing shaped glyph count without introducing charges for empty lines
- **AND** the existing candidate-count and line limits still apply
