## 1. Approval and reproduction

- [x] 1.1 Recheck PR131 head, inspect reported CI failure logs, create isolated correction worktree and planning artifacts.
- [x] 1.2 Record explicit reviewer approval of these complete issue43 artifacts, delegated from the user's OpenSpec authority and subsequent instruction to correct PR131 remotely (2026-10-01).
- [x] 1.3 Reproduce strict Clippy and all three native failures against unchanged base with relevant supported backend/version and concurrency evidence.

## 2. Bounded corrections

- [x] 2.1 Correct RGBA fixed-width byte grouping without semantic changes or warning suppression.
- [x] 2.2 Correct reproduced identity color and nested/compound intent parity at the canonical renderer owner; preserve all independent expectations and thresholds.
- [x] 2.3 Add focused red/green regression evidence, documentation and accurately mapped verification.

## 3. Verification and delivery

- [x] 3.1 Run cargo fmt --check --all; cargo clippy --workspace --all-targets -- -D warnings; cargo test --workspace.
- [x] 3.2 Run actual failed animation_channels native suite, relevant extended_visual_animation fixtures, reviewed goldens and instrumented raster-cache checks with required FFmpeg/FFprobe/font flags.
- [x] 3.3 Run bridge contracts:check, typecheck, lint, unit, MCP integration and packaged smoke; use the documented hermetic Python worker runner where affected.
- [x] 3.4 Verify OpenSpec conformance, protected pre-archive gate and strict validation; sync/archive only after every required implementation check passes, then pass final protected gate.
- [x] 3.5 Prepare the scoped corrective commits and rechecked-head publication payload, plus a separate delivery record for the approved ordinary push, concise PR evidence and exact-head terminal CI.

## Post-archive delivery follow-up

The actual push, PR evidence comment and exact-head terminal CI remain mandatory
for the delegated task and are tracked in the delivery record. They follow the
protected final gate; preparing that record never claims publication or CI success.
This ordering clarification keeps release actions separate from the implementation
checks that must finish before specification archival.
