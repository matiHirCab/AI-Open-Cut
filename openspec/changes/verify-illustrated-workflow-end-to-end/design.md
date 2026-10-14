## Context

The main checkout at `2ccb3f0f831dee25a91647315eeb0bc1806ad8a3` contains the completed native features and #203 optimization. #181's canonical inventory remains historical and does not implement its final targets. Existing required platform tests exercise source and assembled default-package MCP runtimes, with independent component checks. This change adds integration inputs and evidence to those paths without replacing existing coverage.

## Goals / Non-Goals

**Goals:** implement the three original targets as test data, verify actual editable native lifecycle and output, collect an isolated actual desktop observation, and document reproducible release evidence with clear attribution.

**Non-Goals:** product features, alternate renderer/plates, refresh support, public/persisted schema changes, altered contracts, increased tolerances/budgets/timeouts, private fixtures, personal installed desktop inspection, maximum-envelope or portable performance claims, deployment or merge.

## Decisions

1. **Separate test fixtures from historical recipes.** Add an explicitly versioned test-only target description and MCP authoring helper. Every target lasts 3000 ms at 1920×1080/30fps, samples 0/1500/2966 ms, and renders/exports [0,3000) (90 frames). Device roles are device_frame, screen_panel, artwork, label, accent, backdrop; device bounds x260..1140/y180..860, artwork x620..876/y360..616, label x1200..1700/y260..620. Sticker roles are authored_sticker_contour, artwork, dot_backdrop, sun_rays, speed_lines, label; sticker bounds x560..1360/y140..940, rays centered at 960/540, label x560..1360/y900..1020. Team roles are team_parent, role_badges, member_artwork, member_labels, team_title, backdrop; three columns x160..640,720..1200,1280..1760, artwork y260..660, labels y700..820, title y80..200. Reuse supported native primitives/ordinary component operations, with test data extending duration; do not simply rename existing 1-second helpers. Alternative: keep 1-second examples as final targets; rejected because it misses the original integration scope.

2. **Inclusive caps and measured actual inventories.** Independently count expanded visual leaves, instances, text scalars, artwork occurrences and distinct managed assets, maximum image dimension, path commands, and animation channels. Preserve caps respectively: device 24/1/128/1/1024/256/8; sticker 48/2/128/1/1024/1024/16; team 96/6/512/6/1024/2048/32. Counts below caps are valid; state them explicitly. ArtworkCount bounds active expanded occurrences; distinguish distinct active assets from retained stored/history images and do not claim the target cap bounds retained storage. Existing storage guards remain unchanged. Author a three-member team using existing supported components and scaled positions satisfying the original column/artwork/label envelopes. Alternative: force maximum counts; rejected because the caps are ceilings and maximum-envelope performance remains unproven.

3. **Actual MCP and independent oracles.** Import original generated small BMP artwork and a synthetic 3-second PCM signal in owned disposable roots. Use actual tools for creation/customization, renders/exports, source edits, override isolation, close/reopen, revision errors, history, atomic late invalid batches, and cancellation. Verify semantic roles/layout and manually specified interior/gap pixel probes separately from cross-route SSIM ≥0.99. Decode range/export audio to compare RMS ≤0.0001 with nonzero signal; separately compare each decoded signal with the independently authored PCM waveform and original clock, allowing AAC distortion through normalized correlation ≥0.99, with onset/alignment within one 30fps output frame. Silence, omitted audio, and a 40ms clock shift must fail this independent check; confirm 90 frames and timing within one frame. Deliberately omit a role and corrupt a probe/count to demonstrate validator sensitivity. Alternative: render-to-render parity alone; rejected because identical omissions could pass.

4. **Preserve existing gate shape.** Add the workflow to both existing required source/package cases, retaining their names, all prior assertions, 600-second case timeout, 120-second job polling, existing render guards, and required-report validation. Add focused helper tests for fixture/oracle/report negatives. No new optional skip mode or dependency. Owned artifacts/logs remain outside git; committed evidence summaries identify commands, build/source hashes, environment, inventories, route results, and limitations.

