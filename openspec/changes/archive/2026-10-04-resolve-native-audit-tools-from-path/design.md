## Context

Hosted render CI intentionally sets OPENCUT_FFMPEG_PATH=ffmpeg and OPENCUT_FFPROBE_PATH=ffprobe. At c030, the requested-origin helper asserts filesystem existence for these command names before any render; actual CI reports120 passes and6 helper failures. The existing Transform2D native helper reads the configured font and calls Renderer::readiness(), which invokes configured commands through standard process PATH resolution and validates required renderer support.

## Goals / Non-Goals

Accept usable explicitly configured commands and paths, fail closed on unusable tools/fonts, retain all six independent native controls unchanged, and obtain actual amended-head CI success. No production, workflow, schema, codec, fixture-equation, budget, skip-policy or dependency change.

## Decisions

Extract only a small supplied-config readiness helper returning a result. Preserve three configured PathBuf values exactly; read the font as an actual readable file, then invoke Renderer::readiness(). The existing optional/required environment branch remains byte-identical. Tools() unwraps the result with a concrete diagnostic before returning the original paths. Focused tests pass explicit nonexistent tool paths and missing font paths without mutating environment; actual required-suite execution uses bare ffmpeg/ffprobe to prove success.

Using absolute path discovery or modifying CI was rejected because it would replace already-supported configuration and fail to test the public renderer's existing command semantics. Removing dependency checks was rejected because required conformance must fail closed.

## Verification and input reuse

Run fresh formatting, strict workspace Clippy, full required animation_channels with both required flags and bare commands (all existing126 plus new focused negatives), and full canonical contracts because its selected animation integration input changed. Bind the new manifest to c030's342-file snapshot with only this one executable input changed; verify all production/native non-animation test, TypeScript/Python/MCP/package/golden/rules/cache input hashes and tool/environment identities. Reuse their actual passed evidence only after both independent reviewers accept that unchanged input closure. Any changed relevant input invalidates reuse and requires the affected full gate. Obtain independent source/conformance acceptance, formal verification, expected named-change-only prearchive policy rejection, sync/archive, and actual final strict/protected PASS. Publish an ordinary follow-up commit on the same draft branch; preserve c030 bundle and failed CI logs, generate a separate amended bundle, then require all11 checks on the amended exact head. No merge/deploy.

## Risks / Trade-offs

Readiness invokes processes once per native case, matching the existing native helper and remaining within unchanged suite budgets. Required-mode absent/partial configuration behavior remains unchanged. Negative controls must reject both missing FFmpeg and missing FFprobe; unreadable font failure precedes tool execution. Production/static input identity can justify reuse but cannot justify reusing the affected animation gate.

## Compatibility / Migration / Open Questions

Test-only correction; no public contract, persistence or migration change. No open semantic question. CODEOWNER human review remains separately pending; actual delegated approvals are recorded honestly.
