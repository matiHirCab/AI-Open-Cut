## Context
The reviewed main is c4a9625cc56e6c1476b0fc64512852259f3f0b58 (PR222 merged). The normal edit and batch path calls load_project_data, which publishes migration before apply_operation/apply_edit_batch. Existing edit_with_staged_migration already stages core/domain/resource preparation, applies the complete candidate, then publishes project/history/draft updates under the lock and established recoverable journal.

## Goals / Non-Goals
Preserve authoritative project/history bytes and revision on all rejected legacy slot-definition and instance-update edits, including batches that fail late; retain valid migration/edit and recovery behavior. No public contract/schema change or #207 implementation belongs here.

## Decisions
Classify ComponentDefineSlots and ComponentInstanceUpdate as requiring existing composition staging. Reuse the established shared classifier rather than adding a parallel migration or validation implementation. Its draft callers must retain their existing typed behavior and staged resource ownership; exercise relevant suites. An eager migration with rollback was rejected because it creates two publications and needs new rollback semantics. Changing all loaders was rejected as broader than this defect.

Regression fixtures use a genuine schema44 component with no graphic color slots and an existing instance; both retained stacks are genuine schema44. Tests never call a migrating read between capturing bytes and submitting the rejection. Verify definition target ITEM_NOT_FOUND, unknown instance slot ITEM_NOT_FOUND, and a late failed batch after a valid earlier edit. Success cases verify one revision, all snapshots migrated to45, exact settings and stable repeated reopen. Existing commit-point fault injection must preserve original precommit bytes or recover the complete committed target for each route, without duplicate application or managed resource leakage. Existing journal recovery before a request remains authoritative and is not undone.

## Risks / Trade-offs
The shared classifier also routes slot-containing drafts to existing staged draft paths; retain existing aliases, locks, source matching, assets/fonts, error codes and history behavior through their suites. No policy, timeout, fixture, tolerance or renderer relaxation is permitted.

## Verification
First reproduce the three regressions against unchanged main. Then run focused regression/fault/recovery tests, Rust format, strict workspace Clippy/workspace tests, bridge type/lint/unit/contracts/integration/packaged smoke, root policy/agent-context and relevant hermetic/native/platform checks. Record full uncommitted logs and exact limitations. Run strict all-OpenSpec and prearchive protected Moon (only active-change rejection expected), openspec-verify-change, sync/archive only this change, then final protected validation. Commit/push/open a separate draft PR only after local verification and require exact-head protected CI; no merge/deploy.

## Approval
The user explicitly approved the scoped atomicity correction on 2026-10-15. Delegation additionally authorizes issue-scoped specification approval, implementation, verification and draft delivery. The agent approves these bounded artifacts under that instruction; no broader behavior is approved.


## Verified packaging bootstrap amendment

Windows full integration/package attempts retain the same two failures from explicitly Unix shell-provider fixtures. Ubuntu WSL passes the complete unchanged integration suite42/42. Pinned native Bun1.4.0 in this WSL environment fails ELF ftruncate with EACCES, independently reproduced with a one-line program on both DrvFS and an owned ext4 filesystem, using identical native Bun bytes and the official standard build. This is retained as installed compiler readiness evidence, not a passing bootstrap or a source-code finding.

Use the supported Bun1.4.0 Windows-to-Linux compiler route for the unchanged bridge sources, the native Rust1.97.0 release headless built from the scoped patch, and the repository's unchanged assembleRuntimePackage/verifyRuntimePackage functions. Execute the original complete vitest.smoke.config.ts suite against that assembled package. This preserves every39 assertion, timeout, fixture, protocol and package integrity check. The full39/39 suite passes. No compiler, test, CI, policy or rendering source is patched; the temporary verification driver is outside the shipping checkout. The native build script and all protected aggregate jobs remain mandatory at the exact subsequently published head. The existing delegated issue-scoped approval covers this bounded verification-method amendment; no gate or acceptance requirement is waived.
