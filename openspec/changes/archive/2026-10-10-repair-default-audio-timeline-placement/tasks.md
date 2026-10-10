## 1. Approval and independent regressions

- [x] 1.1 Record explicit reviewer approval of proposal/design/deltas/tasks before implementation (acceptance1–7).
- [x] 1.2 Add failing mandatory native decoded-PCM regressions for the delayed ordinary clip and multiple/gapped/overlapping/trimmed clips with local controls (acceptance1–2).
- [x] 1.3 Cover preview crop, neutral/active DSP, routing/ducking and retained/component clocks with independent timing expectations (acceptance3–4).

## 2. Scoped owner repair and compatibility

- [x] 2.1 Correct ordinary sample placement in owning render planning, preserving local/global clock separation and existing mapped clocks; inspect and cover any necessary delay-filter readiness without new public contracts (acceptance1–4).
- [x] 2.2 Verify existing cleanup fixes, typed failures/cancellation/retry, immutable state and timing through undo/redo/reopen; add missing changed-scenario tests (acceptance5–6).
- [x] 2.3 Run verify-only native/golden suites, preserve historical witnesses, present concrete impacted-reference diffs for separate reviewer approval before any governed update, and document the narrow correction/provenance (acceptance6).

## 3. Required conformance and publication

- [x] 3.1 Run Rust fmt/strict workspace Clippy/workspace tests, bridge typecheck/lint/unit/contracts/source MCP integration/packaged smoke, relevant hermetic Python, genuine Linux native media/package and render-parity/release checks; retain full logs and failures (acceptance7).
- [x] 3.2 Run strict specs and prearchive protected gate, verify conformance with openspec-verify-change and prepare synchronization/archival of only this approved verified change. Post-archive protected validation remains mandatory delivery acceptance below (acceptance7).
- [x] 3.3 Prepare the scoped reviewable implementation and acceptance/evidence report for publication, retaining limitations and failed attempts (acceptance7).

## 4. Post-archive delivery acceptance (tracked externally against the exact published head)

1. Synchronize and archive the verified approved deltas; require final protected Moon and strict all-spec validation to pass.
2. Commit/push the scoped verified changes and create a draft PR targeting current main.
3. Monitor every required exact-final-head CI job to terminal; repair only authorized scoped failures, preserving assertions, deadlines and references. Any golden replacement still requires separate concrete reviewer approval.
4. Report the exact final head, terminal CI, real-media evidence and limitations. No merge/deploy.

This ordering preserves all seven approved acceptance criteria. Publication and exact-head CI necessarily follow archival because the protected gate requires an archive-only change inventory; they remain mandatory acceptance, not prematurely completed implementation checkboxes.
