## Context

Issue #43 was implemented and archived under `2026-09-30-render-extended-animation-channels`. A subsequent complete review independently passed the required local suites and reproduced five defects with unchanged production sources. The approved requirements already demand continuous safety, legacy anchor preservation, retained-draft validation, and exact sampling; this change makes the missing cases explicit and restores conformance.

Review evidence is retained outside the repository under `C:/Users/matia/AppData/Local/Temp/opencut43-review/`, principally `focused-results.log`, `focused-source.log`, `candidate-native-results.log`, and the stricter entirely-schema-26 `draft-history-results.log` and `draft-history-source.log`. These logs are diagnostic inputs, not substitute acceptance evidence for the fixes.

The owning layer is editor-core. Headless and MCP submit typed operations and translate core results. No outer-layer domain validator or new dependency edge is permitted. Preserve the preexisting uncommitted implementation and unrelated work.

## Goals / Non-Goals

**Goals:** Correct F1-F5; cover each with independent numeric, byte-preservation, or native-pixel oracles; preserve compatibility, budget accounting, and lifecycle semantics; verify and archive a synchronized correction.

**Non-Goals:** New channels/effects/targets, new public or persisted declarations, a schema bump, provider changes, alternative render semantics, golden regeneration, protected-gate changes, unrelated cleanup, commits, pushes, or PRs.

## Decisions

### F1: Continuous transform bounds apply to every extended occurrence

Remove rotation presence as the condition for checking local/inherited transform magnification. Reuse canonical curve bounds, affine composition, and existing raster/effect limits for every occurrence routed through extended evaluation, including effects-only work, hidden/retained items and expanded content. Account for local and ancestor scale extrema over reachable clocks before committing; endpoint values are insufficient for spring overshoot. Keep conservative rejection, canonical occurrence traversal, left-first subdivision, and the 65536-node final-candidate limit. Do not extend these new checks to legacy-only candidates, increase budgets, or replace interval analysis with timestamp enumeration.

The minimal regression uses a 1000x1000 legacy rectangle, a zero-amount vignette, and scale X/Y channels 1 to 3 over 500 ms with spring mass 1, stiffness 100, damping 1, initialVelocity 0. Edit publication currently succeeds while native preview at 150 ms fails the transformed raster limit. Correct behavior is pre-publication non-retryable INVALID_ARGUMENT and unchanged authoritative bytes/revision. Include a provably safe control and a nested/retained case to guard overbroad rejection and traversal omissions.

Alternatives rejected: bounding only rotation channels leaves the defect; checking only authored keyframes misses extrema; sampling every integer time is neither continuous nor bounded.

### F2: Derive extended legacy-shape affines in the original coordinate basis

When transform2d is absent, preserve the affine represented by legacy position, scale, opacity, and anchor, including the path's nonzero local origin and raster density. Use the core's existing legacy affine algebra and shape origin/bounds conventions. Do not silently replace the legacy origin with the geometric minimum or require a persisted Transform2D/anchor migration. Keep derived offsets process-local; do not serialize otherwise-invalid normalized anchors as a workaround.

Use an independent pixel oracle: a legacy-transform closed rectangular path from (10,20) to (30,40) on a 64x64 canvas occupies red pixels x=10..29 and y=20..39. Identity vignette amount 0 and zero-degree rotation must retain those bounds. Cover nonzero/negative origins, nonidentity legacy position/scale, and existing inherited transforms through canonical evaluated tests; compare frame, draft, range, and export with existing tolerances and preserve audio/stacking. Existing explicit-Transform2D fixtures remain valid controls.

Alternatives rejected: requiring users to convert legacy paths changes compatibility; zero-origin fixtures cannot establish origin preservation; updating goldens to accept the shift would hide the defect.

### F3: Retained-draft validity does not depend on extended fields

During schema-27 adoption under the project lock, validate version-2 draft operations and materializable candidates using the applicable retained base and existing font/resource checks even if both operations and base are legacy-only. An available matching retained base must be used rather than replaying a stale draft against an unrelated current revision. Preserve established stale-draft structural validation and REVISION_CONFLICT behavior when no applicable retained base is available; no new stale-draft replay or base-selection policy is authorized. If existing ownership/policy cannot satisfy these constraints, amend the plan before implementation.

Reject invalid targets with the existing typed error before generation publication, preserving project/history/draft/managed-resource bytes. Keep migrated current/history/draft publication atomic, source-schema rejection, recovery checkpoints, and deterministic no-write schema-27 reopen.

The regression creates a valid version-2 legacy trim draft, changes its target to `missing`, and sets current state plus every retained undo/redo snapshot to schema 26. Reopen must reject with ITEM_NOT_FOUND and no authoritative write; the present code succeeds and later draft access fails. Test valid stale drafts separately, including byte preservation and existing REVISION_CONFLICT, and retain migration fault-injection coverage.

Alternatives rejected: validate only extended drafts leaves the defect; replay every stale draft on current state would reject compatible stale drafts; fix only draft reads permits invalid migration publication.

### F4: Crop correlation must bound actual interpolated values

