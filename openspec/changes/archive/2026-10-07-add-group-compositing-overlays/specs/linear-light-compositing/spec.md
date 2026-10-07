## MODIFIED Requirements

### Requirement: Explicit current layer pipeline
Every uncontrolled existing direct visual MUST follow source rasterization/decode, crop and representable local clipping, declared masks, ordered local effects, local anchor/scale/skew/rotation/position, nearest-to-outer ancestor affine transforms, matte, inherited opacity and transition gain, owning occurrence shutter averaging, then destination blend in evaluated bottom-to-top order. Private named provider products MUST complete their own source/mask/effect/transitive matte/full-world affine/inherited gain/shutter stages and stop there, excluding destination blend/background. Existing crop MUST retain normalized crop/remap behavior; existing SVG clipping/fill/stroke remain source-local. Active static/animated painted masks MUST remain before effects; absent/empty masks identity. Typed track mattes MUST remain after transforms before recipient opacity/transition; absent mattes identity. Provider planes MUST remain isolated transparent premultiplied linear RGBA independent of blend, and matteOnly suppress direct drawing alone. Schema35 MUST execute exactly normal/multiply/screen/overlay/add/darken/lighten under blend-modes equations; absent/normal selection MUST retain the exact existing f32 source-over path and initial opaque black destination for root drawing; controlled-local drawing MUST use transparent local black. Background kinds SHALL NOT activate here; schema37 controlled clip/aggregate/overlay kinds MUST use the explicit exception below. Opacity/transition gain MUST multiply all four working components once per owning/provider layer. Normal source-over MUST retain `Co=Cs+Cd*(1-As)` and `Ao=As+Ad*(1-As)`; non-normal MUST use the approved finite premultiplied blend function and the same source-over alpha. Existing effect support/unstroked anchors MUST remain effective.


Explicit Group/ComponentInstance clip/nonempty effects MUST activate one transparent-local aggregate. Descendants MUST complete source/masks/effects, relative affine, matching-world matte, relative gain and own shutter before canonical child blend; aggregate clip precedes owner effects, then frozen outward owner affine/opacity applies once and root destination blend follows. Only private named providers retain full-world ancestor shutters/opacity excluding ancestor aggregate clip/effects; controlled direct drawing MUST use the motion-blur-sampling boundary. Existing scenarios below MUST retain exact uncontrolled/private-provider behavior, including the historical schema35 opaque-root blend witness. No intermediate encoding round trip MUST be introduced.

#### Scenario: Observe crop effects and affine ordering
- **WHEN** asymmetric cropped content with ordered blur/tint effects is rotated around a noncentral anchor under an ancestor transform
- **THEN** independent evidence shows crop before effects and effects before affine sampling without shifting anchors or applying opacity twice

#### Scenario: Blend mixed current sources in evaluated order
- **WHEN** text, decoded media, shape/SVG/grid, rectangle, solid-color and Caption sources overlap with fractional coverage
- **THEN** every eligible occurrence uses its declared final blend after complete source assembly, retaining canonical paint order including sources without effects/Transform2D

#### Scenario: Preserve absent-stage defaults
- **WHEN** an existing project omits masks, mattes, effects and blend selection
- **THEN** it remains valid after migration, absent stages act as identity and existing normal output/opaque-black initialization remain exact

#### Scenario: Activate authored masks before effects
- **WHEN** an eligible visual carries active static/animated painted alpha/luma masks
- **THEN** metadata is preserved and approved coverage multiplies premultiplied source before effects/transforms with unchanged opacity and existing composition limits

#### Scenario: Activate isolated matte after transforms
- **WHEN** an eligible leaf references transformed alpha/luma provider coverage with matteOnly or its own dependency
- **THEN** isolated coverage multiplies recipient once before recipient opacity/transition and final declared blend, with hidden/inactive provider rules and unchanged audio

#### Scenario: Activate linear blend after source shutter averaging
- **WHEN** a schema35 eligible occurrence uses a non-normal mode with translucent/shutter-averaged source
- **THEN** final premultiplied equations use the current opaque-backed destination, overlay branches on backdrop and bounded add preserves source-over alpha without encoded RGB or blend-per-shutter substitution

#### Scenario: Compose explicit controlled linear aggregates
- **WHEN** schema37 controls isolate overlapping children with positive effects, clipping, local blends and unequal shutters
- **THEN** independent premultiplied local/query/aggregate/outward plates prove exact stage order, no duplicate owner gain and unchanged uncontrolled/provider/root behavior
