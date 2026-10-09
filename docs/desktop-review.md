# Desktop motion graphics review

Open a project with the existing `--project-store` and `--project-id` arguments. Hierarchy and root Timeline share the authoritative scoped selection. Inspector tabs make the existing controls accessible without mixing cue/bus summaries into layer fields:

- **Layer**: selected kind, scope, complete instance path, track, parent, stack order, stored slots, vector/text, animation and applicable masks/mattes/blend/effects. Root controls use existing core edits; component-local occurrences remain read-only.
- **Cues**: root/component scope, bounded 32-row cue pages, local milliseconds and selected marker ID/name. No layer selection is required.
- **Audio**: the selected media's saved alignment/provenance, one segment, authored volume/mute/fades/routing and semantic event snapshot, followed by bounded bus DSP/ducking/master summaries. Authored values are distinct from effective results. Absence remains absent.

Changing tabs discards unfinished Layer field and z-index input. Existing selection, refresh, Undo/Redo and reopen semantics remain core-backed. A conflict requires explicit Refresh and replanning; the app publishes no speculative edit. The tabs use the unchanged bounds and deterministic fixtures described in [hierarchy](desktop-hierarchy.md), [compositing controls](desktop-compositing-controls.md) and [synthetic narration inspection](fixtures/narration-driven.md).

## Review controls

Frame time and range start/end are unsigned integer **project milliseconds**. Click a value, use Ctrl+A to clear, type and press Enter or use Render frame/range. Escape leaves the field. Fractional, negative, nonnumeric and overflowing representations fail before rendering; core decides timeline/range/resolution validity. The range resolution choices are 540p, 720p and Project. Project fps is used. Audio defaults on and can be explicitly disabled. Frame review uses project dimensions.

Rendering runs outside the UI callback through the same EditorCore revision/path validation and Renderer used by native preview/export. Only one local editor/review task runs at a time. Controls and mutations wait while working; ordinary window close is refused until that work finishes. Close again afterwards. Forced process termination is outside this guarantee; the desktop introduces no separate native cancellation protocol.

A successful frame displays the generated image. Range review shows actual captured dimensions/fps/audio, immutable revision, MIME/size/relative path and core warnings. **Reveal MP4 for external playback** explicitly opens the generated local file in the native file manager. Play it with a local video player. This panel has no embedded audio/video player or export control. There is no arbitrary path or network input.

Retained reviews keep their captured identity when edits, refresh or history advance the displayed revision. A STALE label requires rendering the new revision; an old review is never silently relabeled. Renderer failures retain the core error code/message/retryability and publish no replacement success artifact. Review metadata is window-local: a newly opened shell starts without retained artifacts. The UI retains the latest frame and range metadata; preview files follow the existing core output behavior under the project previews directory. This does not change managed-media garbage collection or create a desktop disk-retention quota.

Configuration is frozen when the shell starts: `OPENCUT_FFMPEG_PATH`, `OPENCUT_FFPROBE_PATH` and optional `OPENCUT_DEFAULT_FONT_PATH`, otherwise the existing renderer defaults. Existing project font bindings and resource safety remain canonical core behavior.

## Verification

Run `cargo build -p opencut-desktop`, strict workspace Clippy/full workspace tests and formatting. Configured native desktop review evidence is explicit:

```sh
OPENCUT_FFMPEG_PATH=/usr/bin/ffmpeg OPENCUT_FFPROBE_PATH=/usr/bin/ffprobe cargo test -p opencut-desktop review_tests::desktop_review_native_frame_audio_range_history_and_reopen -- --ignored --exact
```

The named test requires real tools and must report one executed passing test. It reuses #70's unchanged synthetic fixture, verifies frame bytes against the existing Renderer, checks actual audio/video streams and captured revisions, authoritative inventory, core failures, Undo/Redo and reopen. It supplements the existing native audiovisual oracles and genuine GUI tab/preview/edit/history/conflict interactions. GUI screenshots are observation evidence, not a replacement for native pixel/audio/timing parity. #77's ten-capability scene and the proposed creative rescue benchmark remain separate work.
