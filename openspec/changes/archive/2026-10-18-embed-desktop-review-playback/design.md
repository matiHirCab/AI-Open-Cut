## Context

Review artifacts already carry project/revision, range, dimensions, fps, audio, relative path and owned absolute path. Rendering and validation remain core-owned. Presentation must consume the completed artifact without editing the project or repeating rendering.

## Goals / Non-Goals

Embed review playback with truthful identity and lifecycle safety. Do not add inference, speech audition, public wire operations, project fields, cancellation protocols, arbitrary media input or export controls. No claim of Windows/macOS playback support in this slice; both retain explicit unsupported-backend fallback.

## Decisions

1. Keep all implementation in existing review/preview ownership and a nested `review/playback` module, avoiding narration, shell/session, manifests/lockfile and shared contract files. Use the standard Linux dynamic-loader ABI to load the fixed installed `libmpv.so.2` name. A private typed function table owns the library lifetime. No downloaded dependency is bundled with the application.
2. Use libmpv's software render API and normal audio output on a dedicated owned worker thread. Disable mpv config/scripts, terminal/input bindings, network/protocol input and automatic external-track discovery. Commands accept only a captured internally generated artifact; never user-entered paths/URLs or arbitrary mpv commands. Missing/deleted/nonregular files fail visibly. Playback never regenerates missing artifacts.
3. Bound display resolution to fit 1280x720 while preserving aspect; retain original metadata separately. Keep only the latest pending frame, one displayed frame and bounded command admission. Use an aligned software-render buffer and a fixed BMP encoding consumed by GPUI, with old decoded assets and atlas entries explicitly evicted when replaced. No frame/file cache is created on disk.
4. Model loading, paused, playing, ended and error explicitly. Clamp seeks to artifact-relative bounds; show relative and captured project time. Loop means repeating this artifact's captured interval, not reinterpreting the current project. Loading/replacement uses a generation identity so stale work cannot overwrite a newer player. Read-only history changes retain captured revision/stale labels.
5. Each player owns one worker. Drop/stop/replacement closes admission, signals shutdown and joins after freeing the render context before terminating mpv and unloading its library. GUI polling stops on generation mismatch/window release. Backend errors preserve a fixed truthful presentation error and fallback; no raw native paths or error strings are displayed. Existing core render errors remain untouched.

## Risks / Trade-offs

The native ABI is unsafe: every function signature and render parameter must match the installed primary headers, pointers must remain valid/aligned for each synchronous call and destruction order must be tested/reviewed. Codec/backend support and audio hardware remain runtime capabilities. Native tests use an actual software renderer and isolated realtime audio-output monitor, while controller failures are injected. An unpaced ALSA file/null sink is insufficient timing evidence. Physical speaker quality is not established by capture. GUI probes require X11, a window manager and Vulkan; a merely mapped black window is not evidence.

## Verification

Scenario tests cover state transitions, clamps, generation isolation, missing/unsupported/codec errors, metadata/history identity, bounded admission/images and cleanup. Explicit native tests require real libmpv/FFmpeg, verify synthetic pixels, clock/pause/seek/loop with a documented 200ms tolerance and actual nonzero audio samples, and exercise repeated open/close. Actual built GUI interactions must demonstrate displayed changing frames, controls, stale identity, replacement and close. Follow all repository-required checks, separate review, prearchive gate, conformance/sync/archive/final protected gate, then publish a separate draft and inspect exact-head CI. #225 CI is monitored separately and its user-selected ready state is preserved.

## Independent review resolution — approved before follow-up edits

The reviewer identified three implementation mismatches. Resolve all before publication: after render-context creation use only render-thread-safe client APIs (`mpv_command_async`, property observations and event polling), never synchronous client/property calls or waits; release the render context before ordinary client destruction. Parse typed property events into bounded local state. Propagate the active mutable GPUI window to atlas eviction, retaining the no-current-window path only for release outside an active window callback. Require output readiness before reporting audiovisual Playing and reject an ended audio-bearing clip whose output was never ready, including sub-second clips without loop. Cover these with real short/long native controls and GUI cleanup evidence. These corrections enforce the existing threading, bounded lifecycle and truthful audio requirements; approved under the same delegated authority before implementation.

Resource/platform follow-up: keep polling for generation/replacement cleanup, but redraw only when visible state or a decoded image changes, avoiding an idle/error redraw loop on software Vulkan. Express controllable phases as shared values so unsupported-platform builds do not depend on Linux-only enum construction or warning suppression. The isolated native memory probe reproduces codec/allocator warm-up outside GPUI; report measured retained RSS rather than claiming native caches return every byte to the OS. Approved under existing bounded-lifecycle scope before the follow-up implementation.
