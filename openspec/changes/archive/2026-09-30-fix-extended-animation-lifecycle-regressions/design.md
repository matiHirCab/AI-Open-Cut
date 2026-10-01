## Context

Issue #43 and its first review correction are archived, while their implementation remains in the uncommitted working tree. Fresh independent probes reproduced three additional failures. Full source and execution logs are retained outside the repository at `C:/Users/matia/AppData/Local/Temp/opencut43-rereview/`; `verification.md` distinguishes those observations from planned correction evidence.

Applicable requirements include project-persistence's atomic schema-27 migration and stale-draft policy, rendering-export's shared extended rendering and safe export, and agent-bridge's stable safe diagnostics. ADR 0003 assigns every fix to editor-core. The original proposal, design and amendment preserve established clocks, transitions, failure handling and compatibility; these deltas make the newly uncovered cases explicit.

## Goals / Non-Goals

**Goals:** restore project access after draft-base eviction, preserve canonical transition gains for extended visuals, and return safe bounded encoder failures. Demonstrate conformance with independent expected results, failure injection and all affected required suites.

**Non-Goals:** new transition features, draft rebasing, additional history retention, schema or API changes, weaker CI, regenerated goldens, commits, pushes, or PRs. Do not change unrelated working-tree work.

## Decisions

### D1: Validate draft candidates only against their applicable base

Keep structural, font/catalog and resource checks independent of extended-property detection. Materialize and semantically validate retained draft operations when a matching current/undo/redo base exists. An unavailable base remains a stale draft; do not substitute current state. Preserve existing revision-conflict checks before applying or previewing it, and keep discard accessible. Current state and all retained history still receive full migration validation before atomic publication.

Replaying against current state was considered and rejected: deletion after draft creation can make valid old operations invalid on an unrelated revision. Expanding history or silently rebasing changes persisted behavior and is outside scope. Tests must cover eviction through ordinary public edits, reopen, discard, missing-base schema adoption, no-write revision conflicts, and the existing invalid-target matching-base rejection.

### D2: Reuse the canonical occurrence clock for transitions

The sampled-input branch forwards local transition spans to an FFmpeg stream shifted to root time; the legacy affine branch resolves those spans through the occurrence clock. Investigation also reproduced a local visibility interval being compared against root sample timestamps in both preparation and sample preflight. Canonical evaluated-layer helpers resolve half-open root occurrence visibility and local transition gain (using the existing exact/fractional SampleTime). Preflight uses that same visibility before charging cumulative effect work. Preparation multiplies transition gain once into the final premultiplied raster opacity; sampled backend inputs apply no further transition filter. This preserves local effect ordering, opacity, stacking, budgets and audio, and supports trimmed transitions whose root start precedes zero without invalid negative FFmpeg fade parameters. No transport owns a clock interpretation.

Applying transitions with raw local times was rejected because offset instances become invisible prematurely. Baking a second transition into both raster pixels and compositing was rejected because it squares gain. Verify analytic gains plus unaffected legacy controls for root, offset, scaled, nested and repeated occurrences, supported fades/crossfades, fractional times, and half-open boundaries. Decode actual frame/range/draft/export output for the reproduced offset case; use owned semantic clock tests for exact boundaries that video frame quantization cannot represent.

### D3: Reuse the shared process stderr policy

The sampled encoder must pass its already bounded stderr tail through the existing sanitizer/excerpt helper before constructing CoreError. Preserve visual_prepare stage, native exit status, stable code/retryability and bridge defense. No new diagnostic contract is required. Core/headless currently expose the raw encoder tail; bridge already applies secondary sanitization, so the review does not claim an MCP path leak.

Increasing the public limit or relying only on bridge redaction was rejected because direct core/headless users are compatibility consumers too. Fault injection must consume input before failing so BrokenPipe does not mask the intended encoder-status branch. Cover long ASCII and Unicode diagnostics, quoted Windows/POSIX paths, valid UTF-8 limits, process reaping, temporary cleanup and untouched existing destinations/state.

## Risks / Trade-offs

- An unavailable historical base cannot support semantic replay verification. Preserve established structural/resource validation and stale conflict semantics; exercise the matching-base path separately so invalid drafts do not gain validity.
- All render intents can share an incorrect plan. Compare decoded pixels with an analytic 0.5 gain and a separately verified legacy control, not only intent parity.
- Timing and media codecs introduce quantization. Separate exact clock assertions from decoded tolerances, use pinned FFmpeg 7.1.1 and the canonical font, and never regenerate references.
- Process failure tests can accidentally exercise stdin failure instead of encoder exit. Consume stdin fully and assert stage and exit status before checking privacy and cleanup.
- Platform evidence is currently Windows only. Exercise portable Windows/POSIX path strings locally; report unavailable real Linux/macOS execution and remote CI explicitly.

## Migration Plan

No schema bump or data rewrite is introduced beyond the existing schema-27 migration. Under the existing project lock, correct draft validation before publication; keep existing generation recovery and byte preservation tests. Keep previews/exports transactional and retain destinations on failure. Rollback of code must not downgrade schema 27; older binaries retain their rejection behavior.

After approved implementation and passing checks, verify conformance, synchronize the two living specs and archive this change. Add truthful superseding evidence notes to both prior archives. Run the unchanged protected gate after archival; before archival only rejection naming this active change is expected.

## Open Questions

The user explicitly approved these artifacts with "approve" on 2026-09-30. Any newly discovered behavior outside these deltas requires an approved amendment before implementation. No passing correction evidence is claimed until recorded checks execute.
