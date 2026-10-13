# Final FFmpeg startup diagnostics

When the final FFmpeg process cannot be created, OpenCut preserves an optional `ffmpegSpawn` object through core, standalone and persistent headless errors, MCP tool errors and failed jobs. The existing identity remains `FFMPEG_FAILED`, stage `spawn`, message `FFmpeg render failed`, `retryable:false`. `ffmpegExitCode` and `ffmpegStderrExcerpt` remain null because no child exists.

```json
{"kind":"permission_denied","nativeCode":13,"platform":"linux"}
```

`kind` is a closed normalized classification from Rust's I/O error kind: `not_found`, `permission_denied`, `invalid_input`, `invalid_filename`, `argument_list_too_long`, `resource_busy`, `out_of_memory`, or `other`. Unlisted I/O kinds become `other`. `nativeCode` is the signed32 value returned by `std::io::Error::raw_os_error()`, or null when unavailable. It is an OS code, never a FFmpeg child exit code. Interpret it using the producing `platform`: `linux`, `macos`, `windows`, or `other`. Codes and classifications can differ by platform; null does not mean zero or success.

Only enums and a bounded integer/null are exposed. OS error Display/Debug/source strings, executable and output paths, command arguments, environment values and private project/media content are not retained in this metadata. No new automatic retry, path workaround or launch policy is introduced.

Coverage is the actual final `Command::spawn` failure. Missing pipes and unsuccessful children retain their established errors. Readiness still reports `DEPENDENCY_UNAVAILABLE`, and visual decoder/preparation diagnostics belong to their existing paths. Ordinary errors omit `ffmpegSpawn`; project schema44, protocol1, requests and stable codes/retryability remain unchanged. Invalid/stale work retains existing precedence, and startup failure preserves cleanup, state/history/drafts, destinations and no partial publication.

## Evidence and limits

Actual missing-executable tests run on the supported CI platforms; Unix also exercises an owned nonexecutable file. Deterministic native-code/custom-error tests and renderer process-adapter tests cover classification, privacy, cleanup and error precedence. Headless producer tests serialize the actual typed core error. Transport fixtures cover worker/one-shot/MCP job forwarding with execution markers; those fixtures do not demonstrate successful native media rendering. Separate existing real-media native gates provide rendering regression evidence.

Issue178's historical Windows longer-root failure and shorter-root success are an association. The original private project/media and installed-build commit are unavailable, and this change does not claim a path-length root cause or reproduce that private failure. Any future causal claim requires an authorized, controlled Windows comparison with identical bounded preview settings and captured OS diagnostics.

The governed wire authority is `contracts/ffmpeg-spawn-diagnostics-v1.json`, with ownership/CODEOWNERS and exact predecessor tests. Pending budget diagnostics are a separate optional field; the disposable integration check must preserve both fields and their historical contract controls before publication.
