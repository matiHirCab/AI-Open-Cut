# visual-effects Specification

## Purpose

Bounded authored leaf effects and deterministic local composition for visual animation.

## Requirements

### Requirement: Bounded authored effect stacks
Core MUST accept an optional `effects` array on visual media, text, solid-color, rectangle, shape, SVG, and grid items through existing visual-property edits. Omission in stored model data MUST mean an empty stack; existing edit omission MUST preserve the current stack, explicit [] MUST clear it and null MUST reject. Each stack MUST contain at most16 closed tagged records with unique nonempty IDs of at most128 UTF-8 bytes. Supported records MUST remain gaussian_blur with radiusPx, glow with radiusPx/intensity/typed RGBA color, color_tint with typed RGBA color, vignette with amount, and color_adjustment with required exposureStops/contrast/saturation. Radii MUST be finite[0,128], intensity/amount/color components[0,1]; color-adjustment exposureStops MUST be finite[-8,8], contrast/saturation finite[0,2]. Other effect/item kinds MUST return nonretryable INVALID_ARGUMENT; missing references and stale revisions MUST preserve existing stable errors. Effects MUST introduce no paths/URLs/SVG/scripts/resource references. Existing standalone and alias batch array order, stable effect ID targets and public undo/redo/reopen behavior MUST have automated conformance evidence; the fifth tagged variant is additive and requires schema36, while existing four-kind behavior remains unchanged.

#### Scenario: Edit bounded effects transactionally
- **WHEN** an eligible item receives valid ordered stacks through standalone or alias-aware batch edits
- **THEN** IDs/order persist in one atomic undoable revision and actual native pixels before/reorder/undo/redo/reopen reproduce independently expected generations through public APIs/protocols

#### Scenario: Reject invalid stacks and references
- **WHEN** edits have duplicate IDs, excessive stack, unsupported target/type, unknown field, null/out-of-bound value, missing item or stale revision, including a later batch/draft operation
- **THEN** existing typed errors retain meaning and complete current/history/draft/revision/resource inventories remain unchanged

#### Scenario: Preserve omission clear and stable target identity
- **WHEN** existing effects are omitted, cleared with[], reversed with identical IDs or targeted by root/component effect channels
- **THEN** omission/clear retain existing meanings, reordering preserves identity/targets rather than old indices and independently sampled public native output proves the resulting order/clock

### Requirement: Canonical effect composition
Effects MUST operate in declared order on the item's local premultiplied linear-light raster after crop/clip and graphic fill/stroke, before local and ancestor affine transforms, preserving the verified baseline's existing mask/matte/opacity/blend stages. Gaussian blur MUST retain separable normalized Gaussian kernels, sigma=radiusPx, support ceil(3*sigma), transparent extension and zero-radius identity. Glow MUST retain blurred input alpha times intensity/color alpha and original-over-colored-halo composition. Tint MUST mix straight linear RGB toward declared color using color alpha, preserving source alpha. Vignette MUST multiply RGB by `1-amount*clamp((u*u+v*v)/2,0,1)`, u/v ranging−1…1 across unstroked local bounds, preserving alpha. Color adjustment MUST unpremultiply when alpha>0, compute E=RGB*2^exposureStops, C=0.18+contrast*(E-0.18), L=0.2126*C.r+0.7152*C.g+0.0722*C.b and S=L+saturation*(C-L), clamp S to[0,1] only after all three stages, and premultiply by unchanged alpha; alpha0 MUST produce RGB0. Identity(0,1,1) MUST preserve source output without extra quantization. It MUST charge three pixel-pass units per expanded pixel, retaining the aggregate limit. Unstroked anchors MUST remain fixed when blur/glow expand support. Existing16effects/item,4096expanded occurrences and268435456cumulative pixel-pass units/scene sample(each horizontal/vertical tap one pass), stricter surface and shared memory limits MUST remain; excess MUST fail INVALID_ARGUMENT before persistence/output publication without clipping work to hide excess. Existing declared order MUST have independent analytic/native evidence, not merely production-helper inequality.

#### Scenario: Preserve ordered effects and anchors
- **WHEN** asymmetric content uses two differently ordered effects and a noncentral rotated anchor
- **THEN** the outputs reflect declared order and expanded support without moving the authored anchor

#### Scenario: Prove independent nonexpanding effect order
- **WHEN** opaque red asymmetric shape bounds32x24 uses vignette amount.8 and green tint alpha.5 in opposite order with quarter-turn/noncentral anchor and owner opacity.5
- **THEN** independent local(4.5,5.5)→world(32.5,16.5) evidence produces prepared-PAM RGB(114,137,0) versus(114,114,0) within1byte on the existing opaque scene; a complete independently authored plate through the existing conversion boundary predicts final PNG within1byte, preserves source alpha/anchor and distinguishes the declared noncommutativity

#### Scenario: Fail bounded work without side effects
- **WHEN** hidden/retained/expanded/component content exceeds an existing effect/raster/certification budget
- **THEN** owning preflight rejects before excessive allocation or state/artifact publication, with named exact/overflow automation and unchanged limits

#### Scenario: Preserve identity and encoded-color interpretation
- **WHEN** zero blur/vignette/tint-alpha identities or tint encoded green.5 are exercised
- **THEN** identities preserve existing source output and independent color conversion matches existing linear tint semantics; identity-only fixtures SHALL NOT substitute for the noncommuting witness

### Requirement: Deterministic parameterized color-control evidence
The fifth leaf effect MUST retain full closed required fields and stable ID semantics, unchanged eligibility/count/UTF-8 limits, and expose no paths/scripts/URLs/resources. Canonical fixtures MUST own endpoint/overflow/unknown/missing-field cases, linear equations and independent source/output witnesses. Existing Gaussian/glow/tint/vignette algorithms SHALL remain unchanged; active color controls MUST have meaningful identity, negative-intermediate, saturation, exposure, alpha and order evidence rather than merely helper inequality. No new animation-channel property is activated; existing incompatible effect targets MUST fail atomically.

#### Scenario: Validate all parameter endpoints and failures
- **WHEN** a color-control record uses finite inclusive endpoints, identity, missing/unknown field, nonfinite/overflow value, duplicate ID, excessive stack or unsupported target
- **THEN** owning validation accepts only the approved closed subset and rejects invalid standalone/later-batch/draft operations without any authoritative/history/draft/resource publication

#### Scenario: Preserve independent color and alpha semantics
- **WHEN** encoded source colors, partial/zero alpha or negative intermediate contrast values use exposure/contrast/saturation
- **THEN** independent floating expectations prove linear conversion, operation order, final-only clamping, alpha preservation and transparent black with existing lossless1byte bounds

#### Scenario: Preserve stable targets and observable order
- **WHEN** color adjustment is placed before/after tint or Gaussian and existing targeted effects retain IDs, or a targeted older effect is retyped to incompatible color adjustment
- **THEN** native output proves declared order and unchanged target identity, while incompatible channel edits/retyping reject before any mutation
