# visual-effects Specification

## Purpose

Bounded authored leaf effects and deterministic local composition for visual animation.

## Requirements

### Requirement: Bounded authored effect stacks
Core MUST accept an optional `effects` array on visual media, text, solid-color, rectangle, shape, SVG, and grid items through existing visual-property edits. Omission MUST mean an empty stack. Each stack MUST contain at most 16 closed tagged records with unique nonempty IDs of at most 128 UTF-8 bytes. Supported records MUST be `gaussian_blur` with `radiusPx`, `glow` with `radiusPx`, `intensity`, and typed RGBA `color`, `color_tint` with typed RGBA `color`, and `vignette` with `amount`. Radii MUST be finite in [0,128], intensity in [0,1], and amount/color components in [0,1]. Other effect kinds and item kinds MUST return non-retryable `INVALID_ARGUMENT`; missing references and stale revisions MUST preserve existing stable errors. Effects MUST introduce no paths, URLs, SVG, scripts, or resource references.

#### Scenario: Edit bounded effects transactionally
- **WHEN** an eligible item receives a valid ordered stack through standalone or alias-aware batch edits
- **THEN** core preserves IDs and order in one atomic undoable revision and undo/redo/reopen reproduce the stack

#### Scenario: Reject invalid stacks and references
- **WHEN** an edit has duplicate IDs, an excessive stack, unsupported target/type, unknown field, out-of-bound value, missing item, or stale revision
- **THEN** core returns its stable typed error and leaves state, history, revision, and resources unchanged

### Requirement: Canonical effect composition
Effects MUST operate in declared order on the item's local premultiplied linear-light raster after crop/clip and graphic fill/stroke, before local and ancestor affine transforms. Gaussian blur MUST use separable normalized Gaussian kernels, sigma equal to radiusPx, support ceil(3*sigma), transparent extension, and radius zero as identity. Glow MUST blur the input alpha with that kernel, multiply it by intensity and color alpha, colorize with the declared color, and composite the original over the halo. Tint MUST mix straight linear RGB toward the declared color using color alpha as the mixing factor while preserving source alpha. Vignette MUST multiply RGB by `1 - amount * clamp((u*u + v*v)/2,0,1)`, where u and v range from -1 to 1 across the unstroked local bounds, preserving alpha. The unstroked anchor MUST remain fixed when blur/glow expand raster support. Effect work MUST be bounded to 16 effects per item, 4096 expanded visual occurrences, and 268435456 cumulative pixel-pass units per scene sample (each horizontal/vertical kernel tap counts as one pass); existing stricter surface limits MUST also apply. Excess MUST fail with `INVALID_ARGUMENT` before output or persistence publication, without clipping the effect to hide excess work.

#### Scenario: Preserve ordered effects and anchors
- **WHEN** asymmetric content uses two differently ordered effects and a noncentral rotated anchor
- **THEN** the outputs reflect declared order and expanded support without moving the authored anchor

#### Scenario: Fail bounded work without side effects
- **WHEN** hidden, retained, or expanded content exceeds an effect or existing raster budget
- **THEN** core rejects it before allocating beyond the limit or publishing state/artifacts
