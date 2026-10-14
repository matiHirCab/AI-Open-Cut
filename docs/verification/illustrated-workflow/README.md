# Illustrated workflow verification

Issue [#191](https://github.com/matiHirCab/AI-Open-Cut/issues/191) verifies the original
three #181 targets using the selected direct native renderer. This change adds
only test inputs, independent oracles, verification, and documentation. Public
recipes, renderer behavior, guards, tolerances, required case names, timeouts,
performance captures, and CI budgets are unchanged.

## Fresh observed results

The final fixture uses the explicitly reviewed eight-vertex beveled native path
within the original 800px envelope. Exact vertices and independent interior/exterior
probes are documented in the design and tested without changing roles, timing,
caps, renderer behavior or gate budgets. Fresh replacement acceptance is pending.
[Historical curved input](curved-contour-history.md) and
[curved observations/failures](curved-observations.json) remain historical; both
strengthened curved cases timed out at 600 seconds.
[Target input hashes](target-input-comparison.json) prove device/team authoring
inputs unchanged. Fresh sticker GUI evidence is required.

Independent review tightened resource/artifact preservation, exact undo, transform
subset rejection and both stereo channels. The native receipts below precede
those assertion corrections; a fresh corrected two-mode run is required before
acceptance. Device/team GUI evidence remains valid because desktop sources and their
authored inputs are unchanged. Sticker evidence requires the fresh replacement
observation and cannot reuse the curved screenshots.

[native-results.json](native-results.json) contains all six source/default-package
results and wall-clock samples; [validated-inventories.json](validated-inventories.json)
records independent report validation and initial/customized native inventories.
The retained initial two required native cases passed (555.592 and 548.630 seconds), zero failures
or skips, against their existing 600-second limit. All three PNG samples and
90-frame full range/export outputs were checked through actual jobs, semantic
native roles/layout, independent authored probes, decoded video and original
synthetic PCM. SSIM 1, route PCM RMS 0, original correlation 0.9998788883, clock
alignment zero. These measures establish the stated bounded witnesses, not a
complete independent painted image or exact lossy amplitude fidelity.

Source desktop binary SHA-256:
`cd738e713bfbc225077e641dd3e411698d059c0147fcc0cef62f0a140c72d056`.
It was built from the current source and run on owned display `:191` with fresh
project/config/cache/export roots and software Vulkan. GUI evidence records
actual XTest commands, settled screenshots and whole persisted-project checks;
the controller does not substitute screenshots or JSON alone for interaction.
Raw native media and GUI evidence remain outside Git at
`/workspace/issue191-platform-sixth-evidence` and `/workspace/issue191-gui-session`.
[gui-results.json](gui-results.json) records hashes and actual observed transitions;
[attribution.json](attribution.json) records tool/input/build identities. The user's
personal installed application and non-Linux GUIs were not inspected.

## Requirement and scenario traceability

| Requirement / scenarios | Automated or actual evidence |
| --- | --- |
| Original bounded fixtures: all targets; missing roles/excess inventory | `illustrated-targets.ts`, `illustrated-target-oracle.ts`; `illustrated-workflow.test.ts` timing/layout/cap/omission and moved-parent/anchor/rotation/skew negatives; six actual native inventories |
| Editable MCP lifecycle: edits survive reopen; invalid operations preserve state | `illustrated-workflow-native.ts` both modes: typed native source edit, first-instance text/image override, team isolation, exact undo/redo and reconnect/reopen, stale revision and late atomic rollback, actual cancellation/no artifact/project/history/resource mutation; rendered artifacts permit only the actual returned new file and preserve all prior bytes |
| Independent media: all samples/ranges conform; corrupted output rejected | `illustrated-media-oracle.ts`: authored interior/gap/glyph probes for 0/1500/2966 ms and decoded frames 0/45/89, black/corrupt controls, SSIM/audio route checks, original PCM/clock checks on both stereo channels plus silence/omission/40ms-shift and one-sided silence/clock negatives |
| Actual isolated desktop: edit MCP target; incomplete observation blocks | Fresh current-source owned GUI input-event logs, manually inspected settled screenshots, exact persisted transitions, invalid/reset/stale/locked controls, history and close/reopen checks; missing GUI evidence remains an acceptance blocker |
| Attributed documentation: distinguish capabilities; retain timing meaning | `docs/illustrated-workflow.md`, six-mode receipts, active vs retained assets, source/package/cloud-only attribution, immutable #187/#203 references, separate curved contour timings |
| Required verification: draft review; failed/skipped required check blocks | Required local logs/receipt, independent conformance, own specification archive, protected validation, draft PR and exact published-head 18-check/four-workflow CI receipt required before completion |

`illustrated-evidence.ts` validates report attribution, coverage and semantic role
kinds. Focused tests reject omitted/corrupted reports, wrong modes/targets and
wrong leaf kinds. The six actual reports and saved native projects independently
passed this consumer; metadata alone never substitutes for actual render/audio
or desktop evidence.

## Failures retained and limitations

Development failures were retained outside Git: incomplete font setup, transient
parse error, unsupported legacy animation on transform2d, edited artwork probe
requiring the existing three-channel PNG tolerance, and path origin authoring.
Path inputs were corrected to authored local geometry plus explicit transforms;
team probes were corrected from canonical geometry/palette. Existing expectations,
renderer tolerances and timeout limits were not relaxed. GUI coordinate/type
calibration captures are retained and explicitly excluded from acceptance.

The curved sticker cold/edited frames take 21–25 seconds on this host; warm
samples take 2.4–3.0 seconds. Full 90-frame ranges/exports take 52–71 seconds. The source debug GUI
full-resolution renders took minutes; completion is recorded from bounded
observations rather than claimed as a release latency sample.
This is a workload limitation and no general fast-frame claim is made. The
unchanged #187/#203 fixed fourteen-cell 5-second frame/8-second short-range proof
has different fixed inputs. Smaller observed inventories do not establish the
96-leaf/six-instance maximum envelope; retained managed storage is not bounded
by the active artwork occurrence cap. #188/#189 are explicit strategy waivers.

## Delivery state

Implementation checks, independent final conformance, own archive, draft PR and
exact published-head CI must all complete before final release handoff. Local
receipts and exact-head CI attribution are appended only after actual completion.
