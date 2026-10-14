# Illustrated native workflow

The illustrated workflow uses the direct native rendering strategy selected in
[#187](https://github.com/matiHirCab/AI-Open-Cut/issues/187). Native shape/path/grid,
text, managed image, component/slot, group, and animation data remain editable.
It does not require manually generated plates. The conditional refresh work in
[#188](https://github.com/matiHirCab/AI-Open-Cut/issues/188) and
[#189](https://github.com/matiHirCab/AI-Open-Cut/issues/189) was explicitly closed
as not planned; those are strategy waivers, not missing refresh acceptance.
The selected optimization [#203](https://github.com/matiHirCab/AI-Open-Cut/issues/203)
is included through [#205](https://github.com/matiHirCab/AI-Open-Cut/pull/205).

## Create and customize

Use the existing `create_motion_graphics` prompt with `native-hero-card-v1`,
`native-motif-pack-v1`, or `native-role-team-v1` to retrieve the supported native
recipe and its parameters. See [hero](native-hero-card.md),
[motifs](native-motif-pack.md), and [team](native-role-team.md). These existing
examples are distinct from the original final integration targets described below.

For a target, call `project_create` with width 1920, height 1080, and fps 30;
inspect `project_open` for the current revision and overlay/audio track IDs;
import original artwork with `asset_import`. Submit ordinary `component_create`,
slot-definition, instance, group/parent, and animation operations through
`timeline_batch_edit` with `expectedRevision`. The new test-only
[target definitions](../apps/agent-bridge/tests/fixtures/illustrated-targets.ts)
show the complete native operation inputs for each final target. They are not a
new public recipe or an installed runtime asset.

Update an instance through `component_instance_update`, retaining its component
ID and timing fields and replacing only the intended override map entries. A
text override uses `{ "type": "text", "value": "EDITED INTRO" }`; an artwork
override uses `{ "type": "asset", "value": { "kind": "asset", "scope": "project",
"id": "<managed-image-id>" } }`. The device/sticker test labels accept 1–24 Unicode
scalars and artwork slots accept images. Team role labels and its existing color,
asset, and boolean slots retain the [team recipe](native-role-team.md) constraints.
Managed artwork pixels are media, not native child shapes. Native geometry and
paint are editable through `component_update`; preserve existing slots and other
source fields when submitting the complete component definition. Each command
uses the latest authoritative revision. A stale revision fails without writing;
refresh before replanning, rather than automatically overwriting concurrent edits.

Render `preview_render_frame` at 0, 1500, and 2966 ms. Request
`preview_review_range` with startMs 0, endMs 3000, resolution `project`; export via
`project_export_video` with format `mp4`, resolution `project`, an owned relative
path, and overwrite false. Poll actual jobs with `job_get_status`; cancellation
uses `job_cancel`. Project undo/redo and reopening preserve native editability and
managed media ownership. In the desktop, select an instance in the hierarchy and
use **Instance slots**; [desktop controls](desktop-instance-slots.md) document
Apply, clear override, draft reset, conflicts, read-only state, and history.

## Original final target scope

Every final target lasts 3000 ms at 1920×1080/30fps. Full-range preview/export is
[0,3000), 90 output frames. The historical
[#181 inventory](../crates/editor-core/tests/fixtures/illustrated-scene/recipe.json)
remains unchanged; its original 1-second fixture is not relabeled as final coverage.

| Target | Required composition and layout | Inclusive caps: leaves / instances / text scalars / artwork occurrences / image dimension / path commands / animation channels |
| --- | --- | --- |
| `illustrated_device_intro` | Device x260..1140/y180..860; screen, artwork x620..876/y360..616, label within x1200..1700/y260..620, accent, backdrop | 24 / 1 / 128 / 1 / 1024 / 256 / 8 |
| `sticker_motif_intro` | Authored sticker contour x560..1360/y140..940, artwork, dot backdrop, rays around 960/540, speed lines, label x560..1360/y900..1020 | 48 / 2 / 128 / 1 / 1024 / 1024 / 16 |
| `role_badge_team_intro` | Identity team parent, three native badge columns x160..640,720..1200,1280..1760, artwork y260..660, labels y700..820, title y80..200, backdrop | 96 / 6 / 512 / 6 / 1024 / 2048 / 32 |

These are ceilings. Actual smaller inventories must be disclosed; they do not
prove the maximum 96-leaf/six-instance envelope. Count expanded visible leaves
and artwork occurrences separately from component instances and distinct active
or retained managed artwork assets. Replacement images remain managed media and
can be retained for history even when no current instance uses them.

## Reproduce the evidence

Install the pinned repository toolchain and bridge lockfile dependencies. Set
absolute FFmpeg/FFprobe paths and the repository's complete four-face bundled
font directory through `OPENCUT_TEST_FONT_PATH` pointing to its `DejaVuSans.ttf`.
Select an owned, existing evidence directory; use disposable projects/media/export
roots. Run from the repository root:

```sh
OPENCUT_FFMPEG_PATH=/usr/bin/ffmpeg \
OPENCUT_FFPROBE_PATH=/usr/bin/ffprobe \
OPENCUT_TEST_FONT_PATH="$PWD/crates/editor-core/resources/fonts/DejaVuSans.ttf" \
OPENCUT_PLATFORM_EVIDENCE_DIR=/absolute/owned/evidence \
bun run apps/agent-bridge/scripts/run-platform-renderer.ts
```

The existing runner builds source headless and assembles a fresh default package,
then executes both required MCP modes. It retains prior platform assertions and
closed report validation. A skipped required mode is not success. The new workflow
checks semantic roles/layout and bounded inventories from native data independently
of rendered-route parity. Manually specified pixel/gap/glyph probes cover all three
agreed PNG samples and decoded range/export samples. Omission, corruption, and
black-output controls must fail. These probes are limited independent witnesses,
not a full-image painted reference.

Range/export SSIM must be ≥0.99; decoded route-to-route audio RMS ≤0.0001, with
nonzero energy. Both decoded streams are also checked against the independently
created synthetic chirp, coded amplitude envelope, and 120ms onset: normalized
correlation ≥0.99 and original-clock alignment within one 30fps output frame.
Both authored stereo channels are verified independently; silence or a 40ms shift
on either channel must fail. Reports conservatively record the lower channel
correlation and the greater absolute channel displacement. Silence, omission,
and a 40ms shift must fail. Correlation checks waveform agreement,
not exact lossy amplitude fidelity. Frame count is 90 and duration stays within one
output frame of three seconds. The synthetic artwork/audio contain no private or
third-party creative inputs. See the
[workflow](../apps/agent-bridge/tests/illustrated-workflow-native.ts) and
[independent oracles](../apps/agent-bridge/tests/illustrated-media-oracle.ts).

## Capability and acceptance attribution

The initial fresh source bridge and freshly assembled default package each passed all
three final targets, including actual jobs, decoded media, history, native source
edits, cancellation, and reconnect/reopen. After independent review, stronger inventory/exact-undo/transform/stereo assertions
require a fresh two-mode run before final acceptance. Both initial required cases passed with zero
failures/skips within the unchanged 600-second per-case timeout. Every target's
range/export SSIM was 1, decoded route audio RMS was 0, and original waveform
correlation was 0.9998788883 with zero measured clock displacement.

| Target | Measured initial leaves / instances / text scalars / active artwork / distinct active images / stored images / image dimension / paths / channels |
| --- | --- |
| Device | 7 / 1 / 12 / 1 / 1 / 2 / 64 / 0 / 1 |
| Sticker | 7 / 1 / 12 / 1 / 1 / 2 / 64 / 80 / 0 |
| Team | 29 / 4 / 32 / 3 / 1 / 2 / 64 / 120 / 0 |

The customized team has 37 text scalars and two distinct active artwork assets;
its other inventory values are unchanged. Active occurrence caps do not cap
retained storage. The target's identity group is not a component instance.
See [attributed receipts and traceability](verification/illustrated-workflow/README.md).

Current-source desktop interaction is observed on an owned Linux cloud GUI using
software Vulkan, independently of native MCP verification. The fresh GUI covers
text/artwork/color/boolean overrides, invalid text, clear override, draft reset,
undo/redo, stale revision, locked-track rejection, and exact project reopen.
Controls remain visible on a locked track and the authoritative mutation fails
with `TRACK_LOCKED`; this evidence does not claim disabled controls. Historical
#190 GUI and #203 CI/performance evidence remains historical. The cloud source
build does not establish the user's personal installed desktop or Windows/macOS
GUI behavior. No private video files, installed build, configuration, or renders
are inspected. Remaining release gates and published-head CI must pass before
this change is delivered as complete.

## Performance meaning

Retain the original [#187 counterexample](illustrated-scene-benchmark.md) and
[fixed #203 before/after proof](verification/exact-byte-linear-transfer/README.md).
The original fresh 5-second frame/8-second short-range acceptance workloads,
reference host/inputs/captures, guard reservations, and CI budget are unchanged.
Observed timings of the new 90-frame workloads are separate integration data;
they are not replacement fixed samples or those original short-range targets.
No portable performance, isolated optimization-stage effect, statistical tail,
maximum-envelope, or personal installed-runtime claim follows from these samples.

The authored 800-pixel curved sticker contour is a material workload limitation:
source cold frame preparation took 22.0 seconds and the edited frame 25.3 seconds
(packaged: 21.1 and 22.2 seconds). Warm samples took approximately 2.4–3.0 seconds.
The 90-frame sticker range/export took 64–68 seconds; device took 52–55 seconds
and team took 68–71 seconds across these two modes. These are observed wall-clock
samples on this host, without a portable latency or general fast-frame claim.
Source debug GUI full-resolution rendering took minutes and was verified through
completed frames; those observations are not release latency measurements.
The original fixed #203 performance proof is retained unchanged; its thresholds
are not silently extended to this separate curved contour or full 90-frame range.
