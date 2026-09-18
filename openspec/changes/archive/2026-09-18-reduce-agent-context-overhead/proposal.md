## Why

OpenCut agent tasks inherit high planning effort and unrelated plugin descriptions, and can spend additional context on repeated reading and verbose verification output. Reduce this overhead without weakening the existing specification, approval, architecture, or validation safeguards.

## What Changes

- Add project-local Codex settings retaining GPT-6 Astra with medium reasoning and changing planning from inherited xhigh to medium.
- Retain nine project-local marketplace plugin disable overrides and document local re-enablement. Distinguish these settings from observed desktop availability; remote/workspace-managed plugins may remain available, an explicitly accepted limitation.
- Consolidate repeated workflow guidance and define focused reading, archive search exclusions, instruction reuse, concise logs, and evidence-based reuse of successful checks.
- Add configuration and safeguard regression coverage, plus documented fresh-task verification and optional usage comparison.
- Remove synthetic JavaScript merge tests as evidence of Codex inheritance and add regression checks against unsupported complete-removal claims.
- Separate implementation verification from final merge readiness: verify content, synchronize/archive this change, then require the unchanged protected gate to pass. Rejection caused only by this active change is expected before archival, never reported as a passed gate.

## Capabilities

### New Capabilities

- `agent-context-efficiency`: Project-scoped agent defaults and efficient context handling with unchanged contributor safeguards.

### Modified Capabilities

None. Existing `repository-validation` requirements, including archive-only merge readiness, remain unchanged.

## Impact

Project Codex configuration, contributor documentation, and a focused Bun regression test. No application code, public APIs, persisted schemas, dependencies, or migrations change. Override scope is OpenCut only; global configuration and other projects remain untouched. All generated OpenSpec skills and protected CI scripts remain unchanged. Full remote-plugin suppression is not promised.

## Non-goals

No lighter OpenSpec process, removed checks, different model, global plugin removal, or guaranteed savings percentage. Do not modify or archive the independent content-addressed-font-shaping change or its implementation.

## Approval

Original revision: approved by the user with "Approve" and partially implemented. Review revisions: the user selected "Document limitation" for remote plugins, requested the corrective plan, and explicitly approved these revised artifacts with a subsequent "Approve". Corrective implementation is authorized within these revised requirements.
