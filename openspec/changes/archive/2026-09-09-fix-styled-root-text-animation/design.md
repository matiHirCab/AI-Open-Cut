## Context
Affine rendering evaluates legacy keyframes only with layer ancestry. Styled root text currently synthesizes identity ancestry only in measurement, freezing rendered position, scale and opacity.
## Decisions
During apply_ancestors, give parentless styled text identity matrices, opacity 1 and its existing clip span. Keep actual parents and plain text behavior. Remove the local fallback in measure_layer_affine; rendering consumes the same evaluated ancestry. Preserve Transform2D precedence. No renderer-side ancestry duplication.
## Verification
Add evaluator tests for all three keyframe properties, identity ancestry before/after finalization, plain text, non-default anchors, actual group/component ancestors and Transform2D precedence. Native tests sample 0/400/800 ms, compare to independently calculated static transforms/opacity, check displacement/bounds/brightness, and compare frame/range/draft/export with existing tolerances. Do not recapture baseline expectations. Run repository-required checks, verify and archive, then Moon.
## Compatibility and failure behavior
No API, persisted data, capabilities, errors or retryability change. Existing clipping, timing, easing and validation apply. Invalid geometry still fails before artifact publication.
