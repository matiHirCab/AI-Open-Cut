## Why

Issue #24 (MG-M1-08) requires nested component instances to render with local timing and inherited transforms. Definitions and typed slots are stored today, but root placement is rejected and evaluation skips instances, so clients cannot use them in a rendered project.

## What Changes

- Activate bounded component expansion in the shared EvaluatedScene for frame, range, draft preview and export, including media audio, local animation, slot values and internal transitions.
- Add typed `add_component_instance` and `component_instance_update` edits for root overlay tracks, standalone and in aliased batches; retain existing root move/remove/duplicate/order/parent behavior where applicable.
- Specify half-open local time mapping, fractional derived times, composition coordinates, inherited opacity/visibility, hierarchical order and expansion budgets.
- Advance persisted schema 12 to 13 to distinguish root-instance support; atomically migrate current and retained history without changing existing content or output. **BREAKING reader compatibility:** older binaries cannot open schema 13; supported older projects migrate forward and there is no downgrade.
- Advertise additive `component_instance_evaluation` capability under protocol 1, update canonical fixtures and consumers, and document errors and fallback behavior.

## Capabilities

### New Capabilities
- `component-evaluation`: Root instance editing and deterministic bounded nested evaluation, including local time and effective slot values.

### Modified Capabilities
- `component-definitions`: Activate root placement and rendering while preserving local definition validation and unused-definition isolation.
- `motion-graphics-architecture`: Define hierarchical instance evaluation within the private shared scene.
- `rendering-export`: Consume mapped visual/audio facts consistently across all render intents.
- `project-persistence`: Migrate current state and all retained history to schema 13 atomically.
- `motion-graphics-contracts`: Govern the additive runtime instance contract, fixtures and capability.
- `agent-bridge`: Expose typed instance operations and document their semantics.

## Impact

Core model, validation, timeline edits, store/history migration, scene evaluation, render planning and renderer preparation; headless typed requests/status; bridge schemas, MCP registrars and documentation; canonical contracts and parity/render/integration/smoke tests. Ownership remains in editor-core with no new dependency direction or provider protocol. Existing valid requests, aliases, error codes and retryability remain compatible.

## Non-goals

No animated instance transforms, transitions whose endpoint is an instance, looping/reverse playback, time-remap curves, implicit fit-to-parent scaling, isolated component compositing, component canvas masks, new effect/audio-bus systems, external resources, UI authoring, or public serialization of EvaluatedScene. Ordinary content already supported inside definitions remains supported when instantiated.

## Approval

Approved by the user in this task on 2026-09-05 with the message "Approve", covering the proposal, design, delta requirements and implementation tasks for issue #24. Final contract-consumer review was tracked separately and is now complete as recorded below.

Final contract/consumer review and archival were explicitly approved by the user in this task with "Approve" after the implementation verification report was presented. This completes the designated review gate recorded in tasks 6.4 and the runtime contract delta.
