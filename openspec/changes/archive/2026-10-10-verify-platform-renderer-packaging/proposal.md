## Why
Issue #78 requires feature availability and fallback evidence on Windows/macOS/Linux plus portable runtime packaging. Existing EditorCore readiness and typed status already own detection. An owned baseline reproducer shows current version1 package verification accepting an external-to-package symlink with matching bytes; this fails a portable self-contained inventory.

## What Changes
Harden existing four-role version1 manifest validation and package assembly name preflight; preserve valid default layout and filenames. Add meaningful malformed/integrity/path/link/duplicate/case/undeclared-file negatives. Exercise existing canonical renderer/editor/optional-audio readiness and unavailable-dependency behavior through source and actual default assembled-runtime MCP, including genuine frame/A-V range/export/history/reopen. Add a separate pinned three-platform native workflow and its own non-skipping policy tests; all existing required workflows/checks/deadlines remain unchanged. Document availability vs path diagnostics and external dependencies.

## Capabilities
### New Capabilities
- `platform-renderer-packaging`: portable package integrity and platform execution evidence.
### Modified Capabilities
None. No public MCP/headless/provider shape, capability/protocol major, persisted schema44 or renderer semantic change.

## Impact
Bridge packaging helper/tests, bounded default-runtime native harness, platform workflow/policy tests and documentation. Existing #58/#69/#77 fixtures and frozen references remain unchanged; #77 private instrumented-cache evidence is separate from default shipped binaries. No broad fixture cleanup, model accuracy/creative benchmark, merge/deploy or unsupported local Windows/macOS claim.

## Approval
The delegated user explicitly authorizes issue-scoped specification approval, implementation, verification, pushes/draft PRs and continuation. This bounded proposal/design/delta/tasks are approved under that authority before code edits, with actual exact-head three-platform native acceptance required before advancing beyond this issue.
