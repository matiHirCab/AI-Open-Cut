## MODIFIED Requirements

### Requirement: Explicit current layer pipeline
Every existing visual MUST follow source rasterization/decode, crop and representable local clipping, declared masks, ordered local effects, local anchor/scale/skew/rotation/position, nearest-to-outer ancestor affine transforms, matte, inherited opacity and transition gain, then destination blend in evaluated bottom-to-top order. Existing crop MUST retain its normalized crop/remap behavior; existing SVG clipping and fill/stroke remain source-local. Schema33 active static/animated painted path masks MUST remain before effects and absent/empty masks identity. Schema34 MUST execute track-mattes coverage after transforms and before recipient inherited opacity/transition; absent mattes MUST remain identity. Provider planes MUST be isolated transparent premultiplied linear RGBA and matteOnly SHALL suppress direct drawing alone. Absent blend selection MUST remain normal source-over; new clip/non-normal blend/new effect kinds SHALL NOT activate here. Opacity/transition gain MUST multiply all four working components exactly once per owning/provider layer; existing own-copy shutter averaging MUST follow its completed matte/opacity samples and precede destination source-over. Source-over MUST use `Co=Cs+Cd*(1-As)` and `Ao=As+Ad*(1-As)`. Existing effect support extension and unstroked anchors MUST remain effective.

#### Scenario: Observe crop effects and affine ordering
- **WHEN** asymmetric cropped content with ordered blur/tint effects is rotated around a noncentral anchor under an ancestor transform
- **THEN** independent stage-order evidence shows crop before effects and effects before affine sampling, without shifting the authored anchor or applying opacity twice

#### Scenario: Blend mixed current sources in evaluated order
- **WHEN** text, decoded media, shape/SVG/grid, rectangle, solid-color, and Caption sources overlap with fractional coverage
- **THEN** every source participates in the same linear source-over semantics in evaluated paint order, including sources without effects or Transform2D

#### Scenario: Preserve absent-stage defaults
- **WHEN** an existing project omits masks, mattes, effects and blend selection
- **THEN** it remains valid under documented persisted-schema migration/default rules and absent stages act as identities with normal source-over

#### Scenario: Activate authored masks before effects
- **WHEN** a schema33/34 visual carries valid nonempty static/animated painted alpha/luma masks
- **THEN** metadata is preserved and approved coverage multiplies premultiplied source before effects/transforms, preserving all existing opacity/source-over and composition limits

#### Scenario: Activate isolated matte after transforms
- **WHEN** a schema34 eligible leaf references a visible transformed alpha/luma provider with matteOnly or its own dependency
- **THEN** isolated provider coverage multiplies transformed recipient once before recipient inherited opacity/transition and destination blend, with transparent/hidden/inactive provider rules and unchanged audio
