## Context

Issue 14 introduced a test-only golden harness under editor-core. Review found that its directory swap has an interruption window, its URI check recognizes only `://`, its PCM helper does not search offsets when streams have equal length, and its performance report records one cold run plus only the Rust test process. The current change is already archived, so these corrections require a new approved OpenSpec change. Production evaluator, renderer, persistence, protocol, and project formats remain unchanged.

## Goals / Non-Goals

**Goals:**

- Keep one valid reviewed golden generation selected throughout capture failures and process interruption.
- Enforce portable fixture-relative references, including scheme-based URI rejection.
- Calculate RMS after real bidirectional sample alignment within the existing one-frame bound.
- Produce comparable report-only observations from one warm-up and three measured renders, including the complete process tree.
- Preserve deterministic visual, audio, semantic-plan, graph, lifecycle, and failure evidence in Linux CI.

**Non-Goals:**

- No production render behavior, public API, stable error, project schema, history, migration, capability, MCP, or provider change.
- No new SSIM, RMS, timing, or performance budget.
- No general-purpose transaction or telemetry subsystem in editor-core production code.

## Decisions

### Select immutable generations through an atomic pointer

The fixture root will contain a strict versioned `CURRENT` pointer and immutable `generations/<digest>` directories. The digest is the lowercase SHA-256 of the finalized manifest bytes. Update mode writes a complete temporary sibling, validates every retained file and hash, installs the directory under its digest, and atomically replaces `CURRENT`. The pointer replacement is the commit point: interruption before it leaves the prior pointer active; interruption after it leaves the new complete generation active. Recognized stale stages and unreferenced generations are bounded cleanup candidates on the next invocation. Unknown paths are never removed.

Unix uses a same-directory write, file sync, rename-over-existing, and parent sync. Windows uses the native replace/move API with replace-existing and write-through flags; the implementation remains test-only. A cleanup failure after the commit is non-fatal and leaves the old generation for later cleanup.

Alternatives considered: the existing backup-and-rename directory swap leaves the canonical path absent between renames. A recovery journal restores availability only on a later invocation and therefore does not meet the stronger interruption invariant.

### Recognize URI schemes before filesystem interpretation

A reference is a URI when its leading bytes match RFC 3986 scheme syntax: an ASCII letter followed by zero or more ASCII letters, digits, `+`, `-`, or `.`, then `:`. This check precedes `Path` parsing and complements the existing normal-component, absolute-path, canonical-root, and symlink checks.

Alternatives considered: checking only `://` misses valid `file:`, `data:`, and opaque schemes. A general URI parser would add unnecessary runtime surface to test-only validation.

### Search bidirectional PCM offsets

For each integer offset from negative through positive one-frame sample count, the comparison uses the overlapping range and rejects candidates whose overlap is smaller than `min(left.len(), right.len()) - maximum_offset`. It computes RMS for every eligible candidate and returns the minimum. Empty streams, excessive length difference, non-finite samples, and insufficient overlap fail closed. Tests use non-periodic sequences so offset selection cannot pass due only to tone periodicity.

Alternatives considered: aligning only stream ends handles padding but not equal-length codec delay. FFT correlation reduces work but adds complexity for a bounded 48 kHz, one-second test fixture; a direct bounded scan is simpler and remains test-only.

### Separate conformance capture from sampled performance capture

Ordinary conformance performs one render capture. Update, explicit recapture, and report modes first run one discarded warm-up and then three measured captures. Deterministic references from all measured captures must agree under the existing exact/tolerance rules before any report or generation is accepted. Each phase and total duration is the median of the three measurements; peak memory is the maximum of their three observed peaks.

The performance format advances to schema 2 and declares `memoryScope: process_tree`, `timingAggregation: median`, and `memoryAggregation: maximum` along with warm-up and measured counts. A development-only `sysinfo` sampler refreshes every five milliseconds on a dedicated thread and sums resident memory for the test process plus recursively discovered descendants. It starts before each measured capture and stops after render, decode, and probe work completes. This captures FFmpeg and FFprobe without modifying production process APIs.

Alternatives considered: `/proc` alone would omit Windows observations. Instrumenting the production process executor would impose permanent overhead and expand a runtime internal interface for a test-only requirement.

## Risks / Trade-offs

- [Direct offset scanning adds test CPU time] -> Bound it to one video frame and one second of mono fixture audio.
- [A five-millisecond sampler can miss an extremely short child peak] -> Start before process creation, sample continuously, and test with a child whose allocation remains resident long enough to observe.
- [Generation cleanup could remove user data] -> Accept only strict lowercase digest directory names, never remove the active generation, and ignore all unknown entries.
- [A platform-specific atomic primitive can behave differently] -> Test the commit boundary and injected failures on Windows and Linux, including first pointer creation and replacement.
- [Three measured renders increase CI duration] -> Run sampling only for update, recapture, or report requests; ordinary conformance remains one capture.

## Migration Plan

Create generation revision 2 from the reviewed outputs in a Linux environment matching required CI, write the versioned pointer, and remove the former flat fixture files only after the new generation validates. CI continues invoking the same native test and uploads a schema-2 report. Rollback restores the flat revision-1 fixture and prior test-only harness; no runtime or persisted project data requires conversion.

## Open Questions

None.
