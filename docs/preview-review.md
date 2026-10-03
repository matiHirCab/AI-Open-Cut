# Audiovisual range review

Use `preview_review_range` with projectId, expectedRevision, startMs and endMs. Resolution defaults to `project`; select `540p`, `720p`, or custom `{width,height}`. FPS defaults to the project's rate. Audio is enabled by default; use `includeAudio:false` for silent review. Detect support with `preview_review_presets_v1` in ready rendering status.

The height presets use height540 or720 and preserve project aspect ratio. Width rounds to the nearest even pixel, ties upward, minimum2: 1920x1080 becomes960x540 or1280x720; 1080x1920 becomes304x540 or406x720. Project and custom dimensions remain exact, including odd sizes; existing MP4 codec restrictions still apply. Width must be1..7680, height1..4320, fps1..120; oversized aspect-derived widths fail rather than clamp or crop.

Headless clients use additive `render_review_range` with the same fields and resolution union. Legacy `render_preview_range` retains required numeric width/height/fps/includeAudio and historical bounds. Legacy MCP `preview_render_range` retains required custom resolution/fps and silent default. New clients should choose the review tool; old requests retain their meaning.

Review uses an immutable validated revision and the existing export evaluator, audio mix, managed artifacts, cancellable range jobs and stable errors. No project settings, revision, undo/redo history, or persisted schema change. Frame/draft previews continue using project dimensions. Missing media, path escapes, invalid options and revision conflicts fail before artifact publication.
