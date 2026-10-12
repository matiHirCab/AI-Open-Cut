# Native render admission diagnostics

Selected core resource guards return the existing non-retryable `INVALID_ARGUMENT` with an optional `renderBudget`. This means the requested work exceeded an intentional admission guard. It does not indicate an FFmpeg failure or renderer defect. Limits, retryability and artifact publication rules are unchanged.

`limit` and `work` are unsigned decimal strings, preserving all u64 digits through JSON and JavaScript. `work` describes the attempted accounting prefix at the rejecting guard, not complete scene cost. On checked arithmetic overflow it is null and `overflow` is true. Otherwise overflow is false. `measurement: charged` means the exact counter/formula charged by admission; `conservative` means an existing upper-bound allocation/work reservation. Neither measures actual RSS, elapsed time or observed execution cost.

| Budget | Units | Existing limit / meaning | Measurement |
| --- | --- | --- | --- |
| `shape_segments` | segments | Effective tessellation allowance, at most 65,536; aggregate owners can pass a smaller remaining allowance | charged |
| `scene_shape_segments` | segments | 1,048,576 across measured shape occurrences | charged |
| `scene_shape_raster_bytes` | bytes | 824,633,720,832; existing shape raster reservation (12 bytes/pixel, 44 for SVG children), not live composition memory | conservative |
| `candidate_analysis_nodes` | nodes | 65,536 shared candidate certification nodes; first unresolved excess attempts 65,537 | charged |
| `effect_pixel_passes` | pixel_passes | 268,435,456; existing padded surface/convolution/particle scan bounds | conservative |
| `motion_blur_pixel_work` | pixel_samples | 268,435,456; output pixels times charged shutter samples | charged |
| `composition_pixel_work` | pixel_samples | 268,435,456; output-frame pixels times charged composition samples | charged |
| `composition_bytes` | bytes | 1,073,741,824; existing linear composition and continuous mask composition reservations | conservative |

`contributors` contains at most eight anonymous `{kind, work}` entries (`shape`, `grid`, `svg`, `effect`, `motion_blur`). Their work uses the diagnostic's units. Shape aggregate accounting retains the largest eight encountered charges, descending by charge then kind. Other supported owners expose the current charge when available; otherwise the list is empty. The list is partial and never claims to add up to total work. A known rejected charge participates in selection; eight larger charges can exclude it. No source names, IDs, text, paths, source documents or commands are included.

Coverage is deliberately limited to the accounting above. Separate path/curve subdivision, grid clipping/outline, surface dimension, mask/repeater expansion, glyph and other guards keep their existing errors when they do not supply these reviewed accounting values. A missing field does not mean no resource guard exists. Headless standalone/status/worker, bridge jobs and MCP preserve core-origin diagnostics. Speech/transcription providers have no native accounting field. Ordinary errors omit it; malformed diagnostic payloads fail contract validation.

The canonical contract is [render-budget-diagnostics-v1.json](../contracts/render-budget-diagnostics-v1.json). This is an additive version-1 response field with no project schema migration, new operation or new error code. Existing preflight precedence remains effective: rejection occurs before artifact preparation/publication and does not modify project/revision/history/drafts or existing destinations. #179 owns progress; #177/#178 own decoder and spawn diagnostics. The [illustrated benchmark](illustrated-scene-benchmark.md) remains the unchanged healthy workload control.
