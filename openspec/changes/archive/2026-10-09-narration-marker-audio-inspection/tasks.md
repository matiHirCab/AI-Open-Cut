## 1. Approval and predecessor evidence

- [x] 1.1 Obtain explicit user/reviewer approval of proposal, design, both capability specs and this task list; record the approval before implementation. User replied "Approve the proposed scope" on 2026-10-09.
- [x] 1.2 Confirm PR164 final exact-head required CI/review evidence from its archived normalization completion guard; retain full logs and report inaccessible evidence. Public final review5466128898 records all11 CI37886272536/startup37886272512 and twelve full checkout logs; checks page agrees and main/PR164 trees both equal46e8d0ccc15d77beb2a7a5a90174dbd16bba6165. Original full log download is inaccessible under current API policy; local HTML/text snapshots are retained in /tmp/opencut-pr164*. No predecessor success is inferred from merge alone.
- [x] 1.3 Provision pinned Bun1.4.0/Rust1.97.0/Moon2.3.3 and native/desktop dependencies; record tool versions and GUI readiness without changing project pins.
- [x] 1.4 Run existing `cargo test -p opencut-editor-core --test speech_alignment --test speech_markers` and bridge speech tests before changes; verify unchanged saved/explicit alignment and provider catalog evidence (N2/N3). Rust19pass (/tmp/opencut-alignment-baseline.log); bridge39pass (/tmp/opencut-bridge-alignment-baseline.log), both on unchanged main implementation.

## 2. Independent recipe and lifecycle coverage

- [x] 2.1 Author versioned recipe literals and independent visual/audio references before production/presentation edits; map each cue and expected result to N1/N6/N7.
- [x] 2.2 Implement a test-only fresh-store generator using existing typed core operations, local synthetic sources, cue-bound presets/events and ducking/normalization (N1).
- [x] 2.3 Cover saved sentence cues and explicit alignment on unaligned assets, standalone/batch selected-word alias behavior and exact provenance lifecycle (N2).
- [x] 2.4 Cover invalid alignment/selection/source/scope, typed failures, revision priority, late rollback, ordered aliases and cue movement, preserving complete authoritative/resource bytes (N3/N4/N5).
- [x] 2.5 Verify unchanged schema44 current/retained history and exact undo/redo/reopen; document no migration/public contract/worker change (N2/N4).

## 3. Desktop presentation

- [x] 3.1 Add bounded root/component cue inspection and process-local scoped revision-bound page/selection state; cover empty scopes, same-name local cues and maximum paging (D1/D2).
- [x] 3.2 Add selected speech segment/provenance and authored media/event/audio control summaries with bounded bus paging, no path leakage or duplicated core resolver (D3/D4).
- [x] 3.3 Add presentation/session tests for refresh/history/selection invalidation, truthful absence and actual large-collection nonmaterialization (D1/D2/D3/D4).
- [x] 3.4 Build `cargo build -p opencut-desktop --release`; perform/record actual manual inspection, paging, refresh, undo/redo and reopen against generated fixture; retain GUI evidence or explicit blocker (D5).

## 4. Integration and native evidence

- [x] 4.1 Add real bridge/headless coverage using existing standalone/batch operations and aliases; preserve all historical catalogs and speech-worker bytes (N2/N3/N4).
- [x] 4.2 Implement independent full/range/draft preview/export semantic/native cue/preset/compositing/event/ducking checks without changing existing thresholds or expected oracles (N6).
- [x] 4.3 Verify complete-root precodec normalization with independent delivered loudness/true-peak evidence and negative controls; document crop/codec limits (N6/N7).
- [x] 4.4 Document fixture generation, inspection workflow, scope/time/quality semantics, compatibility, verification commands and actual evidence limitations.

## 5. Required validation and review

- [x] 5.1 Run root `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` and focused new fixture/native/desktop tests with configured FFmpeg/FFprobe/font paths. Retain full logs and all original configured native/parity/rules checks.
- [x] 5.2 From apps/agent-bridge run `bun run typecheck`, `bun run lint`, `bun run test`, `bun run contracts:check`, `bun run test:integration`, `bun run test:smoke`. Run the original hermetic worker suite from apps/kokoro-tts via `bun run ../agent-bridge/scripts/run-python-tests.ts`; confirm unchanged provider fixtures.
- [x] 5.3 Run `bunx @fission-ai/openspec@1.5.0 validate --all --strict --no-interactive` and prearchive `moon run root:openspec-validate`; accept only this active-change inventory rejection before archive, report every other failed/skipped required check.
- [x] 5.4 Apply repository `$openspec-verify-change` and substantive implementation/spec/coverage review; resolve all mismatches and obtain designated contract/CODEOWNER review where applicable.
- [ ] 5.5 Apply `$openspec-sync-specs` and `$openspec-archive-change` only after passing implementation/manual/conformance evidence. Run postarchive `moon run root:openspec-validate` and pinned strict-all validation; retain exact-head evidence and required CI acceptance.

## 6. Publication and successor

- [ ] 6.1 Commit scoped verified work, push feature branch and open a draft PR targeting main with implementation, tests, review and limitations; do not merge or deploy. Report GitHub API denial if publication remains blocked.
- [x] 6.2 Read live successor issue/dependency state and recoverable work; select the next dependency-ready issue, prepare or reuse its approved OpenSpec artifacts and continue only within that approved scope.

## Evidence currently retained

Pinned tools are in /tmp, with no repository pin/configuration changes. Release desktop build and real Xvfb/software-Vulkan GUI workflow pass; final screenshots are /tmp/opencut-gui-final-*.png. Bun unit743pass/9optional skips; source integration33pass; packaged smoke30pass; contract parity594TypeScript pass plus all its Rust/desktop suites; hermetic Python12unittest+12pytest pass. Final Rust formatting and strict workspace Clippy pass. Original release golden/performance and all three required rules matrices pass with the reviewed font. All nine optional native bridge tests have been activated and pass; instrumented Rust worker12pass. The full Rust workspace run passes:1486tests/73suites,0failures,9 intentional maintenance/helper ignores (/tmp/opencut-workspace-final-v3.log). Default-feature headless restoration build and83 release transport tests pass; postarchive local acceptance passes; remote-CI/publication acceptance is still pending. All early runtime failures are retained and are not reported as successful.

Successor issue72 proposal/design/spec/tasks explicitly approved by the user on 2026-10-09 ("yes"). Its separate main-based worktree is /workspace/AI-Open-Cut-issue72; initial private preview cache implementation has started without introducing its active change into this checkout.

Final prearchive strict validation56/56pass (/tmp/opencut-spec-all-prearchive-v2.log). The unchanged Moon task /tmp/opencut-openspec-prearchive-v7.log rejects solely the active narration-marker-audio-inspection change, with no other failure; this is the expected prearchive rejection and not a successful protected gate.

Final implementation conformance verified via openspec-verify-change: all7requirements/12scenarios covered; no behavior/design/coverage mismatch. Default headless restoration83pass (/tmp/opencut-headless-default-final.log). Archival/postarchive local validation now passes; remote-CI/publication remain pending and are not inferred complete from local test success.

Both delta capabilities synchronized and change archived2026-10-09. Postarchive Moon gate and pinned strict-all57/57pass (/tmp/opencut-openspec-postarchive-final.log, /tmp/opencut-spec-all-postarchive-final.log). Task5.5 remains unchecked only for remote exact-head required CI acceptance; task6.1 is publication-pending.
