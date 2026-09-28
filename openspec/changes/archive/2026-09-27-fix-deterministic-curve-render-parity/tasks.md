## 1. Approval and focused evidence

- [x] 1.1 Obtain explicit user or reviewer approval of this proposal, design, both delta specs, and tasks before editing any implementation file. (animation-channels, rendering-export)
- [x] 1.2 Reproduce and record the failing native Bézier draft frame and position-Y samples on retained FFmpeg 6.1.1, passing controls on 6.1.1, and the passing curve case on 8.1.2; keep full logs outside tracked files. (rendering-export)
- [x] 1.3 Isolate the first divergent scalar, affine filter expression, or decoded pixel in the complete supported scene on FFmpeg 6, and record the exact cause in the verification evidence before choosing the localized compiler edit. (rendering-export)

## 2. Editor-core regression and correction

- [x] 2.1 Add fixed scalar comparison coverage for Bézier endpoints/interior and underdamped, critical, overdamped, and near-critical spring samples; include the approved `19.99999999999999` damping case and a 64-bit FFmpeg numeric output path with `1e-9` absolute tolerance. (animation-channels)
- [x] 2.2 Correct only the parameterized render expression compiler: preserve finite `f64` control points, coefficients, and segment values at 17 fractional digits; keep the existing formatter for simple and legacy curves and preserve exact endpoints and property bounds. (animation-channels)
- [x] 2.3 Add a private typed render-plan flag for scenes containing cubic Bézier and select `-filter_complex_threads 1` only for those plans; verify command construction keeps the existing policy for spring-only and legacy scenes. Keep the 40-step expression unchanged. (rendering-export)
- [x] 2.4 Strengthen native decoded-media regression coverage for Bézier and spring visual position/scale/opacity and audio gain across matching draft, frame, audiovisual range, and export timestamps, plus invalid-preflight and legacy-only controls. (rendering-export)

## 3. Verification and archival

- [x] 3.1 With `OPENCUT_FFMPEG_PATH`, `OPENCUT_FFPROBE_PATH`, and `OPENCUT_ANIMATION_CHANNEL_RENDER_REQUIRED=1` pointing at retained 6.1.1, run `cargo test -p opencut-editor-core --test animation_channels`; repeat with retained 8.1.2. Capture exact exit status, full logs, and decoded comparison results. (rendering-export)
- [x] 3.2 Run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`, plus affected native render/golden parity with the reviewed FFmpeg/font environment. Preserve full logs and do not refresh canonical goldens to mask a failure. (all changed Rust scenarios)
- [x] 3.3 From `apps/agent-bridge`, run `bun run typecheck`, `bun run lint`, `bun run test`, `bun run contracts:check`, `bun run test:integration`, and `bun run test:smoke`. Python provider tests are unaffected because no worker or provider protocol changes. (compatibility and contract parity)
- [x] 3.4 Run `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive` and pre-archive `moon run root:openspec-validate`; only this active change may cause the expected archive-boundary rejection. Resolve all other failures. (both delta specs)
- [x] 3.5 Run `$openspec-verify-change` against requirements, design, tasks, tests, and code; resolve mismatches, then `$openspec-sync-specs` and `$openspec-archive-change`. (both capabilities)
- [x] 3.6 Rerun post-archive `moon run root:openspec-validate` and `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive`; report command exits, log locations, and the separate FFmpeg 9 tooling limitation. (final protected gate)
