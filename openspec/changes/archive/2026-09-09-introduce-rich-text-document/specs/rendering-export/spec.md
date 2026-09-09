## ADDED Requirements

### Requirement: Shared stored rich text evaluation
Frame, range, draft preview and export MUST evaluate stored documents through the same EvaluatedScene semantics. Effective template text slots MUST replace a document with one unstyled run; rich-text slots MUST replace it with their ordered runs, following existing override/default precedence without mutating shared definitions. Run colors and bold/italic MUST retain existing styled-slot behavior and item typography fallback. Timing MUST retain integer-millisecond half-open intervals, run order MUST be textual order, and common transform/anchor/stacking/opacity semantics MUST remain unchanged. Semantically plain documents MUST preserve legacy rendering and default-font behavior, including after migration. Missing styled font dependencies MUST retain DEPENDENCY_UNAVAILABLE and resource/path safety MUST remain unchanged. Preview/export MUST satisfy existing exact semantic-plan and decoded-content tolerances with fixed resources; no new font hashing/shaping guarantee is introduced.

#### Scenario: Preserve legacy rendered output
- **WHEN** representative plain-text items with Unicode, wrapping, alignment, backgrounds, shadows, outlines, transforms and animation are rendered before and after migration with identical resources
- **THEN** legacy output and evaluated semantics remain equivalent and every render entry point satisfies existing visual/audio/timing tolerances

#### Scenario: Render stored styling and independent slot overrides
- **WHEN** root text and repeated component instances use multirun stored documents with independent text/rich-text slot defaults and overrides
- **THEN** each occurrence renders its effective document deterministically without altering stored base runs and preview/draft/export agree

#### Scenario: Fail safely on unavailable styled fonts
- **WHEN** effective styled text requires an unavailable font face or violates existing resource confinement
- **THEN** preparation returns the existing dependency/path error before artifact publication and never silently drops styling
