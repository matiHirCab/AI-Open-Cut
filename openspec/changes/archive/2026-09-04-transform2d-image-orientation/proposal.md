## Why

The stream-only geometry probe misses JPEG EXIF orientation carried on decoded frames. A reproduced 40x20 image with EXIF orientation 6 displays 20x40 and 800 colored pixels through legacy rendering, but identity Transform2D clips it to 20x20 and 400 pixels.

## What Changes

- Inspect the first image frame through configured FFprobe during read-only preflight, with bounded packet input and metadata output.
- Finalize image affine dimensions from frame dimensions and display-matrix metadata, retaining FFmpeg autorotation exactly once.
- Cover EXIF orientations 1-8 and absent orientation in native regression tests, including shared preparation across render intents.
- Preserve validation precedence, safe bindings, probe reuse, and typed failures.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `motion-graphics-architecture`: Define frame-derived static-image geometry and failure handling.
- `rendering-export`: Explicitly permit bounded image decoding for preflight metadata inspection and require EXIF parity across render intents.

## Impact

Changes remain in editor-core's private process adapter and renderer orchestration, their tests, and documentation. Existing required Transform2D native CI coverage consumes the new cases. No new dependencies, public DTOs, error codes, persisted metadata, schema changes, or migrations are needed. Schema 8, configured tool paths, video probing, and legacy rendering remain compatible.

## Non-goals

Animated-image orientation changes, video geometry changes over time, FFmpeg upgrades, new animation features, public contracts, and migrations.

## Approval

The user explicitly approved the concrete proposal, design, delta specifications, and tasks with "Approve" on 2026-09-04, before implementation.
