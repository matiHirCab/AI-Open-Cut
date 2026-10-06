## Native raw-scene and encoder-boundary verification clarification

This clarification changes verification for tasks5.1/5.2 only, preserving all16
requirements/39scenarios and all original numerical tolerances. Existing final
frame encoding performs scale/pad and format=yuv420p before PNG; one-byte raw
linear-to-sRGB expectations must be checked before that conversion, and final
PNG expectations must independently include it. No production codec, public API,
rendering algorithm, timeout, or acceptance limit changes are authorized.

1. Observe the ACTUAL production-prepared final `linear-scene.pam` for each
   authored candidate/draft, committed and reopened frame. A test-owned portable
   std-only Rust executable proxy is compiled with the already-required Rust
   toolchain in a temporary directory; Renderer uses it as FFmpeg. It copies
   only the existing exact-basename PAM argument before forwarding EVERY original
   OS argument to the configured real FFmpeg, preserving stdout/stderr and exit
   status. Platform executable suffixes and argument boundaries are preserved;
   no shell dependency, global environment mutation, production hook, replacement
   synthetic scene, or platform skip is permitted. Assert copied PAM dimensions,
   format, opaque alpha and independently derived raw RGB within the existing
   uncompressedByteMaximum1. Component witness green linear0.75 encodes225 and
   green linear0.25 encodes137. Raw wrong-average/clock/encoded controls remain.
2. Independently author an RGB reference plate from fixture geometry and analytic
   linear colors (never from captured production pixels). Convert it through the
   exact unchanged final graph route, including the input0 opaque-black base
   through initial format=yuv420p. The independently authored prepared plate
   follows the unchanged fps/settb/setpts/format=rgba input route and
   overlay=format=auto onto that black base, THEN final
   scale=96:64:force_original_aspect_ratio=decrease,pad=96:64:(ow-iw)/2:(oh-ih)/2,
   format=yuv420p, PNG encoding and rgb24 decoding, retaining identical FFmpeg
   options relevant to colors. Never apply the initial black-base YUV conversion
   to the authored RGB plate; an extra plate conversion is a different oracle. Compare public final PNG witness pixels against
   that independently converted plate within the SAME one-byte bound. At chroma
   boundaries author actual neighboring colors rather than isolated point data.
3. Keep draft/current/reopen and undo/redo exact same-intent comparisons, every
   frame/range/draft/export execution, SSIM>=0.99, PCM RMS<=0.0001, frame/timing
   and unchanged matteOnly audio checks. Captured raw bytes alone or a synthetic
   PAM alone cannot replace the public all-intent evidence.

The transformed noncentral-mask/tint/ancestor witness independently predicts
linear green0.026575 (raw green45). Its observed zero must be diagnosed using
actual prepared scene/coordinates/coverage; it cannot be classified as conversion
noise or used to justify a changed oracle/tolerance. Fixture geometry corrections
must be independently derived and documented; genuine owner defects are fixed
within approved matte integration semantics. First native failures remain evidence.
