Correct the independent P2 midpoint finding on PR132 without rewriting its
original implementation or approved archival. A 360-degree five-sample exposure
at 500 ms / 25 FPS now evaluates [484,492,500,508,516], giving the held parent key at 508
its canonical 60/40 coverage. Editor-core floors exact binary64 integer ratios;
all inherited transform/composition/audio/lifecycle contracts and limits remain.

Independent regressions cover every count 1..16, neighboring angles, fractional
frame periods, subnormals and clipping/high roots. Real FFmpeg 6/7 frame/range/draft/
export now have MSE 0 / SSIM 1 and PCM RMS 0 against fixed analytic references. Rust
fmt/strict Clippy/workspace, bridge/type/contracts/unit/integration/mocked smoke,
Python and affected native suites pass. Full legacy golden/rules matrices are
explicitly reused for unchanged no-blur inputs, not claimed as newly executed.

Fresh correction CODEOWNER review and OpenSpec archival/final protected checks
remain pending. Original b03 archival and approval are untouched; the new scoped
change is fix-motion-blur-midpoint-boundaries. Keep PR132 draft. No merge/deploy.