5. **Real desktop evidence with scoped ownership.** Build the current source desktop and run on a fresh owned Xorg dummy/software Vulkan session using available immutable cloud tools. Open MCP-authored fixtures, make actual input events for slot editing and lifecycle operations, retain settled screenshots plus persisted revision/value transitions, and identify the source/binary/environment. Existing #190 control evidence remains historical, not a fresh #191 observation. Never interact with the personal installed app or unrelated video production. Alternative: screenshots or model tests alone; rejected by the living desktop specification.

6. **Truthful release and performance attribution.** Document actual tested source bridge, newly assembled default package, and cloud source desktop capabilities separately from unobserved personal installation. Record #188/#189 not-planned waiver and merged #203. Retain #187 counterexample and immutable #203 before/after evidence; additional 90-frame timings are observations, not the original 5s/8s acceptance workload or proof of maximum-envelope performance. Preserve all required local checks and independently review conformance before synchronization/archive, then verify all 18 exact-head CI checks and unchanged default-120-minute foundation gate.

## Risks / Trade-offs

- Additional full-HD renders can exhaust disk or exceed existing case/job budgets → use bounded actual inventories, owned evidence roots, incremental meaningful verification, and report a real blocker rather than weaken limits.
- Pixel probes can miss errors → check role/layout inventory independently and include omission/corruption negatives; report the oracle's limited coverage rather than claim a full image reference.
- Cloud GUI timing/environment can fail → use settled events and persisted transitions with isolated software rendering; retain actual failures and do not substitute black/screenshots-only evidence.
- Test-only data drift → validate literal duration/layout/caps against the unchanged original inventory and link requirements to focused negatives and actual workflow evidence.

## Migration Plan

No contracts, persisted schemas, dependencies, migration, or deployment changes. Reverting these verification-only additions restores prior test/documentation behavior. Dispose only owned test projects and processes after evidence capture.

## Open Questions

No product decision is outstanding. Runtime/disk/GUI feasibility is a verification result to establish, not permission to relax requirements.

## Reviewed bounded contour selection after runtime verification

The initial test-authored sticker used an 800px contour with four cubic corners.
Its retained native captures observed cold frames of 21–35 seconds and slow
90-frame routes. The strengthened source and default-package run each timed out at the unchanged
600-second case deadline (600042ms and 600023ms), during the final team export.
The complete retained runner confirms both failures. No original requirement specifies Bézier
curvature: #181 and this change require an authored native contour, its original
roles/envelope/timing/caps, and editable native state. The independent reviewer
explicitly approved the bounded alternative below, with fresh acceptance required.

The final fixture therefore authors an original beveled native path with
local vertices (80,0), (720,0), (800,80), (800,720), (720,800), (80,800), (0,720),
(0,80), then close: one move, seven lines, one close, nine commands. Its explicit
transform is (560,140), so its geometric bounds remain x560..1360/y140..940.
This is original synthetic test authorship through the same component/path APIs;
no creative source, manual plate, alternate renderer, or runtime contract is added.
All other authored geometry/media and original target requirements are unchanged.

Retain every existing role/pixel/layout/cap/corruption/lifecycle/stereo assertion.
Add independent contour witnesses at canvas (605,185), expected interior paint
RGB(48,73,104), and (585,165), expected exterior backdrop RGB(24,38,61), safely
inside/outside the authored bevel and away from dots/rays. Record the actual new
path inventory rather than relabeling the curved count. Preserve curved commands,
input identities, timing/failure receipts outside the fresh accepted evidence.

Fresh strengthened source/default-package full acceptance and a fresh sticker
GUI edit/render/reopen are mandatory for the replacement; earlier curved captures
cannot satisfy them. Reuse device/team checks only where relevant inputs are
unchanged. Keep all prior platform assertions, 90-frame ranges/exports, existing
600-second case/120-second polling limits, guards, fixed #187/#203 captures and CI
budgets. This choice makes no claim that the curved contour meets the case budget
or that arbitrary native curves render quickly. Final conformance is still pending.
