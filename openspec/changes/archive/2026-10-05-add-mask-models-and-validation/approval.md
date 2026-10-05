# Issue #50 specification approval

Approved by the parent agent on 2026-10-05 under the user's explicit instruction to approve each issue-scoped specification. Independent Sol Medium reviewers approved the external draft and the promoted artifacts; their findings concerning schema compatibility, digest projection, revision-scoped cache identity, raw JSON duplicate keys, and living pipeline wording were resolved before implementation.

Verified prerequisite: #49 commit `9de55a93`, archived specification `2026-10-05-implement-linear-light-compositing`, draft PR #142. The current branch starts from that verified commit. Pinned strict validation passes for this active change and all 41 items.

Approved scope: typed painted path-mask metadata, alpha/luma, four ordered operations, inversion, canonical Transform2D, bounded feather/expansion and hidden/unused aggregate complexity; existing standalone and alias-aware batch edits; schema 32 atomic current/history/draft migration and fail-closed guards; synchronized contracts and metadata-only `mask_models_v1`; unchanged rendered behavior until #51. No rendering activation, new source resources, matte DAG, new edit operations, or deployment is authorized by this change.

Reviewed artifact SHA-256 values before task-status updates:

- proposal.md: `280ddf169e3cde59fb96859cc012aeaae012b56d9d47cd6839789055956b9921`
- design.md: `9432d38bac86b23201ba218f35853256591ba4dcda1403cfef62d58f0db77229`
- tasks.md: `4ad0339322d8f94aafd110a4dc6d09cae5f2f13b7dc566bdabb2b625cde6cb0a`

All six delta specifications are identical to the independently reviewed external draft. Implementation approval is separate from the designated human contract/CODEOWNER review, which remains pending on the eventual draft PR. Every normative scenario, required check, independent implementation review, synchronization, archival, and final unchanged policy gate must complete before this implementation is presented as verified.

Restack approved on 2026-10-05: the prerequisite branch now includes verified commit `b0a9075f`, which corrects hermetic linear-capability protocol conformance and archives `2026-10-05-fix-linear-capability-contract-readiness`. Its full affected checks and final unchanged policy gates passed. The schema-31 production model, canonical MCP predecessor digest and all six mask-model deltas remain the reviewed baseline. All paused #50 changes were restored without conflicts from the retained recovery stash. Preserve the corrected readiness-aware test when synchronizing #50 contracts.

## Parent approval: current active catalog version markers

The parent independently reviewed and explicitly approves the narrow current-marker amendment before its implementation. Exactly the top-level `projectSchemaVersion` literal advances from 31 to 32 in the six named active animation catalogs and their exact governed expectations. Existing current-version equality remains strict. Feature identifiers, values, bounds, fixtures, historical source/activation-version cases and the approved MCP predecessor projection remain unchanged. Independently pinned predecessor semantic digests, a top-level-only reverse projection and an unrelated-drift negative test certify this limited change.

Reviewed artifact SHA256 values:
- proposal.md: fdafdf3fef2e4fa388b2e3b9f54bac87f5544e70b8e3cb6188e9ad787b20cf7b
- design.md: 17861782f1bbd234db3572f0023d300a4d5a8603648743390345ac417748ccd3
- tasks.md: 432a667f0e36a4d2ccbbfeca4af2815e58173f55826a2e646eb7cecb5e1f7882
- specs/contract-governance/spec.md: 4b783f7d285f0e787b24e5f2fc554b2b1a2047b07f8b9a839ad666fdba3f3850

This approval authorizes that amendment's implementation and verification. It is separate from final conformance approval and designated human CODEOWNER review of the draft PR.

Independent reviewer `/root/review49` also explicitly approved this amendment after reading all four artifacts and confirming current-version equality in the canonical Rust animation/inherited-timing consumers. The reviewer requires the exact narrow scope and complete passing parity/final gates. The parent separately recomputed all six SHA256 predecessor pins directly from `git show b0a9075f:contracts/<catalog>-v1.json`, using recursively locale-sorted object keys, preserved array order/scalars, UTF-8 JSON.stringify and SHA256; every pin exactly matches the captured pre-mutation evidence. Catalog implementation had not begun at approval.
