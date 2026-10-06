# Cache-hit clone refusal control

This closes the approved Meaningful Control2 automation gap; no new budget or public surface.

Add a private artifact-ledger helper in render_artifact/text_measurement_memory.rs:

```rust
pub(super) fn clone_cached_shape(
    memory: &MeasurementMemory,
    shaped: &crate::fonts::shaping::ShapedText,
    transient: u64,
) -> Result<crate::fonts::shaping::ShapedText, CoreError> {
    let required = crate::evaluated_scene::mattes::shaped_heap_bytes(shaped)?
        .checked_mul(3)
        .and_then(|bytes| bytes.checked_add(transient))
        .ok_or_else(invalid)?;
    memory.admit(required)?;
    Ok(shaped.clone())
}
```

Replace the active Some(memory) branch of the existing shaped-cache hit with this helper; None still invokes the existing Clone::clone directly. The reservation and ordering are byte-for-byte equivalent in meaning to the existing pre-admission followed by clone.

The new owning test creates actual bundled-font shaped Text with16 per-glyph paint layers, and places the genuine shape in a HashMap cache with spare capacity. Create a genuine MeasuredText through text::measure using that shape, place it in the retained result HashMap, and use the exact production cache/result byte walkers for the already-live ledger. All source/cache/result objects and test inputs are created before allocator observation.

Let H be the exact owning shaped-heap walker, S a fixed explicit key/document transient, and R the exact retained cache+result ledger. Test limit R+3H+S-1. Call clone_cached_shape with the actual cache.get result inside the existing artifact allocator observer; require INVALID_ARGUMENT/nonretryable, peak<512 bytes (small error diagnostic only) and peak<H. This proves rejection precedes glyph/face/color/paint cloning. Retained cache/result content and capacities must remain unchanged.

Then set limit exactly R+3H+S. The same actual cached object must clone successfully, remain content-equal, and allocate a positive observed heap bounded by3H. Drop the clone before observation ends; input/cache/result remain borrowed and untouched, so the observer requires live0. This is an actual private production seam and exact boundary/+1 control, rather than a mirrored arithmetic test or a fresh-shape budget fixture. Initial source/opaque shaping allocation happens outside observation.

After parent releases source/Cargo: implement helper/call-site/test; format owning files only, run focused new control and existing main cache/path tests. Root then resumes final full checks on the new stable source. No edit is made while its current workspace suite is running.
