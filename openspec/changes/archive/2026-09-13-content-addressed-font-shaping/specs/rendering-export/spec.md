## ADDED Requirements

### Requirement: Shared pinned glyph rendering
For schema-19 text, frame, range, materialized draft and export MUST consume one canonical shaped glyph sequence and layout profile derived from the effective EvaluatedScene text and pinned font bindings. Downstream rendering MUST NOT re-resolve font selectors, reshape strings through FFmpeg drawtext or reconstruct persisted semantics. Equivalent render requests MUST have exact matching semantic plans, visual SSIM at least 0.99, aligned decoded float-PCM RMS error at most 0.0001 and timing alignment within one output frame. Existing timing, alignment, padding, backgrounds, outlines, shadows, opacity, group/component ancestry, legacy animation and Transform2D precedence MUST apply to shaped bounds. Schema-19 shaping supersedes legacy plain-text and per-character rich-text layout guarantees only; non-text and audio behavior MUST remain unchanged. The major text-layout contract and capability report MUST identify the transition and unsupported requested contract versions MUST fail before mutation with INVALID_ARGUMENT.

#### Scenario: Compare every render intent after reopen
- **WHEN** multilingual styled root/component text with slots, repeaters, animation and transforms renders before and after original font removal and project reopen
- **THEN** glyph plans remain exact, all four render intents meet existing decoded-content tolerances and authoritative state is unchanged

#### Scenario: Publish no artifact for invalid font state
- **WHEN** any effective or retained required binding is damaged, path-unsafe or unsupported, or canonical glyph work exceeds limits
- **THEN** canonical preflight returns the specified integrity/path/invalid-input error before output preparation with no fallback

#### Scenario: Advertise the compatibility transition
- **WHEN** clients inspect capabilities or request an unsupported text-layout contract version
- **THEN** supported schema/layout versions are explicit and unsupported negotiation fails with INVALID_ARGUMENT before mutation

## MODIFIED Requirements

### Requirement: Shared stored rich text evaluation
Frame, range, draft preview and export MUST evaluate stored documents through the same EvaluatedScene semantics. Effective template text slots MUST replace a document with one unstyled run; rich-text slots MUST replace it with their ordered runs, following existing override/default precedence without mutating shared definitions. Run colors and bold/italic MUST retain existing styled-slot behavior and item typography fallback. Timing MUST retain integer-millisecond half-open intervals, run order MUST be textual order, and common transform/anchor/stacking/opacity semantics MUST remain unchanged. Semantically plain documents MUST preserve legacy rendering and default-font behavior through schema 18. Schema 19 MUST apply the explicit versioned pinned-font shaping transition to plain and styled text alike; its one-time layout differences supersede earlier text pixel-preservation guarantees only. Missing styled font dependencies MUST retain DEPENDENCY_UNAVAILABLE and resource/path safety MUST remain unchanged. Preview/export MUST satisfy existing exact semantic-plan and decoded-content tolerances with fixed resources; schema 19 MUST additionally satisfy the font-resolution and Shared pinned glyph rendering requirements.

#### Scenario: Preserve legacy rendered output
- **WHEN** representative plain-text items with Unicode, wrapping, alignment, backgrounds, shadows, outlines, transforms and animation are rendered before and after migration through schema 18 with identical resources
- **THEN** legacy output and evaluated semantics remain equivalent and every render entry point satisfies existing visual/audio/timing tolerances

#### Scenario: Render stored styling and independent slot overrides
- **WHEN** root text and repeated component instances use multirun stored documents with independent text/rich-text slot defaults and overrides
- **THEN** each occurrence renders its effective document deterministically without altering stored base runs and preview/draft/export agree

#### Scenario: Fail safely on unavailable styled fonts
- **WHEN** effective styled text requires an unavailable font face or violates existing resource confinement
- **THEN** preparation returns the existing dependency/path error before artifact publication and never silently drops styling

### Requirement: Styled root text retains legacy animation
Ungrouped styled root text MUST retain identity ancestry throughout evaluation, measurement and rendering so position, scale and opacity keyframes follow existing timing/easing semantics. All render intents MUST match independently calculated static styled-text snapshots at equivalent timestamps within existing tolerances. Plain text MUST retain its legacy path through schema 18 and use canonical shaped glyphs in schema 19. Real parent ancestry MUST remain intact and Transform2D MUST retain precedence; font activation MUST NOT freeze legacy animation.

#### Scenario: Animate each supported legacy property
- **WHEN** ungrouped styled text has position, scale or opacity keyframes sampled at 0, 400 and 800 milliseconds
- **THEN** its displacement, visible bounds or brightness changes as expected and output matches static snapshots with independently calculated values

#### Scenario: Preserve ancestry and transform compatibility
- **WHEN** styled text has a non-default anchor, a real group/component parent, or an explicit Transform2D, or text is semantically plain
- **THEN** existing anchor, parent composition, Transform2D precedence and version-appropriate plain rendering semantics remain unchanged and evaluator ancestry is preserved through finalization

#### Scenario: Agree across render intents without freezing
- **WHEN** the animated project renders through frame, range, materialized draft and export at the same selections
- **THEN** outputs satisfy existing decoded-content tolerances, exhibit the independently expected animation and leave authoritative state unchanged
