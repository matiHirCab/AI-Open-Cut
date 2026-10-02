# Motion blur sampling (schema 28)

`motion_blur_sampling_v1` reports support. Existing `update_item` accepts optional
`motionBlur: { shutterAngleDeg, sampleCount }`, including alias-aware batches.
Omission preserves authored settings. Zero shutter disables blur. Angle must be
finite in [0,360]; count is an integer in [1,16]. A single sample retains prior
instantaneous output. Supported leaves are visual media, text, solids, rectangles,
shapes, SVG and grids. Audio, captions, transitions and controllers reject settings.

Exposure is centered on root output time and lasts angle/360 times one authored project frame, even when range output uses a different FPS.
Samples are ascending, equally weighted interval midpoints. Root sample times floor
once to milliseconds; inherited component rates, parent clocks, stagger, loops and
signed repeater offsets retain their fractional mappings. Root intervals clamp to
[0,durationMs-1], preserving duplicate boundary weights. There is no random jitter.

Every sample resolves canonical activity, source time, crop, local clipping, paint,
ordered effects, complete inherited affine, opacity and transition gain. Canvas
rasters average in premultiplied linear light before existing per-layer stacking.
This is per-leaf blur: crossing layers retain their canonical stacking order.
Audio processing and synchronization are unchanged.

The canonical catalog is `contracts/motion-blur-sampling-v1.json`. Existing surface,
geometry, effect and occurrence limits apply. Enabled blur additionally caps
sample-weighted canvas work at 268435456 pixel units per output scene frame.
Validation includes retained/hidden/expanded content and precedes publication.
Failures preserve revisions, history, resources and existing output destinations.

Current state, retained undo/redo and compatible drafts adopt schema28 atomically
under the existing project lock. Older schema documents containing premature blur
fields and future versions fail closed. Valid reopen does not rewrite state.
Existing errors and protocol1 request identifiers remain compatible. Invalid typed edits
return `INVALID_ARGUMENT`; malformed persisted JSON retains its existing
`INTERNAL_ERROR` classification and never publishes a migrated generation.

The existing required native `animation_channels` test target includes the
motion-blur conformance module, so the protected render command executes these
fixtures without changing its CI sequence. For a focused native reproduction,
set explicit `OPENCUT_FFMPEG_PATH`, `OPENCUT_FFPROBE_PATH` and the reviewed font,
set `OPENCUT_ANIMATION_CHANNEL_RENDER_REQUIRED=1`, then run
`cargo test -p opencut-editor-core --test motion_blur_sampling -- --nocapture`.
The independent oracle uses analytic inherited translations and midpoint coverage;
additional fixtures exercise component rates, stagger, signed repeaters, loops,
effect ordering, drafts, differing range FPS, compatibility and decoded PCM.
