## Context

Main84's `render_process::run_command` discards `Command::spawn`'s `std::io::Error`. Its process executor and renderer facade already own cleanup and publication; headless/bridge only translate errors. Readiness failures and visual decoder/preparation failures have separate existing semantics. Issue178 and comments were refreshed before design; there are no comments or declared dependencies. PR197's 18/18 green head25e5175b is a pending overlapping diagnostic lane, preserved separately.

## Goals / Non-Goals

**Goals:** preserve sanitized OS startup classification/native code, compatible core/headless/MCP forwarding, deterministic failure/cleanup/privacy/platform evidence and exact contract governance.

**Non-Goals:** changing readiness, pipe/child-exit errors, decoder behavior, progress, budgets, retryability, launch policy, persisted state, artifact publication or desktop/video environments; attributing the historical Windows failure to path length.

## Decisions

1. Core error primitives own a typed optional `ffmpegSpawn` value: `kind`, `nativeCode` (signed32 integer or null), `platform` (`linux`, `macos`, `windows`, `other`). `kind` is a closed normalized subset of `std::io::ErrorKind`: `not_found`, `permission_denied`, `invalid_input`, `invalid_filename`, `argument_list_too_long`, `resource_busy`, `out_of_memory`, `other`; every unlisted kind maps to `other`. Capture the actual producing target and raw_os_error; never parse localized strings or interpret an OS code as a child exit. Box optional metadata if needed to retain the existing error-size boundary. Existing constructors set it absent. Raw Display/Debug/source text is never serialized. Alternatives of raw OS messages or guessing path length were rejected because they expose private content or invent causality.
2. Attach metadata only where the actual final `Command::spawn` returns Err. Missing pipes remain their existing generic spawn-stage errors; child failures retain existing exit/stderr detail. Keep FFMPEG_FAILED/spawn/retryablefalse/message unchanged. Readiness still reports DEPENDENCY_UNAVAILABLE and revision/validation errors retain precedence. No new dependency edge is required: render_process already consumes core error primitives.
3. Transport serde skips absent metadata; headless standalone/error status/worker use the same typed error conversion. Bridge Zod owns only wire validation, with optional closed strict metadata and signed32 bounds; BridgeError/errorBody/jobs forward it. Do not duplicate process classification in adapters or add runtime test-only launch controls. Existing requests, versions and persisted schema44 are unchanged. Optional detail is additive under ADR0002.
4. Manually govern a new `contracts/ffmpeg-spawn-diagnostics-v1.json` witness/schema, ownership/CODEOWNERS and headless/MCP catalog additions. Every listed consumer reads the witness in the unchanged canonical command. An exact inverse addition projection pins main84 predecessor hashes and preserves all older projections and negative controls. No live-schema catalog generator, unrelated drift masking or golden/reference update.
5. Verify actual run_command missing-executable failure, a nonexecutable file on supported platforms, injected PermissionDenied and host-specific raw codes, hostile custom error text, fallback/no-native-code and unchanged child failure. Existing renderer adapters inject the typed failure to verify real facade cleanup/no-publication for frame/range/export/draft and destination/state/history safety; these controls are not native media success evidence. Headless actual typed serialization and persistent error transport plus source/fresh packaged MCP fixtures verify exact metadata retention. Positive controls prove adapters/markers execute; existing actual native suites provide success/render regressions separately.
6. Own only ffmpegSpawn sections. PR197 owns renderBudget and PR196 owns graphic color slots. Before publication re-fetch main, preserve all merged declarations, and use a disposable exact-head union with PR197 for overlapping error/schema/catalog changes. Record conflicts, manually reconciled semantic union, digest, complete contract gate, and independent parent review separately from human CODEOWNER review. Do not copy a pending lane into this publishing branch without an approved dependency. Existing constructor conflicts and catalog projection composition require semantic review even after a textual merge.

## Risks / Trade-offs

- OS code/kind meanings differ by target → closed producing platform plus host-specific tests; classify through std I/O, no foreign-code guess.
- Error text/private arguments leak → metadata contains only enums and signed32/null, explicit hostile-string and unknown-field controls.
- Readiness hides intended spawn path → actual process unit/renderer adapter tests prove final-start failure; distinguish transport fixtures from native evidence.
- Optional metadata drifts between IPC/job/MCP → canonical witness consumed by every surface, exact predecessor proofs and malformed negatives.
- Original Windows root association is unverified → document source-confirmed diagnostic loss only; private original reproduction and path-length cause remain unavailable. Any later causal claim requires an authorized controlled Windows comparison.
- Pending diagnostic overlap → preserve PR197 and exercise exact-head disposable union/review before publishing178. Do not weaken gates or erase predecessor evidence.

## Migration Plan

No persisted migration or protocol major change. Existing clients can ignore the optional field. Rollback removes only this additive field and its governed declarations. No merge/deployment occurs in this task.

## Open Questions

None blocking the scoped diagnostic fix. Original private-project causality is explicitly outside the available evidence, not silently assumed.
