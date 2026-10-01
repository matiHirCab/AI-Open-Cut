# Corrective verification record

Implementation conformance verified 2026-10-01. Specification synchronization/archive and the protected final gate pass;
external delivery is pending in
`/workspace/AI-Open-Cut-evidence/issue43-corrections-20261001/delivery.md`.
The pinned base is PR131 head e1010bb97174d44a5d19c61a90514bf2336845ca.
Issue44's three commits remain separate and unpublished.

## Completeness, correctness and coherence

The approved plan covers two requirements and four scenarios. Every implementation
scenario has automated coverage. All 11 implementation/workflow tasks are complete. Delivery preparation is
complete. Actual push,
PR evidence and terminal exact-head CI are explicitly pending external steps.

| Requirement / scenario | Implementation and evidence |
| --- | --- |
| Portable conformance / fixed-width RGBA | `render_artifact/extended_visual.rs` uses validated fixed-size byte arrays; strict workspace Clippy passes on Rust 1.97 and 1.98. |
| Portable conformance / native identity and sampled parity | `render_plan.rs` preserves RGBA sampled composition and vector transparent borders; unchanged identity, nested and compound native fixtures pass on Ubuntu FFmpeg6. |
| Isolated delivery / reviewed corrections | Four implementation/test files, bounded approved spec artifacts, independent issue44 worktree and explicit non-force existing-branch payload; publication remains separately pending. |

`extended_visual_animation.rs` preserves identity MSE < 30.0 and every independent
extent, timestamp and SSIM >= 0.99 expectation. It adds an outside-border check
and proves one SSIM stats line at each nested comparison. Aligned PCM RMS <=
0.0001 and timing <= one frame remain unchanged. MCP integration retains every
helper call/assertion and its existing 60-second timeout, with independent
extended-visual work in its own test. No public contract, schema27, persistence,
audio policy, revision, path, error, resource cap, golden or CI configuration changes.
No correctness/coherence mismatches remain in the implementation.

Captured native evidence explains both defects: FFmpeg6 default bilinear `geq`
expanded 400 vector pixels to 441; explicit nearest lookup only for vectors
preserves the transparent border. Media/text lookup remains exactly unchanged.
FFV1 RGBA is byte-identical to produced PAM frames. The nested first frame has
SSIM 0.999466, but untrimmed FFmpeg6 SSIM evaluated eight subsequent frames behind
an output limit and averaged 0.858238. Trimming both inputs fixes the selected
frame measurement, without relaxing encoding conformance.

## Final checks

All checks below passed against the final narrowed vector correction. Full logs,
exit files, tool provenance and red/green captures are retained outside Git in
`/workspace/AI-Open-Cut-evidence/issue43-corrections-20261001/`.

| Check | Result | Log under /tmp |
| --- | --- | --- |
| cargo fmt --check --all | PASS | opencut-43-final-fmt-v3.log |
| cargo clippy --workspace --all-targets -- -D warnings | PASS Rust1.97 and 1.98 | opencut-43-final-clippy197-v3.log; opencut-43-final-clippy198-v4.log |
| cargo test --workspace | 760 passed; 9 existing maintenance/opt-in ignores | opencut-43-final-workspace-v3.log |
| Native animation_channels, Ubuntu FFmpeg6 | 46 passed | opencut-43-final-animation6-v2.log |
| Native extended_visual_animation, system FFmpeg7 | 25 passed, 35.09s | opencut-43-final-extended7-v2.log |
| Native reviewed golden corpus, Ubuntu FFmpeg6 release | PASS all seven captures, 657.93s | opencut-43-final-goldens6-release-v2.log |
| Native core / worker / bridge raster cache | 13 / 3 / 1 passed | opencut-43-final-cache-core6.log; opencut-43-final-cache-worker6.log; opencut-43-final-cache-bridge6.log |
| Default headless / native lifecycle / transform and fonts | 40 / 1 / 29 passed | opencut-43-final-default-headless.log; opencut-43-final-lifecycle6.log; opencut-43-final-geometry-font6.log |
| Performance report validation | PASS | opencut-43-final-report-validator.log |
| Bridge contracts | 184 Rust + 359 TypeScript passed | opencut-43-final-contracts-v2.log |
| Bridge typecheck / lint | PASS | opencut-43-final-type-v2.log; opencut-43-final-lint-v2.log |
| Bridge unit | 426 passed; 1 opt-in cache fixture run separately above | opencut-43-final-unit-v2.log |
| MCP integration | 14 passed, 164.65s | opencut-43-final-integration-v4.log |
| Packaged smoke | 8 passed, 46.38s | opencut-43-final-smoke.log |
| Hermetic Python worker | 10 unittest + 5 pytest passed | opencut-43-final-python.log |

Real rendering uses explicit official Ubuntu6.1.1-3ubuntu5 FFmpeg/FFprobe or system
7.1.5, required native markers and the reviewed font SHA256
`ae7b7855e115a5966d8b1b3f80f254ccc117ec86f9965e202ee2940453837280`.
Smoke mocks are not parity evidence. The final release golden command selects
`--lib`, executing the complete same corpus while avoiding unrelated integration
binary compilation. No golden updates. Captures: rule_card211.008s, shapes122.050s,
SVG27.825s, grids17.408s, repeaters47.607s, rich_text213.791s, sampled16.602s.
A scoped Linux child-subreaper reaps only its own adopted processes for this
container's non-reaping PID1; child exit status and all process guards are retained.

## Superseded failures and cancelled checks

Unchanged e1010bb reproduces Clippy and 3/25 Ubuntu6 render failures (identity
MSE124.97607421875; nested0.858022; compound0.830426). Broad nearest lookup passed
those fixtures but failed unchanged oriented-media occupancy626 versus625;
final correction narrows to vectors and all 13 transform fixtures pass unchanged.
Earlier MCP runs first lacked checkout-local generated binary links, then the
combined independent workflows exceeded 60s even idle. Final isolated case passes
without changing its budget. An exploratory debug golden was deliberately
interrupted; it is cancelled, not parity evidence. Custom source FFmpeg6.1.2 is
not the CI-equivalent backend and is not certification. Superseded successes do
not replace the final results above.

## Protected specification gate and review

The real pinned Moon2.3.3 pre-archive task passes 377 policy/regression tests and
strict validation of all 34 OpenSpec items. Its sole rejection names this active
proposal; expected pre-archive rejection is not gate success. Official proto0.57.4,
Bun1.4.0 and GHCR plugins use temporary caches. Writable XDG_CACHE_HOME and the
documented PROTO_OFFLINE=false runtime option restore this network-enabled
environment without repository/global configuration changes or substitute plugins.

No changed canonical contract requires new contract-owner review for this
correction. Existing PR merge review remains external; no review is fabricated.
Synchronization and archive completed at
`openspec/changes/archive/2026-10-01-fix-extended-animation-ci-parity/`.
The final real Moon protected gate passes (377 policy/regression tests and 34
strict OpenSpec items), log `/tmp/opencut-43-final-protected-v1.log`.
Final strict all-spec validation passes all 34 items, log
`/tmp/opencut-43-final-strict.log`. Only an ordinary PR131 existing-branch update is authorized.
No merge, force-push, deployment, issue44 publication or watch changes.
