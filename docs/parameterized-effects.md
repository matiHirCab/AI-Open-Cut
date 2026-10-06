# Parameterized effects

Issue #55 adds static `color_adjustment` alongside Gaussian blur, glow, tint and vignette. `contracts/parameterized-effects-v1.json` governs the fields and witnesses. Existing `timeline_update_item` and `update_item` batch/draft operations accept the effect. Array order and stable IDs retain their meaning; omission preserves the stack, `[]` clears it and `null` is invalid. Eligible item types and animation properties are unchanged.

| Required field | Accepted values |
| --- | --- |
| `type` | `color_adjustment` |
| `id` | Nonempty, at most 128 UTF-8 bytes, unique in the stack |
| `exposureStops` | Finite −8 through 8 |
| `contrast` | Finite 0 through 2 |
| `saturation` | Finite 0 through 2 |

For positive alpha, unpremultiply the linear RGB sample. Compute `E = RGB * 2^exposureStops`, `C = 0.18 + contrast * (E - 0.18)`, luminance `L = 0.2126*C.r + 0.7152*C.g + 0.0722*C.b`, and `S = L + saturation * (C - L)`. Clamp only S to [0,1], then premultiply by unchanged alpha. Transparent pixels have zero RGB. Identity `(0,1,1)` preserves floating sample precision. The contrast pivot is linear 0.18; encoded-sRGB equations or intermediate clamping give different results.

Color adjustment adds no support padding and reserves three pixel passes per expanded pixel, including identity, within the existing cumulative 268435456 limit. Existing surface, memory, occurrence, stack and deadline guards remain unchanged.

Schema 36 migration advances authentic schemas 1–35, including current/undo/redo, atomically without changing resource bytes or provenance. Color records below source schema 36 are invalid, including component definitions and drafts matched to older bases. Unknown future schemas fail. Existing staged-resource and journal owners govern edits and recovery.

Discovery advertises `parameterized_effect_models_v1` and, with ready rendering, `parameterized_effects_v1`. Protocol 1 and all 78 MCP tools remain. Ten existing catalog markers advance to 36 with complete pinned schema-35 predecessors. MCP projection removes only the exact fifth variant, two capabilities and two literals before checking the complete predecessor digest and every older digest within the same test deadline.

The independent asymmetric 32×24 red path uses a quarter turn, offset anchor and opacity 0.5 on a 64×64 scene. Exposure −1, contrast 0.75 and saturation 0.25 predict straight-linear RGB `(0.19854375,0.10479375,0.10479375)`. Contrast 2 and saturation 0 predict gray 0.2452; premature clamping incorrectly predicts 0.2126. Opposite tint orders and positive sigma-1 normalized seven-tap Gaussian provide full-plate witnesses. Floating raster tests prove partial/zero alpha directly.

With configured FFmpeg, FFprobe and bundled font, run:

```sh
cargo test -p opencut-editor-core --test parameterized_effects_native -- --nocapture
cargo build -p opencut-headless
bun run --cwd apps/agent-bridge test:unit --no-file-parallelism tests/parameterized-effect-native.test.ts
```

These supplement unchanged ordered-effect witnesses. Raw PAM and independently converted PNG differ by at most one byte; encoded range/export SSIM is at least 0.99, decoded real-audio PCM remains identical across lifecycle states, RMS error is at most 0.0001 and alignment is within one frame. Native tests cover frame 0/200/600 ms, range beginning200 ms, draft isolation/commit, undo/redo/reopen and component/ancestor clocks.
