## Why

MG-M6 now has verified complete-scene and release evidence, but readers must reconcile separate capability catalogs, historical schema milestones and workflow documents. Issue #80 needs one current entry point with executable examples that agree with the shipped API.

## What Changes

- Add a release documentation guide covering protocol1/schema44, independently versioned capabilities/catalogs/artifact content, canonical coordinate/timing/ordering/fallback owners, compatibility aliases, migrations, operational readiness and independent verification entry points.
- Add a private versioned teaching-example JSON corpus, executed unchanged through actual source MCP and default compiled packaged smoke. Cover standalone definition creation, aliased atomic definition/slot/instance edits, domain-invalid cycle, missing reference, stale revision, late rollback, one undo step, redo and disk reopen.
- Check example metadata/bindings/error claims and local documentation links; reject missing/duplicate required cases, unsupported example format and unknown runtime bindings.
- Link the guide from README and agent-bridge docs. Clarify schema13 and preparatory fixture statements in component-definitions, component-lifecycle and motion-graphics-contract-fixtures as historical activation context.

## Capabilities

### New Capabilities

- `motion-release-documentation`: current canonical documentation and actual source/default-package example conformance.

### Modified Capabilities

None. Existing runtime and public requirements remain unchanged.

## Impact

Documentation and private tests/examples only. No production code, public operation/schema/capability/provider/catalog change or migration is needed. Canonical public fixtures remain unchanged; the private example format is not a new public contract. Existing Rust/contract/provider/native/GUI evidence is reused only for unchanged inputs; affected bridge tests and final CI rerun.

## Non-goals

No renderer rewrite, contract expansion, model inference, new artistic sample, fixture deduplication or audit-branch edits. The small authoring example does not replace #70 synthetic markers, #77's ten-group scene, #78 default packaging or #79's required native proof. #15 remains report-only; the proposed rescue-video quality benchmark remains separate. No merge or deployment.

## Approval and dependencies

Approved before implementation under the user's explicit standing issue-scoped specification approval. Dependencies #77/#78 are closed. #79 is verified at111a10beacb02b7d0d793a9e0e2ab0f799f67d06: all three complete/default native platforms, focused Windows and all11 standard jobs including aggregate passed. Native fixture controls passed56/56 on all three OS; current main08c8ea27 remains unchanged. This issue's branch includes that verified head and preserves original heads.
