# Issue #70 conformance review

Status: implementation conformance verified. All required local implementation,
manual and native checks pass. Synchronization/archival and postarchive protected
validation pass; draft publication and remote exact-head CI remain
blocked. This is the implementing agent's substantive self-review, not an
independent human approval or a GitHub APPROVED review.

## Base and approval

The user approved proposal/design/deltas/tasks on 2026-10-09 with “Approve the
proposed scope”. Initial checkout/stash/worktree/unreachable-object/remote-branch
inspection recovered no prior #70 implementation. PR164 is merged. `origin/main`
is c0031b6011a9000ba698bb105ecd0a122d29a625; its tree and PR164's final head
49189084d5b43a3832e4c3d44563056666c0af2b both resolve to
46e8d0ccc15d77beb2a7a5a90174dbd16bba6165. Work starts from that main tree.

Published PR164 final review5466128898 records all11 checks from CI37886272536,
focused startup37886272512 and twelve successful full checkout logs. The public
checks page agrees. Local copies are /tmp/opencut-pr164*. Original full logs
cannot be independently downloaded under the current GitHub API policy. Merge
alone was not taken as verification, and this limitation remains explicit.

## Correctness and traceability

| Scenarios | Implementation and automated evidence |
| --- | --- |
| N1 | versioned recipe; tests/support/narration_fixture.rs fresh typed standalone/batch generator; narration_fixture.rs literal six-cue authoring assertions; example generator |
| N2 | saved_and_explicit_alignment_generate_exact_equivalent_scoped_cues; full provenance/history test; unchanged speech_alignment/speech_markers suites cover canonical standalone selected-word policy and batch one-result aliases |
| N3 | unchanged canonical speech_alignment and speech_markers failure/resource suites; narration_fixture integrated missing-source/selected-word failures; bridge speech-alignment/known-text-alignment/speech-markers and contract parity suites |
| N4 | standalone/batch exact history/reopen/resource inventory; late rollback, stale revision and forward alias tests; unchanged markers ambiguous-name tests; new source/packaged MCP workflow and existing full catalog tests |
| N5 | integrated cue move, undo/redo/reopen and invalid dependent interval rollback; renderer semantic moved-draft proof; unchanged moved_marker_changes_preview_draft_range_and_export_pixels; actual GUI moved-cue history |
| N6 | renderer/golden/narration_fixture.rs exact shared nonempty plans, independent PNG/movie plates at all cue boundaries, full/range/draft/export movies, decoded independent AAC and full-root precodec sample equations; original frozen native suites remain unchanged |
| N7 | blank plate, shifted audio, omitted captured gain/events and wrong delivered normalization level controls; independent ebur128/true-peak measurement |
| D1 | desktop Cursor borrows32-cue slices; 4096-cue page/scope identity test, empty component and same-name local markers; actual GUI41-cue paging and component EVERY10ms versus root500ms |
| D2 | successful shell session transitions and item selection reset process-local cursor; revision-bound ID lookup; stale revision/reset tests, core history evidence and actual GUI refresh/undo/redo/reopen |
| D3 | bounded descriptions borrow saved source alignment, show exact metadata/one segment and authored audio/captured event settings; truthful absent source test; exact owned-byte nonmutation test; actual aligned/music/event GUI |
| D4 | 100000-word alignment validated against source duration, pointer equality to stored slice, last segment/empty phoneme navigation and bounded output assertions; allfour existing buses summarized, no whole alignment serialization |
| D5 | release build and actual Xvfb/software-Vulkan desktop workflow; GUI evidence listed below, supplemented by desktop tests |

The test-only recipe's placement under contracts does not introduce a wire
contract. Existing headless/bridge/provider implementations and canonical
catalogs are byte-unchanged. Schema44/protocol1 remain current; no migration or
new ownership/dependency edge exists. Presentation reads already validated core
state and never evaluates marker expressions itself. The `extern crate self`
alias and RecordingProcess visibility/visual delegation are test-only support.
No original oracle threshold, fixture, failure case or golden is rewritten.

## Manual desktop evidence

Release build logs: /tmp/opencut-desktop-release.log and final presentation
layout /tmp/opencut-desktop-release-v3.log. Existing native libraries were linked
through /tmp/opencut-native-lib. Desktop runs on Xvfb:93 with Mesa software
Vulkan; package files and caches are disposable under /tmp. No production
installation or deployment was performed.

Original generated fixture: /tmp/opencut-narration-gui-v2; project
9cae829c-7185-4201-b485-bd5c3b383e11, authored revision6. A disposable batch adds
35 root markers plus a component-local EVERY10ms, producing revision7. These
manual additions use existing headless typed operations; they are not new
recipe/persistence fields. GUI undo/redo produces revisions8/9. A headless cue
move to600ms followed by GUI refresh produces revision10; GUI undo returns500ms
at revision11 and redo restores600ms at revision12. Reopening starts with root
page1, no selected cue/segment, and retained revision12 values.

