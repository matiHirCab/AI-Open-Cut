## Context

Verified main `50ec842ca6c3c6bea72864b6878fecbc953a5a97` includes #181; #182/PR195 is independent and unmerged. The working tree was clean before creating `feat/issue-183-solid-graphic-color-slots`. Existing template slots have a closed eight-kind union, hex6 color values, seven binding identifiers, core-only local target validation and cloned effective component resolution. Native shape fills, stroke paints and dot-grid paints use unassociated sRGB channels and linear alpha in [0,1]. Project schema is44; draft envelope remains2 with existing predecessor1 support. No active unrelated OpenSpec change exists.

## Goals / Non-Goals

Deliver the issue's small explicit matrix with independent instances, exact compatibility/failure semantics and automated lifecycle/native evidence. Non-goals are in the proposal: alpha/gradient slots, arbitrary property paths, SVG/line-grid editing, paint creation, UI, new operations/dependencies, renderer redesign and desktop tasks.

## Decisions

1. Add `shape.fill.color`, `shape.stroke.color`, `grid.dot.paint.color` to the native/bridge closed property enums. Each requires kind=color and an existing solid paint at that exact target. Missing local item remains ITEM_NOT_FOUND; explicit caller-selected scope fields, absent paints, gradients, non-dot grids, SVG and other incompatible combinations use existing non-retryable INVALID_ARGUMENT. Identifiers existing only in root/foreign compositions remain missing local targets (ITEM_NOT_FOUND); IDs are not inferred or resolved across scopes. Duplicate writers retain existing rejection. These names avoid admitting an arbitrary property-binding language.
2. Keep `{type:"color",value:"#RRGGBB"}` unchanged. Parse each byte /255 into unassociated sRGB r/g/b; preserve the base target's a exactly, including0/0.5/1. Existing text-color semantics and original casing round trip unchanged. Do not add an alpha slot/type or reinterpret hex8; doing so would broaden the issue and alter existing consumers. Preserve geometry, all stroke metrics, grid descriptor and other paint/visual fields. Apply through the existing core effective-candidate resolver used by evaluation; transports only submit typed input.
3. Extend manually governed template-slot properties and add a separate `solid-graphic-color-slots-v1.json` conformance artifact/capability `solid_graphic_color_slots_v1`. Update headless metadata/catalog, MCP structural slot enums/project version and reviewed expanded digest, ownership index/CODEOWNERS and Rust/TS parity consumers together. Preserve unrelated canonical sections and all existing identifiers/errors. Synchronize the existing release-report exact project-schema attestation from44 to45 and its synthetic test sample; preserve its report version/shape, numerical limits, release recipe/digest, report-only/hard-gate status and every other assertion. Record section ownership/base/head and request @matiHirCab review on the draft PR; standing delegated specification approval does not fabricate completed human review.
4. Advance project schema44 to45 because new persisted enum identifiers are unreadable by predecessor builds. Reject new bindings when a document claims schema<45, in both deserialization and typed migration paths (including unused definitions). Existing schemas1–44 migrate only through established migration rules plus version adoption, under the project lock and existing atomic current/history publication. Preserve old content/provenance/resources; reject unknown future current/history versions. Draft format remains2: accepted existing component slot/instance operations acquire new enum alternatives, using the existing materialization/preview/commit/rebase rules and font/resource envelope. No new draft operation or parallel migration is introduced. Malformed MCP envelopes retain the existing SDK invalid-arguments rejection before core dispatch; structurally valid wrong-kind bindings/values reach core and retain typed INVALID_ARGUMENT. Native headless structural errors retain their existing mapping; no new bridge/domain validation is added.
5. Use a small native test component containing shape fill, stroke and dot-grid paint plus unrelated ordinary opacity animation. Two root occurrences receive different RGB overrides; exact evaluated facts prove alpha/metrics/geometry and shared-definition isolation. Independently authored pixel locations/expected colors cover both occurrences, alpha compositing over black and a changed draft candidate, with negative blank/wrong-size controls. Invoke shared lifecycle helper in actual source/fresh packaged MCP smoke and the existing required native platform entry points. Native frame/range/draft jobs preserve existing deadlines, RGB3 PNG and RGB8 encoded-range policy; assertions/CI gates are not weakened. Original synthetic geometry, no external media or desktop renders.

