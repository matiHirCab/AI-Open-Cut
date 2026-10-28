# Scenario traceability

| Approved scenario | Executable coverage | Evidence |
| --- | --- | --- |
| Hermetic source and package | Mandatory existing source/package combined workflow; supplied fake FFmpeg; private hash-checked PNG/WAV probe; exact fake output; all common native/provenance/history/durable/reconnect assertions retained | Separate no-ambient interpreter-only PATH source/package proofs and aggregate suites, recorded in verification.md |
| Explicit genuine source and package | Same workflow with explicit genuine selector and absolute tools; independent64x64 RGBA decode and all common assertions retained | Separate genuine source/package runs in verification.md |
| Invalid configuration or fixture | artwork-durable-backends.test.ts three cases: missing/relative/nonexistent tools, invalid mode; supplied version delegate; empty PATH metadata; unknown/changed/malformed PNG checksum/header or WAV, oversize, foreign root, link; foreign bytes preserved | Focused and full unit evidence in verification.md |

Private implementation files are tests/artwork-durable-workflow.ts, tests/fixtures/artwork-durable-probe.mjs and tests/artwork-durable-backends.test.ts only. Runtime, canonical fixtures, dependencies, shared fake-media-tool.mjs, workflows, timeouts and historical archives remain byte-identical to fc48d575. The combined scenario retains its existing Linux-only registration; no new skip is added. Unit version delegation uses the actual interpreter executable on every platform, comparing its direct result to the delegated result, including propagated failure; it does not rely on Unix shebang execution.
