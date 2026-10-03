## Context

The MCP shared responder reads PNG and speech WAV on every completed status poll. Core already supplies validated render metadata. Jobs and speech handles are process-local; a server's dependencies and HTTP transport currently define its access boundary (not a new per-user project ACL).

## Goals / Non-Goals

Provide metadata-first delivery, explicit binary retrieval, stable failures, and opaque links. Preserve core rendering, revision, history, provider and transport behavior; no network resources or persisted schema changes.

## Decisions

- Keep structured job fields; add an optional artifactResource descriptor. Generate it deterministically from an existing UUID job ID and retained metadata. Use fixed display names by job kind/format; never use raw path segments or speech filenames. Defaults perform no artifact file I/O; the status JSON resource uses the same projection. Alternative stat-on-poll adds filesystem dependency and defeats cheap stable polling.
- New canonical `contracts/artifact-delivery-v2.json` governs the breaking content policy (major 2), migration and fixtures; MCP structural catalog remains version 1 with additive fields/template. Advertise `artifact_resources_v2` from bridge status only; headless does not implement MCP delivery. Keep provider metadata sections independent of #59.
- Resource template opencut://jobs/{jobId}/artifact has no listing function. UUID is the existing random job identifier; resource reads recheck registry retention and canonical URL/variables and never accept caller file paths. Possessing a URI does not bypass the existing HTTP authentication: resources/read runs through the same MCP HTTP middleware/session as tools, and both parsed variables and the SDK-normalized complete canonical URI string must match. The SDK normalizes raw URLs before dispatch, including plain and percent-encoded dot segments; equivalent spellings selecting the same retained lowercase UUID are accepted. Do not replace SDK dispatch or inspect private internals. Across servers separate registries do not share ownership; within one server existing authorized clients share access just as job_get_status already does. No invented ACL based on the mutable active project.
- Restrict preview output metadata to previews/ within project UUID roots. Restrict export metadata beneath exports root. Validate relative segments and reject drive, UNC, control characters, percent-encoded traversal and dot segments; verify real paths and disallow symlinks in configured root, ancestors beneath it and final file. Use an opened file handle with O_NOFOLLOW, fstat regular-file check and safe errors. This guards resource I/O, not domain validation. Speech service remains authority for token ownership/expiry; its returned generated-media path is checked against one of config.generatedMediaDirectories (including ttsWorkDirectory) with the same reader. Alternatives: unrestricted join or file:// links expand file access; a new download tool duplicates standard resources/read.
- Resource reads return explicit binary blob for PNG/WAV/MP4. Inline true supports only the legacy PNG/WAV formats; resource links handle videos. No new eager byte buffering on default polling.
- Metadata survives file deletion and speech expiry until job expiry; explicit read resolves current availability. Link expiration is min(job expiry, speech expiry). Existing tokens stay available for commit, not resource addressing. Missing render files map to VALIDATION_FAILED without raw fs exceptions. Missing speech retention uses GENERATED_ARTIFACT_NOT_FOUND. Resource RPC wraps safe errorBody JSON, preserving code without raw details.

## Risks / Trade-offs

- Default content is breaking → version-2 content contract, discovery, migration docs and explicit legacy opt-in.
- Explicit reads buffer bytes using MCP blob semantics → only on request; existing output sizes/retention remain unchanged.
- Filesystem changes between validation and read → reject symlinks and use O_NOFOLLOW on opened files; authorized local filesystem writers remain trusted (no new OS sandbox claim).
- Export overwrite retains existing path-backed artifact semantics → links refer to retained job paths, not immutable content snapshots.
- Parallel lanes edit schemas/catalog → update only job/status fields, artifact resource and digest; do not regenerate entire surface.

## Migration Plan

Clients that render inline previews add includeBinary=true; other clients consume artifactResource/resource_link and call resources/read on demand. New content-policy contract major 2 does not change headless protocol major 1 or persisted schema. Reverting the scoped bridge change restores the old policy; no data rollback is needed.

## Approval and Review

User delegated issue-scoped spec approval/implementation/verified commit/push/draft PR in the source thread; this is not a human CODEOWNER review. Independent spec and implementation reviewers must examine requirements, edge cases, tests and active PR compatibility before delivery. Human CODEOWNER @matiHirCab remains the designated contract reviewer.

Independent spec review completed by spec_review (Sol medium). Clarifications incorporated: fixed names, plural generated-media roots, and identical transport authentication plus canonical URL matching. Delegated issue-scoped spec approval recorded on 2026-10-03; no human CODEOWNER approval is represented.

Independent review amendment: spec_review approved canonical URL equivalence on 2026-10-03 after implementation_review identified SDK pre-dispatch normalization. Accepted aliases retain identical UUID ownership, authentication and filesystem confinement.
