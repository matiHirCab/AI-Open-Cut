# Current-main contract reconciliation

The user explicitly requested fixing conflicting OpenCut PRs after PR138 merged. PR139 now normally merges authoritative main `4c2897e0499281c75bf23b6abac99e2f4b15d2dc`, containing PR137 and PR138 actual head `360f8187193ddf54b38224c16e9ce2e72fe045db`. No rebase, force push, PR merge, deployment or new issue work is authorized by this reconciliation.

Four textual conflicts: the bridge contract-check command, expanded MCP digest/capability assertion, ownership categories and catalog capability array. Resolution preserves speech timestamp fields, artifact_resources_v2/resource template/job metadata/binary opt-in, and issue71 request defaults/selection. New review output references the shared updated DraftPreviewFrameOutput job definition. Both artifactDelivery and previewReview owners/tests remain registered. Legacy render-worker fixture positions from the independently reviewed CI correction are unchanged.

Current merged expanded catalog digest: `77e1e52ca14bfdc25183c3cd8306c7b6ba07fd1a0f8d99a3c2bf2eb2eb395cbf`, deliberately calculated from the manually merged checked-in catalog after semantic resolution. Registered contracts/preview/artifact focused tests57 pass. Shared real MCP/packaged workflow now also checks completed review MP4 artifactResource metadata, default text/resource_link responses and explicit artifact-resource bytes. Final full checks and independent reconciliation review are recorded in verification.md before normal push; exact new-head CI is an external delivery gate.

Independent reviewer reconstructed both expanded catalogs using the same helper. The sole difference between historical8fbe and current77e digests is the last two capability identifiers: historical artifact then review, current review then artifact. All expanded schemas/annotations/tools/resources and object-key ordering are identical; capability sets are unchanged.

All core/headless production, render-worker fixture and native golden inputs are identical to the previously reviewed and tested e098b1be tree. Persisted schema30 and animation domain changes remain untouched.

## Earlier disposable integration evidence

Initial integration used main `5e6472a110bc15dc699d47252533bec337d5d16e`. Publication re-fetch found PR137 already merged into current main `480bd8d7f326eb4cf204dd09f8b9d28f3f61e802`; issue71 was rebased onto that authoritative base. At that earlier checkpoint PR138 remained unmerged and excluded; the current authorized reconciliation above includes only its now-merged main ancestry.

Tested disposable worktree `/tmp/issue71-union` with exact reviewed heads:

- PR137 / issue59: `584df7ed516274e32bc1acaaafa9691ea86aa9be`.
- PR138 / issue74: `dae8cd8404ef56bdb69e6ac5fd0c6c95eaf6f725`.
- Issue71's current semantic diff, new fixtures and tests applied on top.

The first merge conflicted only in the expanded MCP digest. Applying issue71 conflicted in the contract-check command, digest/capability expectation, ownership index and compact MCP catalog. Resolutions retain both preceding lanes' response metadata, artifact resources, binary opt-in and speech timestamp capabilities, add only issue71's review operation/tool/capability/fixtures, and use the shared post-PR138 job output definition for the new tool. No response definition was copied from the old live server.

The combined expanded catalog SHA256 is `8fbe3ec092d5aedf6b6d830eb8a1b8b6c8387f21ff076292811c8685f50a0ed1`, computed from the semantically combined checked-in catalogs. The original pre-PR137 issue71 catalog SHA256 was `19cf9dc05e1e79b288f7778548c2ccc131050d1857bc96b594295c99ac989967`.

`bun run contracts:check` passed in the disposable union: every governed Rust headless/fixture suite and 394 TypeScript tests in16 files. `/tmp/issue71-union-contracts.log` contains the full output. The real MCP review workflow also passed in that worktree (one selected test,15 unselected), including preset/custom/default completed range jobs and exact persisted project/history/draft byte preservation: `/tmp/issue71-union-review-integration-correct.log`.

Merge order can follow independently reviewed, green PRs. Whichever of137/138/71 lands later must preserve the preceding semantic contracts and deliberately recalculate the expanded digest. The disposable union does not replace exact-head CI or human CODEOWNER review. No merge is authorized by this task.

Current main plus issue71 deliberately resolves the expanded catalog digest to `7df40e56127433bfbc9dafbf5839e1d1a49c39bd6e734ee99661827ca3414e28`. Only the digest conflicted during rebase; the speech schema/catalog/ownership additions and review additions both survive. Rust/headless production inputs are byte-identical to the previously tested issue71 tree. The now-merged PR138 job definitions are preserved in the current normal merge above; the earlier exact-head semantic union demonstrates the combined result. A recoverable semantic union patch is retained in the external delivery bundle before disposing the worktree.