Screenshots under /tmp/opencut-gui-final-*.png record:

- root:41 cues/page1; page2:the remaining9 cues; component:local EVERY10ms;
- saved:Estimated producer synthetic-narration-fixture/null model/version,
  six sentences/seven words/no phonemes, EVERY500–900ms;
- word:selected word2/7 SINGLE1000–1400ms; unaligned:music alignment absent;
- event:EVERY+0 expression/synchronized500ms, variant1/seed1/sfx/-6/-3dB,
  captured content hash and alignment absent;
- refresh:root cursor reset; undo/redo:original six versus41 markers restored;
- moved-root:EVERY600ms and visual600–1000ms; cue-redo:captured event binding
  synchronized600ms; cue-undo:visual500–900ms;
- reopen:revision12, root page1, retained600ms, no prior narration selection.

All four bus settings were read in the GUI: music ducking enabled/source
voiceover/gain0.25/attack0/release0; normalization enabled/-24LUFS/7LU/-2dBTP.
The initial dense/clipped inspection was rejected and corrected with labeled
rows and width/wrapping constraints; final saved/event screenshots contain the
complete values. Desktop preview remains the existing placeholder and is not
presented as native rendering evidence.

## Passing evidence and runtime failures

Pinned tools: Bun1.4.0, Rust1.97.0 (rustfmt/Clippy), Moon2.3.3. Tooling, native
libraries and GUI dependencies are under /tmp; project pins are unchanged.
Moon uses its supported MOON_TOOLCHAIN_FORCE_GLOBALS=1 with these pinned
executables, avoiding proxy-unavailable proto version discovery. Protected task
contents and policy checks are unchanged.

- Original alignment baseline:19Rust tests; bridge speech baseline39tests.
- New core fixture lifecycle:3pass in final workspace,256.68s, including the final
  invalid-dependent-interval rollback control.
- New desktop presentation:3pass; final workspace desktop43pass.
- New native fixture before final added negative controls:2pass, including
  decoded AAC, movie SSIM and independent normalization measurement;
  final workspace core library688pass/9 intentional maintenance ignores includes
  the new native fixture and final negative controls; full workspace73 suites
  now pass (1486tests,0failures,9 intentional maintenance/helper ignores).
- Typecheck/lint pass: /tmp/opencut-narration-typecheck-v2.log,
  /tmp/opencut-bridge-lint-v3.log (202files).
- Standard TS unit743pass/9optional native skips: /tmp/opencut-bridge-unit-v2.log.
- MCP integration33pass: /tmp/opencut-bridge-integration-v3.log.
- Packaged smoke30pass: /tmp/opencut-bridge-package.log.
- contracts:check pass: /tmp/opencut-contracts.log; TS594pass plus its Rust and
  desktop suites. Python12unittest+12pytest pass: /tmp/opencut-python-worker.log.
- Final formatting and workspace strict Clippy pass; symbol-free Clippy log:
  /tmp/opencut-clippy-final-v2.log. Removing debug symbols changes neither
  assertions nor optimization; workspace tests retain debug assertions.
- Strict-all56/56pass: /tmp/opencut-spec-all-final.log.
- Prearchive protected task ran unchanged and rejected only this active change:
  /tmp/opencut-openspec-prearchive-v6.log. This is expected rejection, not success.

Earlier failures are retained, not hidden: missing default headless path,
worker exit with observed OOM, orphan-descendant reaping on a nonreaping PID1,
read-only Moon/cache paths and proto offline discovery, initial PCM/int-float
expectations, linear-light visual reference mismatch, independent AAC float/gain
representation mismatch, and final debug build disk exhaustion. Corrections use
normal target paths, a Linux subreaper for test descendants, writable caches,
independent reference equations, and disposable symbol-free build artifacts.
No assertions, thresholds, test timeouts, required selection or CI pins were
weakened. Build/test logs at /tmp/opencut-* retain failed and passing runs.

## Current acceptance blockers

The workspace integration run passes with exit0: /tmp/opencut-workspace-final-v3.log. Activated original native/rules evidence passes (details below). Default headless restoration build and all83 release transport tests pass with exit0 (/tmp/opencut-headless-default-restore.log and /tmp/opencut-headless-default-final.log).
Synchronization/archival is prohibited until implementation checks and this
review pass; postarchive protected validation must then pass. GitHub API still
returns Forbidden in the current session, so live exact-head CI acceptance and
draft-PR creation remain unverified. A supported configuration draft adding only
api.github.com exists; current environment status still reports the old policy.
No merge or deploy is authorized or performed. Issue72 is the next live
dependency-ready issue (71/15 closed); its proposal/design/spec/tasks were explicitly approved on 2026-10-09 ("yes"); implementation has started in an isolated main-based worktree.

