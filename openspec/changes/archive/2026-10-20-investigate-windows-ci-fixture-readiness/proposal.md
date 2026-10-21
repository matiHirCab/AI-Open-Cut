## Why

The delegated report identifies Windows Motion Chocolatey FFmpeg 7.1.1 HTTP 503/499 failures and main job-restart/lifetime test failures. Remote main HEAD is independently verified as 756b7b39ce0e2b1d5fcc65291084838da5331517, matching the clean starting checkout. The authorized GitHub connector confirms all three current main runs failed on Windows: Motion platform run39271035406/job118123075630 and release run39271035366/job118123068613 failed Chocolatey with HTTP503; correctness run39271035312/job118123066972 failed the two reported tests. Shell GitHub access fails independently of the working connector.

## What Changes

Repair private crash/readiness fixtures only: build one self-contained JavaScript crash fixture before controlled tests, publish closed native PID metadata by sibling rename, and keep metadata parse failures within owned-request diagnostic cleanup. The single authorized rerun of each Motion Windows job now passed FFmpeg provisioning; both unchanged Windows jobs completed successfully, so retain workflows unchanged. Preserve every existing assertion and time limit. Implementation details remain conditional on current logs and approval of this concrete proposal.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `job-restart-semantics`: reliable bounded real-process crash verification.
- `master-loudness-normalization`: complete PID readiness publication before observation.

## Impact

Private bridge tests/fixtures only; provisioning investigation is recorded separately. No public or persisted contract changes, migrations, production semantics, secrets, permissions or security settings.

## Non-goals

No #211/#208 work, rewriting #228, timeout increases, retries of semantic tests, weakened gates, merges, deployments or unrelated file cleanup. Do not repeat the archived native-normalization-fixture correction.

## Status

Implementation locally verified; publication and exact-head CI pending. Implementation is limited to the three private bridge test/fixture files. Bun1.4.0, Rust1.97.0 and Moon2.3.3 were installed in /tmp from official distribution artifacts; pinned strict proposal validation passed. Independent repository reviewer approved the concrete scope before implementation; subsequent independent restart code review passed. CI results remain mandatory.
