## 1. Canonical ownership and decision record

- [x] 1.1 Add the versioned contract ownership manifest, headless protocol fixtures, and MCP surface catalog covering requests, responses/events, errors, capabilities, resources, provider protocols, persisted projects, and version negotiation.
- [x] 1.2 Add an ADR selecting fixture-governed manual synchronization, documenting schema-generation and partial-generation alternatives, unsupported type-system features, migration cost, compatibility rules, and the canonical ownership matrix.
- [x] 1.3 Add `.github/CODEOWNERS` coverage for canonical contracts and governed Rust, TypeScript/Zod, MCP, and parity-test consumers with `@matiHirCab` as the required reviewer.
- [x] 1.4 Update `AGENTS.md` and `docs/spec-driven-development.md` with the mandatory public-contract workflow, compatibility classification, fixtures, review evidence, and parity command.

## 2. Contract parity gate

- [x] 2.1 Add Rust parity tests that consume the canonical fixtures/catalogs and prove request strictness, response/event examples, capability identifiers, version negotiation, and stable error code/retryability alignment.
- [x] 2.2 Add TypeScript parity tests that consume the same fixtures/catalogs and prove Zod request/response validation, error alignment, capability identifiers, provider versions, and MCP tool/resource exposure.
- [x] 2.3 Add a repeatable `bun run contracts:check` command and invoke it explicitly from the correctness CI job so a mismatched category or consumer fails the build.

## 3. Representative additive change

- [x] 3.1 Add optional protocol-version selection to the Rust status request and `protocolVersion` to its status response, default omission to version 1, and return non-retryable `INVALID_ARGUMENT` for unsupported explicit versions.
- [x] 3.2 Update the bridge TypeScript/Zod status types and schemas to send and validate protocol version 1 while remaining compatible with callers that omit it.
- [x] 3.3 Expose the optional protocol version through `editor_get_status`, return the negotiated version and capabilities through MCP, and cover supported and unsupported calls in registrar/integration tests.

## 4. Verification and closure

- [x] 4.1 Run `bun run contracts:check` from `apps/agent-bridge` and resolve every cross-language or MCP parity failure.
- [x] 4.2 Run `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` from the repository root.
- [x] 4.3 Run `bun run typecheck`, `bun run lint`, `bun run test`, `bun run test:integration`, and `bun run test:smoke` from `apps/agent-bridge`.
- [x] 4.4 Run `bun run apps/agent-bridge/scripts/run-python-tests.ts` and `moon run openspec-validate` from the repository root.
- [x] 4.5 Run `$openspec-verify-change`, resolve all implementation/spec/design/task mismatches, then archive with `$openspec-archive-change` so living requirements include the accepted contract workflow.
