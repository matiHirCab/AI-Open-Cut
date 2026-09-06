## 1. Regression evidence

- [x] 1.1 Record user approval of the complete fix plan; read the living vector specification and preserve the original archive.
- [x] 1.2 Add shared structural-rejection cases and raw/Value Rust tests covering positional records, nested sites, object-form enums, strict wrappers, reordered keys, and duplicate fields; observe the failures before decoder changes.

## 2. Correct decoding

- [x] 2.1 Implement private streaming map-only decoding for all six production vector record types and the Paint/PathCommand tagged envelopes, plus string-only decoding for caps, joins, and fill rules; preserve public fields, Serialize, tags, and pure validators.
- [x] 2.2 Harden Rust catalog/fixture wrappers and Kind decoding; update both fixture consumers to distinguish structural failures and cover canonical success through raw JSON and Value paths.
- [x] 2.3 Document explicit JSON shape rules, confirm TypeScript production schemas remain unchanged, and map every added scenario to evidence.

## 3. Verification and finalization

- [x] 3.1 Run cargo test -p opencut-editor-core --test vector_primitives --test architecture; cargo fmt --check --all; cargo clippy --workspace --all-targets -- -D warnings; cargo test --workspace.
- [x] 3.2 From apps/agent-bridge run bun run contracts:check, bun run typecheck, bun run lint, bun run test, bun run test:integration, and bun run test:smoke; run bun run scripts/run-python-tests.ts for both hermetic workers.
- [x] 3.3 Run bunx @fission-ai/openspec@1.5.0 validate fix-vector-json-representations --strict --no-interactive; use openspec-verify-change and record conformance, test results, and resolved review findings.
- [x] 3.4 Obtain completed @matiHirCab contract-owner review, synchronize and archive with openspec-archive-change, then pass moon run root:openspec-validate. Record failures/skips explicitly; do not mark complete until the gate passes.
