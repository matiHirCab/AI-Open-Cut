## Why

#218 needs bounded, source-grounded short planning without automatic inference or project edits. Existing core-managed media and timestamped transcription provide the foundation, but timeline captions can move and cannot safely reconstruct original source intervals.

## What Changes

- Add a documented standalone local Bun CLI and injected bridge planning service. Accept explicitly approved, source-timed transcript chunks for an existing managed asset, resolve authoritative asset/revision/integrity through existing headless reads, and return a bounded editable plan.
- Group complete transcript segments deterministically around a target duration, retaining exact source ranges, original quotes and neighboring context. Expose boundary, overlap, context, duration and supplied-claim warnings; apply explicit human boundary/editorial corrections without rewriting evidence.
- Require complete explicit source chunk coverage and provider-limit provenance without silent truncation, inference, network upload or model download. Separate supplied editorial hook/CTA/rationale from transcript evidence and never promise factual correctness or engagement.
- Add independent synthetic fixtures, real core-backed CLI/read-only checks and versioned local CLI examples. No new MCP/headless operations, capabilities, provider protocol or persisted schema.

## Capabilities

### New Capabilities
- `source-grounded-short-planning`: bounded local candidate planning over explicitly approved source transcripts and core-resolved managed media.

### Modified Capabilities
None. Existing `transcription-captions`, managed asset and headless read behavior is consumed unchanged.

## Impact

New `apps/agent-bridge/src/short-candidates*` modules, dedicated tests/fixtures, a versioned CLI contract, documentation and this OpenSpec. Register the new category in the existing ownership index and invoke its independent fixture consumer from the already-required `contracts.test.ts`, preserving the exact contracts:check command, CI configuration/policy and every prior assertion. Extend only the historical fixture projection boundary necessary to invert the exact independently pinned new ownership addition before all unchanged predecessor proofs. Existing MCP/headless/provider schemas, registration, configuration/manifests, narration behavior/#208, desktop/#213, core models/history/GC and providers remain unchanged. Compatibility classification: additive version-1 standalone TypeScript CLI; existing cross-language wire/persisted surfaces have no changes. Shared ownership/test integration files can conflict textually with future #208 integration; record the semantic union rather than omit governance.

## Non-goals

No automatic source transcription or segmentation inference, existing caption-time reinterpretation, score/rank/quality guarantee, assembly/export/publishing, remote/paid APIs, model installation, arbitrary managed-file adoption, new renderer or project mutation. Editorial text is explicitly supplied by the caller; no generated factual assertion is trusted automatically. Windows embedded playback and #214 remain separate.

## Approval

Approved before executable edits on 2026-10-19 under delegated specification authority, following independent read-only review of the complete proposal/design/delta/tasks. The reviewer approved the closed contract, independent examples, exact cut/grouping/bounds semantics, governance and layer ownership with no remaining material design blockers. The user explicitly clarified that shared central files are allowed on an independent cloud branch when no #208 functionality is reimplemented or changed. No device/WIP access is authorized. Final conformance still requires all specified tests, protected gates and exact-head CI.
