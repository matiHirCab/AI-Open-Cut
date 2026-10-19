# Verification: source-grounded short planning (#218)

Status: implementation and local conformance verified; synchronized and archived on 2026-10-19. Postarchive protected and strict gates passed. Draft publication and exact-head required CI remain pending and are tracked in PR metadata.

Current base: b3173e7f, following the user’s external merge of PR226. The #218 branch was fast-forwarded by rebase with its tracked/untracked work preserved. No agent PR merge occurred. All 14 frozen #218 implementation/fixture files, 267 core/headless/build inputs and the actual native executable SHA remained identical. Full TS/Python/own real CLI evidence is reusable because all relevant inputs/toolchain/environment are unchanged; new incoming desktop code is checked separately below.

## Scope and provenance

Approved before executable edits on 2026-10-19 following independent artifact review. Standalone local version-1 Bun CLI; explicitly supplied and approved source transcript, deterministic grouping and supplied editorial fields. Existing core resolves managed identity/integrity/revision twice; no new core/MCP/provider contract, inference, upload, download, timeline mutation or caption-time reconstruction. The actual workflow uses original synthetic PCM media and a manually supplied synthetic transcript, not an ASR accuracy claim.

Only existing integration files changed are contract-ownership-v1.json, contracts.test.ts and the historical narration-profiles projection boundary. This does not change narration implementation. Future #208 integration may require a semantic union of these central additions; no access to the unauthorized device/WIP was attempted.

## Requirement/scenario mapping

| Requirement | Implementation | Automated evidence |
| --- | --- | --- |
| Explicit source-grounded local planning | service.ts checks core-resolved identity/audio/hash/duration/revision before/after; CLI emits sanitized errors | unit read-only call inventory, source mismatches, revision/digest/duration races; actual managed-media CLI workflow and authoritative hash inventory |
| Bounded complete transcript provenance | contract.ts closed records and limits; planner.ts complete ordered chunks, checked offsets, original words | canonical invalid bounds/gaps/provider examples, duplicate/nonordered timing, aggregate segments/words, invalid UTF-8/byte cap, actual CLI input/output caps |
| Editable candidates with honest evidence/context | planner.ts immutable evidence, greedy grouping, explicit selected scope/edits, adjacent context and ordered symmetric warnings | independent canonical negation/uncertainty/original/cut/edit oracles; duration, partial cuts, overlap, invalid edits and unmodified supplied text tests |
| Governed additive interface and isolated ownership | independent CLI fixture/category, real consumer in existing contracts.test.ts; exact own-byte historical projection | captured predecessor raw/SHA, missing/partial addition and consumer negatives, unchanged historical pins/assertions; compact and pretty roundtrip, retained unrelated drift |

## Evidence collected

- Fresh full Rust on original main fa5: fmt, strict workspace/all-target Clippy and cargo test --workspace PASS; 1583 target cases plus 9 nested subprocess replays, 13 existing ignored tests. Logs /tmp/opencut-218-final-{format,clippy,workspace}.log. These core/headless/build inputs are byte-identical after updating main; all 267 relevant inputs were independently hashed before/after. Incoming desktop changes passed fresh workspace format/strict all-target Clippy and 58 desktop cases (2 existing opt-in ignored). Logs /tmp/opencut-218-rebased-{format,clippy,desktop}.log. Combined current-base target coverage is 1588 cases plus 9 nested subprocess replays; unchanged core/headless evidence is reused with the 267-file hash proof. No new warning suppression; Cargo’s existing proc-macro-error2 future-incompatibility advisory remains.

- Full `bun run contracts:check --maxWorkers=2` passed on current main: all selected Rust suites plus configured native oracles, and 646 TypeScript cases across 42 files. `/tmp/opencut-218-second-final-contracts.log`. The native hero oracle that lacked its font in the first run executed successfully with the actual checked-in font.
- Focused corrected historical + own checks: 49 passed, `/tmp/opencut-218-projection-correction.log`.
- Full TypeScript unit suite with native path and container subreaper: 989 passed, 15 existing optional tests skipped; own 29 tests all executed. `/tmp/opencut-218-second-full-ts-unit.log`. Separate required integration/smoke/parity and own actual workflow remain distinct evidence.
- Final corrected typecheck and lint passed; lint checked 296 files. `/tmp/opencut-218-final-corrected-{typecheck,lint}.log`.
- Final rebuilt-native CLI acceptance: debug and release each PASS with zero readiness skips, unchanged 4 authoritative files/source/history and independently authored candidate/cut/editorial/overlap oracles. `/tmp/opencut-218-final-native-{debug,release}.log`; binary digests are recorded in corresponding `.sha256` files.
- Packaged fake-provider smoke: 40 passed using the actual assembled runtime, `/tmp/opencut-218-final-smoke.log`.
- Source MCP integration: 43 passed, `/tmp/opencut-218-final-integration.log`.
- Hermetic Python workers: 12 unittest + 12 pytest passed, `/tmp/opencut-218-final-python.log`.
- Prearchive protected Moon gate: 521 policy tests and strict 76 items passed; only expected own active-change inventory rejection. `/tmp/opencut-218-prearchive-protected.log`. This is not final gate success.
- Actual native workflow exploratory final attempt passed, original tone digest d283f5c2ae433763d4d780c4980fb58cabb6f046128229f4e8c01f6a18ce72b2, captured revision 1, 4 authoritative files unchanged; stale/core/input/limit errors observed and output cap enforced before stdout. `/tmp/opencut-218-third-native-workflow.log`. Frozen-source real native rerun also PASS: /tmp/opencut-218-frozen-native-workflow.log, native SHA256 491cfa6845568ec224352c901452d028bdf972d500748089d78acabeb69217cb. The main-update digest confirmation passed. Subsequent package-specific Cargo testing relinked the debug executable to SHA256 0c0d9014ce2eea4f199d5960e5627bcb1342a55f00b8ef54291352cd08b8fd8a; final own CLI acceptance subsequently passed against both rebuilt debug and release binaries. The source manifest remains unchanged.

