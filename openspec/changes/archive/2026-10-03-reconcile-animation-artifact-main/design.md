## Context

The reviewed issue47 head is `b372185178f44e4cbf78c37c0de470a80b8ae85c`; incoming main is `4c2897e0` with parent `360f8187` and merge base `480bd8d7`. The actual incoming 27-file change adds issue74 artifact resources on top of issue59 speech timestamps. No Rust/headless/Python, lockfile, workflow, policy script or native fixture changed upstream. Root's dry-run merge reports one textual conflict, `MCP_BASELINE_DIGEST` in the contract test; automatic combinations still require semantic review.

## Goals / Non-Goals

Preserve the exact union of already accepted issue47, issue59 and issue74 behavior, retain safe artifact ownership/authentication/confinement/retention, and restore PR136 compatibility with current main. No new rendering, speech, artifact, persisted schema, public error, timeout or security policy is introduced. No remote main merge or deployment is authorized.

## Decisions

1. Use a normal local merge of exact main after preserving the reviewed head and bundles. Do not resolve the direct head-to-main diff by dropping unmerged issue47 content. Review the resulting union against both parents. Squashing/rebuilding imported behavior risks losing specifications and evidence; whole-file ours/theirs replacement risks losing one contract lane.
2. Retain issue74's metadata-first polling without file reads; optional `artifactResource`/resource_link, explicit PNG/WAV `includeBinary=true`, explicit MP4 resources/read, fixed names, safe stable failures, process-local registry ownership, authentication, canonical URI handling, path/symlink confinement and expiry remain governed by the imported artifact-resources specification. Speech timestamp capability semantics remain unchanged. Bridge delivery remains separate from headless/provider/persisted project contracts.
3. Resolve the MCP digest only after confirming the canonical catalog's union of schema31 animation/Pack/clock envelopes, speech timestamp support, artifact descriptors/resources and `artifact_resources_v2`. Compute the pinned digest using the existing reviewed catalog expander and parity procedure; do not regenerate the catalog from live registration or suppress mismatch evidence. Preserve ownership consumers, new artifact-delivery-v2 fixture and imported `contracts:check` coverage of artifact-responses.test.ts.
4. Automatic merging must retain the immutable supporting-surface shallow-copy test setup, all ordered independently partitioned workflow calls, and all issue47 migration/edit scenarios while adding issue74 metadata/link/explicit-read/inline/discard scenarios. Do not weaken individual timeout values, protected duration budgets or assertions.
5. Rerun exact canonical contracts, bridge typecheck/lint/unit tests, full MCP integration, packaged smoke and strict/protected OpenSpec/policy gates. Review artifact metadata and retrieval safety tests plus combined rendering/speech job response consumers. Reuse previous Rust/native/Python checks only with recorded input/toolchain equality; incoming-main absence of changes alone is insufficient if merge resolution changes those inputs. Any new affected inputs invalidate relevant evidence. Final published-head CI must pass all required jobs.

## Risks / Trade-offs

An automatic merge can be semantically incomplete despite one textual digest conflict: independently review live/canonical schemas, capabilities, ownership, resource registration and existing workflow calls. Artifact delivery intentionally changes default content only under imported version2; do not restore eager binary content to make legacy tests pass. Prior remote CI applies only to its recorded head and does not establish the new combined head's success. Preserve complete bundles for rollback; no persisted migration or downgrade is part of this reconciliation.

## Migration Plan

No new data migration. Retain issue74's client migration: PNG/WAV clients requiring inline blocks pass includeBinary=true; MP4 clients use resources/read. Complete checks and conformance before synchronization/archival, then pass post-archive protected gates before publishing the authorized draft update. Stop and report any publication denial without a workaround.

## Open Questions

No semantic decision remains open. Actual conflict resolution, combined-tree checks and exact-head CI are pending implementation and delivery evidence.
