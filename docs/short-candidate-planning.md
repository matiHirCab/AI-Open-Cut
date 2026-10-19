# Source-grounded short planning

The standalone local Bun CLI plans candidate intervals from **explicitly approved source transcripts** and an existing managed audio/video asset. It does not transcribe, download models, upload media, assemble shorts, export or change the project. It uses the existing native core to resolve asset integrity and captured revision before and after planning. A successful plan identifies its exact project, asset, SHA-256 content, revision and duration; the core-resolved managed file path is excluded from output.

From `apps/agent-bridge`, with the existing native headless executable built and configured:

```sh
OPENCUT_CONFIG_ROOT=/path/to/repository \
OPENCUT_PROJECTS_DIR=/path/to/project-store \
OPENCUT_EXPORTS_DIR=/path/to/existing-export-directory \
OPENCUT_ALLOWED_MEDIA_DIRS=/path/to/existing-approved-media-root \
OPENCUT_HEADLESS_PATH=/path/to/repository/target/debug/opencut-headless \
  bun run src/short-candidates-cli.ts < request.json > plan.json
```

Use the existing authorized media root for `OPENCUT_ALLOWED_MEDIA_DIRS`; the native runtime requires it even when the asset is already managed.

On Windows, set the same environment variables to installed Windows paths, then use byte-preserving file redirection from the bridge directory:

```powershell
cmd /d /c "bun run src\short-candidates-cli.ts < request.json > plan.json"
```

This requires the repository/Bun and an available native headless runtime; it is not a newly bundled MCP command or desktop panel. Source portability and CI compilation do not prove an installed runtime is ready. No execution-policy or global security change is required.

## Approved input

Use existing project/asset inspection to obtain the current project ID, asset ID, revision and media duration. Supply an approved transcript with original source timestamps and provider/model/limit provenance. Existing transcription preview segments can be mapped explicitly into this input. **Moved timeline caption timestamps are not original source timestamps.** Caption source text/words alone do not always recover original segment boundaries; do not substitute a current timeline caption interval.

The following independent example describes a 2000ms approved source; replace its identity/revision and transcript with your actual approved source:

```json
{
  "version": 1,
  "projectId": "project-original",
  "assetId": "asset-original",
  "expectedRevision": 4,
  "approvedTranscript": true,
  "targetDurationMs": 1000,
  "maxCandidates": 2,
  "chunks": [
    {
      "id": "c1", "offsetMs": 0, "durationMs": 1000,
      "providerId": "supplied-provider", "modelId": "supplied-model", "maxDurationMs": 1000,
      "segments": [{"id": "s1", "startMs": 0, "endMs": 1000, "text": "I do not recommend that."}]
    },
    {
      "id": "c2", "offsetMs": 1000, "durationMs": 1000,
      "providerId": "supplied-provider", "modelId": "supplied-model", "maxDurationMs": 1000,
      "segments": [{"id": "s1", "startMs": 0, "endMs": 1000, "text": "The result is uncertain."}]
    }
  ]
}
```

Chunks must cover the complete source contiguously and fit their declared provider duration limits. Segment and optional word times are chunk-relative; plans convert them to absolute source time. Provider/model/limit provenance is **supplied**, not independently verified inference. A transcript is not proof of what was actually spoken or whether a statement is true. Review it against authorized source media.

For long sources, provide all explicitly segmented transcript chunks, then optionally select a planning scope with `"selection":{"startMs":250,"endMs":1750}`. The output distinguishes full-source coverage from that selected scope. Over-limit requests fail explicitly; there is no silent truncation. No provider is invoked, so this feature does not bypass an inference provider's existing duration limits.

## Review and correct

The deterministic planner accumulates whole source segments until it reaches the target duration. It keeps a final shorter group and warns when duration is outside 80–120% of target. A selected scope can explicitly cut a segment: the candidate interval remains inside that scope, while evidence preserves the complete original quote/range/words and marks `fullyIncluded:false`.

Regenerate with optional input edits, for example:

```json
"edits": [{
  "candidateId": "candidate-1",
  "startMs": 250,
  "endMs": 1250,
  "hook": "Guaranteed success",
  "cta": "Review the original context",
  "rationale": "Caller-supplied editorial draft"
}]
```

Edits name existing deterministic candidate IDs, stay inside the selected scope, and must intersect source evidence. They do not regroup candidates or rewrite quotations. Hook/CTA/rationale are caller-supplied editorial fields, empty by default. They are separate from original evidence and are never labeled fact-checked, source-verified or guaranteed engaging. The example hook deliberately demonstrates a claim requiring review.

Warnings are explicit and can coexist:

| Warning | Meaning |
| --- | --- |
| `context_review` | Always present: timing cannot establish semantic completeness. |
| `editorial_claim_review` | Supplied editorial assertions require review. |
| `cut_boundary` | The selected interval cuts an original evidence segment. |
| `context_excluded` | Full evidence is cut or neighboring source context lies outside the candidate. |
| `duration_mismatch` | Duration falls outside 80–120% of target. |
| `chunk_boundary` | The candidate intersects multiple source chunks. |
| `candidate_overlap` | Its interval overlaps another candidate; both are marked. |

Immediate before/after transcript segments remain available as context. Neither these neighbors nor the warnings guarantee that all necessary context or unsupported claims were detected. Keep negation, uncertainty and qualifications intact when making the final human cut decision.

## Bounds, failures and verification

Input is strict version-1 JSON, at most 4MiB, with up to 128 chunks, 4096 segments, 32768 optional words and 24 hours of source media. Segment text is at most 2000 UTF-16 code units and word text 256; IDs are nonblank and at most 128. Target duration is 1000–180000ms and candidate count 1–32. Editorial strings are at most 1000 each. Successful output is checked against 8MiB before stdout is written. These are data/output bounds, not a claim of an 8MiB native/process heap quota. Excess candidate count requires an explicitly smaller selection; it never returns a silently shortened plan.

Success goes to stdout. Errors go to stderr with nonzero exit, a bounded fixed planning message and, where applicable, the existing core code/retryability. Missing/stale/integrity errors do not retry, reapprove, infer or mutate. Re-read and explicitly approve current inputs after an edit. The final source identity is captured evidence, not a lease preventing later edits or deletion.

Canonical valid/invalid/original/edit/cut examples are in [the CLI contract](../contracts/short-candidate-planning-cli-v1.json). Its consumer runs from the existing protected contract suite, with an exact ownership-index addition and independently pinned predecessor raw bytes. The existing MCP/headless/provider catalogs, schema versions, package commands and CI policies are unchanged.

Future integration with unpublished #208 may textually conflict in the central ownership index, `contracts.test.ts` or the historical ownership projection boundary. Preserve both approved categories/consumers and the complete predecessor proof chain when resolving that future union. This feature does not import/reconstruct that WIP or change narration/audition behavior.


The explicit real-runtime acceptance driver uses an original synthetic tone and a supplied synthetic transcript, creates an owned temporary project, and fails rather than skipping when native prerequisites are absent:

```sh
OPENCUT_HEADLESS_PATH=/path/to/repository/target/debug/opencut-headless \
OPENCUT_FFMPEG_PATH=/path/to/ffmpeg OPENCUT_FFPROBE_PATH=/path/to/ffprobe \
  bun run tests/short-candidates-workflow.ts
```

Run this from `apps/agent-bridge`. The driver configures its own temporary project/media/export roots, verifies edits and failures plus authoritative hashes, and cleans only that owned fixture. Linux runtime evidence does not establish an installed Windows or macOS runtime is ready.
