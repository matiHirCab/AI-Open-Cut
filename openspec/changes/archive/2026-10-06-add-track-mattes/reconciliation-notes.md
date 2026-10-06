# Stable #51 reconciliation; external planning only

All #52 tasks are pending. No active #52 change, approval, implementation, archive, branch or PR is claimed. On2026-10-05 actual #51 source/contracts and archive-applied living requirements were read. Parent reported #51 fully verified and archived at5f456618b9335a5f87519c5c4375a49e7f52c2d3. All eight raw catalog bytes/full semantic digests and expanded MCP were independently matched to that exact commit with git show. Stop/reconcile any later predecessor mismatch.

## Concrete baseline

- Core PROJECT_SCHEMA_VERSION is33. Existing typed VisualProperties.masks,
  UpdateItem optional masks and15 scoped mask animation properties are active.
- mask_models_v1 and mask_animation_v1 editor capabilities are distinct from
  mask_rendering_v1 complete renderer readiness.
- The eight exact semantic digests are captured in
  predecessor-catalog-digests.json, using the governed existing JS codec.
- Independently expanding actual contracts/mcp-surface-v1.json yielded
  803bf5954ebd4cb47be98dd87b4994e6d261eae20693199c0f569f535452f170.
- The actual mask eligible family is Text, SolidColor, Rectangle, Shape, Svg,
  Grid and known Image/Video Media. Caption/Audio/Group/Repeater/ComponentInstance/
  TemplateInstance/Transition nondefault mattes remain outside proposed #52 scope.
  Their normal existing rendering/default metadata remain valid. This is a concrete
  proposed applicability decision requiring independent acceptance review, not an
  inference that Caption is universally nonvisual. Missing media assets retain
  canonical ASSET_NOT_FOUND rather than being reclassified as an ineligible leaf.
- Mask owner origin is post-crop/clip top-left zero with logical extent W/density,
  H/density. Source/cache PAM bytes remain unmasked; source copies receive mask
  processing once before ordered effects and affine evaluation. Keep no-mask
  exact path, source integrity before caches and revision-scoped admission.
- SharedMasks/SharedChannels preserve empty no-allocation defaults and share
  immutable nonempty programs. Source owning Vec capacities, ID/path/keyframes/
  gradients and S0/no-grid facts are explicitly reserved. The new matte plan
  must not assume one copy per consumer is free or reset shared analysis/work.
- Actual opaque fill is ADDITIVE2048*(S+C+64)+32*(W+H), separate8P Pixmap/result;
  emitted line counts are checked against S+C before the reservation is consumed.
  Retain allocator witnesses for4096S/tiny8x8 and63/64/65 edge capacity boundaries.

## Narrow #52 additions

Schema34 adds only typed scoped matte references and matteOnly lifecycle plus
bounded provider-plane/DAG evaluation. No mask algorithm/property/target, source
family activation, new effect/blend, matte animation target, output background or
backend expression is added. Existing opaque black destination stays unchanged;
only isolated provider planes start transparent. matteOnly alone suppresses direct
drawing; exact bypass requires BOTH default fields on every item.

The proposal retains32→33 active-mask behavior and adds33→34 defaults/current+
retained-history/source-matched draft adoption through existing staged transactions.
Every older source envelope rejects new field presence before defaults; unavailable
valid34 draft bases remain preserved/revision-conflicted instead of replayed on current.

## Promotion review checklist

Confirm exact predecessor hashes and archive-applied living requirements; independently
approve same-family applicability, per-occurrence base+copy aggregation AFTER each
copy's shutter average, bounded nested sample multiplication and exact named graph
limits. Complete the explicit MCP field/definition/literal whitelist and existing
catalog marker-only34→33 proof before implementation. Promote all deltas and plans,
run strict validation, record exact independent/root approval, then author red semantic
public/native evidence and canonical fixtures before governed implementation declarations.
