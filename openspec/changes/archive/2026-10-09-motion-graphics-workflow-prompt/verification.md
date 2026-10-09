# Verification: motion-graphics-workflow-prompt (#75)

Base: verified #73 commit `18c43b942473955f1063396c7ed97a93cb90b1b4`, including merged audit repair #167 through main `eec0c1012f2cd8077c9212ca61e6404dad00909b`. Original work branch/head preserved. This issue adds one MCP prompt, not a native operation, persisted model, renderer feature or autonomous execution agent.

## Conformance review

| Dimension | Evidence | Result |
| --- | --- | --- |
| Completeness | Proposal/design/delta/tasks; registration, canonical identifier, docs, fixture successor and source/package helper | All implementation tasks covered |
| Correctness | Four normative scenarios mapped below; full real MCP discovery/get and rejection tests | No unresolved mismatch |
| Coherence | Guidance-only adapter, unchanged core/headless/schema44/protocol1 and all historical pins | No ownership or compatibility divergence |

Scenario mapping:
1. Discovery/complete workflow: `src/server/context.ts` validated registration; `tests/motion-workflow-prompt.ts` checks required string descriptors, deterministic user text, quoted literal data, published tool identifiers and every workflow stage through in-memory, source STDIO and isolated compiled MCP. Guidance uses project coordinates, integer milliseconds, explicit ordering, typed slots and aliases. Marker timing explicitly uses uniquely scoped markerName, while IDs/aliases address CRUD.
2. Invalid arguments/read-only: missing/empty arguments and unknown prompt reject in each MCP client; non-string wire arguments reject in the in-memory unit. Registrar accepts no editor/provider dependencies, so retrieval cannot invoke them. Existing intro prompt remains retrievable. No project migration applies.
3. Failure recovery: guidance asserts closed shapes and missing-reference resolution, failed-batch unchanged-content/revision inspection and fresh-state replanning after REVISION_CONFLICT. Existing full source/package component, marker, preset and preview workflows exercise actual core invalid-input, reference, rollback, conflict, history and reopen paths. Prompt wording itself does not claim to execute or enforce these agent actions.
4. Compatibility: manual canonical additive identifier; `motion-workflow-projection.ts` removes exactly this successor identifier, rejects malformed/duplicate additions, and is chained ahead of normalization/historical projections. The independently captured #73 raw catalog SHA256 is `5eff94792ee8f029910c655365d7690631b443dc506e30d7f74194c5d0e504bd`; original predecessor digests are untouched. Registration/catalog parity and all prior pin assertions pass. No native union, Zod edit schema or headless capability changes are needed for guidance-only prompt discovery.

## Observed checks

All commands used task-local Bun1.4.0, Rust1.97.0, Moon2.3.3 and the existing frozen bridge installation. Full logs are uncommitted under `/tmp`.

| Command | Observed result | Log |
| --- | --- | --- |
| `cargo fmt --check --all` | exit0 | `/tmp/mg75-fmt.log` |
| `bun run typecheck` | exit0 | `/tmp/mg75-typecheck-final.log` |
| `bun run lint` | exit0;209 files | `/tmp/mg75-lint.log` |
| `bun run test:unit --no-file-parallelism` | exit0;767 pass,11 existing optional skips | `/tmp/mg75-unit.log` |
| `bun run contracts:check` | exit0; every native subcommand plus604 TypeScript tests/38 files | `/tmp/mg75-contracts.log` |
| `bun run test:integration` | exit0;34 source MCP workflows | `/tmp/mg75-integration.log` |
| `bun run test:smoke` | exit0;31 isolated compiled-package workflows | `/tmp/mg75-smoke.log` |
| `openspec validate motion-graphics-workflow-prompt --strict` | exit0 | `/tmp/mg75-spec.log` |

The bridge commands ran through the same task-local process subreaper as #73, preserving all assertions and per-test deadlines; serial unit-file scheduling does not weaken tests. An initial new predecessor-byte test used the older normalization predecessor's hash and failed once (`/tmp/mg75-focused.log`). The new test now uses the independent git-show hash of #73; the observed-exit0 full767 suite repeats it and all contract tests. A focused rerun logged41 passes before executor reconnect but its process handle was lost; completion relies on the later observed full-suite exit0, not an interrupted check claim. The initial ultracite invocation from the wrong directory found no configuration; the proper bridge lint/fix and final lint pass are recorded above. Executor restarts preserved work and completed logs.

## Unaffected evidence reuse

`git diff 18c43b94` changes no Rust source, manifests, compiler configuration, provider Python, native render fixtures, media tools or renderer. Under AGENTS.md's unchanged-input rule, reuse #73's observed strict workspace Clippy exit0, full workspace1490-pass/0-failure/9-intentional-ignore result, Kokoro12/12 and Whisper12/12 hermetic checks, and actual instrumented source/compiled native cache/lifecycle3/3 media workflows. Their full commands/logs and limitations are retained in `../2026-10-09-harden-preview-job-lifecycle/verification.md` after archival. The affected canonical contract gate and every bridge suite were rerun above. No GUI, local Windows/macOS, real model accuracy, subjective creative quality or new native renderer verification is claimed for this guidance change.

## Specification lifecycle and delivery

The exploratory prearchive protected gate passed policy checks and strict59 items, then correctly rejected only the active `motion-graphics-workflow-prompt`. This is expected rejection, not final gate success (`/tmp/mg75-prearchive.log`). Fresh final prearchive conformance, synchronization/archival and final protected/strict results follow below. Draft publication targets main with the full verified #73 lineage. Formal CODEOWNER review and remote exact-head CI remain review/merge-readiness obligations; no merge or deployment is authorized.

Final prearchive protected run (`/tmp/mg75-prearchive-final.log`) again passed strict59 items and rejected only this active change, after all implementation suites passed. Conformance review completed against the approved proposal, design, requirement/four scenarios and tasks with no critical, warning or suggestion issues. The one added requirement was synchronized without changing any earlier requirement. All5 tasks are complete; final merge-readiness gates follow archival.

Postarchive `moon run root:openspec-validate` passed exit0:505 policy tests,58 strict living specs and unchanged CI parity policy (`/tmp/mg75-final-gate.log`). Independent strict all-spec validation passed58/58 (`/tmp/mg75-strict-final.log`). No active changes remain. #73 exact-head Windows startup is successful; OpenCut CI was still in progress at delivery preparation, so full remote completion is not claimed. #75 remote exact-head checks await push/publication. CODEOWNER @matiHirCab is the connected PR author; independent/maintainer contract review remains outstanding on the draft.
