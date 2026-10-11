# Verification: add-native-hero-card (#182)

2026-10-11. Scoped standing approval preceded executable edits. Main baseline `50ec842ca6c3c6bea72864b6878fecbc953a5a97` includes externally merged #181 and the separate doctor fix; neither is reimplemented.

## Completeness, correctness and coherence

All implementation tasks are complete. Four requirements and H1–H8 have automated coverage. The remaining synchronization/archival/post-archive task is the authorized administrative handoff following this conformance review. No critical implementation findings remain. The recipe is static example data in the existing prompt; core still owns validation, edits, resources, history and rendering. Public structural catalogs, schema/protocol pins, renderer limits, native reports and workflows are unchanged.

| Scenario | Implementation and automated evidence |
| --- | --- |
| H1 discovery/read-only | `src/server/context.ts`, `src/recipes/native-hero-card-v1.json`, `tests/native-hero-recipe.test.ts`: actual in-memory MCP listing/retrieval, unchanged prompt arguments, deterministic data, typed existing-operation parsing. Existing motion prompt regressions pass. Shared actual source/packaged workflows also retrieve before edits and verify unchanged state. |
| H2 native creation | `tests/native-hero-workflow.ts`: resolve exact literal placeholders, submit one aliased batch, verify one revision, seven named children and two instances; all source/packaged/native entry points invoke the helper. Five native rectangles, one managed artwork child and one native text child remain editable. |
| H3 independent customization | Existing typed instance update replaces title and managed image; full left-item equality except slot values, unchanged shared component and right item. Actual native before/after PNGs have more than 100 changed title pixels and exactly unchanged entire right-card pixels; independent final samples prove blue replacement on the left and retained red artwork on the right. |
| H4 atomic failures | Append an invalid deletion after valid aliased operations, and separately submit missing component media. Canonical non-retryable ITEM_NOT_FOUND; exact project/history/assets/artifact file hashes and state remain unchanged. Prior separately committed imports are preserved. |
| H5 history/reopen | Repeated undo/redo of customization and creation, exact content equality ignoring only monotonic revision/update timestamps, then actual project reopen; geometry, managed references, slot overrides and ordinary opacity channel persist. |
| H6 stale revision | Stale typed instance update returns retryable REVISION_CONFLICT and leaves all project resource bytes unchanged. Prompt guidance requires refreshed state and replanning. |
| H7 actual native output | Existing required default source and freshly compiled packaged platform entry points use actual FFmpeg/FFprobe and owned fonts: 1920×1080 frame at 500ms and six-frame [0,200)ms range; independent device/screen/label/accent/artwork/glyph samples, initially hidden and finally visible left card, frame/range tolerance parity, immutable job revision and authoritative state equality. |
| H8 negative controls | Blank full-resolution buffers fail visible and hidden-state sample verification; wrong frame sizes also fail. Expected samples are independently authored in the test helper, not derived from the recipe or renderer. |

## Stable-tree local checks

Successful commands exit 0; full uncommitted logs are `/tmp/issue182-*.log`.

