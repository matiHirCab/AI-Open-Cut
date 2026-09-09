## Final contract review for issue #32

Reviewer: @matiHirCab (designated CODEOWNER). Status: approved by the user with “Approve” on 2026-09-09 in response to the final contract-review request. Initial proposal/design/spec/task approval was recorded on 2026-09-09; this document presents the resulting contract surface for final review.

## Resulting behavior

- Protocol 1 advertises `rich_text_documents`. Existing tool/operation names and stable errors/retryability remain unchanged.
- Add-text accepts exactly one of legacy `text` or `document`. Update-item accepts at most one, preserving content when both are omitted. Explicit null and conflicting fields fail. These inputs work in standalone edits, alias batches and durable drafts.
- A document is a closed `{runs:[{text,bold?,italic?,color?}]}` record. It preserves ordered literal Unicode, supports 1-256 runs and 1-4096 aggregate UTF-8 bytes, and retains existing six-digit colors/Boolean flags and item typography defaults. Existing rich-text slot scalar limits remain unchanged.
- Persisted schema 18 requires root/component text items to contain both the document and its exact concatenated `text` projection. Output retains `text` for old readers. Raw legacy component-create/update records may omit document; core supplies a single unstyled run.
- Supported schemas 1-17 and retained undo/redo migrate under the existing locked recoverable transaction. Older versions reject premature text-item documents; schema 18 rejects absent or inconsistent documents; future versions fail closed. No downgrade is supplied.
- Stored styled runs use existing styled-slot preparation and shared evaluation for preview/export. Semantically plain documents preserve legacy rendering. No new font/shaping promise, provider contract, editor UI, range-offset API or dependency edge is introduced.

## Canonical artifacts and consumers

Review contracts/rich-text-documents-v1.json, contract-ownership-v1.json, headless-protocol-v1.json, mcp-surface-v1.json and component-definitions-v1.json together with .github/CODEOWNERS. Rust core/protocol fixtures and TypeScript schema/MCP parity tests consume these surfaces. contracts:check explicitly includes rich-text fixture tests. Documentation is in docs/rich-text-documents.md and docs/project-persistence.md.

The persisted schema is versioned at 18; request/capability/output additions retain protocol 1 compatibility. Component input optionality is distinguished from required persisted output. No error catalog, provider vocabulary or retryability changed.

## Evidence and approval requested

verification.md maps all five requirements and sixteen scenarios to automated evidence. Rust formatting, strict Clippy, workspace tests, forced native rendering, TypeScript type/lint/unit/contract checks, MCP integration, packaged smoke and hermetic Python tests passed. Strict OpenSpec validation passed. Following approval and archival, Moon's final merge gate passed; see verification.md.

The user approved the resulting canonical contracts and governed consumers above on 2026-09-09. Approval authorizes archival and the final Moon gate; their results are recorded in verification.md. No further implementation work is outstanding.
