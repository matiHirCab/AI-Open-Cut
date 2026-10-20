# Investigation evidence — 2026-10-20 UTC

Entries are chronological; later observations supersede earlier availability/status statements. Current conformance and final validation are recorded in verification.md before archival.

- Clean initial branch: work. Initial and remote main HEAD: 756b7b39ce0e2b1d5fcc65291084838da5331517 (`git ls-remote origin HEAD refs/heads/main`, exit 0).
- Isolated local branch: fix/windows-ci-fixture-readiness. No pre-existing edits were modified; no executable changes made.
- `gh auth status`: active GH_TOKEN invalid. `gh repo view --json nameWithOwner,defaultBranchRef`: GraphQL Forbidden. `gh api repos/matiHirCab/AI-Open-Cut/actions/runs?per_page=5`: Forbidden. Anonymous curl API request: proxy CONNECT 403. Git remote read works independently; it does not prove API authorization.
- Bun, Moon, proto, cargo and rustc are absent from PATH. No tests, strict spec validation, CI reruns, push, PR creation or check monitoring performed. No tokens/settings/permissions changed.
- Local findings: direct non-atomic PID file write followed by an existence-only consumer; sequential Bun crash processes under default Vitest timeout; Chocolatey 7.1.1 unchanged in both Motion workflows. These are source observations, not current CI reproduction.
- Prior native fixture correction is already archived at openspec/changes/archive/2026-10-10-stabilize-normalization-lifetime-fixture; it is not proposed again.
- Infrastructure versus code: reported HTTP 503/499 is a provisioning failure; PID readiness is a test publication race; the actual restart timeout cause remains unresolved. Current remote failure status is unknown.
- Blocked actions: current CI/issue/PR reads, permitted rerun, draft PR publication and monitoring (GitHub API access); local automated verification (missing pinned toolchain); executable implementation (current-state prerequisite and unreviewed concrete artifacts). Nothing was denied by automatic approval review. API failures are observed; no mutation had uncertain outcome.

- Coordination received: #211 has an unpublished native acceptance addition to bun-ci.yml in chat 01a150a9-01c3-742b-b4ac-c33d099560cd. This branch does not modify that workflow or governance hashes and does not copy/rewrite the other branch. Coordinate before any future need to touch them.

## Follow-up: connector and toolchain preparation

The previous API/current-state blocker is superseded: the authorized GitHub connector get_repo and GET workflow resources work. gh remains unusable; no credentials changed. Current main failures independently confirmed in runs39271035406,39271035366,39271035312 at756b7b39. Full original logs retained at /tmp/opencut-windows-ci-118123075630.log, /tmp/opencut-windows-ci-118123068613.log and /tmp/opencut-windows-ci-118123066972.log. Windows unit summary984 passed/2 failed/17 existing skipped; timeout5000ms at job-restart.test.ts80 and malformed JSON at master-normalization-lifetime.test.ts212 in processing/cancel.

Official Bun package @oven/bun-linux-x64@1.4.0 downloaded with npm pack into /tmp/opencut-tools and extracted; bun --version1.4.0. bunx is a local sibling symlink to bun. Frozen bridge install passed147 packages. Pinned strict OpenSpec proposal validation passed (log /tmp/opencut-proposal-validation.log). Linux restart baseline9 passed in834ms (log /tmp/opencut-restart-baseline.log). Python setup controls5 passed (log /tmp/opencut-platform-setup-tests.log).

Standard paths /home/agent/.{bun,proto,cargo,rustup}, /usr/local/{cargo,rustup}, /opt/{bun,proto,moon,rust,rustup} are absent; targeted readable-tree search found no executables outside PATH. /opt/containerd was unreadable and not bypassed. README official proto installer URL returns403; installer was not executed. Official Rust manifest https://static.rust-lang.org/dist/channel-rust-1.97.0.toml works. Downloaded x86_64-unknown-linux-gnu rustc,cargo,rust-std,rustfmt,clippy components from its official URLs; every SHA256 verified against manifest before install.sh --prefix=/tmp/opencut-tools/rust --disable-ldconfig. rustc/cargo report1.97.0. Official @moonrepo/core-linux-x64-gnu@2.3.3 npm package extracted under /tmp/opencut-tools/moon. All changes are local tools/cache, without security or credential changes.

