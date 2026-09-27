---
name: opencut-contract-change
description: Guide an OpenCut public or persisted contract change through canonical ownership, compatibility, and cross-language parity.
---

# OpenCut contract changes

Use for changes to a public request, response, event, stable error, capability, MCP surface, provider protocol, or persisted project shape. For ordinary implementation without a contract change, use the applicable OpenSpec workflow directly.

Before implementation, find the applicable living requirements in `openspec/specs/` and the approved change under `openspec/changes/`. If its proposal, delta spec, design, or tasks do not cover the affected behavior, compatibility, failure cases, and verification, update those artifacts and obtain explicit approval before editing implementation.

Use `contracts/contract-ownership-v1.json` and `docs/adr/0002-cross-language-contract-ownership.md` to identify the canonical artifact and every governed consumer. Classify the change as additive or breaking under `AGENTS.md`; breaking changes need a new major contract and migration path. Keep the canonical artifact, all listed consumers, fixtures, and parity tests synchronized. Preserve stable error codes and retryability, and request the designated CODEOWNER review.

Run `bun run contracts:check` from `apps/agent-bridge` when the public cross-language contract is affected, plus all checks required by the approved change and `AGENTS.md`. Record commands, results, and full-log paths. Use `$openspec-verify-change` before synchronization and archival; do not treat a passing local test as a substitute for canonical parity.
