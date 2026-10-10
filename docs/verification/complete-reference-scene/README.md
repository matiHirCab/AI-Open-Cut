# Recorded complete reference-scene evidence

These are original observed Linux renderer outputs from required actual source/compiled MCP checks. The six-second fixture demonstrates the approved ten-capability scene; it is separate from the proposed rescue-video creative benchmark. Synthetic tones and explicitly estimated alignment prove reproducible cue timing, not speech-model accuracy or listening quality.

[Reference video](reference.mp4) · [Original frame at650ms](frame650.png) · [Source report](source-native-evidence.json) · [Compiled report](compiled-native-evidence.json)

Both workflow exports have the same recorded MP4 checksum, so this directory keeps one original sample plus both unmodified reports. Reports retain original managed UUID paths and temporary probe filenames as observed; those temporary roots were subsequently cleaned. This output is recorded evidence, not an independent full-frame golden or a replacement for existing #58/#70 references. Reproduce with [the documented native commands](../../complete-reference-scene.md).

| Artifact | Bytes | SHA256 |
| --- | ---: | --- |
| `compiled-native-evidence.json` | 23738 | `ceb7f2c990e7ad38aa3722e707525f5efb145e860697b3b38ef7cc954f1ffd47` |
| `frame650.png` | 11400 | `d745cbd7f717af55b51d081a0724a83b9671c763aa58a40983ddeafb2a086a85` |
| `reference.mp4` | 198778 | `e5915d227a6d860fecd09d06f30186478ae18b8b7868a1e233767cc5ca1f1b6a` |
| `source-native-evidence.json` | 23738 | `a6e0891b075c6d5aae4ec84e04be59324b1b3fecdb211d4e73f8c761a99c31bb` |

Actual streams:192x108 H264/10fps/60frames/6sec; stereo48kHz AAC. Both workflows retain SSIM1/floatPCM RMS0/sample offset0 at matched requested settings, independent gold/voice-window/gap positive and negative controls, cache reuse and history/fresh-reopen checks. Mix analysis:-23.99LUFS/-18.38dBTP/64bins. No cross-platform rendering, performance or creative-quality claim is inferred from these local observations.
