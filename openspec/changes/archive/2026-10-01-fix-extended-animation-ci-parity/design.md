## Context

CI run 36800533080 reports fixed-width chunks Clippy lint and Linux failures: identity MSE 124.97607421875; nested sample200 SSIM 0.863424; compound sample0 SSIM 0.832159. Recheck current head and terminal CI status before mutation/publication. Initial head remains e1010bb97174d44a5d19c61a90514bf2336845ca.

## Goals / Non-Goals

Restore existing issue43 semantics across supported backends, isolate corrections from issue44 and prove red/green native output. Non-goals are in proposal.md.

## Decisions

Use fixed-width array chunks after existing byte-length validation, preserving linear-premultiplied decoding. Diagnose compositor pixel format, color conversion and sampled lossless preparation before changing code; preserve entire RGBA/alpha semantics through intermediate resources and use documented existing coordinates/clocks. Prefer explicit compatible composition/encoding over FFmpeg version defaults if reproduced evidence identifies defaults as the cause. Keep existing tolerances and independent controls; do not replace failing expectations with output-to-output equality. No dependency direction or security expansion.

## Risks / Trade-offs

FFmpeg version differences and parallel native fixture execution must be isolated. Local FFmpeg7.1.5 focused success is not proof of Ubuntu CI compatibility. Correcting a conversion may affect existing references; compare unchanged goldens and report failures rather than recapturing them.

## Migration / Rollback

No persisted schema change. Make focused corrective commits on latest verified PR131 head. Recheck remote immediately before ordinary push; incorporate concurrent changes without overwriting. Preserve issue44 local commits def74b6d, ec5f67d3 and cd4903d7 unchanged in the isolated worktree until corrected issue43 commits are settled; then reconcile without publishing issue44.

## Approval and reproduced evidence (2026-10-01)

The delegating reviewer explicitly approved the complete proposal, design and tasks
under the user's delegated OpenSpec approval and subsequent request to correct PR131
remotely. This is implementation approval, not external CODEOWNER review, merge,
force-push, deployment or issue44 publication approval.

Remote head remains e1010bb; CI run 36800533080 is terminal failure and there are no
PR reviews. Unchanged base reproduces the Rust 1.98 chunks lint. Official Ubuntu
FFmpeg 6.1.1-3ubuntu5 reproduces identity MSE 124.97607421875, nested sample200 SSIM
0.858022 and compound sample0 SSIM 0.830426 (22 passed, three failed). Debian 7.1.5
passes the focused suite; supported-backend differences require explicit graph
semantics, not tolerance changes. Custom-source 6.1.2 builds are not equivalent CI
reproductions and are not used to certify the fix.

## Diagnosed corrections

Native captures prove two separate causes. Pixelwise FFmpeg 6 `geq` defaults to
bilinear lookup and expands a transparent 22x22 path border: 400 opaque source
pixels become 441 before YUV conversion. Explicit nearest-pixel channel/alpha
lookup preserves the intended border. Sampled overlays explicitly retain RGBA
composition like the existing affine branch.

The lossless FFV1 RGBA streams are byte-identical to produced PAM frames. The
nested first encoded frame has RGB MSE 1.3833 and true single-frame SSIM
0.999466. However, FFmpeg 6 evaluates eight SSIM filter frames ahead of an
output `-frames:v 1` limit, comparing the fixed reference against later animated
frames and averaging 0.858238. Trim each comparison input to its selected first
frame and reset timestamps before SSIM; retain all seek times, thresholds and
independent expectations. A stats-line-count assertion proves exactly one
comparison at each nested sample. This corrects measurement selection, not
encoding quality or tolerances.

Broader native orientation coverage caught a one-pixel opaque video occupancy
change when nearest lookup was applied to every legacy source. Bound the lookup
correction to vector resources with the declared transparent raster border;
retain the established media/text graph exactly. The existing oriented-media
fixture remains unchanged and must pass, alongside the independent vector border
assertion. No expected occupancy, tolerance or reference is changed.

The required MCP integration case combines group, extended-visual and component
workflows that create independent projects. Even an idle run had 12 passes and
that combined case exceeded its existing 60-second budget. Move the complete
extended-visual helper into its own case, retaining every assertion, call and
timeout. This is bounded verification of the approved existing behavior, with
no transport, contract, fixture expectation or CI policy change.

## Delivery ordering clarification

Prepare the exact corrective payload and an explicit delivery record before
archival. Execute the already approved ordinary push and exact-head CI monitoring
only after the protected final gate passes. The implementation task checklist
tracks preparation; actual external actions stay pending in the delivery record
until observed. This resolves the archive-before-publication dependency without
waiving any check or claiming future actions as completed.
