## Verification Report: compact-mcp-surface-catalog

### Summary

| Dimension | Status |
| --- | --- |
| Completeness | 9/9 change tasks complete; a later repository-wide gate rerun is blocked by another active change |
| Correctness | 1/1 delta requirement and 5/5 scenarios covered by catalog equivalence, parity, and regression tests |
| Coherence | Follows ADR 0002: one manually governed canonical JSON artifact, test-only expansion, unchanged runtime declarations and version |

### Requirement and scenario evidence

- The compact catalog has 176 named definitions, 72 tools, version 1, 800,084 bytes, and 28,649 lines. Expansion hashes to the original parsed catalog SHA-256 `2b5c1a5d6c0f74ca80f2c2e813612c7a3d8edcd7bd31ed186c968d3131b40752` twice, proving complete storage equivalence, including schemas, annotations, identifiers, resource templates, and version.
- `apps/agent-bridge/tests/fixtures/mcp-surface-catalog.ts` expands local references and rejects missing, cyclic, malformed, non-local, sibling-bearing, invalid, and unused definitions. The regression tests exercise these failures and a shared-definition change affecting all five draft output tools.
- `apps/agent-bridge/tests/contracts.test.ts` compares all live tool input/output structural schemas and annotations with the expanded catalog. It checks registered tool names, prompts, resource name/template pairs, resource URI constants, and the canonical capability identifiers. Existing direct input, output, annotation, and description-only normalization tests remain in place. Synthetic supporting-surface mutations fail comparison.
- The public MCP server, Zod declarations, persisted data, protocol negotiation, typed failures, and revision-conflict behavior were not edited. Source, integration, and packaged tests passed.

### Check evidence

- `bun run contracts:check`: exit 0, 352 TypeScript contract tests passed plus focused Rust and type checks. Log: `C:\Users\matia\AppData\Local\Temp\opencut-contracts-check-final-b9d526d4-9946-49f3-ba75-ca452a343612.log`.
- `bun run typecheck` and `bun run lint`: exit 0. Logs: `C:\Users\matia\AppData\Local\Temp\opencut-typecheck-resource-c8e3168e-9947-4bd6-9471-b4e602a2932c.log`, `C:\Users\matia\AppData\Local\Temp\opencut-lint-resource-final-596c21f7-6ad5-49ad-aee9-3a76726b1e7b.log`.
- `bun run test:unit`: exit 0, 418 passed, one optional raster-cache test skipped by its configured environment guard. Log: `C:\Users\matia\AppData\Local\Temp\opencut-unit-final-987d12bd-7679-4c82-8354-f71842c0f2c4.log`.
- `bun run test:integration --testTimeout 180000`: exit 0, 12 passed. The first run with the default 60-second timeout failed one combined shape-history case; that case passed in 44 seconds when rerun with the test-only timeout. Log: `C:\Users\matia\AppData\Local\Temp\opencut-integration-extended-d2698939-8347-41c7-a74b-55bb9d3f1b48.log`.
- `bun run test:smoke`: exit 0, six packaged tests passed. Log: `C:\Users\matia\AppData\Local\Temp\opencut-packaged-smoke-759e69a3-e3f0-4d73-b19a-2da21608c963.log`.
- `cargo fmt --check --all`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace`: exit 0. Logs: `C:\Users\matia\AppData\Local\Temp\opencut-rust-fmt-86b0cab7-da10-4aca-aff5-3245e40de0bd.log`, `C:\Users\matia\AppData\Local\Temp\opencut-clippy-69b669f0-bc62-4bba-9bd2-78edd84139f3.log`, `C:\Users\matia\AppData\Local\Temp\opencut-cargo-workspace-test-cd0af4ce-15a6-4ad6-9bd7-a212be973699.log`.
- Hermetic Python worker runner `bun run apps/agent-bridge/scripts/run-python-tests.ts`: exit 0, ten Kokoro unittest cases and five transcription pytest cases passed. Direct system-Python unittest failed because `soundfile` was absent; the repository's pinned runner supplied it. Log: `C:\Users\matia\AppData\Local\Temp\opencut-python-hermetic-4bb4d617-9290-4539-b831-c35b47b7aac1.log`.
- Pinned strict all-spec validation: exit 0, 30 passed. Log: `C:\Users\matia\AppData\Local\Temp\opencut-openspec-strict-final-prearchive-1e5c5d97-218b-4bfe-9f1d-be5a344feb32.log`.
- Pinned Moon `root:openspec-validate`: exit 1 solely because `compact-mcp-surface-catalog` is active. Its strict validation passed 30/30; this is the documented pre-archive policy result. Log: `C:\Users\matia\AppData\Local\Temp\opencut-moon-final-prearchive-4e4f8d98-07ef-4a7f-b596-0820c481983c.log`.
- Post-archive pinned strict validation: exit 0, 29 passed and 0 failed. Log: `C:\Users\matia\AppData\Local\Temp\opencut-openspec-postarchive-5821a78a-1dab-4e74-b979-eb90d436063f.log`.
- Post-archive pinned Moon `root:openspec-validate`: exit 0; strict specs and CI parity gate policy passed. Log: `C:\Users\matia\AppData\Local\Temp\opencut-moon-postarchive-713f8110-9dc8-4fbc-89e6-e1c0ee05a462.log`.
- Final pinned strict all-spec validation after the archived completion record: exit 0, 30 passed and 0 failed. Log: `C:\Users\matia\AppData\Local\Temp\opencut-openspec-postarchive-final-a2aa291a-b4d6-4c13-bb2c-a7c3b561c3ff.log`.
- Final protected Moon rerun: exit 1 because the separate active change `add-opencut-agent-skills` appeared in the workspace. Strict validation passed 30/30 in that run. Log: `C:\Users\matia\AppData\Local\Temp\opencut-moon-postarchive-final-a812c273-e049-48b5-9527-9a0bcf45f2ac.log`.

### Closure

No implementation/spec/design/test mismatch remains. The user explicitly approved the verified changes in this chat on 2026-09-26; the signed-in GitHub identity `matiHirCab` matches the designated CODEOWNER. The delta requirement is synchronized into the living `contract-governance` spec, and the change is archived. Strict post-archive validation and the protected gate passed immediately after archival. A subsequent repository-wide protected gate rerun is blocked by the unrelated active `add-opencut-agent-skills` change. That change and the unrelated `.codex/config.toml` edit remain untouched.
