## Decisions

Use an exhaustive command match in `Player::send`: forward Pause and Loop payloads unchanged and keep the existing bounded Seek clamp. This gives the common production controller ownership of all typed payloads; a wildcard merely forwarding the enum leaves the pause field unread when native Linux code is excluded. Do not add warning allowances, dummy field reads or conditional suppression.

Extend the existing injected-backend scenario with Playing → Paused and loop on → off, observing real controller state instead of testing source strings. Existing native playback proves actual video/audio/pause/seek/loop. Unsupported platforms remain unsupported; only their compilation and injected controller paths are claimed until exact-head platform CI passes.

## Verification and reuse boundaries

Run Rust format, workspace-wide strict Clippy/all targets and all desktop unit tests on this follow-up, then the explicit real native playback test. Reuse the successful full-workspace evidence only for unchanged editor-core/headless targets and the successful TS/contract/integration/packaged/provider evidence only for their unchanged inputs/environment. Record this composed target coverage explicitly rather than claiming a second fresh whole-workspace execution. Backend ABI/native sampling, GUI/event code, fixtures and their prior genuine GUI observations are unchanged; the changed command-admission path receives fresh controller and native coverage. Any further implementation change invalidating those inputs requires affected checks again.

Required OpenSpec prearchive rejection must name only this active change. Verify conformance, synchronize/archive only this follow-up and pass final protected/strict gates. Push only the locally verified correction to the existing draft, then require fresh exact-head Linux/macOS/Windows and aggregate CI. Do not treat the first failed head as complete or weaken its checks.
