# Platform renderer packaging conformance

Approved change: `verify-platform-renderer-packaging`, implementing issue #78 under the delegated authorization for bounded specification approval, implementation, verification and draft publication. Reviewed proposal, design, tasks and all three requirements/seven scenarios using the repository OpenSpec verification workflow.

This report separates verified local implementation from platform acceptance. Exact published-head Windows/macOS/Linux native execution and all original required CI must reach terminal success before accepting #78 or advancing #79. Those remote results are pending publication; no Windows/macOS execution is claimed here.

## Completeness, correctness and coherence

All four local implementation tasks are complete. Three requirements have implementation and automated coverage; all seven scenarios map below. No unresolved implementation mismatch was found. The external acceptance condition for P6 remains pending. No public/project/provider DTO, capability identifier, protocol1 or persisted schema44 changed. Existing EditorCore readiness owns filter and media-tool detection; the bridge gained no competing scene or validation rules. Existing protected workflows, checks and deadlines retain their original bytes.

| Scenario | Implementation and coverage |
| --- | --- |
| P1 healthy package | `scripts/package-runtime.ts` keeps four roles/version1 and normal layout. Positive plain/`.exe` manifest tests and actual default package verify regular bytes, hashes, inventory and POSIX executable permissions. |
| P2 corruption/escapes | 49 package tests cover closed/future/malformed manifests/entries, unsafe byte counts/hash, multi-chunk digest/size drift, portable paths, case/worker-directory collisions, links/junctions/root/manifest/nonregular/missing/extra files and empty directories. Failed source/name preflight preserves the prior destination. An extra POSIX filename resembling a worker path after backslash conversion is independently rejected. |
| P3 availability | Both actual SDK clients exercise real healthy tools and a native controlled filter-inventory fixture. Existing canonical status determines base/optional capabilities. Readable fixture path diagnostics are explicitly distinguished from usable filters. Requested master DSP with missing optional filters fails instead of dropping processing. |
| P4 fallback lifecycle | Missing FFmpeg, missing FFprobe, missing base filters and incomplete optional filters preserve editor capabilities. Each case creates, edits, undoes/redoes and freshly reopens a project. Failed renders retain authoritative files/revisions, return `DEPENDENCY_UNAVAILABLE` and publish no artifact. |
| P5 actual media | Source/default-package clients use the actual release headless and compiled bridge with both original workers. Genuine PNG/H264/AAC, independent gold/black pixel and nonzero audio witnesses, matched range/export metrics, invalid/missing-reference/stale rollback, undo/redo and fresh state/frame reopen are exercised. The sealed runtime verifies before and after execution. |
| P6 all platforms | Separate `motion-platform-renderer.yml` matrix executes all three OSes with pinned Rust/Bun/Moon/Python, actual FFmpeg/FFprobe and existing fixed font bytes. Driver requires both expected native assertions to pass in the actual JSON report with zero skips. Local Linux executed; exact-head Windows/macOS/CI remain the explicit acceptance condition. |
| P7 orchestration refusal | 14 workflow policy tests reject changed platforms/pins/commands, optional conditions, failure masks and private instrumentation substitutions. Nine report tests plus an actual all-skipped invocation reject absent, skipped, failed or unrelated execution. Five hermetic Python setup cases cover fixed-family identity and missing/tampered/unusable dependencies. |

Manifest reads are bounded16KiB; binary verification hashes64KiB chunks. The verifier checks static caller-owned package integrity, including links, canonical containment and undeclared directories. It does not claim a concurrent filesystem publication transaction or enforce project-path policy. Existing bundled font provenance/license and canonical identities are unchanged; setup verifies all four faces without adding assets or downloads.

## Passing implementation evidence

All listed local checks passed. Long logs and exit files remain outside the repository.

