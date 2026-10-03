# Disposable contract integration evidence

Initial integration used main `5e6472a110bc15dc699d47252533bec337d5d16e`. Publication re-fetch found PR137 already merged into current main `480bd8d7f326eb4cf204dd09f8b9d28f3f61e802`; issue71 was rebased onto that authoritative base. PR138 remains unmerged at the exact head below; its commits are absent from the publishing branch.

Tested disposable worktree `/tmp/issue71-union` with exact reviewed heads:

- PR137 / issue59: `584df7ed516274e32bc1acaaafa9691ea86aa9be`.
- PR138 / issue74: `dae8cd8404ef56bdb69e6ac5fd0c6c95eaf6f725`.
- Issue71's current semantic diff, new fixtures and tests applied on top.

The first merge conflicted only in the expanded MCP digest. Applying issue71 conflicted in the contract-check command, digest/capability expectation, ownership index and compact MCP catalog. Resolutions retain both preceding lanes' response metadata, artifact resources, binary opt-in and speech timestamp capabilities, add only issue71's review operation/tool/capability/fixtures, and use the shared post-PR138 job output definition for the new tool. No response definition was copied from the old live server.

The combined expanded catalog SHA256 is `8fbe3ec092d5aedf6b6d830eb8a1b8b6c8387f21ff076292811c8685f50a0ed1`, computed from the semantically combined checked-in catalogs. The original pre-PR137 issue71 catalog SHA256 was `19cf9dc05e1e79b288f7778548c2ccc131050d1857bc96b594295c99ac989967`.

`bun run contracts:check` passed in the disposable union: every governed Rust headless/fixture suite and 394 TypeScript tests in16 files. `/tmp/issue71-union-contracts.log` contains the full output. The real MCP review workflow also passed in that worktree (one selected test,15 unselected), including preset/custom/default completed range jobs and exact persisted project/history/draft byte preservation: `/tmp/issue71-union-review-integration-correct.log`.

Merge order can follow independently reviewed, green PRs. Whichever of137/138/71 lands later must preserve the preceding semantic contracts and deliberately recalculate the expanded digest. The disposable union does not replace exact-head CI or human CODEOWNER review. No merge is authorized by this task.

Current main plus issue71 deliberately resolves the expanded catalog digest to `7df40e56127433bfbc9dafbf5839e1d1a49c39bd6e734ee99661827ca3414e28`. Only the digest conflicted during rebase; the speech schema/catalog/ownership additions and review additions both survive. Rust/headless production inputs are byte-identical to the previously tested issue71 tree. PR138 must subsequently reconcile its shared job definitions and preserve this preview request surface; the earlier exact-head semantic union demonstrates the combined result. A recoverable semantic union patch is retained in the external delivery bundle before disposing the worktree.