## Additional final native evidence

- All nine optional bridge native tests explicitly activated: eight pass in
  /tmp/opencut-bridge-native-final-v2.log; the instrumented persistent worker
  passes in /tmp/opencut-instrumented-worker-ts.log. Standard unit skips do not
  substitute for these runs.
- Instrumented Rust worker12pass: /tmp/opencut-instrumented-worker-rust.log.
- Original release golden conformance and sampled performance pass:
  /tmp/opencut-original-golden-final.log and
  /tmp/opencut-golden-performance-final.json. Independent report validation
  passes in /tmp/opencut-performance-report-final.log.
- Required original rules matrices at960x540,1280x720,1920x1080 all pass:
  /tmp/opencut-rules-960x540-final.log, /tmp/opencut-rules-1280x720-final.log,
  /tmp/opencut-rules-1920x1080-final.log. Existing PR scope is unchanged.
- Initial native bridge debug-sidecar hero tests exceeded their unchanged120s
  timeout; the supported release-sidecar override passes. The original reviewed
  font SHAae7b7855e115a5966d8b1b3f80f254ccc117ec86f9965e202ee2940453837280
  fixes the earlier system-font guard failures without rewriting any oracle.

## Final conformance decision before synchronization

Completeness: all seven approved requirements and twelve scenarios have the
mapped automated evidence, supplemented by the required actual GUI workflow.
All local implementation checks pass on the stable code tree; full workspace
1486pass and restored default headless83pass. No implementation requirement is
missing. Publication/remote-CI and postarchive tasks remain accurately pending;
they are phase/acceptance blockers, not silently checked off.

Correctness: presentation borrows authoritative core state, bounds collections,
resets stale selections and performs no domain mutation/inference. Both existing
alignment paths and selected-word aliases retain canonical typed behavior. The
independent recipe/native oracles cover timing, linear-light compositing, captured
events, routing/ducking, complete-root normalization and negative controls.
Neither predecessor thresholds nor expected reference files change.

Coherence: implementation follows approved design and existing inward ownership;
no new dependency edge, schema migration or public contract change occurs.
CODEOWNERS still names @matiHirCab for covered files, but the designated
cross-language contract-change approval gate is not triggered by this test-only
recipe or cfg(test) support. A later PR review remains distinct from this self-review.

No unresolved behavior/spec/test mismatch was found. The verified deltas may be
synchronized and archived under the approved lifecycle. Postarchive protected
validation must pass before claiming local merge readiness. Remote required CI
acceptance and draft creation remain unverified; no merge/deploy is authorized.

## Postarchive local acceptance

Both capabilities are synchronized into living specs and the verified change is
archived at openspec/changes/archive/2026-10-09-narration-marker-audio-inspection.
The protected Moon task passes unchanged (/tmp/opencut-openspec-postarchive-final.log);
pinned strict-all validation57/57pass (/tmp/opencut-spec-all-postarchive-final.log).
All local required checks pass. Remote exact-head CI and draft-PR publication
remain acceptance blockers; task5.5's remote-CI portion and task6.1 remain unchecked.
The approved task/design lifecycle authorizes archiving verified behavior before
external publication; no pending external result is fabricated as completed.

## CI repair follow-up (2026-10-09)

Original head `124a73533a5feaed406985c7ecd16d733a18b157`, CI run `37958536586`: Linux job `113915152790`, Windows `113915152942` and macOS `113915153009` all fail `chunks_exact_to_as_chunks` at `renderer/golden/narration_fixture.rs:127`. Their Rust tests are skipped after Clippy, so these logs do not establish a readiness-test regression. The synthetic merge checkout pairs this head with main `c0031b6011a9000ba698bb105ecd0a122d29a625`.

The approved N6/N7 test helper now reads complete `[u8; 4]` chunks and decodes each with the same `f32::from_le_bytes`. The existing length-divisibility assertion remains. Complete-chunk iteration and any trailing-byte exclusion remain equivalent. Rust's array chunk methods are stable since 1.88, below pinned 1.97. No fixture, oracle, threshold, test selection, timeout or warning policy changes.

Repair conformance review follows `openspec-verify-change`: scope is the archived approved native-evidence tasks, N6/N7 requirements and design decision 4. Completeness of external CI acceptance remains pending; correctness and coherence have no repair-specific mismatch. This is agent review, not independent human approval. Full logs are uncommitted under `/tmp/opencut-ci-logs/`; final committed-head validation and remote results will be reported separately.