## Risks / Trade-offs

- Closed property enums and current-project schema appear throughout public catalogs/tests → manually update only owned sections, retain predecessor raw/digest evidence and run actual canonical parity. Inspect affected current-version fixtures; historical fixtures remain historical.
- Alpha preservation does not allow palette-driven alpha editing → document it explicitly; existing visual.opacity and native paint editing remain available.
- Existing complex test infrastructure can mask self-derived expected output → canonical fixtures test wire behavior; independently authored pixels/blank controls and native diagnostics test rendered behavior.
- Required human CODEOWNER review is asynchronous → request it on the concrete draft PR, report pending review accurately; do not merge.

## Migration Plan

Implement approved models/guards/migration and focused predecessor/current/history/draft tests before publication. Verify repeat-open determinism and byte-level atomic rejection for corrupt/future history. Reopen predecessors with schema45 adoption and retained slots/paint/animations/media unchanged. No downgrade of schema45 is promised; restore a prior checkout/project backup for rollback. Existing protocol major1 and ordinary requests remain valid. Run all required checks, conformance, sync/archive only this change, protected post-archive gate and exact published-head CI.

## Open Questions

None blocking the issue-scoped design. Human CODEOWNER review remains a separate PR requirement, not delegated approval.

## Native fixture sampling refinement

The first actual native pilot passed PNG/draft samples but the encoded range blended the initial four-pixel stroke beyond the existing eight-level RGB bound. Use a twelve-pixel stroke in the authored fixture so the declared stroke sample lies in a codec-safe interior. Expected colors, RGB3/RGB8 bounds, negative controls and existing native deadlines remain unchanged. Core exact whole-item comparisons still prove preservation of the authored stroke metrics. This is fixture design refinement within G3/G9, approved under standing issue-scoped authorization before the fixture edit. Preserve the failed pilot log.

The next native pilot confirms the thicker stroke meets the original encoded bound and identifies the same codec-edge problem in the radius-four cyan dot. Use radius-eight dots at the existing spacing and independently declared centers to provide a codec-safe interior, retaining quarter-alpha and all expected RGB values/bounds/deadlines. Exact core comparisons preserve these authored grid metrics. Approval and failed logs follow the same G3/G9 fixture refinement.

## Current-schema test consumer synchronization

All existing actual-runtime workflow/status/state/migration assertions and synthetic current-project schema fixtures must use the new canonical solid-graphic fixture schema45. Historical feature catalogs and comparisons between their schema44 adoption markers remain unchanged; composed rollback pins remain authoritative. This required G6/G10 consumer synchronization includes existing non-graphic lifecycle/native tests and is approved before their edits.

## Composed capability rollback preservation

The new semantic MCP inverse removes exactly one owned graphic capability identifier while preserving unrelated capability edits, so older negative tests can still exercise their own missing-capability failures. The other three owned schema paths must match the captured after-values. Raw canonical rollback separately still requires the exact pre-change digest and rejects every unrelated drift. This scoped compositional correction is approved before its edit; no captured pin is rewritten.

The current motion-graphics release guide and its private executable teaching example/parser also attest the persisted current version: update only that schema marker44→45 and runtime expectation. Keep the public example version, operation corpus, reference-recipe fingerprint and historical feature markers unchanged. These current-documentation consumers are included in G6/G10 approval before edits.

## Protected canonical-command synchronization

The protected gate first failed because validate-ci-gates.ts attests the exact old contracts:check command. Add the new focused Rust and TypeScript conformance consumers to that exact expected command in the same positions as package.json. Add omission and failure-masking negatives for both consumers; preserve every old consumer, bootstrap condition, deadline, report/duration bound and workflow command. This strengthens mandatory G10 parity and is approved before these gate-consumer edits; the original failed pre-archive log is retained.

