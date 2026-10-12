## Why

Issue #178 reports final FFmpeg startup failures whose OS error is discarded. Current main independently confirms this loss, preventing clients from distinguishing missing executables, access errors and other process-creation failures.

## What Changes

- Add optional, bounded `ffmpegSpawn` metadata to actual final FFmpeg process-start failures, containing a closed error kind, native OS code or null, and producing platform.
- Preserve `FFMPEG_FAILED`, stage `spawn`, non-retryability and null child exit/stderr diagnostics. Forward the typed metadata through standalone and persistent headless transport and MCP errors/jobs.
- Govern canonical fixtures, consumers, exact predecessor projections and privacy/cleanup/platform tests in the same change.

## Capabilities

### New Capabilities

- `ffmpeg-spawn-diagnostics`: sanitized final process-start diagnostics with compatible transport, cleanup and platform evidence.

### Modified Capabilities

None. Existing renderer readiness, failure/publication and contract-governance requirements remain applicable and unchanged; the new capability adds optional diagnostic detail.

## Impact

Core error primitives and final process executor; headless error serialization; bridge error schema/translation/job forwarding; canonical contract catalogs, ownership/CODEOWNERS and tests/docs. No project/protocol major version change, persisted schema change, migration, new operation or dependency edge is intended. Current base is main `84da152fac0378396099d8f01fc3b7339690361f`; issue #178 is open with no comments/dependencies or duplicate implementing PR found. PR197 is separately preserved at its verified 18/18 green head and is not stacked into this branch.

## Non-goals

No path-length root-cause claim, path workaround, retry, process-launch policy change, decoder/read failure investigation (#177), preparation progress (#179), budget changes (#186), desktop/video task access, merge or deployment. The original private media/project and installed-build commit are unavailable; Windows cross-platform adapter tests cannot establish the original project's cause. An authorized Windows root comparison is required before any future path-length claim.