## Failures retained and corrected

Initial typechecking/formatting and fixture mistakes were corrected without suppressions. The full TS first run exposed a real new raw-projector incompatibility with unchanged compact historical input. An independently reviewed exact compact-suffix inversion corrected it without normalizing unrelated bytes or changing existing pins/assertions. That initial run also lacked the existing test native executable environment and container orphan subreaper, causing ENOENT/process-reaping failures; the correctly configured full rerun passed. No deadlines, warning policy, CI workflow, scripts or security settings changed.

The first full contracts command accidentally configured FFmpeg/ffprobe (enabling an existing optional native masked-hero oracle) without its required OPENCUT_TEST_FONT_PATH. It failed explicitly before that oracle could run. Retained log: /tmp/opencut-218-final-contracts.log. Integration/packaged commands did not start. The rerun supplies the actual checked-in DejaVuSans test font and keeps the native oracle enabled; no test/assertion was removed or weakened.

The first real workflow omitted the existing required OPENCUT_EXPORTS_DIR; fixture/documentation were corrected. Its next output-cap assertion used insufficient data to exceed 8MiB; increasing original fixture text within the existing 2000-character allowed limit exercised the real failure. Product limits and behavior were unchanged. Logs remain outside Git under /tmp/opencut-218-*.

## Limits and remaining work

Current checks execute on Linux; local CLI source portability is distinct from installed Windows/macOS runtime acceptance. Transcript provenance is supplied, not independently verified, and warnings do not prove semantic completeness/factual correctness. No new graphical behavior is introduced. #213 remains a Windows playback gap and #214 is paused.

The repository CODEOWNER retains final human review; independent agent review is not a recorded GitHub CODEOWNER approval. Publication, when verified, is draft only; no merge or deployment.

Independent final code/spec review: all 4 requirements and 8 scenarios reviewed, complete/correct/coherent with no implementation blockers or unsupported evidence claims. Explicit issue-only synchronization/archive approved after final results. Exact byte-only projector correction and unchanged historical negative controls confirmed. All required implementation suites are now complete. Final protected/strict closure and exact-head remote gates remain separate, observed milestones.

Final prearchive gates after all implementation checks: 521 policy tests and 76 strict items PASS; protected gate rejects only the own active change (expected, not gate success). Separate pinned strict 76 PASS. Logs /tmp/opencut-218-final-prearchive-{protected,strict}.log.

Source task 4.2 records preparation/checklist completion only. Synchronization/archive and postarchive protected+strict success were subsequently observed below. Exact-head required CI must still be observed before delivery, tracked in PR metadata without a future-CI claim in this source report.

## Final runtime artifact identities

dfd3bce7b7e43f5be9f36a8511395a8f5377310dbb5d009d91151d6e85be8ac7  /workspace/AI-Open-Cut/target/debug/opencut-headless
f95fbe45701ecbbb4197624c86b016b9bcf2de5a7c0f7ea67e38f5867b53a86c  /workspace/AI-Open-Cut/target/release/opencut-headless

Both artifacts passed the same original managed-media CLI acceptance after the required builds completed.

## Local conformance scorecard

| Dimension | Assessment |
| --- | --- |
| Completeness | 8/8 source preparation/implementation tasks, 4/4 requirements and 8/8 scenarios covered. |
| Correctness | Independent canonical outputs, negative controls, actual managed-media CLI and unchanged authoritative hashes agree with the approved specification. |
| Coherence | Existing core ownership, isolated bridge planning, unchanged wire/package/policy surfaces and historical proofs preserved. |

No unresolved implementation critical issues or warnings. Published-head release gates remain mandatory; no future CI result is asserted.

## Observed source closure

The only new living capability, source-grounded-short-planning, was synchronized with 4 requirements/8 scenarios. Only this change was archived at 2026-10-19-plan-source-grounded-short-candidates; its .openspec.yaml and approved artifacts were preserved. The separate unapproved #215 proposal remains in its own worktree and was not imported or archived.

Postarchive moon run root:openspec-validate PASS: 521 policy tests, 76 strict living items, valid unchanged CI parity policy and archive-only inventory. Separate pinned strict validation PASS: 76 items. Logs /tmp/opencut-218-final-protected.log and /tmp/opencut-218-final-strict.log. No unrelated changes were archived and no policy, deadline, warning rule or security configuration was changed.

Toolchain/environment reuse: same Linux x86_64 workspace, pinned Rust 1.97.0/Bun 1.4.0/Moon 2.3.3/proto 0.57.4, native system tools and dependency cache. All 267 core/headless/build inputs, 14 own frozen implementation/contract/test inputs and the unchanged wire/package/policy hashes were compared across the main update; only incoming desktop inputs needed and received a fresh complete desktop check. Later native relinks received their own actual CLI acceptance above.

Release gate after this source closure: push and create a draft PR, then require all configured exact-head workflows and the protected aggregate before declaring delivery complete. CODEOWNER remains pending; no merge or deployment is authorized/performed.
