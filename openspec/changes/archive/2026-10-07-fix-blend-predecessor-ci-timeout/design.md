## Context

Run37558455469 head dcc6b28bf4f60bc3d18bb5673090f6e9e810b004 passed ten leaf foundations except Windows correctness (nine leaf successes); its annotation names blend-modes.test.ts:100 and a5000ms timeout. MacOS succeeded. Logs from results-receiver are denied; the concrete annotation is sufficient to investigate the repeated projection work.

## Goals / Non-Goals

Reject malformed unchanged blend nodes before whole-catalog predecessor transforms, preserving complete positive projection and every negative control. No production/API/schema/renderer/CI changes or timeout increases; no merge/deployment.

## Decisions

Extract the existing29-field read-only exact validator and call it on current input before projectParameterizedMcpPredecessor. Call the same validator in the existing schema35 projector too, retaining capabilities, literal markers, historical transformations and full digest checks. Current37→36→35 transforms do not alter blendMode nodes; valid input still traverses the complete pipeline. No cache, generated authority, weakened digest or assumptions about unrelated fields.

Automated early-rejection witness uses a guarded unrelated subtree on malformed input: rejection must occur without reading it. Existing87malformed/29missing/source-frozen checks and full projected MCP pin remain unchanged; add valid-input unrelated-drift discrimination and complete non-mutation where needed. Early diagnostics are test-only and can identify the malformed blend node before unrelated predecessor failures.

## Risks / Trade-offs

Additional read-only29-node scan on valid input is small; malformed cases eliminate repeated full clones. Validate behavior with all historical projection suites under unchanged deadlines. Reuse previously passing required checks only where complete relevant inputs/toolchain/environment remain unchanged, with hashes and explicit evidence; rerun affected typecheck/lint/unit/contracts/policy/strict/protected checks and exact-head all11 CI. Retain failed run/annotation receipts honestly.

## Lifecycle

Independent specification approval precedes code. Independent source/conformance review follows passing checks. Prearchive protected rejection must name only this change; sync/archive only this accepted delta and preserve all prior archives/unrelated requirements/production sources/catalogs. Postarchive protected and strict checks must pass before commit/push to existing PR150 branch. Exact-head CI must succeed before #57. Publication receipts remain external to avoid self-head circularity.
