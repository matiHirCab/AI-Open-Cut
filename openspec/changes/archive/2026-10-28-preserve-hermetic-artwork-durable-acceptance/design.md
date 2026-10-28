## Context

The helper overrides configured fake tools with ambient names. Shared fake FFprobe reports1x1/100ms, unlike the real3x3 PNG/400ms WAV.

## Goals / Non-Goals

Repair test isolation and retain genuine proof. No production, lifecycle, dependency, workflow, profile or timeout changes.

## Decisions

1. Hermetic mode uses supplied fake FFmpeg. A private probe delegates version checks to supplied FFprobe and derives actual header facts from bounded ordinary owned PNG/WAV files; reject unknown/malformed/changed/out-of-root inputs. Keep shared fake backend verbatim.
2. Both modes retain all existing core/source/provenance/narration/history/admission/start/reconnect/old-job/stale-revision assertions. Hermetic output must match exact existing fake bytes/size; it is not pixel evidence.
3. Explicit genuine mode requires explicit genuine paths, retains the independent64x64 RGBA decoding assertion, and fails for missing tools without fallback/skip. Exercise source and assembled package in both modes with separate evidence.
4. Regress configured-tool and owned-input failures plus absence of ambient tools. Existing native/release gates remain unchanged.

## Risks / Migration

Bounds/header/hash checks prevent fabricated probe success. Explicit mode prevents evidence confusion. No persisted/public migration. Preserve fc48d575/failed CI; publish only a verified corrective PR229 descendant.
