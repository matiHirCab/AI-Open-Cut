## Why

Issue #71 requires convenient 540p, 720p, and project audiovisual review while retaining custom dimensions. Existing MCP `preview_render_range` requires custom dimensions and silently defaults to audio off; changing that default would break its public meaning under ADR 0002.

## What Changes

- Add `preview_review_range`, a uniquely named MCP tool with `resolution` accepting `540p`, `720p`, `project`, or the existing custom `{width,height}` object. Omission selects project dimensions; audio is enabled unless explicitly false; omission of fps selects project fps.
- Add a uniquely named typed headless `render_review_range` request with the same resolution selection and defaults. Preserve existing `render_preview_range` exactly, including its historically broader numeric bounds.
- Resolve preset sizing and validate conflicts/limits exclusively in editor-core before renderer I/O. Report `preview_review_presets_v1` when rendering is ready.
- Keep existing MCP `preview_render_range` behavior unchanged. Document the new default workflow and explicit silent opt-out.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `rendering-export`: preset dimensions, canonical resolution validation, audiovisual review defaults, immutable snapshot semantics and parity.
- `agent-bridge`: additive review tool and support discovery without changing legacy tool meaning.

## Impact

Editor-core render options; headless request union/status; bridge schemas/types/render registration/instructions; canonical headless/MCP fixtures/catalogs/ownership and parity tests; native and MCP workflow tests; review documentation.

Compatibility: additive uniquely named headless operation and MCP tool, protocol major remains 1. The legacy MCP tool retains audio-off omission. There is no persisted shape or schema version change.

## Non-goals

Frame/draft preview selection (these retain project dimensions), export presets, desktop redesign, compositor/evaluator redesign, project/history migrations, job result/resource redesign, speech clocks, or pending PRs #136/#137/#138. Range previews are read-only ephemeral artifacts, not timeline edits, so batch aliases, undo operations, and migrations are inapplicable; tests prove state/history remain unchanged across review and reopen.