Normal repository preparation remains the README proto install followed by proto use, bridge frozen install, cargo build -p opencut-headless and platform prerequisites. Alternate isolated official artifacts preserve exact pins and avoid writing home/global configuration. Python available3.12.14 versus CI3.11; Node24.19.0; ffmpeg/ffprobe available. Desktop xcb/xkbcommon pkg-config dependencies are not all available; assess before mandatory workspace checks.

One authorized rerun each requested successfully using connector: original jobs118123075630 and118123068613. Latest Windows jobs118390114629 and118390131861 are in progress and Windows FFmpeg succeeded in both. Do not rerun again. Workflow changes excluded pending terminal outcomes. No modification of bun-ci.yml, protected hashes or peer branches.

## Implementation and observed local verification

Independent reviewer approved concrete artifacts before edits (/root/review_windows_ci_spec; approval.md). Three executable test/fixture files changed; original restart cases from first old test through EOF remain byte-identical to base756b7b39. No workflow, governance hash, production or contract edits. Independent review of restart changes approved; all original time bounds/assertions retained. Main agent reviewed normalization stage/rename, error diagnostic boundary and native failure controls.

Observed: full bridge typecheck and lint pass (296 files); strict workspace all-target Clippy pass in2m08s (pre-existing proc-macro-error2 future-compatibility notice remains); Rust workspace formatting passes;52 Motion workflow policy controls pass; hermetic workers12 unittest+12pytest pass. Focused normalization58/58 (including3 Rust publication controls) passes in51.29s (/tmp/opencut-normalization-changed.log), restart independent12/12 passes3.49s (/tmp/opencut-review-restart.log).

Initial unmodified Linux normalization baseline27 failed/29passed because both observed process PIDs were zombies under PID1 (/proc5139 and5141 stateZ PPid1), unrelated to the current Windows JSON failure. Retained /tmp/opencut-normalization-baseline.log. Verification launcher /tmp/opencut-subreaper.py sets PR_SET_CHILD_SUBREAPER only on its own launcher and waitpid reaps only its orphaned descendants. No PID assertion or test timeout changed, no production/repo launcher added. The complete changed focused suite passes under that owned child-reaping launcher.

Moon2.3.3 exact pinned tools work with PATH pointing at isolated official binaries, MOON_TOOLCHAIN_FORCE_GLOBALS=true, PROTO_HOME=/tmp/opencut-proto, MOON_HOME=/tmp/opencut-moon and XDG_CACHE_HOME=/tmp/opencut-cache. Initial errors retained: default personal Wasmtime cache was read-only; isolated proto-managed setup then reported offline. No permission or policy bypass was used. With explicit exact-version global tool resolution, prearchive gate passed521 policy tests/77specification items and failed only on this active change as expected (/tmp/opencut-preimplementation-protected.log). This is not final protected success.

Full Rust workspace tests, bridge full unit/integration/packaged smoke, final conformance/archive/gate and exact-head PR CI are still pending. Motion reruns still in progress; their provisioning steps passed, final renderer results not yet asserted.

For reproducibility in this environment, source /tmp/opencut-ci-env.sh (tool paths and cache locations only, no credentials). /tmp/opencut-crash-startup-measurement.json records four source startups287.80/424.20/310.54/271.92ms and four bundled startups163.82/189.71/171.66/292.51ms under concurrent Cargo build. This limited Linux sample supports reduced startup work but is not a controlled cross-platform benchmark or proof of the Windows fix.

Original log SHA256 (normalized LF text saved from connector): platform3516332223aae8a39d70e8fd5d095bdfbf062bd9a7d3f35f1d14e5c5b0e109a7; release5249f9121fb5cc604d6501123ad322d7809b9fc82428fc099af696be3a406965; correctnessfb6243cbb6af28a7d82f05e2a2e8ae7db9d349eeb6409d912ebfe6f21dac72a5.