- Focused recipe and existing motion-prompt tests: `unit-focused-final.log`, 4 passed. Typecheck: `typecheck-evidence.log`; full lint: `lint.log`, 234 files.
- Full bridge unit: `unit.log`, 896 passed, 15 existing opt-in skips in 10 files.
- Canonical `bun run contracts:check --no-file-parallelism`: `contracts.log`, all required Rust sections and 608 TypeScript tests passed; unchanged structural catalogs and pins verified.
- Full source integration: `integration.log`, 38 passed. Freshly assembled package smoke: `package.log`, 35 passed. Fake-media smoke establishes actual MCP lifecycle, not native image output.
- Required configured `bun run scripts/run-platform-renderer.ts`: `native.log`, 2 passed in 89.75s; actual source and fresh package, unchanged 600s test deadlines and report assertions.
- `cargo fmt --check --all`: `rustfmt.log`; strict workspace Clippy: `clippy.log` (existing dependency future-incompatibility notice only); fresh `cargo test --workspace`: `workspace.log`, 1519 passed across 77 result blocks, 12 existing ignored native tests. Required published-head native/foundation jobs remain mandatory.
- Existing `bun run scripts/run-python-tests.ts`: `python-hermetic.log`, 12 unittest and 12 pytest passed. An initial direct system-Python invocation lacked soundfile (`python.log`); the existing isolated locked test environment resolves it without repository changes.
- Pinned strict validation: `spec-all.log`, 65 items passed including this active change. Pre-archive `moon run root:openspec-validate`: `prearchive-gate.log`, 65 strict items and bootstrap/policy tests passed, then expected exit 1 naming only `add-native-hero-card` as unarchived. This is not a passed protected gate.

Linux subprocess checks use the existing external subreaper wrapper to compensate for container PID 1 not reaping adopted children. No assertion, deadline, repository runner or public behavior is weakened. The tail of native functional verification overlapped other checks; suite duration is not a performance measurement. No repeated passing check is required absent new inputs.

## Native attribution and limits

Owned captures and runtime manifest are retained in `/tmp/issue182-native-evidence`. Both source and package produced PNG SHA256 `c7fd548b838fd3af8606907d23d90ddb64d4ac88d8e4cf29c123271e5186966c` and MP4 `f79b1ecd016c26438835ffb3b733c1c202d886ede4fa0aff4fdba570dc216100`. Their recipe SHA256 over JSON.stringify of retrieved data is `9ca3bdb66acb0fc5e7f1ea1573858845023574e8fa214b0122fcf42cf6fa9731`. Prompt source SHA256 is `490ea6015cb71a972f5657118813d3ea02850100ca66aa078ba4d2f9b6b5c146`; recipe file SHA256 is `31f745244ee9cd793d034f86a809c0f3f401eed3ffa154ba3922ebeafa7822ff`. Workflow source hash is `ce110ffd956af99bf0b42ebee03c9c3c05b4fd4d56c77680b8976c93fa00b346`; platform test hash `3e49604ff7854d0e4ba14d80f6803ece4697617abaaeeea63b51be2448d437cd`. Reviewed regular font SHA256 is `ae7b7855e115a5966d8b1b3f80f254ccc117ec86f9965e202ee2940453837280`. Tool versions and separate runtime file hashes are in the retained mode records/manifest (FFmpeg 7.1.5 Debian, Bun 1.4.0, Rust 1.97.0). Packaged bridge SHA256 is `a0d2b52136df001b0f4170639a531d915aeb7ab7deff63695a180f69dbab4e0c`.

The captures identify main50ec842c plus the uncommitted recipe/test inputs above; they are local implementation evidence, not exact published-head CI. Visually inspected the retained frame against the declared geometry and independent customization. Interior/glyph comparisons are narrower than a full-image golden. This two-instance/fourteen-leaf/one-channel/six-frame workload is distinct from #181's static one-instance benchmark; no portable performance, dense pack, concurrent-heavy-export, arbitrary-artwork auto-fit or separately installed desktop claim follows. Original synthetic BMP artwork only; no desktop files, configuration, builds or renders were touched.

Sync only the four new native-hero-card requirements, preserve unrelated specifications, then archive this change and pass the protected gate again. Publication and all exact-head CI are pending at report creation and must be recorded in the PR/external delivery receipt. No merge, auto-merge or deployment.

## Post-archive readiness

Only this completed change was synchronized and archived. `moon run root:openspec-validate` passed (postarchive-gate.log, exit 0), including policy/bootstrap assertions and 65 strict specifications. Separate pinned `validate --all --strict --no-interactive` passed 65 specifications (postarchive-strict.log, exit 0). All seven scoped tasks are complete; publication/exact-head CI remain the delivery step.
