---
name: opencut-bridge-verification
description: Verify OpenCut agent-bridge and MCP changes across contracts, integration, packaged smoke, and provider boundaries.
---

# OpenCut bridge verification

Use when verifying agent-bridge workflows, MCP adapters, provider orchestration, or headless transport changes. Start with the approved OpenSpec change and applicable requirements in `openspec/specs/agent-bridge/`, plus feature-specific specs. If requested implementation lacks approved coverage for behavior, failure cases, compatibility, and verification, update the change artifacts and obtain explicit approval before editing code.

Use `AGENTS.md` and `docs/spec-driven-development.md` for the complete mandatory gate. Select focused evidence by affected surface: `bun run contracts:check` for governed public contracts; `bun run typecheck`, `bun run lint`, and `bun run test:unit` for bridge code; `bun run test:integration` for bridge/headless interaction; `bun run test:smoke` for packaging; and hermetic Python worker tests for worker changes. Run commands from `apps/agent-bridge` unless their documented invocation specifies another directory. Include the workspace Rust and OpenSpec checks required by the repository even when focused checks pass.

Keep transport handlers thin and domain decisions in `editor-core`. Record each command, exit status, relevant failure, and full uncommitted log path. Mark unavailable or skipped checks explicitly. Use `$openspec-verify-change` before synchronization and archival; never infer conformance from compilation alone.
