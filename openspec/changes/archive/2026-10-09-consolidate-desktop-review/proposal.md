## Why
Issue #76 requires inspectable agent-created hierarchy, markers, compositing, audio buses and preview controls. Existing scoped inspectors are merged, but narration/bus detail pushes layer controls down one long panel and the preview remains a placeholder.

## What Changes
Consolidate the inspector into explicit Layer/Cues/Audio tabs, retaining every existing scoped editor and bounded read-only inspection. Replace the placeholder with explicit frame and audio-enabled range review controls using existing EditorCore/Renderer facades. Show captured revision, options, warnings, a rendered frame and a reveal action for the generated MP4. Keep external playback explicit and label it truthfully.

## Capabilities
### New Capabilities
- `desktop-review-surfaces`: consolidated desktop inspection and native review controls.
### Modified Capabilities
None; all existing hierarchy/narration/compositing requirements remain applicable through the tabs.

## Impact
Desktop presentation/session helpers, tests and docs. No MCP/headless/public/persisted contract changes, migrations or renderer-semantic change. No new fixture replacement: preserve #70's complete synthetic fixture and leave #77's ten-capability scene separate.

## Approval
Approved under the user's standing authorization for issue-scoped specification approval and implementation of #76. A video player, renderer rewrite, new editing operation, export UI, cancellation system and fixture deduplication are outside this scope.
