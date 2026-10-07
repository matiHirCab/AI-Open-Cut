# Verification: add-desktop-compositing-controls

All12 implementation/verification tasks complete. Completeness:14/14 requirements and31/31 scenarios covered in scenario-coverage.json. Correctness: named desktop, core, fresh headless and real MCP evidence plus actual built GPUI observations satisfy accepted requirements. Coherence: presentation-only controls use existing typed core transactions; no parallel domain validation, renderer, migration, schema or public surface change. Independent Sol Medium review approved with no remaining findings; every finding was corrected and retested.

Required local evidence:36 log-bound checks passed, including strict workspace Clippy/workspace tests, final40desktop tests,18compositing tests,458root policy tests, bridge583unit tests,442TypeScript contract tests,23integration tests,20packaged smoke tests, hermetic Python, configured native six MCP suites, release golden/report, three rules resolutions, raster cache hooks and default binary restore. Seven opt-in unit skips are separately covered by configured native execution. Integration uses a documented local Node --max-opt=0 memory workaround with unchanged tests; standard remote exact-head CI remains required.

Actual GPUI acceptance:168 screenshot/state/file-bound receipts on a deterministic project, unchanged production sources/binary, all7effect fields/defaults, masks/path variants/readonly gradients/order, blend/matte, aggregate effects/clip, failures/reset/history/selection/conflict/explicit refresh and fresh reopen. Visible preview remains a labelled placeholder; media claims derive from separate actual native render gates. Error screenshots were independently viewed, including ITEM_NOT_FOUND nonretryable and REVISION_CONFLICT retryable.

Preservation:1771/1788 predecessor tracked files unchanged;17approved tracked edits, all1144prior archives/living specs and predecessor contract pins untouched before own sync. Synchronization must preserve493unrelated requirement blocks and apply exactly1approved modification plus13additions.

No critical, warning or suggestion findings remain. Approved for own-only sync/archive. Final protected+strict postarchive gates and exact-head11-jobCI/publication remain PENDING administrative requirements; they are never inferred from this prepublication snapshot. No merge/deployment/issue closure is authorized.

Postarchive completion: exact accepted-block/preservation proof passed; protected Moon gate and strict all-spec validation both passed44items. Exact-head publication CI remains pending.
