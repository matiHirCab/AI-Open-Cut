# Complete reference scene conformance

## Scope and mapping
The recipe adds test-owned data, fixture helpers/example, tests and documentation only. No production, public/provider/persisted contract, protocol1, schema44 or renderer change. Existing #58/#70 independent references remain untouched. The rescue-video proposal and fixture deduplication are separate.

| Requirement/scenario | Implementation and automated proof |
| --- | --- |
| Complete ten-group recipe / F1 | `contracts/complete-reference-scene-v1.json`67 operations; `tests/support/reference_scene.rs` existing typed standalone/batch authoring; core `complete_reference_standalone_and_atomic_authoring_witnesses`; canonical TypeScript DTO parity |
| Independent completeness / F2 | core `complete_reference_coverage_rejects_removed_groups_operations_overrides_and_bindings` closes ten group names/aliases and required authored fields/operations; group deletion, slot/timing/event/blend mutations fail |
| Authoritative lifecycle / F3 | core `complete_reference_failures_history_and_reopen_preserve_all_content` plus actual source/compiled MCP `reference-scene-native.test.ts`; invalid slot, missing reference, stale and late rollback preserve authoritative files/resources; duplicate overrides, parent edit, exact undo/redo/reopen with revision/time separately checked |
| Native audiovisual evidence / F4 | actual source/compiled MCP with genuine FFmpeg/FFprobe/fixed DejaVu family; frame650, full6sec A/V review, cold/warm540p/720p cache, final export, actual mix analysis and restored/reopened frame byte equality |
| Fail closed / F5 | required missing font/tool invocations fail; independent literal voice windows/gaps and gold pixel negative controls fail; Python strict synthetic protocol/source tests; changed-witness core controls fail |

The synthetic worker returns the unchanged synthesis DTO without timestamps. The existing marker operation receives explicitly estimated recipe alignment; this is not measured speech-model accuracy. Core fixture provenance additionally records synthetic alignment. Presets explicitly clear legacy transform2d where existing validation requires it; that validation is unchanged.

## Passing local checks
- Final focused core3 tests at final recipe/helper inputs: `/tmp/mg77-reference-delivery.log`, exit0,603.66s.
- Rust workspace captured run: `/tmp/mg77-workspace.log/.exit0`,1505 passed/0 failed/10 intentional configured-native ignores,74 result blocks. Its compiled reference fixture preceded final blend/helper refinements; final focused3-test evidence covers those changed inputs. Other workspace inputs and pinned toolchain/environment are unchanged. Captured reference run754.80s is not a performance baseline.
- Final Rust formatting and strict workspace Clippy all targets/all features: `/tmp/mg77-fmt-final.log`, `/tmp/mg77-clippy-final.log`, `/tmp/mg77-rust-style-final.exit0`.
- Typecheck and lint210 files: `/tmp/mg77-type-native-corrected.log`, `/tmp/mg77-lint-native-corrected.log`, successful set-e progression into required native checks.
- Hermetic bridge unit768 passed/13 optional skips: `/tmp/mg77-bridge-unit.log`, `/tmp/mg77-bridge-checks.exit0`. Eleven predecessor optional skips and two new configured-native skips are explicit; final modified opt-in bodies are separately executed below. Unchanged default-unit inputs reuse that evidence.
- New Python fixture3 passed: `/tmp/mg77-python-reference.log/.exit0`; unchanged predecessor provider suites reuse recorded #73/#75 evidence for unchanged files/environment.
- Full contract command: `/tmp/mg77-contracts-retry.log/.exit0`, all Rust parity plus605 TypeScript tests/38 files. Same command/deadlines/assertions under four-CPU affinity to match local quota and avoid competing native renders.
- Real source MCP integration34 passed: `/tmp/mg77-integration.log/.exit0`; assembled default-runtime fake-provider smoke31 passed: `/tmp/mg77-smoke.log/.exit0`.
- Required actual source+compiled MCP native checks2 passed/0 skipped: `/tmp/mg77-native-corrected.log/.exit0`,909.09s. Preserved instrumented `/workspace/toolchain/opencut-headless-reference` avoids unrelated default Cargo builds overwriting the private cache-test binary. This configured bridge evidence is distinct from assembled default-runtime platform acceptance owned by #78.

