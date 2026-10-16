## Why

Issue #207 needs reusable, explicitly applied narration defaults so two projects can share approved pronunciation without later profile edits changing committed speech.

## What Changes

- Add a bounded local profile library owned by editor-core under the configured projects root, with immutable identity and optimistic versions.
- Expose create/read/list/update/apply through the headless transport and provider-neutral MCP workflow.
- Resolve explicit request overrides over the selected version; persist profile identity/version and the exact effective speech request using existing generated-asset lifecycle.
- Preserve legacy requests, preview retention, stale-revision handling, history and existing provider readiness failures.

## Capabilities

### New Capabilities
- `narration-profiles`: Versioned local profile storage and explicit deterministic application.

### Modified Capabilities
- `speech-generation`: Optional resolved profile attribution in persisted speech intent.

## Impact

Additive headless/MCP contracts and persisted intent require canonical fixtures, governed consumers and current/history migration coverage. Core owns validation, storage and resolution; bridge owns provider readiness and speech orchestration; transports stay thin. No new dependencies are planned.

## Non-goals

No automatic application, cloud sync, inference provider, paid API, model download, global rewrite, sentence editing, channel pack, batch production, merge or deployment.

The compatibility slice also closes an existing cancellation race in provider preflight: cancelled speech must not start a worker request after asynchronous executable checks. No inference protocol or provider capability changes.