| Check | Result / evidence |
| --- | --- |
| Stable full bridge unit | 824 passed,15 existing/opt-in skips; `/tmp/mg78-unit-final.log`, exit0. Includes49 package and9 native-report cases. Two new optional native bodies are separately required/executed below. |
| Type/lint/scoped style | `/tmp/mg78-alias-{type,lint,style}.log`; TypeScript passes, lint212 files and scoped changed files pass. |
| Actual native SDK/default package | Two passed/zero skips on Linux; `/tmp/mg78-native-final-reviewed.log`, exit0; actual JSON report and original media in `docs/verification/platform-renderer-packaging`. |
| Source integration | 34 passed; `/tmp/mg78-integration.log`, exit0. |
| Default packaged fake-provider smoke | 31 passed; `/tmp/mg78-smoke.log`, exit0. This remains distinct from actual-media native evidence. |
| Workflow policy | 14 passed; `/tmp/mg78-policy-sealed-final.log`, exit0. |
| New Python setup | Five passed; `/tmp/mg78-python-sealed-final.log`, exit0. Actual tools/font setup passed separately in `/tmp/mg78-setup-actual-retry.log`, exit0. |
| Formatting | Current `cargo fmt --check --all`, `/tmp/mg78-fmt-sealed-final.log`, exit0. New standalone Rust inventory fixture is rustfmt checked and actually compiled/executed by the native driver. |
| Required negative invocations | Missing required font input exits1; missing actual FFmpeg executes two failing native cases and exits1. Actual all-skipped native report is rejected; outer control exits0. Logs `/tmp/mg78-{required-input-negative,missing-tool-negative,skip-negative-final}.log`. These are refusal evidence, not native success. |

Native observations:192x108/10fps/10 frames/one second H264, stereo48k AAC. Both clients retain SSIM0.998298 and floatPCM RMS0, meeting>=0.99/<=0.0001 and one-frame timing bounds. Solid-color interior allows three RGB rounding levels and rejects a black control; audio has independently required nonzero energy. Actual tool versions and hashes remain recorded. Local Python is3.12.14; the matrix explicitly uses3.11, which is not claimed as locally executed. No listening, model-inference quality, desktop GUI, creative-quality or performance claim follows from this small portability scene. Existing #70/#77 fixture requirements and the separate rescue proposal remain intact.

Unchanged Cargo/core/headless/desktop/public-contract/provider inputs reuse their verified #77 lineage evidence: full workspace1505 passed/zero failed/10 configured ignores (74 blocks), latest focused reference3 passed603.66s, final lineage formatting/strict all-target/all-feature Clippy/all84 headless tests, full Rust plus605 TS contract checks and unchanged provider Python checks. The original logs are identified in the #77 archive/checkpoint. New package policy/harness inputs were independently rerun. No required Rust assertion, budget or workflow was weakened.

## Retained failures and corrections

The first full unit invocation failed36 cases:33 unchanged descendant cleanup checks observed zombie liveness in this executor without its existing subreaper, and three new package collision tests saw the previous module while implementation was being edited. Both logs remain; the stable unchanged command under the owned subreaper/taskset0–3 subsequently passes. Assertions and deadlines were preserved. A later stable full rerun includes all current inputs and passes824 cases.

Review reproduced a flaw in the inventory rewrite: converting all backslashes on POSIX could accept an extra filename as a declared path. `/tmp/mg78-inventory-alias-baseline.log` records the failed regression before correction. Conversion now uses only the actual host separator; the final full suite includes the negative. No vulnerable version was published.

Initial native harness failures for incomplete font faces, RGB conversion rounding and reading a disposed preview after shutdown were corrected in the harness, with original logs retained. Original frame bytes are retained before closing the client for reopen comparison. SourceForge download returned403 and was not a setup pass; existing bundled fonts remove that unnecessary dependency. Initial local setup lacked an existing `RUNNER_TEMP` parent; creating the owned parent allowed the actual setup to pass. No production renderer, provider or frozen reference was changed to mask a failure.

## OpenSpec and delivery

Pre-archive protected gate validates61 items and rejects only this active change: `/tmp/mg78-prearchive-final.log`, exit1. That rejection is expected and is not reported as a passed gate. Synchronization adds the reviewed capability to living specs; archive follows under standing authorization. Final protected/strict gates must pass after archival and before draft publication. Remote exact-head platform/original CI remains an acceptance blocker until terminal success; no merge/deploy is authorized.

Final delivery gate: after synchronization/archive, `moon run root:openspec-validate` passes and independent pinned strict validation passes61/61 items, zero failures. Logs `/tmp/mg78-final-gate.log` and `/tmp/mg78-strict-final.log`, both exit0. The archive contains all four checked local implementation tasks and the exact accepted delta; remote platform/original CI acceptance is still pending draft publication. `git diff --check` passes. Unrelated repair worktree is clean.