## Actual media observations
Committed original media and both unmodified reports are in `docs/verification/complete-reference-scene/`; artifact checksums and interpretation limits are in its README. Separate original source/compiled reports are `/workspace/mg77-native-evidence/{source,compiled}/evidence.json`; original MP4s in the same folders. Both198778B exports SHA256 `e5915d227a6d860fecd09d06f30186478ae18b8b7868a1e233767cc5ca1f1b6a`:192x108 H264,60 frames/10fps/6s, stereo48kHz AAC. Matched range/export SSIM1, aligned floatPCM RMS0/sample offset0, retained tolerances>=0.99/<=0.0001/one frame. These are equivalence comparisons, not an invented independent full-frame golden.

Both workflows observed cache(hit,miss,final execution) triples [(0,1,1),(0,2,2),(0,3,3),(1,3,3),(1,4,4),(2,4,4)] and distinct immutable artifact paths/identical warm bytes. Literal gold RGB255/203/0 and independently declared six voice windows/zero silence gaps pass, with black/missing-signal negative controls failing. Actual mix analysis reports-23.99LUFS/-18.38dBTP,288000 frames and64 bins. Retained source frame650 original SHA256 `d745cbd7f717af55b51d081a0724a83b9671c763aa58a40983ddeafb2a086a85` was visually inspected (three cards, diagonal grid, outlined red EVERY, radar and gold decoration). No listening/intelligibility, speech-model quality, performance baseline or Windows/macOS scene-rendering claim.

## Preserved failed evidence and corrections
Initial core assertion used a nonexistent serialized preset field; corrected to existing animationPresetProvenance channel keys. Early native setup failures (Python path, strict voice ID, confined font roots/family) were corrected in the test harness. Initial analysis call omitted required waveformBins; corrected to existing field64. These failed logs remain `/tmp/mg77-*`; no production contract, tolerance, deadline or validation was relaxed.

Initial full contract run `/tmp/mg77-contracts.log` failed8 original deadlines while full Rust/native/Vitest competed for a four-CPU quota. Retry passes all605 with no assertion/deadline change. Required missing-font/tool negative commands exit1 (two cases fail), `/tmp/mg77-missing-font-negative.log` and `/tmp/mg77-missing-tool-negative.log`; their outer negative controls pass and are not claimed as media acceptance passes.

## Review and delivery boundary
Completeness/correctness/coherence reviewed against all3 requirements/5 scenarios; implementation tasks1–4 complete. No domain validation duplication, schema migration or new public capability is needed. Prearchive protected gate `/tmp/mg77-prearchive-gate.log/.exit1` validates60 strict items and rejects ONLY this active change, as expected; this is not final gate success. All11 standard CI jobs and separate focused Windows job pass on predecessor #170 exact headfd2124113487ce4a2831f83ab95ddb66d9ec4e34, including full native Windows descendant proof and foundation. The issue77 branch fast-forward includes that verified lineage; tracked diff and every untracked-file hash were verified preserved. Affected fmt/Clippy/headless checks are refreshed after the lineage update, followed by sync/archive, protected/independent strict gates and publication. The standing delegated authority covers bounded approval/sync/archive/push/draftPR without further prompts. No merge/deploy.

## Final conformance and sync decision
Completeness:5/5 implementation tasks,3/3 requirements and5/5 scenarios covered; correctness/coherence:0 critical issues,0 implementation warnings. Authorized delivery operations are recorded separately to avoid a self-referential pre-archive publication checkbox. Refreshed lineage formatting/strict Clippy/all84 headless tests pass, `/tmp/mg77-lineage-rust.exit0` with separate logs. Prearchive gate after verified lineage validates60 items and rejects ONLY complete-reference-scene (`/tmp/mg77-prearchive-lineage.log/.exit1`).

Sync assessment: add new living complete-reference-scene capability with its3 requirements/5 scenarios; no existing living requirements or frozen references removed/changed. Standing explicit delegated authority approves synchronization and archival. Postarchive protected/strict gates and publication follow; remote new-head CI remains a draft delivery check, not an asserted pass.

Final postarchive protected gate `/tmp/mg77-final-gate.log/.exit0` and independent strict validation `/tmp/mg77-strict-final.log/.exit0` pass with60 living specs. The synchronized archive has5/5 completed implementation tasks. Draft publication to main follows these passed gates; its new-head remote checks and independent review remain explicitly pending.