## Terminal unchanged Motion reruns

Both authorized Windows reruns completed successfully at unchanged main756b7b39. Platform job118390114629: https://github.com/matiHirCab/AI-Open-Cut/actions/runs/39271035406/job/118390114629 ; release job118390131861: https://github.com/matiHirCab/AI-Open-Cut/actions/runs/39271035366/job/118390131861 . Windows FFmpeg and complete native renderer/release steps succeeded, not merely package installation. Full rerun logs retained under /tmp/opencut-windows-ci-rerun-{jobid}.log. No further rerun performed. No workflow, download logic, pin, protected hashes or security setting changed. Report the original HTTP503 as transient provisioning infrastructure; this does not repair the still-failing original main correctness run.

## Additional preparation failures and their limits

First cargo test --workspace failed at desktop link only: missing development link names for xcb/xkbcommon/xkbcommon-x11 (/tmp/opencut-workspace-tests.log). System runtime libraries are installed and ldconfig reports their actual paths. Attempted unprivileged apt-get download could not resolve packages; it also reported an unreadable managed apt config. No apt operation was retried and no permissions/configuration changed. An independent local test linker directory /tmp/opencut-native-link contains only three .so symlinks to those already-installed actual system runtime libraries; LIBRARY_PATH points there for the workspace test command. This is temporary test preparation, not a replacement library, repository change or system installation. The prepared full workspace run remains in progress at this entry.

Full default bridge unit run passed994 tests with15 unchanged configured optional/platform omissions in88 files (78 executable/10 skipped); log /tmp/opencut-unit-all.log. Actual packaged smoke passed40/40 using the existing assembled release runtime and unchanged test command, log /tmp/opencut-packaged-smoke.log.

Initial source integration run41/43 passed with two failures: existing grid job polling remained running at its matcher deadline and audio-analysis PID absence failed. That run accidentally omitted the owned subreaper and ran while full native Rust tests were busy. These facts do not prove the grid failure cause. Full original log /tmp/opencut-integration.log is retained. Rerun complete unchanged integration after native load settles and with the owned subreaper; do not alter grid/audio tests, assertions or deadlines.

Prepared source integration rerun passed all43 cases with the original test command/assertions/timeouts (/tmp/opencut-integration-prepared.log), including both initially failing grid/audio cases. The initial41/43 result remains retained; no grid/audio implementation or test was changed. The successful rerun used the owned child reaper and substantially lower native build load; no precise causal claim is made for the grid timing failure.

Read-only confirmation of the separate HTTP499 report: PR228 original platform job118159264979 and release job118159265146 logs both fail the same Chocolatey V2 FFmpeg7.1.1 feed request with status499 before native testing. Full logs /tmp/opencut-pr228-download-{jobid}.log. URLs: https://github.com/matiHirCab/AI-Open-Cut/actions/runs/39282460233/job/118159264979 and https://github.com/matiHirCab/AI-Open-Cut/actions/runs/39282460324/job/118159265146 . No PR228 rerun, edit, branch update or rewrite performed. Main's independently authorized attempt2 success remains the current infrastructure observation.

## Completed local implementation checks

Prepared full cargo test --workspace completed with exit0 (/tmp/opencut-workspace-tests-prepared.log), including desktop linkage to installed libraries, all ordinary workspace tests and doc tests; existing ignored optional evidence cases remain unchanged. Full source integration subsequently passed43/43 and assembled release smoke40/40. All required implementation checks are now complete. No source changed after passing bridge unit/types/lint/Clippy/fixture checks; later edits only record specification/evidence and delivery bookkeeping.

## Postarchive local closure

Only this verified change was archived and its two approved requirements synchronized without modifying unrelated living requirements. Postarchive protected gate passed521 policy controls/76 living specifications and separate strict validation76/76; logs /tmp/opencut-postarchive-protected.log and /tmp/opencut-postarchive-strict.log. Final frozen-tree prepublication gate/strict runs follow this documentation update. Publication and terminal exact-head CI are still future observations recorded in PR metadata; no main repair or merge is claimed.