Use the correlated-sum shortcut only when the canonical scalar interpolation cannot clamp either component in a way that invalidates the sum bound. Otherwise include the clamp in conservative coupled bounds using the existing interval machinery. Preserve exact keyframes and hold samples below the positive interpolation floor; do not raise their authored values or change the floor of 0.000001. Retain acceptance of provably safe correlated crops and stable budget/error behavior.

The regression uses linear X 0.9999999 to 0.9999998 and width 0.0000001 to 0.0000002 over 500 ms, static Y=0/height=1. Authored endpoint sums equal 1, but the actual 250-ms interpolation yields X=0.99999985, width=0.000001, sum=1.00000085. Reject before publication with INVALID_ARGUMENT. Test analogous height/Y behavior, exact tiny endpoints, holds, safe correlated controls, and fractional inherited samples.

Alternatives rejected: raw endpoint sums miss the floor; clamping exact endpoints breaks the approved sampler; independently bounding everything can reject safe correlated cases unnecessarily, so retain the safe shortcut.

### F5: Keep integer clocks through extended segment/loop selection

Provide one canonical internal sample-selection path that preserves integer arithmetic for root clocks and other exactly integer timing paths. Select segments, exact keyframes, holds, and repeat/ping-pong seams without first casting absolute timestamps to f64. Convert integer differences only when computing interpolation progress. Preserve established fractional inherited-clock behavior through the canonical fractional path; do not floor/round inherited clocks or route them through an integer-only approximation. Share selection semantics across scalar, path-point, gradient-stop, and tint consumers and legacy transform scalars on extended items.

Use independent known-value fixtures at origin 9007199254740993 and adjacent timestamps, including a 0-to-100 four-ms rotation ramp: origin+1, +2, +3 must yield 25, 50, 75. Test exact endpoints, adjacent hold switches, finite/repeating/reflected loops, static fallbacks, compound component values, and near-u64-limit arithmetic without overflow. Exercise actual extended evaluated consumers, not just the existing integer helper. Preserve fractional inherited-clock fixtures with their established numeric tolerance.

Public timestamp representations and wire types are unchanged. Native large-u64 fixtures must not rely on lossy JavaScript number literals as an exact-value oracle. Native intent parity uses bounded normal windows; large-time evaluated/frame/window tests are used where supported, without attempting an enormous-duration export. Record any native platform limits separately from automated evaluated coverage.

Alternatives rejected: patching only the integer helper does not fix current consumers; converting all clocks to integers loses fractional timing; widening f64 comparisons or epsilon-matching timestamps cannot recover already-lost bits.

### Verification and truthful historical evidence

Convert the temporary repro cases to owned automated tests before declaring fixes complete, using analytic expected values and authoritative bytes/native pixels rather than duplicating production calculations. Maintain an F1-F5 requirement/scenario/test mapping in this change's verification record. New tests must fail on the reviewed implementation and pass after the corresponding correction.

Run all affected and repository-required checks with pinned tools and full external logs: Rust formatting, strict workspace Clippy, workspace tests; TS formatting/typecheck/lint/unit; canonical contract parity; MCP integration; packaged smoke; official hermetic Python runner; native animation/Transform2D/fonts/headless lifecycle; golden and report-mode conformance plus report validation; all three PR rules-screen shards; core and instrumented worker/bridge cache checks with default binary restoration. Preserve reviewed fonts/tool configuration, opt-in fail-closed flags, and existing goldens.

After checks, run the unchanged protected Moon gate and strict all-spec/change validation. Before archival, a gate rejection caused only by this active change is expected and must be reported as rejection, not success; other failures block archival. Use openspec-verify-change, resolve mismatches, synchronize/archive, then require final protected and strict all-spec validation to pass.

The original archive must receive a dated correction note linking this change's final verified record and identifying the five disproved guarantees. Preserve historical approvals/logs rather than rewriting them as though the first verification was correct. Never claim remote CI, CODEOWNER approval, or unavailable platform evidence from local results.

## Risks / Trade-offs

- More conservative transform/crop bounds can reject valid extended inputs: use safe controls and retained approved correlation/budget rules; do not change legacy-only behavior.
- Origin correction can disturb explicit Transform2D or density handling: test both transform representations and offset geometry independently.
- Unconditional draft replay can break stale drafts or stage resources: retain base selection/error precedence and pre-publication resource handling with byte/fault tests.
- Integer/fractional sampler divergence can create seam or hold bugs: share selection semantics and use exact analytic plus fractional inherited fixtures.
- Native checks are lengthy and platform-specific: retain full command/exit logs; distinguish passing Windows evidence, skipped opt-in tests covered separately, and unavailable Linux/macOS/POSIX evidence.

## Migration Plan

No new schema, persisted fields, or migration version. Repair schema-27 adoption and schema-27 sampling/certification in place. Valid projects keep IDs, revisions, provenance, histories, drafts, resources and output. Invalid migration candidates fail without publication; supported interrupted writes continue to recover one complete generation. Existing schema-27 projects still receive canonical validation; do not rewrite bytes merely to adopt the fix. Rollback of failed edits/migration remains the existing transactional path; no backward-schema conversion is introduced.

## Open Questions

The user approved the concrete artifacts on 2026-09-30 with "Approve"; this approval is recorded in proposal.md. Any need to change public/persisted contracts or stale-base policy requires a reviewed amendment first.
