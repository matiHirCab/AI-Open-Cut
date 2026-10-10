# Recorded Linux platform-renderer evidence

Original observed artifacts from the final required source/default-packaged native driver. The actual JSON report requires both named cases to pass with zero skips. Worker files are the original package sources, and the release headless/compiled bridge are the default shipped binaries; no private cache instrumentation is substituted.

[Video](portable.mp4) · [Frame at500ms](frame500.png) · [Source report](source-evidence.json) · [Package report](packaged-evidence.json) · [Execution report](native-results.json) · [Actual package manifest](manifest.json)

Both clients produced identical sampled export/frame bytes, so one original sample is retained alongside both unmodified reports. Reports include original managed UUID/temp paths that were subsequently cleaned. These observations are neither an independent full-frame golden nor the #70/#77 complete fixture or proposed rescue-video benchmark.

Actual192x108/10fps/10frames/one-second H264 and stereo48kAAC; SSIM0.998298 and floatPCM RMS0 for each matched range/export comparison. Independent solid-color/black and nonzero-audio controls, invalid/missing/stale input, history/fresh reopen and four dependency fallback lifecycles are required in the two passing cases. Local Linux does not establish Windows/macOS acceptance, model/listening quality, desktop GUI or performance.

| Artifact | Bytes | SHA256 |
| --- | ---: | --- |
| `frame500.png` | 1962 | `be1b081e9d24f9907645ab9b40b0eb4e04ab9a3cdbaf4057b770875f86a8f53d` |
| `manifest.json` | 680 | `0e8a73aea311e9f11d7716afe9ae17d472531a450e7a0acad3ebb13077b93836` |
| `native-results.json` | 1631 | `21831be56a0335714709795a9ceea3cd0f1201315cc4598b58bc96cf85b7b7fc` |
| `packaged-evidence.json` | 64630 | `b8f6613de391c6eb8eafb4a0943e77254354a1d9f527754585fc698471f1ab2f` |
| `portable.mp4` | 19604 | `1b63a7fa1831645be9f2b8f8c4a4d768092674b6a5c2c86d920b9ff005cfc16a` |
| `source-evidence.json` | 64543 | `77d76254192e82d22c1699917f771b634382c7f68ba2d9fcb7e1b858cdbbfb10` |

Reproduce using [the documented mandatory driver](../../platform-renderer-packaging.md). Exact-head three-platform and prior required CI must finish successfully before acceptance.
