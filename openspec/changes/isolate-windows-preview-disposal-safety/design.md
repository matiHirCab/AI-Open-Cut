## Context

The clean saved checkout initially contained main predecessor `ae0a6cbc`. Git fetch independently confirmed main remains `8cb21a6b`; branch `fix/windows-preview-disposal-safety` starts there. The disposer and failing fixture are unchanged between these revisions. Root instructions, README, OpenSpec workflow, active-change inventory (empty before this work), artifact-resources L4–L10, native implementation and prior disposal design were read.

The authenticated GitHub connector fetched Windows job119176954562: source acceptance passed in435759ms; packaged acceptance failed in770ms at the outside-sentinel read, after twelve disposal calls. FFmpeg7.1.1 setup passed. Windows Server2025, image20261007.264.1, Rust1.97.0 and Bun1.4.0 appear in the log. The actual test-process runtime/version must still be recorded; Bun orchestrating Vitest does not establish that Vitest uses Bun rather than Node.

The race starts before sentinel publication, renames previews, creates a junction targeting the fixture root, recursively removes that junction, then restores previews. Native disposal opens root/project/previews without delete sharing, refuses reparse points and uses SetFileInformationByHandle on the target. These facts identify competing deletion paths but do not establish causality. An earlier occurrence recorded by PR229 independently predates the narration merge. PR229 and PR228 are the only open PRs; neither is modified here. #208 remains untouched.

## Goals / Non-Goals

**Goals:** produce attributable Windows evidence, retain outside preservation and bounded native cleanup assertions, and change only an evidenced fixture/runtime-sensitive removal if diagnosed.

**Non-goals:** production/native behavioral changes without revised approval, dependency upgrades, broad filesystem refactoring, CI/secret/permission changes, skipped assertions, merges, deployments, or other issues.

## Decisions

### Separate controls before changing deletion

Use fresh independent temporary fixtures for (1) junction creation/removal with no native process, (2) ordinary native disposal without racing and (3) the existing source/package race. Record sentinel preservation at phase boundaries, replacement count, expected sharing refusals, actual executable identity and process runtime. Use controlled handoffs where needed so the diagnostic can identify the first destructive operation instead of merely detecting a missing sentinel at the end. Publish the sentinel before starting mutation.

Keep the actual native safety race and all twelve calls. A fixture-only failure implicates setup/teardown or its runtime; a native-only failure implicates native disposal; an overlap-only failure needs finer controlled evidence. A passing Linux or Windows run alone cannot attribute an intermittent failure. A zero-replacement race must be identified rather than silently accepted as replacement coverage. Expected Windows refusal while held handles are open remains legitimate evidence, separate from observed replacement between calls.

Alternative rejected: replace rm immediately and infer the cause from green CI. That can conceal native loss. Another rejected alternative is adding retries, skipping Windows, or increasing timeouts.

### Conditional fixture correction

Only after decisive Windows evidence identifies junction teardown as destructive, use nonrecursive removal appropriate to the link type (Windows directory junction versus Unix directory symlink), without target traversal. Apply it consistently to the fixture's race and static/debt reparse cases. Verify the target sentinel and directory remain unchanged and restore the original previews directory before further operations. Teardown failures must remain visible; never silently fall back to recursive deletion. Bounded retries, if genuinely needed for observed Windows sharing errors, must apply only to link removal, never semantic assertions or native disposal.

No helper API or precise syscall is chosen before runtime evidence. Core continues owning production semantics. A native defect requires a concrete amended design, regression plan and approval.

### Keep scope and lifecycle honest

No public, persisted or cross-language contract changes; no schema migration. Diagnostics contain synthetic fixture information only and remain outside production responses. Rollback is the scoped private-test change. The active specification stays unarchived until implementation, required checks and conformance pass. No check is labeled passed solely because the active-change policy rejection was expected.

## Risks / Trade-offs

- Linux cannot execute Windows junction/sharing semantics: exact-head Windows source and package evidence is required.
- Diagnostics may affect interleavings: retain the adversarial race alongside deterministic controls.
- Fixture final recursive cleanup could mask an earlier failure or traverse remaining links: explicitly remove owned links before final tree cleanup and preserve the primary failure.
- Native evidence uploads currently occur only after test success: inspect logs and retain diagnostic results in the test output; do not alter workflow security/configuration to publish them.

## Open Questions

- Which exact operation removes the sentinel on Windows, and under which test runtime/version?
- Does fixture-only junction teardown reproduce the loss, or is native overlap required?
- Can an existing supported CI execution provide the controls without changing workflows?

## Approval

Explicit delegated reviewer approval granted on2026-10-21 for review diff SHA2562d46814469057528303e89cb32695e044f7e38f7f1d3614ae2e487db49a0747e in parent thread01a1583b-f8d9-70ae-8c32-75210bced0e5. Locally checked diagnostics may be published as an investigation draft with Windows verification pending, while the active specification remains unarchived and its policy rejection remains expected. Production changes still require amended design and review.
