## 1. Approved regressions

- [x] 1.1 Transcribe user-approved plan into proposal, design and delta specs; preserve schema 19 and draft/layout v2.
- [x] 1.2 Add failing core regressions for scoped selector capture, operation matching and mandatory separators, including atomic ambiguity and native parity.

## 2. Implementation

- [x] 2.1 Core assets/store: capture unresolved scoped identities and align retained steps by structure then font intent; preserve matches and reject ambiguity atomically.
- [x] 2.2 Core shaping: mandatory breaks, correct paragraph bidi, original clusters, CRLF and empty-line/work limits.
- [x] 2.3 Document approved v2 conformance correction and draft-update matching/failure behavior.

## 3. Verification and archival

- [x] 3.1 Run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` with required native fixtures.
- [x] 3.2 In apps/agent-bridge run `bun run contracts:check`, `bun run typecheck`, `bun run lint`, `bun run test:unit --maxWorkers=2`, `bun run test:integration`, `bun run test:smoke`; run hermetic Python via `bun run ../agent-bridge/scripts/run-python-tests.ts` in apps/kokoro-tts.
- [x] 3.3 Run strict OpenSpec validation and `moon run root:openspec-validate`; use openspec-verify-change and maintain scenario evidence in verification.md.
- [x] 3.4 Sync accepted specs and archive using openspec-archive-change; rerun Moon and report unrelated active-change blockers without altering them.

Post-archive Moon: all 26 spec items pass; exit 1 solely for the separate active
`reduce-agent-context-overhead` change. Repository-wide merge readiness remains
blocked. See verification.md for full evidence and uncommitted log locations.