The modified template-slots requirement now spells out explicit caller-selected scope fields for INVALID_ARGUMENT, distinguishing that structural violation from an ordinary local identity whose identifier exists only elsewhere (ITEM_NOT_FOUND). This matches the approved G1/G2 decision, new real-root/foreign tests and existing resolver; no new domain behavior is introduced.

## Required active-version consumer correction

Full Rust conformance identified seven active catalogs whose existing unchanged assertion requires equality with the current core schema. Advance only their projectSchemaVersion markers44→45: animation channels/presets/extended visual/inherited timing/motion blur/initial pack/mask models. Capture their exact authoritative main raw hashes before editing; add owned marker rollback before existing normalization projections. Keep all historical captured pins, activation bounds and other catalog fields unchanged. Update only their current-marker TypeScript expectations. Preserve the Rust current-core equality assertion.

Normalization migration tests must assert the new current version for migrated state/history and use current+1 for the unknown-future refusal; retain schema43/44 premature/null cases and all atomicity assertions. Actual release-measurement output must attest the current core schema45 while retaining its exact report shape, recipe hash and budgets. Speech-alignment raw compatibility tests first undo exactly the seven owned marker changes and verify their captured main digests before all previous rollback/digest assertions. These required G6/G7/G10 synchronization edits are approved before executable changes. The initial full Rust/parity failures and superseded check attempts remain recorded.

The existing typed-template guide also lists the supported color bindings: synchronize its color row and link the new solid-paint/alpha/schema45 guide. Preserve historical activation and closed-record compatibility text. This documentation consumer is within approved G1/G3/G6/G10 and is updated before its upcoming TypeScript/documentation checks.

The complete workspace run identified three further literal current-schema assertions in narration_fixture.rs, reference_scene.rs and the configured motion_release.rs native witness. Synchronize them to the current core version, retaining their exact scene/history/audio/pixel/report assertions and reference recipe fingerprints. This is the already-approved G6/G10 current-consumer rule applied to concrete required native consumers; approve this explicit list before edits.

## External main reconciliation

PR195/#182 merged externally into main84da152fac0378396099d8f01fc3b7339690361f during verification. Reconcile that main before publication, preserving its complete read-only hero recipe and all hero source/package/native assertions alongside the new graphic assertions. No hero behavior or fixed initial paints are changed. Main adds no Rust or canonical contract input; verify captured input hashes before reusing successful fmt/Clippy/workspace evidence under AGENTS. Run all bridge/source/fresh package/native/Python checks on the reconciled tree and retain complete exact published-head CI obligations. Original captured predecessor digests and initial base50 provenance remain unchanged. Standing issue-scoped approval explicitly covers this necessary base reconciliation before checkout edits.

## Historical projection dispatch correction

The full reconciled unit suite exposes eight historical projection tests whose common speech-alignment marker dispatch admits39–44 but never routes the seven new current45 markers through the already-owned graphic45→44/master-normalization chain. Add exactly45 to this existing dispatch, preserving every previous version/control, predecessor digest, unsupported-marker failure and unrelated-drift assertion. Existing eight failures provide scenario coverage for all affected downstream projections. This necessary G10 compositional consumer correction is approved under standing authorization before edits. No canonical artifact or captured hash changes.

## Explicit PNG dimension conformance

Conformance review identified that G9 range dimensions were explicitly probed while frame/draft PNGs only checked decoded pixel count and independent samples. Strengthen G9 by independently probing width/height for both PNG captures with the existing required FFprobe. Preserve every range/frame-count/pixel/durable-byte assertion, expectation, tolerance and deadline. This fills the already-approved native-dimension requirement and is approved before this native-verifier edit. Repeat bridge verification on the final helper; final native modes must attest its exact new hash.
