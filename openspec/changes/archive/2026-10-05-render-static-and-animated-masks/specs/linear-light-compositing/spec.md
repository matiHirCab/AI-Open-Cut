## MODIFIED Requirements

### Requirement: Explicit current layer pipeline
Every existing visual MUST follow source rasterization/decode, crop and representable local clipping, declared masks, ordered local effects, local anchor/scale/skew/rotation/position, nearest-to-outer ancestor affine transforms, matte, inherited opacity and transition gain, then destination blend in evaluated bottom-to-top order. Existing crop MUST retain its normalized crop/remap behavior; existing SVG clipping and fill/stroke remain source-local. Schema33 MUST execute authored static/animated painted path masks under mask-rendering before ordered effects; absent/empty masks MUST remain identity. Absent mattes MUST remain identity and absent blend selection normal source-over; authored matte, new clip, non-normal blend and new effect kinds SHALL NOT be activated by the mask-model milestone. Opacity/transition gain MUST multiply all four working components exactly once. Source-over MUST use `Co=Cs+Cd*(1-As)` and `Ao=As+Ad*(1-As)` on premultiplied linear components. The raster support extension and unstroked anchor semantics of existing effects MUST remain effective.

#### Scenario: Observe crop effects and affine ordering
- **WHEN** asymmetric cropped content with ordered blur/tint effects is rotated around a noncentral anchor under an ancestor transform
- **THEN** independent stage-order evidence shows crop before effects and effects before affine sampling, without shifting the authored anchor or applying opacity twice

#### Scenario: Blend mixed current sources in evaluated order
- **WHEN** text, decoded media, shape/SVG/grid, rectangle, solid-color, and Caption sources overlap with fractional coverage
- **THEN** every source participates in the same linear source-over semantics in evaluated paint order, including sources without effects or Transform2D

#### Scenario: Preserve absent-stage defaults
- **WHEN** an existing project omits masks, mattes, effects and blend selection
- **THEN** it remains valid under the documented persisted-schema migration/default rules and those stages act as identities with normal source-over

#### Scenario: Activate authored masks before effects
- **WHEN** a schema 33 visual carries a valid nonempty static/animated painted alpha/luma mask stack
- **THEN** metadata is preserved and approved coverage multiplies premultiplied source before existing effects/transforms, with unchanged opacity/source-over and all existing composition limits
